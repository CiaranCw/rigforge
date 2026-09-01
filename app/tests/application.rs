mod common;

use common::{certify, unpublished_graph, valid_graph};
use rigforge_app::{Application, FakeWorker, JobRunState};

#[test]
fn application_queries_keep_asset_kinds_separate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = valid_graph();
    app.catalog_mut()
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &certify(g.derived.clone()),
            &certify(g.derived_version.clone()),
        )
        .unwrap();

    let characters = app.list_characters().unwrap();
    let motions = app.list_motions().unwrap();
    let derived = app.list_derived_variants().unwrap();
    assert_eq!(characters.len(), 1);
    assert_eq!(motions.len(), 1);
    assert_eq!(derived.len(), 1);
    assert_eq!(characters[0].display_name, "Knight");
    assert_eq!(motions[0].display_name, "Walk Carry");
    assert_eq!(derived[0].display_name, "Knight Walk Carry");
    assert_ne!(characters[0].logical_id, motions[0].logical_id);
    assert_ne!(characters[0].logical_id, derived[0].logical_id);

    let exact = app
        .resolve_exact_character_version(&g.character_version.id().canonical())
        .unwrap();
    assert_eq!(exact.as_record().id(), g.character_version.id());
    assert!(!Application::requires_network());
    assert!(!Application::requires_blender());
}

#[test]
fn selected_version_is_exact_and_job_status_does_not_mutate_product() {
    let mut app = Application::open_in_memory().unwrap();
    let g = unpublished_graph();
    app.catalog_mut()
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.mapping_version.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.policy_version.clone()))
        .unwrap();
    let spec = certify(g.job.clone());
    let spec_json = rigforge_domain::to_json(&spec).unwrap();
    let run = app.enqueue_job(spec).unwrap();
    app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(&run.run_id, &mut worker).unwrap();
    let status = app.job_status(&run.run_id).unwrap();
    assert_eq!(status.state, JobRunState::Running);
    let loaded = app.load_job_spec(&g.job.id().canonical()).unwrap();
    assert_eq!(rigforge_domain::to_json(&loaded).unwrap(), spec_json);
}
