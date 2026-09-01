mod common;

use std::path::Path;

use common::sha256_file;
use rigforge_app::{
    generate_mapping_proposal, Application, MappingAssistProfile, WorkerCapabilityProfile,
};
use rigforge_blender_worker::{enforce_pin, BlenderPin, BlenderSkeletonInspector};
use rigforge_domain::*;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";
const KNIGHT_SHA: &str = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";

fn evidence(path: &str, sha: &str) -> SourceArtifactEvidence {
    let bytes = std::fs::metadata(path).unwrap().len();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(sha).unwrap(),
        bytes,
        "application/octet-stream",
        Some("V1-4 inspect checkpoint".into()),
        None,
    )
    .unwrap()
}

#[test]
fn real_frozen_pair_mapping_and_preflight() {
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(KNIGHT)), KNIGHT_SHA);
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);

    let mut app = Application::open_in_memory().unwrap();
    let mut character = CharacterAsset::new("Knight_Male").unwrap();
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        "Knight_Male frozen FBX",
        evidence(KNIGHT, KNIGHT_SHA),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2_Standard").unwrap();
    let mut motion = MotionAsset::new("UAL2_Standard").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "UAL2 Walk_Carry_Loop",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            CLIP,
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence(UAL2, UAL2_SHA),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());

    let mut policy = RetargetPolicy::new("rest-relative-proven").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());

    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character.clone()).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion.clone()).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy.clone()).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();

    let inspector = BlenderSkeletonInspector::production().unwrap();
    let target_summary = app
        .inspect_and_store_character_summary(&character_version.id().canonical(), &inspector)
        .unwrap();
    let source_summary = app
        .inspect_and_store_source_summary(
            &source_skeleton.id().canonical(),
            &inspector,
            &motion_version.id().canonical(),
        )
        .unwrap();

    assert_eq!(
        target_summary.as_record().subject_character_version_id(),
        Some(character_version.id())
    );
    assert_eq!(
        source_summary.as_record().subject_source_skeleton_ref_id(),
        Some(source_skeleton.id())
    );
    assert!(!target_summary.as_record().joints().is_empty());
    assert!(!source_summary.as_record().joints().is_empty());

    let proposal = generate_mapping_proposal(
        source_summary.as_record(),
        target_summary.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    assert!(!proposal.entries.is_empty());

    let (logical, draft) = app
        .store_mapping_draft(
            "knight-ual2-v14",
            &character_version.id().canonical(),
            &source_skeleton.id().canonical(),
            &proposal,
            &source_summary.as_record().id().canonical(),
            &target_summary.as_record().id().canonical(),
        )
        .unwrap();
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    assert_eq!(published.as_record().lifecycle(), Lifecycle::Published);
    assert!(published.as_record().review().reviewed());
    assert_eq!(
        published.as_record().review().review_kind(),
        Some(MappingReviewKind::AutomaticConfirmed)
    );
    assert!(published.as_record().generated_from_candidates());
    assert!(!published.as_record().user_modified());
    assert_eq!(
        published.as_record().review().review_kind(),
        Some(MappingReviewKind::AutomaticConfirmed)
    );
    assert!(published.as_record().review().reviewed());
    assert!(published.as_record().generated_from_candidates());
    assert!(!published.as_record().user_modified());

    let result = app
        .run_compatibility_preflight(
            &character_version.id().canonical(),
            &motion_version.id().canonical(),
            &published.as_record().id().canonical(),
            &policy_version.id().canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();

    let mapped_pairs: Vec<_> = published
        .as_record()
        .entries()
        .iter()
        .map(|e| {
            serde_json::json!({
                "source": e.source().joint_key().as_str(),
                "target": e.target().joint_key().as_str(),
                "participation": format!("{:?}", e.participation()),
                "evidence": e.evidence()
            })
        })
        .collect();
    let unmapped_source: Vec<_> = published
        .as_record()
        .unmapped_source()
        .iter()
        .map(|u| {
            serde_json::json!({
                "joint": u.joint_key().as_str(),
                "reason": u.reason(),
                "disposition": format!("{:?}", u.disposition())
            })
        })
        .collect();
    let unmapped_target: Vec<_> = published
        .as_record()
        .unmapped_target()
        .iter()
        .map(|u| {
            serde_json::json!({
                "joint": u.joint_key().as_str(),
                "reason": u.reason(),
                "disposition": format!("{:?}", u.disposition())
            })
        })
        .collect();
    let evidence_path = std::env::temp_dir().join("rigforge_v14_real_mapping_checkpoint.json");
    let payload = serde_json::json!({
        "character_version_id": character_version.id().canonical(),
        "motion_version_id": motion_version.id().canonical(),
        "character_sha256": KNIGHT_SHA,
        "motion_sha256": UAL2_SHA,
        "clip": CLIP,
        "source_summary_id": source_summary.as_record().id().canonical(),
        "target_summary_id": target_summary.as_record().id().canonical(),
        "source_joint_count": source_summary.as_record().joints().len(),
        "target_joint_count": target_summary.as_record().joints().len(),
        "mapping_id": logical.as_record().id().canonical(),
        "mapping_version_id": published.as_record().id().canonical(),
        "review_kind": published.as_record().review().review_kind().map(|k| format!("{k:?}")),
        "review_reviewed": published.as_record().review().reviewed(),
        "generated_from_candidates": published.as_record().generated_from_candidates(),
        "user_modified": published.as_record().user_modified(),
        "review_method": published.as_record().review().method(),
        "ambiguities": published.as_record().review().ambiguities(),
        "unmapped_source_count": published.as_record().unmapped_source().len(),
        "unmapped_target_count": published.as_record().unmapped_target().len(),
        "unmapped_source": unmapped_source,
        "unmapped_target": unmapped_target,
        "mapped_pairs": mapped_pairs,
        "proposal_confirmation_required": proposal.confirmation_required,
        "mapped_entries": published.as_record().entries().len(),
        "policy_version_id": policy_version.id().canonical(),
        "compatibility_id": result.as_record().id().canonical(),
        "mapping_completeness": format!("{:?}", result.as_record().mapping_completeness()),
        "structural_compatibility": format!("{:?}", result.as_record().structural_compatibility()),
        "method_eligibility": format!("{:?}", result.as_record().method_eligibility()),
        "motion_suitability": format!("{:?}", result.as_record().motion_suitability()),
        "result_acceptability": format!("{:?}", result.as_record().result_acceptability()),
        "summary": format!("{:?}", result.as_record().summary()),
        "notes": result.as_record().notes(),
        "blender": "5.2.1 LTS",
        "build": "9e2066aef7ef",
        "transfer_executed": false,
        "worker_result_created": false,
        "derived_variant_created": false,
        "product_qc_run": false
    });
    std::fs::write(&evidence_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    assert_eq!(result.as_record().result_acceptability(), Judgment::Unknown);
}

#[test]
fn wrong_source_skeleton_is_unsupported() {
    let mut app = Application::open_in_memory().unwrap();
    let g = rigforge_domain::SourceSkeletonReference::new("other").unwrap();
    let mut character = CharacterAsset::new("c").unwrap();
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        "c",
        SourceArtifactEvidence::new(
            LocationEvidence::filesystem_path("C:/research/c.bin").unwrap(),
            ContentDigest::parse(&"a".repeat(64)).unwrap(),
            1,
            "application/octet-stream",
            None,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source = SourceSkeletonReference::new("src").unwrap();
    let mut motion = MotionAsset::new("m").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "m",
        g.id(),
        TimeDomainProvenance::new(
            "clip",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(2, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        SourceArtifactEvidence::new(
            LocationEvidence::filesystem_path("C:/research/m.bin").unwrap(),
            ContentDigest::parse(&"b".repeat(64)).unwrap(),
            1,
            "application/octet-stream",
            None,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    let mut mapping = BoneMapping::new("m").unwrap();
    let mut mapping_version = BoneMappingVersion::draft(
        mapping.id(),
        character_version.id(),
        source.id(),
        vec![BoneMappingEntry::new(
            JointRef::source(JointKey::new("root").unwrap()),
            JointRef::target(JointKey::new("Bone").unwrap()),
            JointParticipation::Required,
            None,
            "manual",
        )
        .unwrap()],
        MappingReviewProvenance::with_kind(
            true,
            Some("manual".into()),
            vec![],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    mapping_version.publish().unwrap();
    mapping.bind_published(mapping_version.id());
    let mut policy = RetargetPolicy::new("p").unwrap();
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
        .put_validated(&Validated::certify(source).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(g).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(mapping).unwrap(),
            &Validated::certify(mapping_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();

    let result = app
        .run_compatibility_preflight(
            &character_version.id().canonical(),
            &motion_version.id().canonical(),
            &mapping_version.id().canonical(),
            &policy_version.id().canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    assert_eq!(result.as_record().motion_suitability(), Judgment::Fail);
    assert_eq!(result.as_record().summary(), CompatibilitySummary::Unsupported);
}
