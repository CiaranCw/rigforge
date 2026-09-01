//! Translate the proven V1-1 RetargetPolicy. Unknown modes fail before mutation.

use rigforge_app::AppError;
use rigforge_domain::{
    ChannelPolicy, IkPolicy, MissingSourceJointPolicy, QuaternionContinuityPolicy,
    QuaternionInterpolationPolicy, QuaternionNormalizationPolicy, RestAlignmentPolicy,
    RetargetPolicyVersion, RootPolicy, ScalePolicy, TimeBakePolicy, UnmappedTargetChannelPolicy,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyProjection {
    pub root_policy: &'static str,
    pub channel_policy: &'static str,
    pub quaternion_normalization_policy: &'static str,
    pub quaternion_continuity_policy: &'static str,
    pub quaternion_interpolation_policy: &'static str,
    pub time_bake_policy: &'static str,
    pub rest_alignment_policy: &'static str,
    pub unmapped_target_joints: &'static str,
    pub missing_source_joint: &'static str,
    pub scale_policy: &'static str,
    pub ik_policy: &'static str,
}

pub fn project_supported_policy(
    policy: &RetargetPolicyVersion,
) -> Result<PolicyProjection, AppError> {
    if policy.root_policy() != RootPolicy::CopyWorldTranslationDelta {
        return unsupported("root_policy");
    }
    if policy.channel_policy() != ChannelPolicy::RotationOnlyMappedNonRoot {
        return unsupported("channel_policy");
    }
    if policy.quaternion_normalization_policy() != QuaternionNormalizationPolicy::NormalizeBeforeKey
    {
        return unsupported("quaternion_normalization_policy");
    }
    if policy.quaternion_continuity_policy() != QuaternionContinuityPolicy::ConsecutiveHemisphere {
        return unsupported("quaternion_continuity_policy");
    }
    if policy.quaternion_interpolation_policy()
        != QuaternionInterpolationPolicy::BackendInterpolationAfterNormalizedKeys
    {
        return unsupported("quaternion_interpolation_policy");
    }
    if policy.time_bake_policy() != TimeBakePolicy::EverySourceFrame {
        return unsupported("time_bake_policy");
    }
    if policy.rest_alignment_policy() != RestAlignmentPolicy::RestRelativeWorldDelta {
        return unsupported("rest_alignment_policy");
    }
    if policy.missing_channel_policy().unmapped_target_joints()
        != UnmappedTargetChannelPolicy::RemainAtTargetRest
    {
        return unsupported("unmapped_target_joints");
    }
    if policy.missing_channel_policy().missing_source_joint()
        != MissingSourceJointPolicy::FailClosedDoNotInvent
    {
        return unsupported("missing_source_joint");
    }
    if policy.scale_policy() != ScalePolicy::KeepTargetRestScale {
        return unsupported("scale_policy");
    }
    if policy.ik_policy() != IkPolicy::ExplicitNoIk {
        return unsupported("ik_policy");
    }
    Ok(PolicyProjection {
        root_policy: "copy_world_translation_delta",
        channel_policy: "rotation_only_mapped_non_root",
        quaternion_normalization_policy: "normalize_before_key",
        quaternion_continuity_policy: "consecutive_hemisphere",
        quaternion_interpolation_policy: "backend_interpolation_after_normalized_keys",
        time_bake_policy: "every_source_frame",
        rest_alignment_policy: "rest_relative_world_delta",
        unmapped_target_joints: "remain_at_target_rest",
        missing_source_joint: "fail_closed_do_not_invent",
        scale_policy: "keep_target_rest_scale",
        ik_policy: "explicit_no_ik",
    })
}

fn unsupported(field: &str) -> Result<PolicyProjection, AppError> {
    Err(AppError::Worker(format!(
        "unsupported Policy/{field}; fail before blender mutation"
    )))
}
