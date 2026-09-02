mod common;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{certify, matching_success, unpublished_graph};
use rigforge_app::{
    DispatchReceipt, FakeWorker, JobRunState, SqliteCatalog, DISPATCH_INTENT_DIRNAME,
};
use rigforge_domain::Validated;

fn temp_db() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rigforge-v18-runtime-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join("catalog.sqlite")
}

fn seed_catalog(catalog: &mut SqliteCatalog) -> Validated<rigforge_domain::JobSpec> {
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

#[test]
fn dispatchable_without_intent_survives_reopen() {
    let path = temp_db();
    let run_id;
    {
        let mut catalog = SqliteCatalog::open(&path).unwrap();
        let spec = seed_catalog(&mut catalog);
        let run = catalog.enqueue_job(spec).unwrap();
        catalog.mark_dispatchable(&run.run_id).unwrap();
        run_id = run.run_id;
    }
    let catalog = SqliteCatalog::open(&path).unwrap();
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Dispatchable);
}

#[test]
fn spawn_to_running_intent_without_durable_running_fails_closed_on_reopen() {
    let path = temp_db();
    let run_id;
    {
        let mut catalog = SqliteCatalog::open(&path).unwrap();
        let spec = seed_catalog(&mut catalog);
        let run = catalog.enqueue_job(spec).unwrap();
        catalog.mark_dispatchable(&run.run_id).unwrap();
        let dir = catalog.dispatch_intent_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{}.intent.json", run.run_id)),
            br#"{"simulated":"spawn-without-running"}"#,
        )
        .unwrap();
        assert_eq!(DISPATCH_INTENT_DIRNAME, "rigforge-dispatch-intents");
        run_id = run.run_id;
    }
    let catalog = SqliteCatalog::open(&path).unwrap();
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Failed);
    let reason = run.failure_reason.unwrap_or_default();
    assert!(
        reason.contains("spawn-to-RUNNING") || reason.contains("dispatch intent"),
        "{reason}"
    );
}

#[test]
fn running_without_in_process_handle_fails_closed_on_reopen() {
    let path = temp_db();
    let spec;
    let run_id;
    {
        let mut catalog = SqliteCatalog::open(&path).unwrap();
        spec = seed_catalog(&mut catalog);
        let run = catalog.enqueue_job(spec.clone()).unwrap();
        catalog.mark_dispatchable(&run.run_id).unwrap();
        let mut worker = FakeWorker::default();
        catalog.dispatch(&run.run_id, &mut worker).unwrap();
        assert_eq!(
            catalog.load_job_run(&run.run_id).unwrap().state,
            JobRunState::Running
        );
        run_id = run.run_id;
    }
    let mut catalog = SqliteCatalog::open(&path).unwrap();
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Failed);
    let reason = run.failure_reason.clone().unwrap_or_default();
    assert!(reason.contains("RUNNING"), "{reason}");
    let err = catalog
        .complete_success(&run_id, &matching_success(spec.as_record(), &run))
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("illegal job transition") || msg.contains("RUNNING") || msg.contains("FAILED"),
        "{msg}"
    );
}

#[test]
fn failed_orphan_cannot_publish_from_stale_worker_result() {
    let path = temp_db();
    let spec;
    let run_id;
    let receipt: DispatchReceipt;
    {
        let mut catalog = SqliteCatalog::open(&path).unwrap();
        spec = seed_catalog(&mut catalog);
        let run = catalog.enqueue_job(spec.clone()).unwrap();
        catalog.mark_dispatchable(&run.run_id).unwrap();
        let mut worker = FakeWorker::default();
        let (running, rec) = catalog.dispatch(&run.run_id, &mut worker).unwrap();
        assert_eq!(running.state, JobRunState::Running);
        run_id = run.run_id;
        receipt = rec;
        let _ = receipt;
    }
    let mut catalog = SqliteCatalog::open(&path).unwrap();
    let run = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Failed);
    let forged = matching_success(spec.as_record(), &run);
    let err = catalog.complete_success(&run_id, &forged).unwrap_err();
    assert!(
        err.to_string().contains("illegal job transition")
            || err.to_string().contains("orchestration"),
        "{err}"
    );
}
