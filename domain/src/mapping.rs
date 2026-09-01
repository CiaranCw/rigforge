//! Mapping, Compatibility, and Retarget Policy. Product-owned, backend-neutral.

use crate::backend::assert_lifecycle_transition;
use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    assert_backend_neutral_text, expect_record_type, expect_schema_version, BoneMappingId,
    BoneMappingVersionId, CharacterAssetVersionId, CompatibilityResultId, JointKey, Lifecycle,
    MotionAssetVersionId, RecordType, RetargetPolicyId, RetargetPolicyVersionId,
    SourceSkeletonReferenceId, SCHEMA_VERSION,
};
use crate::record::DomainRecord;
use serde::{Deserialize, Serialize};

fn require_nonempty(value: &str, field: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        Err(DomainError::new(
            ErrorCode::MissingRequiredField,
            format!("{field} is required"),
        ))
    } else {
        Ok(())
    }
}

fn freeze(current: Lifecycle) -> Result<Lifecycle, DomainError> {
    assert_lifecycle_transition(current, Lifecycle::Published)?;
    Ok(Lifecycle::Published)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonSide {
    Source,
    Target,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointRef {
    skeleton_side: SkeletonSide,
    joint_key: JointKey,
}

impl JointRef {
    pub fn source(joint_key: JointKey) -> Self {
        Self {
            skeleton_side: SkeletonSide::Source,
            joint_key,
        }
    }

    pub fn target(joint_key: JointKey) -> Self {
        Self {
            skeleton_side: SkeletonSide::Target,
            joint_key,
        }
    }

    pub fn skeleton_side(&self) -> SkeletonSide {
        self.skeleton_side
    }

    pub fn joint_key(&self) -> &JointKey {
        &self.joint_key
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointParticipation {
    Required,
    Optional,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneMappingEntry {
    source: JointRef,
    target: JointRef,
    participation: JointParticipation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role_profile: Option<String>,
    evidence: String,
}

impl BoneMappingEntry {
    pub fn new(
        source: JointRef,
        target: JointRef,
        participation: JointParticipation,
        role_profile: Option<String>,
        evidence: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            source,
            target,
            participation,
            role_profile,
            evidence: evidence.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.source.skeleton_side != SkeletonSide::Source {
            return Err(DomainError::new(
                ErrorCode::MappingTooThin,
                "mapping source joint must be SkeletonSide::Source",
            ));
        }
        if self.target.skeleton_side != SkeletonSide::Target {
            return Err(DomainError::new(
                ErrorCode::MappingTooThin,
                "mapping target joint must be SkeletonSide::Target",
            ));
        }
        require_nonempty(&self.evidence, "mapping evidence")?;
        assert_backend_neutral_text(&self.evidence, "mapping evidence")?;
        if let Some(role) = &self.role_profile {
            assert_backend_neutral_text(role, "role_profile")?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnmappedJoint {
    skeleton_side: SkeletonSide,
    joint_key: JointKey,
    reason: String,
}

impl UnmappedJoint {
    pub fn validate(&self) -> Result<(), DomainError> {
        require_nonempty(&self.reason, "unmapped reason")?;
        assert_backend_neutral_text(&self.reason, "unmapped reason")
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingReviewProvenance {
    reviewed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ambiguities: Vec<String>,
}

impl MappingReviewProvenance {
    pub fn new(reviewed: bool, method: Option<String>, ambiguities: Vec<String>) -> Result<Self, DomainError> {
        let value = Self {
            reviewed,
            method,
            ambiguities,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn reviewed(&self) -> bool {
        self.reviewed
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if let Some(method) = &self.method {
            assert_backend_neutral_text(method, "mapping review method")?;
        }
        for item in &self.ambiguities {
            assert_backend_neutral_text(item, "mapping ambiguity")?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneMapping {
    schema_version: u32,
    record_type: RecordType,
    id: BoneMappingId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_version_id: Option<BoneMappingVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    draft_version_id: Option<BoneMappingVersionId>,
}

impl BoneMapping {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::BoneMapping,
            id: BoneMappingId::generate(),
            display_name: display_name.into(),
            published_version_id: None,
            draft_version_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> BoneMappingId {
        self.id
    }

    pub fn bind_published(&mut self, version_id: BoneMappingVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::BoneMapping)?;
        require_nonempty(&self.display_name, "display_name")
    }
}

impl DomainRecord for BoneMapping {
    const RECORD_TYPE: RecordType = RecordType::BoneMapping;
    fn validate(&self) -> Result<(), DomainError> {
        BoneMapping::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneMappingVersion {
    schema_version: u32,
    record_type: RecordType,
    id: BoneMappingVersionId,
    mapping_id: BoneMappingId,
    lifecycle: Lifecycle,
    target_character_version_id: CharacterAssetVersionId,
    source_skeleton_ref_id: SourceSkeletonReferenceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_motion_version_id: Option<MotionAssetVersionId>,
    entries: Vec<BoneMappingEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    unmapped_source: Vec<UnmappedJoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    unmapped_target: Vec<UnmappedJoint>,
    review: MappingReviewProvenance,
}

impl BoneMappingVersion {
    pub fn draft(
        mapping_id: BoneMappingId,
        target_character_version_id: CharacterAssetVersionId,
        source_skeleton_ref_id: SourceSkeletonReferenceId,
        entries: Vec<BoneMappingEntry>,
        review: MappingReviewProvenance,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::BoneMappingVersion,
            id: BoneMappingVersionId::generate(),
            mapping_id,
            lifecycle: Lifecycle::Draft,
            target_character_version_id,
            source_skeleton_ref_id,
            source_motion_version_id: None,
            entries,
            unmapped_source: Vec::new(),
            unmapped_target: Vec::new(),
            review,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> BoneMappingVersionId {
        self.id
    }
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn target_character_version_id(&self) -> CharacterAssetVersionId {
        self.target_character_version_id
    }
    pub fn source_skeleton_ref_id(&self) -> SourceSkeletonReferenceId {
        self.source_skeleton_ref_id
    }
    pub fn entries(&self) -> &[BoneMappingEntry] {
        &self.entries
    }
    pub fn review(&self) -> &MappingReviewProvenance {
        &self.review
    }

    pub fn bind_source_motion_version(
        &mut self,
        motion_version_id: MotionAssetVersionId,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.source_motion_version_id = Some(motion_version_id);
        self.validate()
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::BoneMappingVersion)?;
        if self.entries.is_empty() {
            return Err(DomainError::new(
                ErrorCode::MappingTooThin,
                "BoneMappingVersion must contain joint correspondence entries; a dict of names is not sufficient by itself",
            ));
        }
        for entry in &self.entries {
            entry.validate()?;
        }
        for u in self.unmapped_source.iter().chain(self.unmapped_target.iter()) {
            u.validate()?;
        }
        self.review.validate()
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        self.lifecycle = freeze(self.lifecycle)?;
        Ok(())
    }

    pub fn invalidate(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        assert_lifecycle_transition(self.lifecycle, Lifecycle::Invalidated)?;
        self.lifecycle = Lifecycle::Invalidated;
        Ok(())
    }
}

impl DomainRecord for BoneMappingVersion {
    const RECORD_TYPE: RecordType = RecordType::BoneMappingVersion;
    fn validate(&self) -> Result<(), DomainError> {
        BoneMappingVersion::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Judgment {
    Pass,
    PassWithWarnings,
    Fail,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilitySummary {
    Ready,
    ReadyWithWarnings,
    MappingConfirmationRequired,
    Unsupported,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityResult {
    schema_version: u32,
    record_type: RecordType,
    id: CompatibilityResultId,
    character_version_id: CharacterAssetVersionId,
    motion_version_id: MotionAssetVersionId,
    mapping_version_id: BoneMappingVersionId,
    policy_version_id: RetargetPolicyVersionId,
    mapping_completeness: Judgment,
    structural_compatibility: Judgment,
    method_eligibility: Judgment,
    motion_suitability: Judgment,
    result_acceptability: Judgment,
    summary: CompatibilitySummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

impl CompatibilityResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        character_version_id: CharacterAssetVersionId,
        motion_version_id: MotionAssetVersionId,
        mapping_version_id: BoneMappingVersionId,
        policy_version_id: RetargetPolicyVersionId,
        mapping_completeness: Judgment,
        structural_compatibility: Judgment,
        method_eligibility: Judgment,
        motion_suitability: Judgment,
        result_acceptability: Judgment,
        summary: CompatibilitySummary,
        notes: Vec<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::CompatibilityResult,
            id: CompatibilityResultId::generate(),
            character_version_id,
            motion_version_id,
            mapping_version_id,
            policy_version_id,
            mapping_completeness,
            structural_compatibility,
            method_eligibility,
            motion_suitability,
            result_acceptability,
            summary,
            notes,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn mapping_completeness(&self) -> Judgment {
        self.mapping_completeness
    }
    pub fn structural_compatibility(&self) -> Judgment {
        self.structural_compatibility
    }
    pub fn method_eligibility(&self) -> Judgment {
        self.method_eligibility
    }
    pub fn motion_suitability(&self) -> Judgment {
        self.motion_suitability
    }
    pub fn result_acceptability(&self) -> Judgment {
        self.result_acceptability
    }
    pub fn summary(&self) -> CompatibilitySummary {
        self.summary
    }

    fn decision_critical_fail(&self) -> bool {
        matches!(self.mapping_completeness, Judgment::Fail)
            || matches!(self.structural_compatibility, Judgment::Fail)
            || matches!(self.method_eligibility, Judgment::Fail)
            || matches!(self.motion_suitability, Judgment::Fail)
            || matches!(self.result_acceptability, Judgment::Fail)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::CompatibilityResult)?;
        if self.summary == CompatibilitySummary::Ready && self.decision_critical_fail() {
            return Err(DomainError::new(
                ErrorCode::CompatibilityContradiction,
                "CompatibilitySummary::Ready cannot coexist with a decision-critical FAIL dimension",
            ));
        }
        if self.summary == CompatibilitySummary::ReadyWithWarnings && self.decision_critical_fail() {
            return Err(DomainError::new(
                ErrorCode::CompatibilityContradiction,
                "CompatibilitySummary::ReadyWithWarnings cannot coexist with a decision-critical FAIL dimension",
            ));
        }
        for note in &self.notes {
            assert_backend_neutral_text(note, "compatibility note")?;
        }
        Ok(())
    }
}

impl DomainRecord for CompatibilityResult {
    const RECORD_TYPE: RecordType = RecordType::CompatibilityResult;
    fn validate(&self) -> Result<(), DomainError> {
        CompatibilityResult::validate(self)
    }
}

/// Product-owned, machine-readable retarget intent proven by W0 / Blender E2E.
/// Unknown serialized tags fail closed. Domain does not execute retargeting.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootPolicy {
    CopyWorldTranslationDelta,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelPolicy {
    RotationOnlyMappedNonRoot,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuaternionNormalizationPolicy {
    NormalizeBeforeKey,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuaternionContinuityPolicy {
    ConsecutiveHemisphere,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuaternionInterpolationPolicy {
    BackendInterpolationAfterNormalizedKeys,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeBakePolicy {
    EverySourceFrame,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestAlignmentPolicy {
    RestRelativeWorldDelta,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmappedTargetChannelPolicy {
    RemainAtTargetRest,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingSourceJointPolicy {
    FailClosedDoNotInvent,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissingChannelPolicy {
    unmapped_target_joints: UnmappedTargetChannelPolicy,
    missing_source_joint: MissingSourceJointPolicy,
}

impl MissingChannelPolicy {
    pub fn proven() -> Self {
        Self {
            unmapped_target_joints: UnmappedTargetChannelPolicy::RemainAtTargetRest,
            missing_source_joint: MissingSourceJointPolicy::FailClosedDoNotInvent,
        }
    }

    pub fn unmapped_target_joints(&self) -> UnmappedTargetChannelPolicy {
        self.unmapped_target_joints
    }

    pub fn missing_source_joint(&self) -> MissingSourceJointPolicy {
        self.missing_source_joint
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalePolicy {
    KeepTargetRestScale,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IkPolicy {
    ExplicitNoIk,
}

/// Distinct Policy contract states. V1-1 recognizes/supports/audits the proven
/// set and does not execute retargeting. Worker capability negotiation is V1-3.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyContractStatus {
    recognized: bool,
    supported: bool,
    executed: bool,
    audited: bool,
}

impl PolicyContractStatus {
    pub fn v1_1_proven_unexecuted() -> Self {
        Self {
            recognized: true,
            supported: true,
            executed: false,
            audited: true,
        }
    }

    pub fn recognized(&self) -> bool {
        self.recognized
    }
    pub fn supported(&self) -> bool {
        self.supported
    }
    pub fn executed(&self) -> bool {
        self.executed
    }
    pub fn audited(&self) -> bool {
        self.audited
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        let expected = Self::v1_1_proven_unexecuted();
        if *self != expected {
            return Err(DomainError::new(
                ErrorCode::InvalidRecordType,
                "V1-1 Policy contract status must be recognized/supported/audited and not executed",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetargetPolicy {
    schema_version: u32,
    record_type: RecordType,
    id: RetargetPolicyId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_version_id: Option<RetargetPolicyVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    draft_version_id: Option<RetargetPolicyVersionId>,
}

impl RetargetPolicy {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::RetargetPolicy,
            id: RetargetPolicyId::generate(),
            display_name: display_name.into(),
            published_version_id: None,
            draft_version_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> RetargetPolicyId {
        self.id
    }

    pub fn bind_published(&mut self, version_id: RetargetPolicyVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::RetargetPolicy)?;
        require_nonempty(&self.display_name, "display_name")
    }
}

impl DomainRecord for RetargetPolicy {
    const RECORD_TYPE: RecordType = RecordType::RetargetPolicy;
    fn validate(&self) -> Result<(), DomainError> {
        RetargetPolicy::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetargetPolicyVersion {
    schema_version: u32,
    record_type: RecordType,
    id: RetargetPolicyVersionId,
    policy_id: RetargetPolicyId,
    lifecycle: Lifecycle,
    root_policy: RootPolicy,
    channel_policy: ChannelPolicy,
    quaternion_normalization_policy: QuaternionNormalizationPolicy,
    quaternion_continuity_policy: QuaternionContinuityPolicy,
    quaternion_interpolation_policy: QuaternionInterpolationPolicy,
    time_bake_policy: TimeBakePolicy,
    rest_alignment_policy: RestAlignmentPolicy,
    missing_channel_policy: MissingChannelPolicy,
    scale_policy: ScalePolicy,
    ik_policy: IkPolicy,
    contract_status: PolicyContractStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    explanation: Option<String>,
}

impl RetargetPolicyVersion {
    pub fn proven_draft(policy_id: RetargetPolicyId) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::RetargetPolicyVersion,
            id: RetargetPolicyVersionId::generate(),
            policy_id,
            lifecycle: Lifecycle::Draft,
            root_policy: RootPolicy::CopyWorldTranslationDelta,
            channel_policy: ChannelPolicy::RotationOnlyMappedNonRoot,
            quaternion_normalization_policy: QuaternionNormalizationPolicy::NormalizeBeforeKey,
            quaternion_continuity_policy: QuaternionContinuityPolicy::ConsecutiveHemisphere,
            quaternion_interpolation_policy: QuaternionInterpolationPolicy::BackendInterpolationAfterNormalizedKeys,
            time_bake_policy: TimeBakePolicy::EverySourceFrame,
            rest_alignment_policy: RestAlignmentPolicy::RestRelativeWorldDelta,
            missing_channel_policy: MissingChannelPolicy::proven(),
            scale_policy: ScalePolicy::KeepTargetRestScale,
            ik_policy: IkPolicy::ExplicitNoIk,
            contract_status: PolicyContractStatus::v1_1_proven_unexecuted(),
            explanation: Some(
                "W0/E2E-proven rest-relative rotation-only transfer; no hidden corrective IK"
                    .to_string(),
            ),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> RetargetPolicyVersionId {
        self.id
    }
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn root_policy(&self) -> RootPolicy {
        self.root_policy
    }
    pub fn channel_policy(&self) -> ChannelPolicy {
        self.channel_policy
    }
    pub fn quaternion_normalization_policy(&self) -> QuaternionNormalizationPolicy {
        self.quaternion_normalization_policy
    }
    pub fn quaternion_continuity_policy(&self) -> QuaternionContinuityPolicy {
        self.quaternion_continuity_policy
    }
    pub fn ik_policy(&self) -> IkPolicy {
        self.ik_policy
    }
    pub fn contract_status(&self) -> PolicyContractStatus {
        self.contract_status
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::RetargetPolicyVersion)?;
        self.contract_status.validate()?;
        if let Some(explanation) = &self.explanation {
            require_nonempty(explanation, "explanation")?;
            assert_backend_neutral_text(explanation, "policy explanation")?;
        }
        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        self.lifecycle = freeze(self.lifecycle)?;
        Ok(())
    }

    pub fn invalidate(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        assert_lifecycle_transition(self.lifecycle, Lifecycle::Invalidated)?;
        self.lifecycle = Lifecycle::Invalidated;
        Ok(())
    }
}

impl DomainRecord for RetargetPolicyVersion {
    const RECORD_TYPE: RecordType = RecordType::RetargetPolicyVersion;
    fn validate(&self) -> Result<(), DomainError> {
        RetargetPolicyVersion::validate(self)
    }
}
