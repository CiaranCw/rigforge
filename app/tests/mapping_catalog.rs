mod common;

use common::{certify, mapping_entry, source, unpublished_graph};
use rigforge_app::{
    evaluate_compatibility, generate_mapping_proposal, Application, MappingAssistProfile,
    WorkerCapabilityProfile,
};
use rigforge_domain::*;

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

fn seed_pair(app: &mut Application) -> UnpublishedGraphIds {
    let g = unpublished_graph();
    app.catalog_mut()
        .put_validated_pair(&certify(g.character.clone()), &certify(g.character_version.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(&certify(g.motion.clone()), &certify(g.motion_version.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(&certify(g.policy.clone()), &certify(g.policy_version.clone()))
        .unwrap();
    UnpublishedGraphIds {
        character_version: g.character_version.id(),
        motion_version: g.motion_version.id(),
        source_skeleton: g.source_skeleton.id(),
        policy_version: g.policy_version.id(),
        producer: g.backend.clone(),
    }
}

struct UnpublishedGraphIds {
    character_version: CharacterAssetVersionId,
    motion_version: MotionAssetVersionId,
    source_skeleton: SourceSkeletonReferenceId,
    policy_version: RetargetPolicyVersionId,
    producer: BackendExecutionContext,
}

fn pair_summaries(ids: &UnpublishedGraphIds) -> (SkeletonSummary, SkeletonSummary) {
    let source = SkeletonSummary::new(
        SkeletonSubjectKind::SourceSkeletonReference,
        None,
        Some(ids.source_skeleton),
        ids.producer.clone(),
        vec![
            j("root", None, true, "deforming"),
            j("head", Some("root"), false, "deforming"),
        ],
        vec!["source fixture".into()],
    )
    .unwrap();
    let target = SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(ids.character_version),
        None,
        ids.producer.clone(),
        vec![
            j("Bone", None, true, "deforming"),
            j("Head", Some("Bone"), false, "deforming"),
        ],
        vec!["target fixture".into()],
    )
    .unwrap();
    (source, target)
}

#[test]
fn store_load_skeleton_summary_exact_subject() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let stored = app.store_skeleton_summary(target.clone()).unwrap();
    let loaded = app
        .load_skeleton_summary(&stored.as_record().id().canonical())
        .unwrap();
    assert_eq!(
        loaded.as_record().subject_character_version_id(),
        Some(ids.character_version)
    );
    let stored_s = app.store_skeleton_summary(source).unwrap();
    assert_eq!(
        stored_s.as_record().subject_source_skeleton_ref_id(),
        Some(ids.source_skeleton)
    );
}

#[test]
fn store_publish_mapping_and_forbid_overwrite() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "fixture-map",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    assert_eq!(draft.as_record().lifecycle(), Lifecycle::Draft);
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    assert_eq!(published.as_record().lifecycle(), Lifecycle::Published);
    let err = app
        .override_mapping_entry(
            &published.as_record().id().canonical(),
            "head",
            Some("Head"),
            "should fail",
        )
        .unwrap_err();
    match err {
        rigforge_app::AppError::Domain(d) => assert_eq!(d.code, ErrorCode::ImmutableVersion),
        other => panic!("{other}"),
    }
}

#[test]
fn manual_override_persisted_on_draft() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (_logical, draft) = app
        .store_mapping_draft(
            "override-map",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    let edited = app
        .override_mapping_entry(
            &draft.as_record().id().canonical(),
            "head",
            Some("Head"),
            "user chose Head",
        )
        .unwrap();
    assert!(edited
        .as_record()
        .entries()
        .iter()
        .any(|e| e.evidence().contains("user override")));
}

#[test]
fn compatibility_exact_version_and_reopen() {
    let dir = std::env::temp_dir().join(format!("rf-v14-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("catalog.sqlite");
    let mut app = Application::open(&path).unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "compat-map",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
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
    let result = app
        .run_compatibility_preflight(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &published.as_record().id().canonical(),
            &ids.policy_version.canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    drop(app);
    let app = Application::open(&path).unwrap();
    let loaded = app
        .load_compatibility_result(&result.as_record().id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().character_version_id(), ids.character_version);
    assert_eq!(loaded.as_record().mapping_version_id(), published.as_record().id());
    assert_eq!(loaded.as_record().result_acceptability(), Judgment::Unknown);
}

#[test]
fn changed_mapping_or_motion_does_not_reuse_old_preflight() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "reuse-map",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
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
    let first = app
        .run_compatibility_preflight(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &published.as_record().id().canonical(),
            &ids.policy_version.canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    let other_mapping = BoneMappingVersionId::generate();
    let reused = app
        .latest_compatibility_for_exact_set(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &other_mapping.canonical(),
            &ids.policy_version.canonical(),
        )
        .unwrap();
    assert!(reused.is_none());
    let same = app
        .latest_compatibility_for_exact_set(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &published.as_record().id().canonical(),
            &ids.policy_version.canonical(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(same.as_record().id(), first.as_record().id());
}

#[test]
fn mapping_reusable_across_motions_of_same_source_skeleton() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "shared-skel-map",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    assert!(draft.as_record().source_motion_version_id().is_none());
    let _ = logical;
}

#[test]
fn unconfirmed_mapping_is_confirmation_required() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (_logical, draft) = app
        .store_mapping_draft(
            "unconfirmed",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    let result = app
        .run_compatibility_preflight(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &draft.as_record().id().canonical(),
            &ids.policy_version.canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
}

#[test]
fn unsupported_method_when_worker_lacks_capability() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "cap-fail",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
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
    let mut capability = WorkerCapabilityProfile::v1_3_isolated_worker();
    capability.proven_rest_relative_policy = false;
    let result = app
        .run_compatibility_preflight(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &published.as_record().id().canonical(),
            &ids.policy_version.canonical(),
            &capability,
        )
        .unwrap();
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Unsupported);
    assert_eq!(result.as_record().method_eligibility(), Judgment::Fail);
}

fn published_mapping_with_unmapped(
    ids: &UnpublishedGraphIds,
    disposition: UnmappedDisposition,
    reason: &str,
) -> BoneMappingVersion {
    let mapping = BoneMapping::new("typed-unmap").unwrap();
    let mut version = BoneMappingVersion::draft(
        mapping.id(),
        ids.character_version,
        ids.source_skeleton,
        vec![mapping_entry("root", "Bone"), mapping_entry("head", "Head")],
        MappingReviewProvenance::with_kind(
            true,
            Some("manual review".into()),
            vec![],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    version
        .set_unmapped(
            vec![UnmappedJoint::new(
                SkeletonSide::Source,
                JointKey::new("spare").unwrap(),
                reason,
                disposition,
            )
            .unwrap()],
            vec![],
        )
        .unwrap();
    version.publish().unwrap();
    version
}

fn completeness_for(
    app: &Application,
    ids: &UnpublishedGraphIds,
    mapping: &BoneMappingVersion,
    source: &SkeletonSummary,
    target: &SkeletonSummary,
) -> Judgment {
    let character = app
        .catalog()
        .load_character_version(&ids.character_version.canonical())
        .unwrap();
    let motion = app
        .catalog()
        .load_motion_version(&ids.motion_version.canonical())
        .unwrap();
    let skeleton = app
        .catalog()
        .load_source_skeleton(&ids.source_skeleton.canonical())
        .unwrap();
    let policy = app
        .catalog()
        .load_policy_version(&ids.policy_version.canonical())
        .unwrap();
    evaluate_compatibility(
        character.as_record(),
        motion.as_record(),
        skeleton.as_record(),
        mapping,
        policy.as_record(),
        Some(source),
        Some(target),
        &WorkerCapabilityProfile::v1_3_isolated_worker(),
    )
    .unwrap()
    .mapping_completeness()
}

#[test]
fn automatic_unedited_explicit_accept_becomes_automatic_confirmed() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "auto-confirm",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
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
    assert_eq!(
        published.as_record().review().review_kind(),
        Some(MappingReviewKind::AutomaticConfirmed)
    );
}

#[test]
fn edited_explicit_accept_becomes_user_override() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "user-override",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    app.override_mapping_entry(
        &draft.as_record().id().canonical(),
        "head",
        Some("Head"),
        "reaffirm Head",
    )
    .unwrap();
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    assert_eq!(
        published.as_record().review().review_kind(),
        Some(MappingReviewKind::UserOverride)
    );
}

#[test]
fn required_unmapped_with_reason_containing_optional_still_fails() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let mapping = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Blocking,
        "this is optional and looks helper",
    );
    let (source, target) = pair_summaries(&ids);
    let judgment = completeness_for(&app, &ids, &mapping, &source, &target);
    assert_eq!(judgment, Judgment::Fail);
}

#[test]
fn required_unmapped_with_reason_containing_helper_still_fails() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let mapping = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Blocking,
        "helper control leftover",
    );
    let (source, target) = pair_summaries(&ids);
    let judgment = completeness_for(&app, &ids, &mapping, &source, &target);
    assert_eq!(judgment, Judgment::Fail);
}

#[test]
fn typed_optional_unmapped_is_non_blocking() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let mapping = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Optional,
        "required-looking reason text must not matter",
    );
    let (source, target) = pair_summaries(&ids);
    let judgment = completeness_for(&app, &ids, &mapping, &source, &target);
    assert_ne!(judgment, Judgment::Fail);
}

#[test]
fn typed_helper_unmapped_is_non_blocking() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let mapping = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Helper,
        "required source joint leftover",
    );
    let (source, target) = pair_summaries(&ids);
    let judgment = completeness_for(&app, &ids, &mapping, &source, &target);
    assert_ne!(judgment, Judgment::Fail);
}

#[test]
fn manual_reason_does_not_change_participation() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "reason-trap",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    let edited = app
        .override_mapping_entry(
            &draft.as_record().id().canonical(),
            "head",
            None,
            "optional helper",
        )
        .unwrap();
    assert_eq!(
        edited
            .as_record()
            .unmapped_source()
            .iter()
            .find(|u| u.joint_key().as_str() == "head")
            .unwrap()
            .disposition(),
        UnmappedDisposition::Blocking
    );
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &edited.as_record().id().canonical(),
        )
        .unwrap();
    let result = app
        .run_compatibility_preflight(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &published.as_record().id().canonical(),
            &ids.policy_version.canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    assert_eq!(result.as_record().mapping_completeness(), Judgment::Fail);
}

#[test]
fn mapping_completeness_uses_typed_unmapped_semantics() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let blocking = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Blocking,
        "optional",
    );
    let optional = published_mapping_with_unmapped(
        &ids,
        UnmappedDisposition::Optional,
        "required",
    );
    let (source, target) = pair_summaries(&ids);
    assert_eq!(
        completeness_for(&app, &ids, &blocking, &source, &target),
        Judgment::Fail
    );
    assert_ne!(
        completeness_for(&app, &ids, &optional, &source, &target),
        Judgment::Fail
    );
}

#[test]
fn mapping_draft_rejects_source_summary_from_other_skeleton() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let other = SourceSkeletonReference::new("other-skel").unwrap();
    let joints = source.joints().to_vec();
    let source = SkeletonSummary::new(
        SkeletonSubjectKind::SourceSkeletonReference,
        None,
        Some(other.id()),
        ids.producer.clone(),
        joints,
        vec!["wrong source".into()],
    )
    .unwrap();
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let err = app
        .store_mapping_draft(
            "wrong-src",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap_err();
    match err {
        rigforge_app::AppError::Catalog(msg) => assert!(msg.contains("source SkeletonSummary")),
        other => panic!("{other}"),
    }
}

#[test]
fn mapping_draft_rejects_target_summary_from_other_character_version() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let joints = target.joints().to_vec();
    let target = SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(CharacterAssetVersionId::generate()),
        None,
        ids.producer.clone(),
        joints,
        vec!["wrong target".into()],
    )
    .unwrap();
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let err = app
        .store_mapping_draft(
            "wrong-tgt",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap_err();
    match err {
        rigforge_app::AppError::Catalog(msg) => assert!(msg.contains("target SkeletonSummary")),
        other => panic!("{other}"),
    }
}

#[test]
fn mapping_draft_rejects_reversed_summary_sides() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let err = app
        .propose_mapping(
            &target.as_record().id().canonical(),
            &source.as_record().id().canonical(),
            MappingAssistProfile::None,
        )
        .unwrap_err();
    match err {
        rigforge_app::AppError::Catalog(msg) => {
            assert!(
                msg.contains("propose_mapping")
                    || msg.contains("subject")
                    || msg.contains("sides"),
                "{msg}"
            );
        }
        other => panic!("{other}"),
    }
}

#[test]
fn failed_summary_binding_persists_no_mapping_records() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let before = app.list_mappings().unwrap().len();
    let (source, target) = pair_summaries(&ids);
    let joints = target.joints().to_vec();
    let wrong = SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(CharacterAssetVersionId::generate()),
        None,
        ids.producer.clone(),
        joints,
        vec!["wrong target".into()],
    )
    .unwrap();
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(wrong).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    assert!(app
        .store_mapping_draft(
            "no-persist",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .is_err());
    assert_eq!(app.list_mappings().unwrap().len(), before);
}

#[test]
fn correct_exact_summary_binding_succeeds() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (_logical, draft) = app
        .store_mapping_draft(
            "bound",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    assert_eq!(
        draft.as_record().source_skeleton_summary_id(),
        Some(source.as_record().id())
    );
    assert_eq!(
        draft.as_record().target_skeleton_summary_id(),
        Some(target.as_record().id())
    );
}

fn persist_reviewed_manual_mapping(
    app: &mut Application,
    ids: &UnpublishedGraphIds,
    source: &Validated<SkeletonSummary>,
    target: &Validated<SkeletonSummary>,
    publish: bool,
) -> (Validated<BoneMapping>, Validated<BoneMappingVersion>) {
    let mut logical = BoneMapping::new("manual-reviewed").unwrap();
    let mut version = BoneMappingVersion::draft(
        logical.id(),
        ids.character_version,
        ids.source_skeleton,
        vec![mapping_entry("root", "Bone"), mapping_entry("head", "Head")],
        MappingReviewProvenance::with_kind(
            true,
            Some("manual review".into()),
            vec![],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    version
        .bind_skeleton_summaries(source.as_record().id(), target.as_record().id())
        .unwrap();
    if publish {
        version.publish().unwrap();
        logical.bind_published(version.id());
    } else {
        logical.bind_draft(version.id());
    }
    let logical = Validated::certify(logical).unwrap();
    let version = Validated::certify(version).unwrap();
    app.catalog_mut()
        .put_validated_pair(&logical, &version)
        .unwrap();
    (logical, version)
}

fn preflight(
    app: &mut Application,
    ids: &UnpublishedGraphIds,
    mapping_version_id: &str,
) -> Validated<CompatibilityResult> {
    app.run_compatibility_preflight(
        &ids.character_version.canonical(),
        &ids.motion_version.canonical(),
        mapping_version_id,
        &ids.policy_version.canonical(),
        &WorkerCapabilityProfile::v1_3_isolated_worker(),
    )
    .unwrap()
}

#[test]
fn reviewed_manual_draft_cannot_be_ready() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let (_logical, draft) = persist_reviewed_manual_mapping(&mut app, &ids, &source, &target, false);
    let result = preflight(&mut app, &ids, &draft.as_record().id().canonical());
    assert_ne!(result.as_record().summary(), CompatibilitySummary::Ready);
    assert_ne!(
        result.as_record().summary(),
        CompatibilitySummary::ReadyWithWarnings
    );
}

#[test]
fn reviewed_manual_draft_requires_mapping_confirmation() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let (_logical, draft) = persist_reviewed_manual_mapping(&mut app, &ids, &source, &target, false);
    let result = preflight(&mut app, &ids, &draft.as_record().id().canonical());
    assert_eq!(result.as_record().mapping_completeness(), Judgment::Unknown);
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
}

#[test]
fn automatic_reviewed_draft_cannot_be_ready() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (_logical, draft) = app
        .store_mapping_draft(
            "auto-reviewed-draft",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
            &proposal,
            &source.as_record().id().canonical(),
            &target.as_record().id().canonical(),
        )
        .unwrap();
    let mut version = draft.into_record();
    version
        .replace_entries_and_review(
            version.entries().to_vec(),
            MappingReviewProvenance::with_kind(
                true,
                Some("explicit Product acceptance".into()),
                vec![],
                MappingReviewKind::AutomaticConfirmed,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(version.lifecycle(), Lifecycle::Draft);
    let version = Validated::certify(version).unwrap();
    app.catalog_mut().put_validated(&version).unwrap();
    let result = preflight(&mut app, &ids, &version.as_record().id().canonical());
    assert_ne!(result.as_record().summary(), CompatibilitySummary::Ready);
    assert_ne!(
        result.as_record().summary(),
        CompatibilitySummary::ReadyWithWarnings
    );
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
}

#[test]
fn published_manual_mapping_can_be_ready() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let (_logical, published) =
        persist_reviewed_manual_mapping(&mut app, &ids, &source, &target, true);
    let result = preflight(&mut app, &ids, &published.as_record().id().canonical());
    assert_eq!(published.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Ready);
}

#[test]
fn published_automatic_confirmed_mapping_can_be_ready() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "auto-confirmed-ready",
            &ids.character_version.canonical(),
            &ids.source_skeleton.canonical(),
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
    assert_eq!(
        published.as_record().review().review_kind(),
        Some(MappingReviewKind::AutomaticConfirmed)
    );
    let result = preflight(&mut app, &ids, &published.as_record().id().canonical());
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Ready);
}

#[test]
fn draft_mapping_preflight_does_not_mutate_or_publish_mapping() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let (_logical, draft) = persist_reviewed_manual_mapping(&mut app, &ids, &source, &target, false);
    let before_id = draft.as_record().id();
    let _ = preflight(&mut app, &ids, &draft.as_record().id().canonical());
    let loaded = app
        .load_mapping_version(&before_id.canonical())
        .unwrap();
    assert_eq!(loaded.as_record().lifecycle(), Lifecycle::Draft);
    assert_eq!(
        loaded.as_record().review().review_kind(),
        Some(MappingReviewKind::Manual)
    );
    assert!(loaded.as_record().review().reviewed());
}

#[test]
fn override_optional_entry_preserves_optional_participation() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let mut logical = BoneMapping::new("optional-remap").unwrap();
    let mut version = BoneMappingVersion::draft(
        logical.id(),
        ids.character_version,
        ids.source_skeleton,
        vec![
            mapping_entry("root", "Bone"),
            BoneMappingEntry::new(
                JointRef::source(JointKey::new("head").unwrap()),
                JointRef::target(JointKey::new("Head").unwrap()),
                JointParticipation::Optional,
                None,
                "optional head correspondence",
            )
            .unwrap(),
        ],
        MappingReviewProvenance::with_kind(
            false,
            Some("deterministic candidate generation".into()),
            vec![],
            MappingReviewKind::AutomaticCandidate,
        )
        .unwrap(),
    )
    .unwrap();
    version.mark_generated_from_candidates().unwrap();
    version
        .bind_skeleton_summaries(source.as_record().id(), target.as_record().id())
        .unwrap();
    logical.bind_draft(version.id());
    let logical = Validated::certify(logical).unwrap();
    let version = Validated::certify(version).unwrap();
    app.catalog_mut()
        .put_validated_pair(&logical, &version)
        .unwrap();
    let edited = app
        .override_mapping_entry(
            &version.as_record().id().canonical(),
            "head",
            Some("Head"),
            "user remapped optional head",
        )
        .unwrap();
    let head = edited
        .as_record()
        .entries()
        .iter()
        .find(|e| e.source().joint_key().as_str() == "head")
        .unwrap();
    assert_eq!(head.participation(), JointParticipation::Optional);
    assert_eq!(head.target().joint_key().as_str(), "Head");
}

fn persist_mapping(
    app: &mut Application,
    name: &str,
    character_version: CharacterAssetVersionId,
    source_skeleton: SourceSkeletonReferenceId,
    publish: bool,
) -> Validated<BoneMappingVersion> {
    let mut logical = BoneMapping::new(name).unwrap();
    let mut version = BoneMappingVersion::draft(
        logical.id(),
        character_version,
        source_skeleton,
        vec![mapping_entry("root", "Bone"), mapping_entry("head", "Head")],
        MappingReviewProvenance::with_kind(
            true,
            Some("manual review".into()),
            vec![],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    if publish {
        version.publish().unwrap();
        logical.bind_published(version.id());
    } else {
        logical.bind_draft(version.id());
    }
    let logical = Validated::certify(logical).unwrap();
    let version = Validated::certify(version).unwrap();
    app.catalog_mut()
        .put_validated_pair(&logical, &version)
        .unwrap();
    version
}

fn result_for(
    ids: &UnpublishedGraphIds,
    mapping_id: BoneMappingVersionId,
    completeness: Judgment,
    structural: Judgment,
) -> Validated<CompatibilityResult> {
    Validated::certify(
        CompatibilityResult::from_preflight(
            ids.character_version,
            ids.motion_version,
            mapping_id,
            ids.policy_version,
            completeness,
            structural,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Unknown,
            vec!["catalog graph fixture".into()],
        )
        .unwrap(),
    )
    .unwrap()
}

fn persist_rejected(err: rigforge_app::AppError) -> String {
    match err {
        rigforge_app::AppError::Catalog(msg) => msg,
        rigforge_app::AppError::NotFound { what, id } => format!("{what} not found: {id}"),
        other => panic!("unexpected persist error: {other}"),
    }
}

#[test]
fn ready_compatibility_bound_to_draft_mapping_rejected_on_persist() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let draft = persist_mapping(
        &mut app,
        "draft-ready",
        ids.character_version,
        ids.source_skeleton,
        false,
    );
    let result = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Ready);
    let err = app.catalog_mut().put_validated(&result).unwrap_err();
    let msg = persist_rejected(err);
    assert!(msg.contains("Published"), "{msg}");
    let latest = app
        .latest_compatibility_for_exact_set(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &draft.as_record().id().canonical(),
            &ids.policy_version.canonical(),
        )
        .unwrap();
    assert!(latest.is_none());
}

#[test]
fn ready_with_warnings_bound_to_draft_mapping_rejected_on_persist() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let draft = persist_mapping(
        &mut app,
        "draft-warn",
        ids.character_version,
        ids.source_skeleton,
        false,
    );
    let result = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Pass,
        Judgment::PassWithWarnings,
    );
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::ReadyWithWarnings
    );
    let err = app.catalog_mut().put_validated(&result).unwrap_err();
    let msg = persist_rejected(err);
    assert!(msg.contains("Published"), "{msg}");
}

#[test]
fn untrusted_validated_ready_result_for_draft_mapping_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let draft = persist_mapping(
        &mut app,
        "untrusted-draft",
        ids.character_version,
        ids.source_skeleton,
        false,
    );
    let result = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    let json = to_json(result.as_record()).unwrap();
    let ingested = ingest_validated::<CompatibilityResult>(&json).unwrap();
    assert_eq!(ingested.as_record().summary(), CompatibilitySummary::Ready);
    let err = app.catalog_mut().put_validated(&ingested).unwrap_err();
    let msg = persist_rejected(err);
    assert!(msg.contains("Published"), "{msg}");
}

#[test]
fn ready_result_with_mapping_for_other_character_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let mut other = CharacterAsset::new("Other").unwrap();
    let mut other_version =
        CharacterAssetVersion::draft(other.id(), "Other v1", source("other-char", 7)).unwrap();
    other_version.publish().unwrap();
    other.bind_published(other_version.id());
    app.catalog_mut()
        .put_validated_pair(&certify(other), &certify(other_version.clone()))
        .unwrap();
    let mapping = persist_mapping(
        &mut app,
        "other-char-map",
        other_version.id(),
        ids.source_skeleton,
        true,
    );
    let result = result_for(
        &ids,
        mapping.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    let err = app.catalog_mut().put_validated(&result).unwrap_err();
    let msg = persist_rejected(err);
    assert!(msg.contains("target_character_version_id"), "{msg}");
}

#[test]
fn ready_result_with_mapping_for_other_source_skeleton_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let other_source = SourceSkeletonReference::new("other-source").unwrap();
    app.catalog_mut()
        .put_validated(&certify(other_source.clone()))
        .unwrap();
    let mapping = persist_mapping(
        &mut app,
        "other-skel-map",
        ids.character_version,
        other_source.id(),
        true,
    );
    let result = result_for(
        &ids,
        mapping.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    let err = app.catalog_mut().put_validated(&result).unwrap_err();
    let msg = persist_rejected(err);
    assert!(
        msg.contains("SourceSkeletonReference") || msg.contains("Published"),
        "{msg}"
    );
}

#[test]
fn compatibility_result_with_missing_exact_mapping_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let missing = BoneMappingVersionId::generate();
    let result = result_for(&ids, missing, Judgment::Pass, Judgment::Pass);
    let err = app.catalog_mut().put_validated(&result).unwrap_err();
    let msg = persist_rejected(err);
    assert!(msg.contains("not found"), "{msg}");
}

#[test]
fn published_mapping_ready_result_persists() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let published = persist_mapping(
        &mut app,
        "published-ready",
        ids.character_version,
        ids.source_skeleton,
        true,
    );
    let result = result_for(
        &ids,
        published.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Ready);
    app.catalog_mut().put_validated(&result).unwrap();
    let loaded = app
        .load_compatibility_result(&result.as_record().id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().summary(), CompatibilitySummary::Ready);
}

#[test]
fn draft_mapping_confirmation_required_result_persists() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let draft = persist_mapping(
        &mut app,
        "draft-confirm",
        ids.character_version,
        ids.source_skeleton,
        false,
    );
    let result = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Unknown,
        Judgment::Pass,
    );
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
    app.catalog_mut().put_validated(&result).unwrap();
    let loaded = app
        .load_compatibility_result(&result.as_record().id().canonical())
        .unwrap();
    assert_eq!(
        loaded.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
}

#[test]
fn normal_run_compatibility_preflight_still_persists() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let (source, target) = pair_summaries(&ids);
    let source = app.store_skeleton_summary(source).unwrap();
    let target = app.store_skeleton_summary(target).unwrap();
    let (_logical, published) =
        persist_reviewed_manual_mapping(&mut app, &ids, &source, &target, true);
    let result = preflight(&mut app, &ids, &published.as_record().id().canonical());
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Ready);
    let loaded = app
        .load_compatibility_result(&result.as_record().id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().id(), result.as_record().id());
}

#[test]
fn latest_compatibility_for_exact_set_cannot_return_forged_draft_ready() {
    let mut app = Application::open_in_memory().unwrap();
    let ids = seed_pair(&mut app);
    let draft = persist_mapping(
        &mut app,
        "forged-ready",
        ids.character_version,
        ids.source_skeleton,
        false,
    );
    let forged = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Pass,
        Judgment::Pass,
    );
    assert!(app.catalog_mut().put_validated(&forged).is_err());
    let diagnostic = result_for(
        &ids,
        draft.as_record().id(),
        Judgment::Unknown,
        Judgment::Pass,
    );
    app.catalog_mut().put_validated(&diagnostic).unwrap();
    let latest = app
        .latest_compatibility_for_exact_set(
            &ids.character_version.canonical(),
            &ids.motion_version.canonical(),
            &draft.as_record().id().canonical(),
            &ids.policy_version.canonical(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        latest.as_record().summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
    assert_ne!(latest.as_record().summary(), CompatibilitySummary::Ready);
}

