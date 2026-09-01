mod common;

use common::{certify, failed_worker_for, successful_worker_for, unpublished_graph};
use rigforge_app::{AppError, FakeWorker, JobRunState, SqliteCatalog, WorkerPort};
use rigforge_domain::{to_json, JobSpec, Validated};

fn seeded_spec(catalog: &mut SqliteCatalog) -> Validated<JobSpec> {
    let g = unpublished_graph();
    catalog
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.mapping_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.policy_version.clone()))
        .unwrap();
    certify(g.job)
}

fn running_job(catalog: &mut SqliteCatalog) -> (Validated<JobSpec>, String) {
    let spec = seeded_spec(catalog);
    let run = catalog.enqueue_job(spec.clone()).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&run.run_id, &mut worker).unwrap();
    (spec, run.run_id)
}

#[test]
fn enqueue_validated_job_spec_starts_queued() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let spec_json = to_json(&spec).unwrap();
    let run = catalog.enqueue_job(spec.clone()).unwrap();
    assert_eq!(run.state, JobRunState::Queued);
    assert_eq!(run.job_spec_id, spec.as_record().id().canonical());
    let loaded = catalog.load_job_spec(&run.job_spec_id).unwrap();
    assert_eq!(to_json(&loaded).unwrap(), spec_json);
}

#[test]
fn valid_transitions_and_fake_worker_receive_exact_spec() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let spec_id = spec.as_record().id().canonical();
    let character_id = spec.as_record().character_version_id().canonical();
    let motion_id = spec.as_record().motion_version_id().canonical();
    let spec_json_before = to_json(&spec).unwrap();
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    let (running, receipt) = catalog.dispatch(&run.run_id, &mut worker).unwrap();
    assert_eq!(running.state, JobRunState::Running);
    assert_eq!(worker.last_job_spec_id.as_deref(), Some(spec_id.as_str()));
    assert_eq!(
        worker.last_character_version_id.as_deref(),
        Some(character_id.as_str())
    );
    assert_eq!(
        worker.last_motion_version_id.as_deref(),
        Some(motion_id.as_str())
    );
    assert_eq!(receipt.worker_execution_ref, "fake-worker:v1-2");
    let after = catalog.load_job_spec(&spec_id).unwrap();
    assert_eq!(to_json(&after).unwrap(), spec_json_before);
    catalog.complete_success(&run.run_id, &successful_worker_for(after.as_record())).unwrap();
    let done = catalog.load_job_run(&run.run_id).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    let after_success = catalog.load_job_spec(&spec_id).unwrap();
    assert_eq!(to_json(&after_success).unwrap(), spec_json_before);
}

#[test]
fn invalid_transitions_fail() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let run = catalog.enqueue_job(spec).unwrap();
    let err = catalog
        .transition(&run.run_id, JobRunState::Running)
        .unwrap_err();
    match err {
        AppError::InvalidTransition {
            from: JobRunState::Queued,
            to: JobRunState::Running,
        } => {}
        other => panic!("unexpected {other}"),
    }
    let err = catalog
        .transition(&run.run_id, JobRunState::Succeeded)
        .unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("complete_success"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&run.run_id, &mut worker).unwrap();
    let err = catalog.cancel(&run.run_id).unwrap_err();
    match err {
        AppError::InvalidTransition {
            from: JobRunState::Running,
            to: JobRunState::Cancelled,
        } => {}
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn cancel_from_queued_and_dispatchable_is_deterministic() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let queued = catalog.enqueue_job(spec.clone()).unwrap();
    let cancelled = catalog.cancel(&queued.run_id).unwrap();
    assert_eq!(cancelled.state, JobRunState::Cancelled);
    let err = catalog.mark_dispatchable(&queued.run_id).unwrap_err();
    match err {
        AppError::InvalidTransition {
            from: JobRunState::Cancelled,
            ..
        } => {}
        other => panic!("unexpected {other}"),
    }

    let second = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&second.run_id).unwrap();
    let cancelled = catalog.cancel(&second.run_id).unwrap();
    assert_eq!(cancelled.state, JobRunState::Cancelled);
}

#[test]
fn failure_persists_diagnostics_without_mutating_job_spec() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let spec_json = to_json(&spec).unwrap();
    let spec_id = spec.as_record().id().canonical();
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&run.run_id, &mut worker).unwrap();
    let failed = catalog
        .complete_failure(&run.run_id, "fake worker produced no output")
        .unwrap();
    assert_eq!(failed.state, JobRunState::Failed);
    assert_eq!(
        failed.failure_reason.as_deref(),
        Some("fake worker produced no output")
    );
    assert_eq!(to_json(&catalog.load_job_spec(&spec_id).unwrap()).unwrap(), spec_json);
}

#[test]
fn fake_worker_refusal_does_not_start_running() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker {
        fail: true,
        ..FakeWorker::default()
    };
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    match err {
        AppError::Worker(_) => {}
        other => panic!("unexpected {other}"),
    }
    let still = catalog.load_job_run(&run.run_id).unwrap();
    assert_eq!(still.state, JobRunState::Failed);
    assert!(
        still
            .failure_reason
            .as_deref()
            .unwrap_or("")
            .contains("fake worker refused dispatch"),
        "{still:?}"
    );
}

#[test]
fn dispatch_does_not_resolve_latest() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let exact_character = spec.as_record().character_version_id().canonical();
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&run.run_id, &mut worker).unwrap();
    assert_eq!(
        worker.last_character_version_id.as_deref(),
        Some(exact_character.as_str())
    );
}

#[test]
fn worker_port_is_usable_without_blender() {
    let mut worker = FakeWorker::default();
    let g = unpublished_graph();
    let spec = certify(g.job);
    let receipt = WorkerPort::dispatch(&mut worker, &spec, "attempt-from-orchestrator").unwrap();
    assert_eq!(receipt.worker_execution_ref, "fake-worker:v1-2");
    assert_eq!(receipt.attempt_id, "attempt-from-orchestrator");
}

#[test]
fn mismatched_worker_result_job_spec_cannot_complete_success() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let other = unpublished_graph();
    let mismatched = successful_worker_for(&other.job);
    let err = catalog
        .complete_success(&run_id, &mismatched)
        .unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("job_spec_id"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Running);
    assert!(run.worker_result_id.is_none());
    match catalog.load_worker_result(&mismatched.as_record().id().canonical()) {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("mismatched WorkerResult must not persist, got {other:?}"),
    }
    assert_eq!(
        catalog
            .load_job_spec(&spec.as_record().id().canonical())
            .unwrap()
            .as_record()
            .id(),
        spec.as_record().id()
    );
}

#[test]
fn successful_run_requires_worker_result() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (_spec, run_id) = running_job(&mut catalog);
    let err = catalog
        .transition(&run_id, JobRunState::Succeeded)
        .unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("WorkerResult"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    assert_eq!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Running
    );
}

#[test]
fn failed_worker_result_cannot_mark_succeeded() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let failed = failed_worker_for(spec.as_record());
    let err = catalog.complete_success(&run_id, &failed).unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("worker_success"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Running);
    match catalog.load_worker_result(&failed.as_record().id().canonical()) {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("failed WorkerResult must not be stored on SUCCEEDED path, got {other:?}"),
    }
}

#[test]
fn worker_dispatch_failure_persists_failed_state_and_reason() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker {
        fail: true,
        ..FakeWorker::default()
    };
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    match err {
        AppError::Worker(_) => {}
        other => panic!("unexpected {other}"),
    }
    let failed = catalog.load_job_run(&run.run_id).unwrap();
    assert_eq!(failed.state, JobRunState::Failed);
    assert!(
        failed
            .failure_reason
            .as_deref()
            .unwrap_or("")
            .contains("fake worker refused dispatch")
    );
}

#[test]
fn attempt_id_is_stable_across_dispatch() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let run = catalog.enqueue_job(spec).unwrap();
    let attempt_id = run.attempt_id.clone();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    let (running, receipt) = catalog.dispatch(&run.run_id, &mut worker).unwrap();
    assert_eq!(running.attempt_id, attempt_id);
    assert_eq!(receipt.attempt_id, attempt_id);
    assert_eq!(worker.last_attempt_id.as_deref(), Some(attempt_id.as_str()));
}

#[test]
fn success_transaction_stores_matching_worker_result_and_state_atomically() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let worker_result = successful_worker_for(spec.as_record());
    let worker_id = worker_result.as_record().id().canonical();
    let done = catalog.complete_success(&run_id, &worker_result).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    assert_eq!(done.worker_result_id.as_deref(), Some(worker_id.as_str()));
    let loaded = catalog.load_worker_result(&worker_id).unwrap();
    assert_eq!(loaded.as_record().job_spec_id(), spec.as_record().id());
    assert!(loaded.as_record().worker_success());
}
