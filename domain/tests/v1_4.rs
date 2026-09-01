mod common;

use rigforge_domain::*;

fn producer() -> BackendExecutionContext {
    common::backend()
}

fn j(
    key: &str,
    parent: Option<&str>,
    root: bool,
    deform: &str,
) -> JointObservation {
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

#[test]
fn derive_ready_allows_unknown_result_acceptability() {
    assert_eq!(
        CompatibilityResult::derive_summary(
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Unknown,
        ),
        CompatibilitySummary::Ready
    );
}

#[test]
fn derive_confirmation_and_unsupported_matrix() {
    assert_eq!(
        CompatibilityResult::derive_summary(
            Judgment::Unknown,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Unknown,
        ),
        CompatibilitySummary::MappingConfirmationRequired
    );
    assert_eq!(
        CompatibilityResult::derive_summary(
            Judgment::Fail,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Unknown,
        ),
        CompatibilitySummary::Unsupported
    );
    assert_eq!(
        CompatibilityResult::derive_summary(
            Judgment::Pass,
            Judgment::PassWithWarnings,
            Judgment::Pass,
            Judgment::Pass,
            Judgment::Unknown,
        ),
        CompatibilitySummary::ReadyWithWarnings
    );
}

#[test]
fn from_preflight_rejects_ui_chosen_summary() {
    let g = common::valid_graph();
    let result = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["pre-transfer".into()],
    )
    .unwrap();
    assert_eq!(result.summary(), CompatibilitySummary::Ready);
    assert_eq!(result.result_acceptability(), Judgment::Unknown);
}

#[test]
fn automatic_candidate_cannot_claim_reviewed() {
    let err = MappingReviewProvenance::with_kind(
        true,
        Some("auto".into()),
        vec![],
        MappingReviewKind::AutomaticCandidate,
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn published_mapping_cannot_be_edited_in_place() {
    let mut g = common::unpublished_graph();
    let err = g
        .mapping_version
        .replace_entries_and_review(
            g.mapping_version.entries().to_vec(),
            MappingReviewProvenance::new(true, Some("edit".into()), vec![]).unwrap(),
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
}

#[test]
fn mapping_without_humanoid_slots_is_valid() {
    let summary = SkeletonSummary::new(
        SkeletonSubjectKind::SourceSkeletonReference,
        None,
        Some(SourceSkeletonReference::new("critter").unwrap().id()),
        producer(),
        vec![
            j("root", None, true, "deforming"),
            j("segment_a", Some("root"), false, "deforming"),
            j("segment_b", Some("segment_a"), false, "deforming"),
            j("appendage", Some("root"), false, "deforming"),
        ],
        vec!["non-humanoid fixture".into()],
    )
    .unwrap();
    assert!(summary.joints().iter().all(|j| {
        optional_humanoid_not_required(j.display_name())
    }));
}

fn optional_humanoid_not_required(name: &str) -> bool {
    !matches!(name, "hips" | "spine" | "head" | "left_arm")
}

fn candidate_mapping_draft() -> BoneMappingVersion {
    let g = common::unpublished_graph();
    let mapping = BoneMapping::new("candidate").unwrap();
    let mut version = BoneMappingVersion::draft(
        mapping.id(),
        g.character_version.id(),
        g.source_skeleton.id(),
        vec![common::mapping_entry("root", "Bone")],
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
}

#[test]
fn automatic_candidate_domain_publish_rejected() {
    let mut version = candidate_mapping_draft();
    let err = version.publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn published_automatic_candidate_json_rejected() {
    let g = common::valid_graph();
    let mut value = serde_json::to_value(&g.mapping_version).unwrap();
    value["review"]["review_kind"] = serde_json::json!("automatic_candidate");
    value["review"]["reviewed"] = serde_json::json!(false);
    value["generated_from_candidates"] = serde_json::json!(true);
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<BoneMappingVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn automatic_candidate_cannot_be_accepted_as_manual() {
    let mut version = candidate_mapping_draft();
    version
        .replace_entries_and_review(
            version.entries().to_vec(),
            MappingReviewProvenance::with_kind(
                true,
                Some("explicit Product acceptance".into()),
                vec![],
                MappingReviewKind::Manual,
            )
            .unwrap(),
        )
        .unwrap();
    let err = version.publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn user_edited_candidate_cannot_be_accepted_as_automatic_confirmed() {
    let mut version = candidate_mapping_draft();
    version.mark_user_modified().unwrap();
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
    let err = version.publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn duplicate_source_mapping_entries_rejected() {
    let g = common::unpublished_graph();
    let mapping = BoneMapping::new("dup-src").unwrap();
    let err = BoneMappingVersion::draft(
        mapping.id(),
        g.character_version.id(),
        g.source_skeleton.id(),
        vec![
            common::mapping_entry("root", "Bone"),
            common::mapping_entry("root", "Body"),
        ],
        MappingReviewProvenance::new(false, Some("draft".into()), vec![]).unwrap(),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn duplicate_target_mapping_entries_rejected() {
    let g = common::unpublished_graph();
    let mapping = BoneMapping::new("dup-dst").unwrap();
    let err = BoneMappingVersion::draft(
        mapping.id(),
        g.character_version.id(),
        g.source_skeleton.id(),
        vec![
            common::mapping_entry("root", "Bone"),
            common::mapping_entry("pelvis", "Bone"),
        ],
        MappingReviewProvenance::new(false, Some("draft".into()), vec![]).unwrap(),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn published_mapping_still_immutable() {
    let mut g = common::unpublished_graph();
    let err = g.mapping_version.set_unmapped(Vec::new(), Vec::new()).unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
}

#[test]
fn unknown_mapping_plus_ready_summary_rejected() {
    let g = common::valid_graph();
    let err = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Unknown,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        CompatibilitySummary::Ready,
        vec![],
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn failed_dimension_plus_confirmation_summary_rejected() {
    let g = common::valid_graph();
    let err = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Fail,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        CompatibilitySummary::MappingConfirmationRequired,
        vec![],
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn all_pass_preflight_plus_ready_valid() {
    let g = common::valid_graph();
    let result = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        CompatibilitySummary::Ready,
        vec![],
    )
    .unwrap();
    assert_eq!(result.summary(), CompatibilitySummary::Ready);
}

#[test]
fn warning_matrix_plus_ready_with_warnings_valid() {
    let g = common::valid_graph();
    let result = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::PassWithWarnings,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        CompatibilitySummary::ReadyWithWarnings,
        vec![],
    )
    .unwrap();
    assert_eq!(result.summary(), CompatibilitySummary::ReadyWithWarnings);
}

#[test]
fn untrusted_json_cannot_choose_summary() {
    let g = common::valid_graph();
    let result = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["pre-transfer".into()],
    )
    .unwrap();
    let mut value = serde_json::to_value(&result).unwrap();
    value["summary"] = serde_json::json!("unsupported");
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<CompatibilityResult>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn from_preflight_still_derives_summary() {
    let g = common::valid_graph();
    let result = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Unknown,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec![],
    )
    .unwrap();
    assert_eq!(
        result.summary(),
        CompatibilitySummary::MappingConfirmationRequired
    );
}

#[test]
fn target_side_entry_in_unmapped_source_rejected() {
    let mut version = candidate_mapping_draft();
    let err = version
        .set_unmapped(
            vec![UnmappedJoint::new(
                SkeletonSide::Target,
                JointKey::new("Head").unwrap(),
                "wrong container",
                UnmappedDisposition::Optional,
            )
            .unwrap()],
            vec![],
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn source_side_entry_in_unmapped_target_rejected() {
    let mut version = candidate_mapping_draft();
    let err = version
        .set_unmapped(
            vec![],
            vec![UnmappedJoint::new(
                SkeletonSide::Source,
                JointKey::new("head").unwrap(),
                "wrong container",
                UnmappedDisposition::Optional,
            )
            .unwrap()],
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn untrusted_wrong_side_unmapped_json_rejected() {
    let mut version = candidate_mapping_draft();
    version
        .set_unmapped(
            vec![UnmappedJoint::new(
                SkeletonSide::Source,
                JointKey::new("spare").unwrap(),
                "optional leftover",
                UnmappedDisposition::Optional,
            )
            .unwrap()],
            vec![],
        )
        .unwrap();
    let mut value = serde_json::to_value(&version).unwrap();
    value["unmapped_source"][0]["skeleton_side"] = serde_json::json!("target");
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<BoneMappingVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
}

#[test]
fn wrong_side_optional_record_cannot_hide_required_source_joint() {
    let mut version = candidate_mapping_draft();
    let err = version
        .set_unmapped(
            vec![UnmappedJoint::new(
                SkeletonSide::Target,
                JointKey::new("head").unwrap(),
                "optional looking hide",
                UnmappedDisposition::Optional,
            )
            .unwrap()],
            vec![],
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MappingTooThin);
    assert!(err.message.contains("unmapped_source"));
}

#[test]
fn correct_source_and_target_unmapped_sides_valid() {
    let mut version = candidate_mapping_draft();
    version
        .set_unmapped(
            vec![UnmappedJoint::new(
                SkeletonSide::Source,
                JointKey::new("spare").unwrap(),
                "optional leftover",
                UnmappedDisposition::Optional,
            )
            .unwrap()],
            vec![UnmappedJoint::new(
                SkeletonSide::Target,
                JointKey::new("Pole").unwrap(),
                "helper leftover",
                UnmappedDisposition::Helper,
            )
            .unwrap()],
        )
        .unwrap();
    assert_eq!(
        version.unmapped_source()[0].skeleton_side(),
        SkeletonSide::Source
    );
    assert_eq!(
        version.unmapped_target()[0].skeleton_side(),
        SkeletonSide::Target
    );
}
