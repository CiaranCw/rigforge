//! Compatibility preflight. Does not launch Transfer or create WorkerResult.

use rigforge_domain::{
    BoneMappingVersion, CharacterAssetVersion, CompatibilityResult, IkPolicy, Judgment, Lifecycle,
    MappingReviewKind, MotionAssetVersion, RetargetPolicyVersion, ScalePolicy, SkeletonSummary,
    SourceSkeletonReference, TimeKind,
};

use crate::capability::WorkerCapabilityProfile;
use crate::error::AppError;

pub fn evaluate_compatibility(
    character: &CharacterAssetVersion,
    motion: &MotionAssetVersion,
    source_skeleton: &SourceSkeletonReference,
    mapping: &BoneMappingVersion,
    policy: &RetargetPolicyVersion,
    source_summary: Option<&SkeletonSummary>,
    target_summary: Option<&SkeletonSummary>,
    capability: &WorkerCapabilityProfile,
) -> Result<CompatibilityResult, AppError> {
    if mapping.target_character_version_id() != character.id() {
        return Err(AppError::Catalog(
            "Compatibility mapping is not bound to this CharacterAssetVersion".into(),
        ));
    }

    let mut notes = Vec::new();
    let mapping_completeness =
        mapping_completeness(mapping, source_summary, target_summary, &mut notes);
    let structural =
        structural_compatibility(mapping, source_summary, target_summary, &mut notes);
    let method = method_eligibility(policy, motion, capability, &mut notes);
    let motion_j = motion_suitability(motion, source_skeleton, mapping, &mut notes);
    notes.push(
        "result_acceptability is post-execution only; V1-4 preflight records UNKNOWN and is not QC"
            .into(),
    );

    CompatibilityResult::from_preflight(
        character.id(),
        motion.id(),
        mapping.id(),
        policy.id(),
        mapping_completeness,
        structural,
        method,
        motion_j,
        Judgment::Unknown,
        notes,
    )
    .map_err(AppError::from)
}

fn mapping_completeness(
    mapping: &BoneMappingVersion,
    source_summary: Option<&SkeletonSummary>,
    target_summary: Option<&SkeletonSummary>,
    notes: &mut Vec<String>,
) -> Judgment {
    if mapping.lifecycle() != Lifecycle::Published {
        notes.push(
            "mapping is not Published; explicit Product acceptance is required before Ready".into(),
        );
        return Judgment::Unknown;
    }
    let review = mapping.review();
    if review.confirmation_required()
        || matches!(review.review_kind(), Some(MappingReviewKind::AutomaticCandidate))
        || !review.reviewed()
    {
        notes.push("mapping is not explicitly accepted or still requires confirmation".into());
        return Judgment::Unknown;
    }
    if !review.ambiguities().is_empty() && review.reviewed() {
        notes.push("accepted Mapping still records unresolved ambiguities".into());
    }

    let mut missing_required_names = Vec::new();
    let mut optional_unmapped = false;
    for u in mapping.unmapped_source() {
        match u.disposition() {
            rigforge_domain::UnmappedDisposition::Blocking => {
                missing_required_names.push(u.joint_key().as_str().to_string());
            }
            rigforge_domain::UnmappedDisposition::Optional
            | rigforge_domain::UnmappedDisposition::Helper => {
                optional_unmapped = true;
            }
        }
    }
    if mapping
        .unmapped_target()
        .iter()
        .any(|u| !u.disposition().is_blocking())
    {
        optional_unmapped = true;
    }
    if mapping
        .unmapped_target()
        .iter()
        .any(|u| u.disposition().is_blocking())
    {
        notes.push("blocking unmapped target joint(s) remain".into());
        return Judgment::Fail;
    }

    if let (Some(source), Some(_target)) = (source_summary, target_summary) {
        for joint in source.joints() {
            let mapped = mapping
                .entries()
                .iter()
                .any(|e| e.source().joint_key() == joint.joint_key());
            if mapped {
                continue;
            }
            let recorded = mapping
                .unmapped_source()
                .iter()
                .any(|u| u.joint_key() == joint.joint_key());
            if recorded {
                continue;
            }
            missing_required_names.push(joint.display_name().to_string());
        }
    } else {
        notes.push("completeness judged from Mapping unmapped dispositions; summaries were not both supplied".into());
    }

    if !missing_required_names.is_empty() {
        notes.push(format!(
            "{} required source joint(s) are unmapped: {}",
            missing_required_names.len(),
            missing_required_names.join(", ")
        ));
        return Judgment::Fail;
    }
    if optional_unmapped
        || !mapping.unmapped_source().is_empty()
        || !mapping.unmapped_target().is_empty()
    {
        notes.push("optional or helper joints remain unmapped".into());
        Judgment::PassWithWarnings
    } else {
        Judgment::Pass
    }
}

fn structural_compatibility(
    mapping: &BoneMappingVersion,
    source_summary: Option<&SkeletonSummary>,
    target_summary: Option<&SkeletonSummary>,
    notes: &mut Vec<String>,
) -> Judgment {
    let (Some(source), Some(target)) = (source_summary, target_summary) else {
        notes.push("structural compatibility UNKNOWN without both SkeletonSummary records".into());
        return Judgment::Unknown;
    };
    if source.subject_source_skeleton_ref_id() != Some(mapping.source_skeleton_ref_id()) {
        notes.push("source summary subject does not match mapping SourceSkeletonReference".into());
        return Judgment::Fail;
    }
    if target.subject_character_version_id() != Some(mapping.target_character_version_id()) {
        notes.push("target summary subject does not match mapping CharacterAssetVersion".into());
        return Judgment::Fail;
    }

    let mut missing = 0usize;
    let mut hierarchy_warnings = 0usize;
    for entry in mapping.entries() {
        if source.joint(entry.source().joint_key()).is_none() {
            missing += 1;
        }
        if target.joint(entry.target().joint_key()).is_none() {
            missing += 1;
        }
    }
    if missing > 0 {
        notes.push("mapped joints are missing from SkeletonSummary".into());
        return Judgment::Fail;
    }

    let src_roots: Vec<_> = source.joints().iter().filter(|j| j.is_root()).collect();
    let dst_roots: Vec<_> = target.joints().iter().filter(|j| j.is_root()).collect();
    if src_roots.len() == 1 && dst_roots.len() == 1 {
        let root_mapped = mapping.entries().iter().any(|e| {
            e.source().joint_key() == src_roots[0].joint_key()
                && e.target().joint_key() == dst_roots[0].joint_key()
        });
        if !root_mapped {
            notes.push("unique roots are not mapped to each other".into());
            hierarchy_warnings += 1;
        }
    }

    for entry in mapping.entries() {
        let Some(sj) = source.joint(entry.source().joint_key()) else {
            continue;
        };
        let Some(tj) = target.joint(entry.target().joint_key()) else {
            continue;
        };
        if let (Some(sp), Some(tp)) = (sj.parent_key(), tj.parent_key()) {
            let parent_pair = mapping.entries().iter().any(|e| {
                e.source().joint_key() == sp && e.target().joint_key() == tp
            });
            let source_parent_mapped_elsewhere = mapping
                .entries()
                .iter()
                .any(|e| e.source().joint_key() == sp);
            if source_parent_mapped_elsewhere && !parent_pair {
                hierarchy_warnings += 1;
            }
        }
    }

    if hierarchy_warnings > 0 {
        notes.push("mapped parent relationships are not fully coherent".into());
        Judgment::PassWithWarnings
    } else {
        Judgment::Pass
    }
}

fn method_eligibility(
    policy: &RetargetPolicyVersion,
    motion: &MotionAssetVersion,
    capability: &WorkerCapabilityProfile,
    notes: &mut Vec<String>,
) -> Judgment {
    if !capability.proven_rest_relative_policy {
        notes.push("current worker does not declare proven rest-relative Policy support".into());
        return Judgment::Fail;
    }
    if policy.ik_policy() != IkPolicy::ExplicitNoIk && !capability.ik_supported {
        notes.push("unsupported IK requirement".into());
        return Judgment::Fail;
    }
    if policy.scale_policy() != ScalePolicy::KeepTargetRestScale {
        notes.push("unsupported scale policy token".into());
        return Judgment::Fail;
    }
    if preflight_policy_supported(policy).is_err() {
        notes.push("unsupported Policy token relative to V1-3 proven set".into());
        return Judgment::Fail;
    }
    if capability.integral_frames_only {
        let start = motion.time().start();
        let end = motion.time().end();
        if start.kind() == TimeKind::Frames && !start.is_integral_frame() {
            notes.push("unsupported fractional-frame execution".into());
            return Judgment::Fail;
        }
        if end.kind() == TimeKind::Frames && !end.is_integral_frame() {
            notes.push("unsupported fractional-frame execution".into());
            return Judgment::Fail;
        }
    }
    notes.push("method eligibility uses backend-neutral worker capability tokens".into());
    Judgment::Pass
}

fn motion_suitability(
    motion: &MotionAssetVersion,
    source_skeleton: &SourceSkeletonReference,
    mapping: &BoneMappingVersion,
    notes: &mut Vec<String>,
) -> Judgment {
    if motion.source_skeleton_ref_id() != source_skeleton.id() {
        notes.push("Motion references a different Source Skeleton".into());
        return Judgment::Fail;
    }
    if mapping.source_skeleton_ref_id() != motion.source_skeleton_ref_id() {
        notes.push("Mapping Source Skeleton does not match Motion Source Skeleton".into());
        return Judgment::Fail;
    }
    if let Some(bound) = mapping.source_motion_version_id() {
        if bound != motion.id() {
            notes.push(
                "Mapping records a different Motion version as additional evidence; Source Skeleton still matches"
                    .into(),
            );
            return Judgment::PassWithWarnings;
        }
    }
    if motion.time().clip_identity_evidence().trim().is_empty() {
        notes.push("Motion is missing clip identity evidence".into());
        return Judgment::Fail;
    }
    Judgment::Pass
}

pub(crate) fn preflight_policy_supported(
    policy: &RetargetPolicyVersion,
) -> Result<(), AppError> {
    use rigforge_domain::{
        ChannelPolicy, MissingSourceJointPolicy, QuaternionContinuityPolicy,
        QuaternionInterpolationPolicy, QuaternionNormalizationPolicy, RestAlignmentPolicy,
        RootPolicy, TimeBakePolicy, UnmappedTargetChannelPolicy,
    };
    if policy.root_policy() != RootPolicy::CopyWorldTranslationDelta
        || policy.channel_policy() != ChannelPolicy::RotationOnlyMappedNonRoot
        || policy.quaternion_normalization_policy() != QuaternionNormalizationPolicy::NormalizeBeforeKey
        || policy.quaternion_continuity_policy() != QuaternionContinuityPolicy::ConsecutiveHemisphere
        || policy.quaternion_interpolation_policy()
            != QuaternionInterpolationPolicy::BackendInterpolationAfterNormalizedKeys
        || policy.time_bake_policy() != TimeBakePolicy::EverySourceFrame
        || policy.rest_alignment_policy() != RestAlignmentPolicy::RestRelativeWorldDelta
        || policy.missing_channel_policy().unmapped_target_joints()
            != UnmappedTargetChannelPolicy::RemainAtTargetRest
        || policy.missing_channel_policy().missing_source_joint()
            != MissingSourceJointPolicy::FailClosedDoNotInvent
        || policy.scale_policy() != ScalePolicy::KeepTargetRestScale
        || policy.ik_policy() != IkPolicy::ExplicitNoIk
    {
        return Err(AppError::Catalog("unsupported Policy token".into()));
    }
    Ok(())
}
