mod common;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{certify, successful_worker_for, unpublished_graph, valid_graph};
use rigforge_app::{FakeWorker, JobRunState, SqliteCatalog};

fn temp_db() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rigforge-v1-2-reopen-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join("catalog.sqlite")
}

#[test]
fn reopen_preserves_catalog_and_job_state() {
    let path = temp_db();
    let g = valid_graph();
    let spec_id;
    let character_version_id = g.character_version.id().canonical();
    let artifact_id = g.persistence.id().canonical();
    let run_id;
    let queued_run_id;
    let spec_json;
    {
        let mut catalog = SqliteCatalog::open(&path).unwrap();
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
        catalog.put_validated(&certify(g.job.clone())).unwrap();
        catalog
            .put_validated(&certify(g.persistence.clone()))
            .unwrap();
        catalog
            .put_validated(&certify(g.verification.clone()))
            .unwrap();
        spec_json = rigforge_domain::to_json(&g.job).unwrap();
        spec_id = g.job.id().canonical();
        let spec = certify(g.job.clone());
        let queued = catalog.enqueue_job(spec.clone()).unwrap();
        queued_run_id = queued.run_id.clone();
        let run = catalog.enqueue_job(spec).unwrap();
        catalog.mark_dispatchable(&run.run_id).unwrap();
        let mut worker = FakeWorker::default();
        catalog.dispatch(&run.run_id, &mut worker).unwrap();
        catalog.complete_success(&run.run_id, &successful_worker_for(&g.job)).unwrap();
        run_id = run.run_id;
    }

    let catalog = SqliteCatalog::open(&path).unwrap();
    let loaded_version = catalog
        .load_character_version(&character_version_id)
        .unwrap();
    assert_eq!(loaded_version.as_record(), &g.character_version);
    let loaded_spec = catalog.load_job_spec(&spec_id).unwrap();
    assert_eq!(rigforge_domain::to_json(&loaded_spec).unwrap(), spec_json);
    let artifact = catalog.load_artifact_metadata(&artifact_id).unwrap();
    assert_eq!(artifact.as_record(), &g.persistence);
    let succeeded = catalog.load_job_run(&run_id).unwrap();
    assert_eq!(succeeded.state, JobRunState::Succeeded);
    let queued = catalog.load_job_run(&queued_run_id).unwrap();
    assert_eq!(queued.state, JobRunState::Queued);
    assert_eq!(catalog.path(), Some(path.as_path()));
}

#[test]
fn reopen_empty_file_still_migrated() {
    let path = temp_db();
    {
        let catalog = SqliteCatalog::open(&path).unwrap();
        assert_eq!(catalog.db_schema_version().unwrap(), 1);
    }
    let catalog = SqliteCatalog::open(&path).unwrap();
    assert_eq!(catalog.db_schema_version().unwrap(), 1);
    let _g = unpublished_graph();
}
