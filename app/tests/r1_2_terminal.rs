mod common;

use common::{
    certify, matching_failure, matching_success, persist_core, successful_worker_for,
    unpublished_graph,
};
use rigforge_app::{
    AppError, Application, FakeWorker, JobRunState, TerminalOutcome, WorkerFailureClass,
};
use rigforge_domain::JobSpec;

fn running_app() -> (Application, String, JobSpec) {
    let mut app = Application::open_in_memory().unwrap();
    let g = unpublished_graph();
    persist_core(app.catalog_mut(), &g);
    let spec = certify(g.job.clone());
    let job: JobSpec = spec.as_record().clone();
    let run = app.enqueue_job(spec).unwrap();
    app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(&run.run_id, &mut worker).unwrap();
    (app, run.run_id, job)
}

#[test]
fn apply_terminal_outcome_success_persisted_exactly_once() {
    let (mut app, run_id, job) = running_app();
    let run = app.job_status(&run_id).unwrap();
    let result = matching_success(&job, &run);
    let worker_id = result.as_record().id().canonical();
    let (done, _) = app
        .apply_terminal_outcome(&run_id, TerminalOutcome::Success(result.clone()))
        .unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    assert_eq!(done.worker_result_id.as_deref(), Some(worker_id.as_str()));
    let loaded = app.catalog().load_worker_result(&worker_id).unwrap();
    assert_eq!(loaded.as_record().id().canonical(), worker_id);
    let err = app
        .apply_terminal_outcome(&run_id, TerminalOutcome::Success(result))
        .unwrap_err();
    match err {
        AppError::InvalidTransition { from, .. } => assert_eq!(from, JobRunState::Succeeded),
        other => panic!("expected InvalidTransition, got {other}"),
    }
    assert_eq!(
        app.job_status(&run_id).unwrap().worker_result_id.as_deref(),
        Some(worker_id.as_str())
    );
}

#[test]
fn apply_terminal_outcome_failed_preserves_matching_failure() {
    let (mut app, run_id, job) = running_app();
    let run = app.job_status(&run_id).unwrap();
    let failed = matching_failure(&job, &run);
    let worker_id = failed.as_record().id().canonical();
    let (done, _) = app
        .apply_terminal_outcome(
            &run_id,
            TerminalOutcome::Failed {
                class: WorkerFailureClass::StructuredWorkerFail,
                reason: "injected terminal failure".into(),
                worker_result: Some(failed),
            },
        )
        .unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    assert_eq!(done.worker_result_id.as_deref(), Some(worker_id.as_str()));
    let loaded = app.catalog().load_worker_result(&worker_id).unwrap();
    assert!(!loaded.as_record().worker_success());
}

#[test]
fn apply_terminal_outcome_attempt_mismatch_fails_closed() {
    let (mut app, run_id, job) = running_app();
    let run = app.job_status(&run_id).unwrap();
    let wrong = successful_worker_for(
        &job,
        "not-this-attempt",
        run.worker_execution_ref.as_deref().unwrap(),
    );
    let err = app
        .apply_terminal_outcome(&run_id, TerminalOutcome::Success(wrong))
        .unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("attempt_id"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    assert_eq!(app.job_status(&run_id).unwrap().state, JobRunState::Failed);
}

#[test]
fn apply_terminal_outcome_execution_ref_mismatch_fails_closed() {
    let (mut app, run_id, job) = running_app();
    let run = app.job_status(&run_id).unwrap();
    let wrong = successful_worker_for(&job, &run.attempt_id, "not-this-ref");
    let err = app
        .apply_terminal_outcome(&run_id, TerminalOutcome::Success(wrong))
        .unwrap_err();
    match err {
        AppError::Orchestration(msg) => assert!(msg.contains("worker_execution_ref"), "{msg}"),
        other => panic!("unexpected {other}"),
    }
    assert_eq!(app.job_status(&run_id).unwrap().state, JobRunState::Failed);
}

#[test]
fn catalog_collect_reuses_apply_terminal_outcome() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/catalog.rs"));
    assert!(src.contains("pub fn apply_terminal_outcome("));
    let collect = src
        .split("pub fn collect<C: WorkerCompletionPort + ?Sized>(")
        .nth(1)
        .expect("collect");
    let body = collect.split("pub fn assemble_worker_dispatch_request").next().unwrap();
    assert!(
        body.contains("self.apply_terminal_outcome(run_id, outcome)"),
        "collect must persist through apply_terminal_outcome after wait"
    );
    assert!(
        !body.contains("self.complete_success(run_id, result)"),
        "collect must not duplicate success persistence after the split"
    );
}
