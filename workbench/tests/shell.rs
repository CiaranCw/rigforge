mod common;

use common::{MemoryArtifactInspector, MemoryPersistenceReopener};
use rigforge_app::rigforge_domain::*;
use rigforge_app::{
    generate_mapping_proposal, sha256_file, Application, FakeWorker, MappingAssistProfile,
    TransferOutcomeKind, WorkerCapabilityProfile,
};
use rigforge_workbench::{PreviewEmbeddingSlot, WorkbenchApp};

fn digest(n: u8) -> ContentDigest {
    ContentDigest::parse(&format!("{n:02x}").repeat(32)).unwrap()
}

fn evidence(name: &str, n: u8) -> SourceArtifactEvidence {
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(format!("C:/research/{name}.bin")).unwrap(),
        digest(n),
        1024,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".to_string()),
        None,
    )
    .unwrap()
}

fn j(key: &str, parent: Option<&str>, root: bool, deform: &str) -> JointObservation {
    JointObservation::new(
        JointKey::new(key).unwrap(),
        key,
        parent.map(|p| JointKey::new(p).unwrap()),
        root,
        Some(deform.into()),
        Some("rest".into()),
    )
    .unwrap()
}

struct Seeded {
    app: Application,
    character_version: String,
    motion_version: String,
    policy_version: String,
    mapping_id: String,
    draft_id: String,
}

fn seed() -> Seeded {
    let mut app = Application::open_in_memory().unwrap();
    let producer = BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", evidence("knight", 1)).unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            "clip:walk-carry",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence("motion", 2),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    let mut policy = RetargetPolicy::new("rest-relative").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();
    let source = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::SourceSkeletonReference,
                None,
                Some(source_skeleton.id()),
                producer.clone(),
                vec![
                    j("root", None, true, "deforming"),
                    j("head", Some("root"), false, "deforming"),
                ],
                vec!["source".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let target = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::CharacterAssetVersion,
                Some(character_version.id()),
                None,
                producer,
                vec![
                    j("Bone", None, true, "deforming"),
                    j("Head", Some("Bone"), false, "deforming"),
                ],
                vec!["target".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "wb-map",
            &character_version.id().canonical(),
            &source_skeleton.id().canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    Seeded {
        app,
        character_version: character_version.id().canonical(),
        motion_version: motion_version.id().canonical(),
        policy_version: policy_version.id().canonical(),
        mapping_id: logical.as_record().id().canonical(),
        draft_id: draft.as_record().id().canonical(),
    }
}

#[test]
fn mapping_tray_supports_v1_4_workflow_state() {
    let mut shell = WorkbenchApp::empty();
    shell.select_character_version("character-version");
    shell.select_motion_version("motion-version");
    assert_eq!(shell.selected_character_version(), Some("character-version"));
    assert_eq!(shell.selected_motion_version(), Some("motion-version"));
    shell.show_mapping_proposal(
        vec!["root → Bone".into()],
        vec!["PoleTarget.L optional".into()],
        vec!["mid: equal candidates".into()],
    );
    assert_eq!(shell.mapping_entries().len(), 1);
    assert_eq!(shell.ambiguities().len(), 1);
    assert!(!shell.mapping_accepted());
    shell.show_compatibility(
        vec![("mapping_completeness".into(), "pass".into())],
        "ready",
    );
    assert_eq!(shell.compatibility_summary(), Some("ready"));
    assert!(shell.compatibility_id().is_none());
    assert!(!shell.transfer_available());
    assert!(shell.transfer_eligibility_label().contains("CompatibilityResult"));
    assert_eq!(
        WorkbenchApp::preview_unavailable_reason(),
        "NOT AVAILABLE — V1-6"
    );
}

#[test]
fn workbench_cannot_claim_acceptance_without_published_mapping() {
    let mut shell = WorkbenchApp::empty();
    shell.show_mapping_proposal(vec!["root → Bone".into()], vec![], vec![]);
    assert!(!shell.mapping_accepted());
    assert!(shell.accepted_mapping_version_id().is_none());
}

#[test]
fn workbench_accept_operation_persists_mapping() {
    let mut seeded = seed();
    let draft = seeded
        .app
        .load_mapping_version(&seeded.draft_id)
        .unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    assert!(!shell.mapping_accepted());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    assert!(shell.mapping_accepted());
    let loaded = seeded
        .app
        .load_mapping_version(shell.accepted_mapping_version_id().unwrap())
        .unwrap();
    assert_eq!(loaded.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        loaded.as_record().review().review_kind(),
        Some(MappingReviewKind::AutomaticConfirmed)
    );
}

#[test]
fn workbench_override_calls_application_workflow() {
    let mut seeded = seed();
    let draft = seeded
        .app
        .load_mapping_version(&seeded.draft_id)
        .unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell
        .override_current_mapping(&mut seeded.app, "head", Some("Head"), "user chose Head")
        .unwrap();
    assert!(!shell.mapping_accepted());
    let loaded = seeded
        .app
        .load_mapping_version(shell.mapping_version_id().unwrap())
        .unwrap();
    assert!(loaded.as_record().user_modified());
    assert_eq!(loaded.as_record().lifecycle(), Lifecycle::Draft);
}

#[test]
fn workbench_compatibility_display_uses_actual_result() {
    let mut seeded = seed();
    let draft = seeded
        .app
        .load_mapping_version(&seeded.draft_id)
        .unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    let result = seeded
        .app
        .run_compatibility_preflight(
            &seeded.character_version,
            &seeded.motion_version,
            shell.accepted_mapping_version_id().unwrap(),
            &seeded.policy_version,
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    shell.apply_compatibility_result(result.as_record());
    assert_eq!(
        shell.compatibility_id(),
        Some(result.as_record().id().canonical().as_str())
    );
    assert!(shell
        .compatibility_dimensions()
        .iter()
        .any(|(n, _)| n == "result_acceptability"));
    let expected = format!("{:?}", result.as_record().summary());
    assert_eq!(shell.compatibility_summary(), Some(expected.as_str()));
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    match result.as_record().summary() {
        CompatibilitySummary::Ready => assert!(shell.transfer_available()),
        CompatibilitySummary::ReadyWithWarnings => {
            assert!(!shell.transfer_available());
            assert!(shell.transfer_requires_acknowledgement());
            assert!(!shell.warnings_acknowledged());
            shell
                .acknowledge_compatibility_warnings(&seeded.app)
                .unwrap();
            assert!(shell.warnings_acknowledged());
            assert!(shell.transfer_available());
        }
        other => panic!("unexpected summary {other:?}"),
    }
}

#[test]
fn workbench_reopen_reads_persisted_mapping_state() {
    let dir = std::env::temp_dir().join(format!(
        "rf-v14-wb-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("catalog.sqlite");
    {
        let mut seeded = seed_on_path(&path);
        let draft = seeded
            .app
            .load_mapping_version(&seeded.draft_id)
            .unwrap();
        let mut shell = WorkbenchApp::empty();
        shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
        shell.accept_current_mapping(&mut seeded.app).unwrap();
        assert!(shell.mapping_accepted());
    }
    let reopened = Application::open(&path).unwrap();
    let shell = WorkbenchApp::from_application(&reopened).unwrap();
    assert!(shell.mapping_accepted());
    assert!(shell.accepted_mapping_version_id().is_some());
}

fn seed_on_path(path: &std::path::Path) -> Seeded {
    let mut app = Application::open(path).unwrap();
    let producer = BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", evidence("knight", 3)).unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            "clip:walk-carry",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence("motion", 4),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    let mut policy = RetargetPolicy::new("rest-relative").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();
    let source = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::SourceSkeletonReference,
                None,
                Some(source_skeleton.id()),
                producer.clone(),
                vec![
                    j("root", None, true, "deforming"),
                    j("head", Some("root"), false, "deforming"),
                ],
                vec!["source".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let target = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::CharacterAssetVersion,
                Some(character_version.id()),
                None,
                producer,
                vec![
                    j("Bone", None, true, "deforming"),
                    j("Head", Some("Bone"), false, "deforming"),
                ],
                vec!["target".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "wb-map-file",
            &character_version.id().canonical(),
            &source_skeleton.id().canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    Seeded {
        app,
        character_version: character_version.id().canonical(),
        motion_version: motion_version.id().canonical(),
        policy_version: policy_version.id().canonical(),
        mapping_id: logical.as_record().id().canonical(),
        draft_id: draft.as_record().id().canonical(),
    }
}

fn backend() -> BackendExecutionContext {
    BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap()
}

fn persist_character(app: &mut Application, name: &str, n: u8) -> CharacterAssetVersion {
    let mut character = CharacterAsset::new(name).unwrap();
    let mut version =
        CharacterAssetVersion::draft(character.id(), format!("{name} v1"), evidence(name, n))
            .unwrap();
    version.publish().unwrap();
    character.bind_published(version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character).unwrap(),
            &Validated::certify(version.clone()).unwrap(),
        )
        .unwrap();
    version
}

fn persist_source(app: &mut Application, name: &str) -> SourceSkeletonReference {
    let source = SourceSkeletonReference::new(name).unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source.clone()).unwrap())
        .unwrap();
    source
}

fn persist_motion(
    app: &mut Application,
    name: &str,
    source_id: SourceSkeletonReferenceId,
    n: u8,
) -> MotionAssetVersion {
    let mut motion = MotionAsset::new(name).unwrap();
    let mut version = MotionAssetVersion::draft(
        motion.id(),
        format!("{name} v1"),
        source_id,
        TimeDomainProvenance::new(
            "clip:walk-carry",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence(name, n),
    )
    .unwrap();
    version.publish().unwrap();
    motion.bind_published(version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(version.clone()).unwrap(),
        )
        .unwrap();
    version
}

fn persist_accepted_mapping(
    app: &mut Application,
    display_name: &str,
    character_version: CharacterAssetVersionId,
    source_skeleton: SourceSkeletonReferenceId,
    producer: BackendExecutionContext,
) -> String {
    let source = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::SourceSkeletonReference,
                None,
                Some(source_skeleton),
                producer.clone(),
                vec![
                    j("root", None, true, "deforming"),
                    j("head", Some("root"), false, "deforming"),
                ],
                vec!["source".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let target = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::CharacterAssetVersion,
                Some(character_version),
                None,
                producer,
                vec![
                    j("Bone", None, true, "deforming"),
                    j("Head", Some("Bone"), false, "deforming"),
                ],
                vec!["target".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            display_name,
            &character_version.canonical(),
            &source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    published.as_record().id().canonical()
}

#[test]
fn workbench_reopen_does_not_bind_mapping_for_other_character() {
    let mut app = Application::open_in_memory().unwrap();
    let producer = backend();
    let alpha = persist_character(&mut app, "Alpha", 20);
    let zulu = persist_character(&mut app, "Zulu", 21);
    let source = persist_source(&mut app, "skel-zulu");
    let _motion = persist_motion(&mut app, "Z-walk", source.id(), 22);
    let other_id = persist_accepted_mapping(
        &mut app,
        "aaa-other-character",
        zulu.id(),
        source.id(),
        producer,
    );
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert_eq!(
        shell.selected_character_version(),
        Some(alpha.id().canonical().as_str())
    );
    assert!(!shell.mapping_accepted());
    assert!(shell.accepted_mapping_version_id().is_none());
    assert_ne!(shell.accepted_mapping_version_id(), Some(other_id.as_str()));
}

#[test]
fn workbench_reopen_does_not_bind_mapping_for_other_source_skeleton() {
    let mut app = Application::open_in_memory().unwrap();
    let producer = backend();
    let character = persist_character(&mut app, "Alpha", 30);
    let source_a = persist_source(&mut app, "skel-a");
    let source_b = persist_source(&mut app, "skel-b");
    let motion_a = persist_motion(&mut app, "A-walk", source_a.id(), 31);
    let _motion_b = persist_motion(&mut app, "B-run", source_b.id(), 32);
    let other_id = persist_accepted_mapping(
        &mut app,
        "map-for-b",
        character.id(),
        source_b.id(),
        producer,
    );
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert_eq!(
        shell.selected_motion_version(),
        Some(motion_a.id().canonical().as_str())
    );
    assert!(!shell.mapping_accepted());
    assert!(shell.accepted_mapping_version_id().is_none());
    assert_ne!(shell.accepted_mapping_version_id(), Some(other_id.as_str()));
}

#[test]
fn workbench_reopen_binds_matching_published_mapping() {
    let mut app = Application::open_in_memory().unwrap();
    let producer = backend();
    let character = persist_character(&mut app, "Alpha", 40);
    let source = persist_source(&mut app, "skel-match");
    let motion = persist_motion(&mut app, "A-walk", source.id(), 41);
    let mapping_id = persist_accepted_mapping(
        &mut app,
        "matching-map",
        character.id(),
        source.id(),
        producer,
    );
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert_eq!(
        shell.selected_character_version(),
        Some(character.id().canonical().as_str())
    );
    assert_eq!(
        shell.selected_motion_version(),
        Some(motion.id().canonical().as_str())
    );
    assert!(shell.mapping_accepted());
    assert_eq!(
        shell.accepted_mapping_version_id(),
        Some(mapping_id.as_str())
    );
}

#[test]
fn multiple_published_mappings_do_not_use_arbitrary_unrelated_first() {
    let mut app = Application::open_in_memory().unwrap();
    let producer = backend();
    let alpha = persist_character(&mut app, "Alpha", 50);
    let zulu = persist_character(&mut app, "Zulu", 51);
    let source_alpha = persist_source(&mut app, "skel-alpha");
    let source_zulu = persist_source(&mut app, "skel-zulu-2");
    let _motion_alpha = persist_motion(&mut app, "A-walk", source_alpha.id(), 52);
    let _motion_zulu = persist_motion(&mut app, "Z-walk", source_zulu.id(), 53);
    let unrelated = persist_accepted_mapping(
        &mut app,
        "aaa-unrelated-first",
        zulu.id(),
        source_zulu.id(),
        producer.clone(),
    );
    let matching = persist_accepted_mapping(
        &mut app,
        "zzz-matching-later",
        alpha.id(),
        source_alpha.id(),
        producer,
    );
    let listed = app.list_mappings().unwrap();
    assert_eq!(listed[0].display_name, "aaa-unrelated-first");
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert_eq!(
        shell.selected_character_version(),
        Some(alpha.id().canonical().as_str())
    );
    assert!(shell.mapping_accepted());
    assert_eq!(
        shell.accepted_mapping_version_id(),
        Some(matching.as_str())
    );
    assert_ne!(
        shell.accepted_mapping_version_id(),
        Some(unrelated.as_str())
    );
}

#[test]
fn transfer_disabled_until_application_authorizes() {
    let shell = WorkbenchApp::empty();
    assert!(!shell.transfer_available());
    assert!(shell
        .transfer_eligibility_label()
        .contains("CompatibilityResult"));
    assert_eq!(
        WorkbenchApp::preview_unavailable_reason(),
        "NOT AVAILABLE — V1-6"
    );
}

#[test]
fn shell_initializes_without_network_or_blender() {
    assert!(!WorkbenchApp::requires_network());
    assert!(!WorkbenchApp::requires_blender());
    assert!(!Application::requires_network());
    assert!(!Application::requires_blender());
    let _options = WorkbenchApp::native_options();
    let slot = PreviewEmbeddingSlot::default();
    assert!(!slot.occupied);
    assert_eq!(PreviewEmbeddingSlot::viewer_library(), None);
    assert_eq!(PreviewEmbeddingSlot::payload_format(), None);
}

#[test]
fn workbench_reads_application_lists_without_blender() {
    let app = Application::open_in_memory().unwrap();
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert!(shell.selected_character_version().is_none());
    assert!(shell.job_states().is_empty());
    assert!(!shell.mapping_accepted());
}

fn write_staged(bytes: &[u8]) -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rf-v15-wb-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("derived_result.blend");
    std::fs::write(&path, bytes).unwrap();
    path
}

fn complete_shell_success(app: &mut Application, run_id: &str, bytes: &[u8]) -> std::path::PathBuf {
    app.mark_dispatchable(run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(run_id, &mut worker).unwrap();
    let running = app.job_status(run_id).unwrap();
    let spec = app.load_job_spec(&running.job_spec_id).unwrap();
    let path = write_staged(bytes);
    let sha = sha256_file(&path).unwrap();
    assert_ne!(
        sha, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "staged fixture must not be empty: {}",
        path.display()
    );
    let result = WorkerResult::new(
        spec.as_record().id(),
        backend(),
        true,
        "completed",
        ExecutionCorrelation::new(
            &running.attempt_id,
            running.worker_execution_ref.as_deref().unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
    .with_staged_artifact_digests(vec![ContentDigest::parse(&sha).unwrap()])
    .unwrap();
    app.complete_success(run_id, &Validated::certify(result).unwrap())
        .unwrap();
    path
}

#[test]
fn transfer_disabled_for_unaccepted_mapping_confirmation() {
    let mut seeded = seed();
    let draft = seeded.app.load_mapping_version(&seeded.draft_id).unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    let result = seeded
        .app
        .run_compatibility_preflight(
            &seeded.character_version,
            &seeded.motion_version,
            &seeded.draft_id,
            &seeded.policy_version,
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    shell.apply_compatibility_result(result.as_record());
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
    assert!(!shell.transfer_available());
}

#[test]
fn transfer_disabled_for_unsupported() {
    let mut seeded = seed();
    let draft = seeded.app.load_mapping_version(&seeded.draft_id).unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    let mapping_version = shell.accepted_mapping_version_id().unwrap();
    let result = CompatibilityResult::from_preflight(
        CharacterAssetVersionId::parse(&seeded.character_version).unwrap(),
        MotionAssetVersionId::parse(&seeded.motion_version).unwrap(),
        BoneMappingVersionId::parse(mapping_version).unwrap(),
        RetargetPolicyVersionId::parse(&seeded.policy_version).unwrap(),
        Judgment::Fail,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["unsupported fixture".into()],
    )
    .unwrap();
    assert_eq!(result.summary(), CompatibilitySummary::Unsupported);
    seeded
        .app
        .catalog_mut()
        .put_validated(&Validated::certify(result.clone()).unwrap())
        .unwrap();
    shell.apply_compatibility_result(&result);
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    assert!(!shell.transfer_available());
    assert!(shell.transfer_eligibility_label().contains("Unsupported"));
}

#[test]
fn ready_with_warnings_requires_explicit_acknowledgement() {
    let mut seeded = seed();
    let draft = seeded.app.load_mapping_version(&seeded.draft_id).unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    let mapping_version = shell.accepted_mapping_version_id().unwrap();
    let result = CompatibilityResult::from_preflight(
        CharacterAssetVersionId::parse(&seeded.character_version).unwrap(),
        MotionAssetVersionId::parse(&seeded.motion_version).unwrap(),
        BoneMappingVersionId::parse(mapping_version).unwrap(),
        RetargetPolicyVersionId::parse(&seeded.policy_version).unwrap(),
        Judgment::PassWithWarnings,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["optional helper remains unmapped".into()],
    )
    .unwrap();
    assert_eq!(result.summary(), CompatibilitySummary::ReadyWithWarnings);
    seeded
        .app
        .catalog_mut()
        .put_validated(&Validated::certify(result.clone()).unwrap())
        .unwrap();
    shell.apply_compatibility_result(&result);
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    assert!(shell.transfer_requires_acknowledgement());
    assert!(!shell.warnings_acknowledged());
    assert!(!shell.transfer_available());
    shell
        .acknowledge_compatibility_warnings(&seeded.app)
        .unwrap();
    assert!(shell.warnings_acknowledged());
    assert!(shell.transfer_available());
}

#[test]
fn transfer_uses_application_authorization_and_displays_publication() {
    let mut seeded = seed();
    let draft = seeded.app.load_mapping_version(&seeded.draft_id).unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    let result = seeded
        .app
        .run_compatibility_preflight(
            &seeded.character_version,
            &seeded.motion_version,
            shell.accepted_mapping_version_id().unwrap(),
            &seeded.policy_version,
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    shell.apply_compatibility_result(result.as_record());
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    if shell.transfer_requires_acknowledgement() {
        assert!(!shell.warnings_acknowledged());
        shell
            .acknowledge_compatibility_warnings(&seeded.app)
            .unwrap();
    }
    assert!(shell.transfer_available());
    let run_id = shell.request_transfer(&mut seeded.app, None).unwrap();
    let staged = complete_shell_success(&mut seeded.app, &run_id, b"wb-published");
    let outcome = seeded
        .app
        .finalize_transfer_for_test(
            &run_id,
            &staged,
            &MemoryArtifactInspector::passing(None).with_joints(vec![
                "Bone".into(),
                "Head".into(),
            ]),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    shell.apply_transfer_outcome(&outcome);
    assert_eq!(
        shell.derived_variant_version_id(),
        Some(outcome.derived_variant_version_id.as_str())
    );
    assert_eq!(shell.publication_state(), Some("Published"));
    assert_eq!(shell.qc_verdict(), Some("Pass"));
    assert!(shell.persistence_verification_id().is_some());
    assert_eq!(
        WorkbenchApp::preview_unavailable_reason(),
        "NOT AVAILABLE — V1-6"
    );
}

#[test]
fn qc_fail_displays_publication_denied() {
    let mut seeded = seed();
    let draft = seeded.app.load_mapping_version(&seeded.draft_id).unwrap();
    let mut shell = WorkbenchApp::empty();
    shell.bind_mapping_draft(&seeded.mapping_id, draft.as_record());
    shell.accept_current_mapping(&mut seeded.app).unwrap();
    let result = seeded
        .app
        .run_compatibility_preflight(
            &seeded.character_version,
            &seeded.motion_version,
            shell.accepted_mapping_version_id().unwrap(),
            &seeded.policy_version,
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    shell.apply_compatibility_result(result.as_record());
    shell.refresh_transfer_authorization(&seeded.app).unwrap();
    if shell.transfer_requires_acknowledgement() {
        shell
            .acknowledge_compatibility_warnings(&seeded.app)
            .unwrap();
    }
    let run_id = shell.request_transfer(&mut seeded.app, None).unwrap();
    let staged = complete_shell_success(&mut seeded.app, &run_id, b"wb-qc-fail");
    let outcome = seeded
        .app
        .finalize_transfer_for_test(
            &run_id,
            &staged,
            &MemoryArtifactInspector::failing_finite_transforms(None).with_joints(vec![
                "Bone".into(),
                "Head".into(),
            ]),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::PublicationDenied);
    shell.apply_transfer_outcome(&outcome);
    assert_eq!(shell.publication_state(), Some("Publication denied"));
    assert_eq!(shell.qc_verdict(), Some("Fail"));
}
