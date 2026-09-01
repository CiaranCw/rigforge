mod common;

use rigforge_app::{
    generate_mapping_proposal, MappingAssistProfile, MemorySkeletonInspector,
};
use rigforge_domain::*;

fn producer() -> BackendExecutionContext {
    common::backend()
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

fn summary_source(id: SourceSkeletonReferenceId, joints: Vec<JointObservation>) -> SkeletonSummary {
    SkeletonSummary::new(
        SkeletonSubjectKind::SourceSkeletonReference,
        None,
        Some(id),
        producer(),
        joints,
        vec!["fixture".into()],
    )
    .unwrap()
}

fn summary_target(id: CharacterAssetVersionId, joints: Vec<JointObservation>) -> SkeletonSummary {
    SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(id),
        None,
        producer(),
        joints,
        vec!["fixture".into()],
    )
    .unwrap()
}

#[test]
fn exact_name_match() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![j("Head", None, true, "deforming"), j("Neck", Some("Head"), false, "deforming")],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![j("Head", None, true, "deforming"), j("Neck", Some("Head"), false, "deforming")],
    );
    let proposal = generate_mapping_proposal(&src, &dst, MappingAssistProfile::None).unwrap();
    assert!(proposal.entries.iter().any(|e| e.source_key == "Head" && e.target_key == "Head"));
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.signals.contains(&rigforge_app::CandidateSignal::ExactName)));
}

#[test]
fn normalized_name_match() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("upperarm_l", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("UpperArm.L", Some("Bone"), false, "deforming"),
        ],
    );
    let proposal =
        generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "upperarm_l" && e.target_key == "UpperArm.L"));
}

#[test]
fn left_right_distinction() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("hand_l", Some("root"), false, "deforming"),
            j("hand_r", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("Fist.L", Some("Bone"), false, "deforming"),
            j("Fist.R", Some("Bone"), false, "deforming"),
        ],
    );
    let proposal =
        generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "hand_l" && e.target_key == "Fist.L"));
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "hand_r" && e.target_key == "Fist.R"));
}

#[test]
fn parent_context_disambiguation() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("child", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("ChildA", Some("Bone"), false, "deforming"),
        ],
    );
    let proposal = generate_mapping_proposal(&src, &dst, MappingAssistProfile::None).unwrap();
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "root" && e.target_key == "Bone"));
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "child" && e.target_key == "ChildA"));
}

#[test]
fn ambiguous_equal_candidates_require_confirmation() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("mid", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("A", Some("Bone"), false, "deforming"),
            j("B", Some("Bone"), false, "deforming"),
        ],
    );
    let proposal = generate_mapping_proposal(&src, &dst, MappingAssistProfile::None).unwrap();
    assert!(proposal.confirmation_required);
    assert!(!proposal.ambiguities.is_empty() || proposal.unmapped_source.iter().any(|u| u.reason().contains("required") || u.reason().contains("pending")));
}

#[test]
fn required_target_collision_not_silently_accepted() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("alpha", Some("root"), false, "deforming"),
            j("beta", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![j("Bone", None, true, "deforming"), j("Only", Some("Bone"), false, "deforming")],
    );
    let proposal =
        generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    let only_count = proposal
        .entries
        .iter()
        .filter(|e| e.target_key == "Only")
        .count();
    assert!(only_count <= 1);
    assert!(proposal.confirmation_required || proposal.unmapped_source.len() >= 1);
}

#[test]
fn helper_optional_unmapped_non_blocking() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("PoleTarget.L", Some("root"), false, "helper"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![j("Bone", None, true, "deforming")],
    );
    let proposal = generate_mapping_proposal(&src, &dst, MappingAssistProfile::None).unwrap();
    assert!(proposal
        .unmapped_source
        .iter()
        .any(|u| u.joint_key().as_str() == "PoleTarget.L"));
    assert!(!proposal.confirmation_required);
}

#[test]
fn ball_toe_optional_unmapped_non_blocking() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("foot_l", Some("root"), false, "deforming"),
            j("ball_l", Some("foot_l"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("Foot.L", Some("Bone"), false, "deforming"),
            j("Foot.L_end", Some("Foot.L"), false, "helper"),
        ],
    );
    let proposal =
        generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    assert!(proposal
        .entries
        .iter()
        .any(|e| e.source_key == "foot_l" && e.target_key == "Foot.L"));
    assert!(proposal
        .unmapped_source
        .iter()
        .any(|u| u.joint_key().as_str() == "ball_l"));
    assert!(!proposal.confirmation_required);
}

#[test]
fn deterministic_repeated_candidate_result() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("head", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("Bone", None, true, "deforming"),
            j("Head", Some("Bone"), false, "deforming"),
        ],
    );
    let a = generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    let b = generate_mapping_proposal(&src, &dst, MappingAssistProfile::OptionalHumanoid).unwrap();
    assert_eq!(a, b);
}

#[test]
fn memory_inspector_is_not_blender() {
    let g = common::unpublished_graph();
    let source = summary_source(
        g.source_skeleton.id(),
        vec![j("root", None, true, "deforming")],
    );
    let target = summary_target(
        g.character_version.id(),
        vec![j("Bone", None, true, "deforming")],
    );
    let inspector = MemorySkeletonInspector {
        character: target,
        source,
    };
    let _ = inspector;
}

#[test]
fn non_humanoid_mapping_without_humanoid_slots() {
    let src = summary_source(
        SourceSkeletonReferenceId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("segment_a", Some("root"), false, "deforming"),
            j("segment_b", Some("segment_a"), false, "deforming"),
            j("appendage", Some("root"), false, "deforming"),
        ],
    );
    let dst = summary_target(
        CharacterAssetVersionId::generate(),
        vec![
            j("root", None, true, "deforming"),
            j("segment_a", Some("root"), false, "deforming"),
            j("segment_b", Some("segment_a"), false, "deforming"),
            j("appendage", Some("root"), false, "deforming"),
        ],
    );
    let proposal = generate_mapping_proposal(&src, &dst, MappingAssistProfile::None).unwrap();
    assert!(proposal.entries.iter().any(|e| e.source_key == "segment_a"));
    assert!(proposal.entries.iter().all(|e| e.role_profile.is_none()));
}
