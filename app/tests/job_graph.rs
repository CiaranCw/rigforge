mod common;

use common::{certify, source, unpublished_graph};
use rigforge_app::{AppError, JobRunState, SqliteCatalog};
use rigforge_domain::{
    BoneMappingVersion, CharacterAsset, CharacterAssetVersion, ErrorCode, JobSpec,
    MotionAssetVersion, RetargetPolicyVersion, SourceSkeletonReference,
};

fn store_inputs(
    catalog: &mut SqliteCatalog,
    character: Option<&CharacterAssetVersion>,
    motion: Option<&MotionAssetVersion>,
    skeleton: Option<&SourceSkeletonReference>,
    mapping: Option<&BoneMappingVersion>,
    policy: Option<&RetargetPolicyVersion>,
) {
    if let Some(record) = character {
        catalog.put_validated(&certify(record.clone())).unwrap();
    }
    if let Some(record) = skeleton {
        catalog.put_validated(&certify(record.clone())).unwrap();
    }
    if let Some(record) = motion {
        catalog.put_validated(&certify(record.clone())).unwrap();
    }
    if let Some(record) = mapping {
        catalog.put_validated(&certify(record.clone())).unwrap();
    }
    if let Some(record) = policy {
        catalog.put_validated(&certify(record.clone())).unwrap();
    }
}

fn assert_enqueue_did_not_persist(catalog: &SqliteCatalog, spec_id: &str) {
    match catalog.load_job_spec(spec_id) {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("JobSpec must not persist after failed enqueue, got {other:?}"),
    }
    let runs = catalog.list_job_runs().unwrap();
    assert!(
        runs.iter().all(|run| run.job_spec_id != spec_id),
        "JobRun must not persist after failed enqueue: {runs:?}"
    );
}

#[test]
fn enqueue_fails_when_character_version_missing() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        None,
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    let err = catalog.enqueue_job(spec).unwrap_err();
    match err {
        AppError::NotFound { .. } => {}
        other => panic!("expected NotFound, got {other}"),
    }
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_motion_version_missing() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        None,
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    catalog.enqueue_job(spec).unwrap_err();
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_source_skeleton_missing() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        None,
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    catalog.enqueue_job(spec).unwrap_err();
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_mapping_missing() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        None,
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    catalog.enqueue_job(spec).unwrap_err();
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_policy_missing() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        None,
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    catalog.enqueue_job(spec).unwrap_err();
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_mapping_targets_wrong_character() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let mut other = CharacterAsset::new("Other").unwrap();
    let mut other_version =
        CharacterAssetVersion::draft(other.id(), "Other v1", source("other", 8)).unwrap();
    other_version.publish().unwrap();
    other.bind_published(other_version.id());
    store_inputs(
        &mut catalog,
        Some(&other_version),
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let job = JobSpec::new(
        other_version.id(),
        g.motion_version.id(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let spec = certify(job);
    let spec_id = spec.as_record().id().canonical();
    let err = catalog.enqueue_job(spec).unwrap_err();
    match err {
        AppError::Domain(domain) => assert_eq!(domain.code, ErrorCode::GraphMismatch),
        other => panic!("expected GraphMismatch, got {other}"),
    }
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn enqueue_fails_when_motion_uses_wrong_source_skeleton() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let other_skeleton = SourceSkeletonReference::new("other-skel").unwrap();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        Some(&other_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let job = JobSpec::new(
        g.character_version.id(),
        g.motion_version.id(),
        other_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let spec = certify(job);
    let spec_id = spec.as_record().id().canonical();
    let err = catalog.enqueue_job(spec).unwrap_err();
    match err {
        AppError::Domain(domain) => assert_eq!(domain.code, ErrorCode::GraphMismatch),
        other => panic!("expected GraphMismatch, got {other}"),
    }
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn invalidated_input_cannot_enqueue() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let mut g = unpublished_graph();
    g.character_version.invalidate().unwrap();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let spec_id = spec.as_record().id().canonical();
    let err = catalog.enqueue_job(spec).unwrap_err();
    match err {
        AppError::Domain(domain) => assert_eq!(domain.code, ErrorCode::InvalidatedInput),
        other => panic!("expected InvalidatedInput, got {other}"),
    }
    assert_enqueue_did_not_persist(&catalog, &spec_id);
}

#[test]
fn valid_exact_graph_enqueues() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    store_inputs(
        &mut catalog,
        Some(&g.character_version),
        Some(&g.motion_version),
        Some(&g.source_skeleton),
        Some(&g.mapping_version),
        Some(&g.policy_version),
    );
    let spec = certify(g.job.clone());
    let run = catalog.enqueue_job(spec.clone()).unwrap();
    assert_eq!(run.state, JobRunState::Queued);
    assert_eq!(
        catalog
            .load_job_spec(&run.job_spec_id)
            .unwrap()
            .as_record()
            .id(),
        spec.as_record().id()
    );
}
