mod common;

use common::{
    certify, failed_worker_for, matching_failure, matching_success, successful_worker_for,
    unpublished_graph,
};
use rigforge_app::{
    AppError, DispatchReceipt, FakeWorker, JobRunState, SqliteCatalog, TerminalOutcome,
    WorkerCompletionPort, WorkerFailureClass, WorkerPort,
};
use rigforge_domain::{to_json, ExecutionCorrelation, JobSpec, Validated, WorkerResult};

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
    catalog
        .put_validated(&certify(g.compatibility.clone()))
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
    assert_eq!(
        receipt.worker_execution_ref,
        format!("fake-worker:{}", receipt.attempt_id)
    );
    let after = catalog.load_job_spec(&spec_id).unwrap();
    assert_eq!(to_json(&after).unwrap(), spec_json_before);
    catalog
        .complete_success(&run.run_id, &matching_success(after.as_record(), &running))
        .unwrap();
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
    assert_eq!(
        receipt.worker_execution_ref,
        "fake-worker:attempt-from-orchestrator"
    );
    assert_eq!(receipt.attempt_id, "attempt-from-orchestrator");
}

#[test]
fn mismatched_worker_result_job_spec_cannot_complete_success() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let other = unpublished_graph();
    let mismatched = successful_worker_for(
        &other.job,
        &run.attempt_id,
        run.worker_execution_ref.as_deref().unwrap(),
    );
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
    let run = catalog.load_job_run(&run_id).unwrap();
    let failed = matching_failure(spec.as_record(), &run);
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
    let run = catalog.load_job_run(&run_id).unwrap();
    let worker_result = matching_success(spec.as_record(), &run);
    let worker_id = worker_result.as_record().id().canonical();
    let done = catalog.complete_success(&run_id, &worker_result).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    assert_eq!(done.worker_result_id.as_deref(), Some(worker_id.as_str()));
    let loaded = catalog.load_worker_result(&worker_id).unwrap();
    assert_eq!(loaded.as_record().job_spec_id(), spec.as_record().id());
    assert!(loaded.as_record().worker_success());
}

struct CollectErrWorker;

impl WorkerCompletionPort for CollectErrWorker {
    fn collect(
        &mut self,
        _receipt: &DispatchReceipt,
    ) -> Result<TerminalOutcome, AppError> {
        Err(AppError::Worker(
            "injected terminal wait failure".into(),
        ))
    }
}

#[test]
fn collection_failure_does_not_leave_running() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (_spec, run_id) = running_job(&mut catalog);
    assert_eq!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Running
    );
    let (done, outcome) = catalog
        .collect(&run_id, &mut CollectErrWorker)
        .unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    assert!(done.worker_result_id.is_none());
    assert!(
        done.failure_reason
            .as_deref()
            .unwrap_or("")
            .contains("injected terminal wait failure"),
        "{:?}",
        done.failure_reason
    );
    match outcome {
        TerminalOutcome::Failed {
            worker_result,
            reason,
            ..
        } => {
            assert!(worker_result.is_none());
            assert!(reason.contains("terminal collection failure"));
        }
        other => panic!("{other:?}"),
    }
}

struct FailedOutcomePort {
    result: Option<Validated<WorkerResult>>,
}

impl WorkerCompletionPort for FailedOutcomePort {
    fn collect(
        &mut self,
        _receipt: &DispatchReceipt,
    ) -> Result<TerminalOutcome, AppError> {
        Ok(TerminalOutcome::Failed {
            class: WorkerFailureClass::StructuredWorkerFail,
            reason: "injected terminal failure".into(),
            worker_result: self.result.clone(),
        })
    }
}

fn assert_failed_without_result(catalog: &SqliteCatalog, run_id: &str, needle: &str) {
    let run = catalog.load_job_run(run_id).unwrap();
    assert_eq!(run.state, JobRunState::Failed);
    assert!(run.worker_result_id.is_none());
    let reason = run.failure_reason.as_deref().unwrap_or("");
    assert!(reason.contains("injected terminal failure"), "{reason}");
    assert!(reason.contains("evidence rejected"), "{reason}");
    assert!(reason.contains(needle), "{reason}");
}

#[test]
fn same_jobspec_different_attempt_cannot_reuse_worker_result() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let a = catalog.enqueue_job(spec.clone()).unwrap();
    let b = catalog.enqueue_job(spec.clone()).unwrap();
    catalog.mark_dispatchable(&a.run_id).unwrap();
    catalog.mark_dispatchable(&b.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&a.run_id, &mut worker).unwrap();
    catalog.dispatch(&b.run_id, &mut worker).unwrap();
    let run_a = catalog.load_job_run(&a.run_id).unwrap();
    let run_b = catalog.load_job_run(&b.run_id).unwrap();
    assert_ne!(run_a.attempt_id, run_b.attempt_id);
    let result = matching_success(spec.as_record(), &run_a);
    catalog.complete_success(&a.run_id, &result).unwrap();
    let err = catalog.complete_success(&b.run_id, &result).unwrap_err();
    match err {
        AppError::Orchestration(msg) => {
            assert!(
                msg.contains("attempt_id") || msg.contains("already authorizes"),
                "{msg}"
            );
        }
        other => panic!("unexpected {other}"),
    }
    let still = catalog.load_job_run(&b.run_id).unwrap();
    assert_eq!(still.state, JobRunState::Running);
    assert!(still.worker_result_id.is_none());
}

#[test]
fn wrong_attempt_worker_result_cannot_complete_success() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let wrong = successful_worker_for(
        spec.as_record(),
        "not-this-attempt",
        run.worker_execution_ref.as_deref().unwrap(),
    );
    let err = catalog.complete_success(&run_id, &wrong).unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("attempt_id"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    assert_eq!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Running
    );
}

#[test]
fn wrong_execution_ref_worker_result_cannot_complete_success() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let wrong = successful_worker_for(spec.as_record(), &run.attempt_id, "not-this-ref");
    let err = catalog.complete_success(&run_id, &wrong).unwrap_err();
    match err {
        AppError::Orchestration(msg) => {
            assert!(msg.contains("worker_execution_ref"), "{msg}")
        }
        other => panic!("unexpected {other}"),
    }
    assert_eq!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Running
    );
}

#[test]
fn matching_attempt_worker_result_completes_success() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let result = matching_success(spec.as_record(), &run);
    let rec = result.as_record();
    assert_eq!(rec.job_spec_id().canonical(), run.job_spec_id);
    assert_eq!(rec.attempt_id(), run.attempt_id);
    assert_eq!(
        rec.worker_execution_ref(),
        run.worker_execution_ref.as_deref().unwrap()
    );
    assert!(rec.worker_success());
    let done = catalog.complete_success(&run_id, &result).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    assert_eq!(
        done.worker_result_id.as_deref(),
        Some(rec.id().canonical().as_str())
    );
}

#[test]
fn one_worker_result_id_cannot_complete_two_runs() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let spec = seeded_spec(&mut catalog);
    let a = catalog.enqueue_job(spec.clone()).unwrap();
    let b = catalog.enqueue_job(spec.clone()).unwrap();
    catalog.mark_dispatchable(&a.run_id).unwrap();
    catalog.mark_dispatchable(&b.run_id).unwrap();
    let mut worker = FakeWorker::default();
    catalog.dispatch(&a.run_id, &mut worker).unwrap();
    catalog.dispatch(&b.run_id, &mut worker).unwrap();
    let run_a = catalog.load_job_run(&a.run_id).unwrap();
    let result = matching_success(spec.as_record(), &run_a);
    let worker_id = result.as_record().id().canonical();
    catalog.complete_success(&a.run_id, &result).unwrap();
    let err = catalog.complete_success(&b.run_id, &result).unwrap_err();
    match err {
        AppError::Orchestration(msg) => {
            assert!(
                msg.contains(&worker_id) || msg.contains("attempt_id") || msg.contains("already authorizes"),
                "{msg}"
            );
        }
        other => panic!("unexpected {other}"),
    }
    assert_eq!(
        catalog.load_job_run(&b.run_id).unwrap().state,
        JobRunState::Running
    );
    assert_eq!(
        catalog
            .load_job_run(&a.run_id)
            .unwrap()
            .worker_result_id
            .as_deref(),
        Some(worker_id.as_str())
    );
}

#[test]
fn failed_outcome_wrong_jobspec_marks_failed_without_result() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (_spec, run_id) = running_job(&mut catalog);
    let other = unpublished_graph();
    let run = catalog.load_job_run(&run_id).unwrap();
    let bad = failed_worker_for(
        &other.job,
        &run.attempt_id,
        run.worker_execution_ref.as_deref().unwrap(),
    );
    let bad_id = bad.as_record().id().canonical();
    let (done, _) = catalog
        .collect(
            &run_id,
            &mut FailedOutcomePort {
                result: Some(bad),
            },
        )
        .unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    assert!(done.worker_result_id.is_none());
    match catalog.load_worker_result(&bad_id) {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("rejected WorkerResult must not persist, got {other:?}"),
    }
    assert_failed_without_result(&catalog, &run_id, "job_spec_id");
}

#[test]
fn failed_outcome_success_valued_result_marks_failed_without_result() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let bad = matching_success(spec.as_record(), &run);
    let bad_id = bad.as_record().id().canonical();
    catalog
        .collect(
            &run_id,
            &mut FailedOutcomePort {
                result: Some(bad),
            },
        )
        .unwrap();
    match catalog.load_worker_result(&bad_id) {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("success-valued failure evidence must not persist, got {other:?}"),
    }
    assert_failed_without_result(&catalog, &run_id, "worker_success");
}

#[test]
fn failed_outcome_wrong_attempt_marks_failed_without_result() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (spec, run_id) = running_job(&mut catalog);
    let run = catalog.load_job_run(&run_id).unwrap();
    let bad = failed_worker_for(
        spec.as_record(),
        "other-attempt",
        run.worker_execution_ref.as_deref().unwrap(),
    );
    catalog
        .collect(
            &run_id,
            &mut FailedOutcomePort {
                result: Some(bad),
            },
        )
        .unwrap();
    assert_failed_without_result(&catalog, &run_id, "attempt_id");
}

#[test]
fn rejected_failure_evidence_never_leaves_running() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (_spec, run_id) = running_job(&mut catalog);
    let other = unpublished_graph();
    catalog
        .collect(
            &run_id,
            &mut FailedOutcomePort {
                result: Some(failed_worker_for(&other.job, "nope", "nope-ref")),
            },
        )
        .unwrap();
    assert_ne!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Running
    );
    assert_eq!(
        catalog.load_job_run(&run_id).unwrap().state,
        JobRunState::Failed
    );
}

#[test]
fn worker_result_execution_correlation_is_required_and_read_only() {
    let g = unpublished_graph();
    let corr = ExecutionCorrelation::new("attempt-a", "worker-ref-a").unwrap();
    let result = WorkerResult::new(g.job.id(), g.backend.clone(), true, "completed", corr).unwrap();
    assert_eq!(result.attempt_id(), "attempt-a");
    assert_eq!(result.worker_execution_ref(), "worker-ref-a");
    let json = to_json(&result).unwrap();
    assert!(json.contains("execution_correlation"));
    assert!(json.contains("attempt-a"));
}
