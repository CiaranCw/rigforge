mod common;

use common::*;
use rigforge_domain::*;

#[test]
fn product_id_is_not_path() {
    let err = CharacterAssetId::parse(r"F:\assets\hero.fbx").unwrap_err();
    assert_eq!(err.code, ErrorCode::PathUsedAsId);
    let err = MotionAssetVersionId::parse("/tmp/clip.fbx").unwrap_err();
    assert_eq!(err.code, ErrorCode::PathUsedAsId);
}

#[test]
fn product_id_is_not_artifact_hash() {
    let hex = "ab".repeat(32);
    let err = DerivedVariantVersionId::parse(&hex).unwrap_err();
    assert_eq!(err.code, ErrorCode::DigestUsedAsId);
    let digest = ContentDigest::parse(&hex).unwrap();
    assert_ne!(
        digest.sha256().len(),
        CharacterAssetId::generate().canonical().len()
    );
}

#[test]
fn exact_version_references_required() {
    let g = valid_graph();
    g.job.validate().unwrap();
    assert_eq!(g.job.character_version_id(), g.character_version.id());
    assert_eq!(g.job.motion_version_id(), g.motion_version.id());
    assert_eq!(g.job.mapping_version_id(), g.mapping_version.id());
    assert_eq!(g.job.policy_version_id(), g.policy_version.id());
}

#[test]
fn published_versions_are_immutable() {
    let mut g = valid_graph();
    assert_eq!(g.character_version.lifecycle(), Lifecycle::Published);
    let err = g
        .character_version
        .try_set_display_name("mutated")
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
}

#[test]
fn draft_may_be_replaced_in_place() {
    let character = CharacterAsset::new("Hero").unwrap();
    let mut version =
        CharacterAssetVersion::draft(character.id(), "Hero draft", source("hero", 5)).unwrap();
    version.try_set_display_name("Hero draft 2").unwrap();
    assert_eq!(version.display_name(), "Hero draft 2");
}

#[test]
fn derived_variant_version_references_exact_inputs() {
    let g = valid_graph();
    g.derived_version.validate().unwrap();
    assert_eq!(
        g.derived_version.character_version_id(),
        g.character_version.id()
    );
    assert_eq!(g.derived_version.motion_version_id(), g.motion_version.id());
    assert_eq!(
        g.derived_version.source_skeleton_ref_id(),
        g.source_skeleton.id()
    );
    assert_eq!(g.derived_version.mapping_version_id(), g.mapping_version.id());
    assert_eq!(g.derived_version.policy_version_id(), g.policy_version.id());
    assert_eq!(g.derived_version.job_spec_id(), g.job.id());
    assert_eq!(g.derived_version.worker_result_id(), g.worker.id());
    assert_eq!(g.derived_version.qc_report_id(), Some(g.qc.id()));
}

#[test]
fn persistence_artifact_is_separate_from_derived_variant_version() {
    let g = valid_graph();
    assert_ne!(
        g.persistence.id().canonical(),
        g.derived_version.id().canonical()
    );
    let regenerated = g.persistence.regenerate(digest(9), 8192).unwrap();
    assert_eq!(
        regenerated.bound_derived_variant_version_id(),
        g.derived_version.id()
    );
    assert_ne!(regenerated.instance_id(), g.persistence.instance_id());
    assert_ne!(regenerated.digest(), g.persistence.digest());
}

#[test]
fn preview_artifact_is_separate_from_product_identity() {
    let g = valid_graph();
    assert_ne!(g.preview.id().canonical(), g.derived_version.id().canonical());
    assert_ne!(g.preview.id().canonical(), g.preview.digest().sha256());
    assert!(g.preview.derived() && g.preview.rebuildable() && !g.preview.authoritative());
}

#[test]
fn preview_binds_exact_product_version() {
    let g = valid_graph();
    g.preview.validate().unwrap();
    assert_eq!(
        g.preview.bound_derived_variant_version_id(),
        Some(g.derived_version.id())
    );
}

#[test]
fn job_spec_is_backend_neutral() {
    let g = valid_graph();
    g.job.validate().unwrap();
    let json = to_json(&g.job).unwrap();
    assert!(!json.to_ascii_lowercase().contains("bpy"));
    assert!(!json.to_ascii_lowercase().contains("posebone"));
}

#[test]
fn worker_success_does_not_imply_qc_pass_or_publication() {
    let g = valid_graph();
    assert!(g.worker.worker_success());
    assert!(!g.worker.implies_qc_pass());
    assert!(!g.worker.implies_publication());
    let err = assert_not_used_as_publication(&g.worker).unwrap_err();
    assert_eq!(err.code, ErrorCode::WorkerNotPublicationAuthority);
}

#[test]
fn qc_subject_binding() {
    let g = valid_graph();
    g.qc.validate().unwrap();
    assert_eq!(
        g.qc.subject_derived_variant_version_id(),
        Some(g.derived_version.id())
    );
}

#[test]
fn schema_version_recognized() {
    let g = valid_graph();
    assert_eq!(g.character_version.schema_version(), SCHEMA_VERSION);
    g.character_version.validate().unwrap();
}

#[test]
fn unsupported_schema_version_fails_closed() {
    let g = valid_graph();
    let text = json_with(serde_json::to_value(&g.job).unwrap(), "schema_version", serde_json::json!(99));
    let err = from_json_validated::<JobSpec>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnsupportedSchemaVersion);
}

#[test]
fn missing_required_provenance_fails() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.qc).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("subject_derived_variant_version_id".into(), serde_json::Value::Null);
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<QcReport>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::SubjectUnion);
}

#[test]
fn round_trip_character_asset_version() {
    let g = valid_graph();
    g.character_version.validate().unwrap();
    let back: CharacterAssetVersion = round_trip(&g.character_version).unwrap();
    assert_eq!(g.character_version, back);
}

#[test]
fn round_trip_motion_asset_version() {
    let g = valid_graph();
    let back: MotionAssetVersion = round_trip(&g.motion_version).unwrap();
    assert_eq!(g.motion_version, back);
}

#[test]
fn round_trip_bone_mapping_version() {
    let g = valid_graph();
    let back: BoneMappingVersion = round_trip(&g.mapping_version).unwrap();
    assert_eq!(g.mapping_version, back);
}

#[test]
fn round_trip_retarget_policy_version() {
    let g = valid_graph();
    let back: RetargetPolicyVersion = round_trip(&g.policy_version).unwrap();
    assert_eq!(g.policy_version, back);
}

#[test]
fn round_trip_job_spec() {
    let g = valid_graph();
    let back: JobSpec = round_trip(&g.job).unwrap();
    assert_eq!(g.job, back);
}

#[test]
fn round_trip_derived_variant_version() {
    let g = valid_graph();
    let back: DerivedVariantVersion = round_trip(&g.derived_version).unwrap();
    assert_eq!(g.derived_version, back);
}

#[test]
fn round_trip_preview_artifact() {
    let g = valid_graph();
    let back: PreviewArtifact = round_trip(&g.preview).unwrap();
    assert_eq!(g.preview, back);
}

#[test]
fn wrong_product_version_binding_fails_publication() {
    let mut g = unpublished_graph();
    g.replace_qc(
        QcReport::for_derived_variant(
            DerivedVariantVersionId::generate(),
            g.policy_version.id(),
            g.worker.id(),
            g.persistence.id(),
            g.persistence.instance_id(),
            g.persistence.digest().clone(),
            passing_structural_qc_checks(),
        )
        .unwrap(),
    );
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::QcSubjectMismatch);
}

#[test]
fn missing_mapping_version_on_job_spec_fails_deserialize() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.job).unwrap();
    value.as_object_mut().unwrap().remove("mapping_version_id");
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<JobSpec>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidJson);
}

#[test]
fn artifact_hash_cannot_be_used_as_product_id_in_json() {
    let hex = "cd".repeat(32);
    let err = CharacterAssetVersionId::parse(&hex).unwrap_err();
    assert_eq!(err.code, ErrorCode::DigestUsedAsId);
    let json = format!("\"{hex}\"");
    assert!(serde_json::from_str::<CharacterAssetVersionId>(&json).is_err());
}

#[test]
fn preview_bound_to_wrong_derived_variant_fails() {
    let g = valid_graph();
    let preview = PreviewArtifact::for_derived_variant(
        DerivedVariantVersionId::generate(),
        digest(4),
        2048,
        "application/octet-stream",
        g.backend.id(),
    )
    .unwrap();
    let err = preview
        .assert_bound_derived_variant(g.derived_version.id())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn job_spec_with_blender_api_leak_fails() {
    let g = valid_graph();
    let err = JobSpec::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "bpy.ops.object.mode_set",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::BackendNeutralViolation);
}

#[test]
fn unknown_field_fails_closed() {
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.policy_version).unwrap(),
        "unexpected",
        serde_json::json!("nope"),
    );
    let err = from_json_validated::<RetargetPolicyVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnknownField);
}

#[test]
fn worker_result_cannot_publish_without_qc() {
    let mut g = unpublished_graph();
    let mut checks = passing_structural_qc_checks();
    checks[0] = QcCheck::new(QcCheckName::FiniteTransforms, QcCheckOutcome::Fail);
    g.replace_qc(
        QcReport::for_derived_variant(
            g.derived_version.id(),
            g.policy_version.id(),
            g.worker.id(),
            g.persistence.id(),
            g.persistence.instance_id(),
            g.persistence.digest().clone(),
            checks,
        )
        .unwrap(),
    );
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::WorkerNotPublicationAuthority);
}

#[test]
fn compatibility_keeps_separable_dimensions() {
    let g = valid_graph();
    let result = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::PassWithWarnings,
        Judgment::Pass,
        Judgment::Unknown,
        Judgment::Unknown,
        CompatibilitySummary::ReadyWithWarnings,
        vec!["motion suitability not yet judged".to_string()],
    )
    .unwrap();
    result.validate().unwrap();
    let back: CompatibilityResult = round_trip(&result).unwrap();
    assert_eq!(result, back);
}

#[test]
fn skeleton_summary_is_derived_evidence() {
    let g = valid_graph();
    let summary = SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(g.character_version.id()),
        None,
        g.backend.clone(),
        vec![JointObservation::new(
            JointKey::new("Bone").unwrap(),
            "Bone",
            None,
            true,
            Some("deform".to_string()),
            Some("rest pose observation".to_string()),
        )
        .unwrap()],
        vec!["derived evidence only".to_string()],
    )
    .unwrap();
    summary.validate().unwrap();
}

#[test]
fn frames_are_not_seconds() {
    let err = TimeDomainProvenance::new(
        "clip:walk-carry",
        TimePoint::frames(1, 30, 1).unwrap(),
        TimePoint::seconds(2, 1).unwrap(),
        SamplingInterpretation::BakedEverySourceFrame,
        "unmapped target joints remain at target rest",
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn mapping_is_more_than_name_pairs() {
    let g = valid_graph();
    assert!(!g.mapping_version.entries().is_empty());
    assert!(g.mapping_version.review().reviewed());
    let mut value = serde_json::to_value(&g.mapping_version).unwrap();
    value["lifecycle"] = serde_json::json!("draft");
    value["entries"] = serde_json::json!([]);
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<BoneMappingVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn schema_version_is_not_entity_version() {
    let g = valid_graph();
    assert_eq!(g.character_version.schema_version(), 1);
    assert_ne!(
        g.character_version.id().canonical(),
        g.character_version.schema_version().to_string()
    );
}
