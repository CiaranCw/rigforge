//! Mapping, Compatibility, and Retarget Policy. Product-owned, backend-neutral.

use std::collections::HashSet;

use crate::backend::assert_lifecycle_transition;
use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    assert_backend_neutral_text, expect_record_type, expect_schema_version, BoneMappingId,
    BoneMappingVersionId, CharacterAssetVersionId, CompatibilityResultId, JointKey, Lifecycle,
    MotionAssetVersionId, RecordType, RetargetPolicyId, RetargetPolicyVersionId,
    SkeletonSummaryId, SourceSkeletonReferenceId, SCHEMA_VERSION,
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

    pub fn source(&self) -> &JointRef {
        &self.source
    }

    pub fn target(&self) -> &JointRef {
        &self.target
    }

    pub fn participation(&self) -> JointParticipation {
        self.participation
    }

    pub fn role_profile(&self) -> Option<&str> {
        self.role_profile.as_deref()
    }

    pub fn evidence(&self) -> &str {
        &self.evidence
    }
}

/// Typed unmapped semantics. Reason text is explanation only, never decision authority.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmappedDisposition {
    Blocking,
    Optional,
    Helper,
}

impl UnmappedDisposition {
    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Blocking)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnmappedJoint {
    skeleton_side: SkeletonSide,
    joint_key: JointKey,
    reason: String,
    disposition: UnmappedDisposition,
}

impl UnmappedJoint {
    pub fn new(
        skeleton_side: SkeletonSide,
        joint_key: JointKey,
        reason: impl Into<String>,
        disposition: UnmappedDisposition,
    ) -> Result<Self, DomainError> {
        let value = Self {
            skeleton_side,
            joint_key,
            reason: reason.into(),
            disposition,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn skeleton_side(&self) -> SkeletonSide {
        self.skeleton_side
    }

    pub fn joint_key(&self) -> &JointKey {
        &self.joint_key
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn disposition(&self) -> UnmappedDisposition {
        self.disposition
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_nonempty(&self.reason, "unmapped reason")?;
        assert_backend_neutral_text(&self.reason, "unmapped reason")
    }
}

/// How an accepted or candidate Mapping was reviewed. Optional on historical
/// records. Automatic generation is never silent Product acceptance.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingReviewKind {
    AutomaticCandidate,
    AutomaticConfirmed,
    UserOverride,
    Manual,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingReviewProvenance {
    reviewed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ambiguities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review_kind: Option<MappingReviewKind>,
}

impl MappingReviewProvenance {
    pub fn new(reviewed: bool, method: Option<String>, ambiguities: Vec<String>) -> Result<Self, DomainError> {
        let value = Self {
            reviewed,
            method,
            ambiguities,
            review_kind: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn with_kind(
        reviewed: bool,
        method: Option<String>,
        ambiguities: Vec<String>,
        review_kind: MappingReviewKind,
    ) -> Result<Self, DomainError> {
        let value = Self {
            reviewed,
            method,
            ambiguities,
            review_kind: Some(review_kind),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn reviewed(&self) -> bool {
        self.reviewed
    }

    pub fn method(&self) -> Option<&str> {
        self.method.as_deref()
    }

    pub fn ambiguities(&self) -> &[String] {
        &self.ambiguities
    }

    pub fn review_kind(&self) -> Option<MappingReviewKind> {
        self.review_kind
    }

    pub fn confirmation_required(&self) -> bool {
        match self.review_kind {
            Some(MappingReviewKind::AutomaticCandidate) => true,
            Some(_) => !self.ambiguities.is_empty() && !self.reviewed,
            None => !self.reviewed,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if let Some(method) = &self.method {
            assert_backend_neutral_text(method, "mapping review method")?;
        }
        for item in &self.ambiguities {
            assert_backend_neutral_text(item, "mapping ambiguity")?;
        }
        if let Some(kind) = self.review_kind {
            match kind {
                MappingReviewKind::AutomaticCandidate if self.reviewed => {
                    return Err(DomainError::new(
                        ErrorCode::MappingTooThin,
                        "automatic_candidate review cannot claim Mapping was reviewed/accepted",
                    ));
                }
                MappingReviewKind::AutomaticConfirmed
                | MappingReviewKind::UserOverride
                | MappingReviewKind::Manual
                    if !self.reviewed =>
                {
                    return Err(DomainError::new(
                        ErrorCode::MappingTooThin,
                        "accepted Mapping review kinds require reviewed=true",
                    ));
                }
                _ => {}
            }
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

    pub fn published_version_id(&self) -> Option<BoneMappingVersionId> {
        self.published_version_id
    }

    pub fn draft_version_id(&self) -> Option<BoneMappingVersionId> {
        self.draft_version_id
    }

    pub fn bind_published(&mut self, version_id: BoneMappingVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn bind_draft(&mut self, version_id: BoneMappingVersionId) {
        self.draft_version_id = Some(version_id);
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_skeleton_summary_id: Option<SkeletonSummaryId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_skeleton_summary_id: Option<SkeletonSummaryId>,
    /// True when this version was created from automatic candidate generation.
    #[serde(default)]
    generated_from_candidates: bool,
    /// True when an explicit Product override/unmap edited this version after generation.
    #[serde(default)]
    user_modified: bool,
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
            source_skeleton_summary_id: None,
            target_skeleton_summary_id: None,
            generated_from_candidates: false,
            user_modified: false,
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
    pub fn unmapped_source(&self) -> &[UnmappedJoint] {
        &self.unmapped_source
    }
    pub fn unmapped_target(&self) -> &[UnmappedJoint] {
        &self.unmapped_target
    }
    pub fn source_motion_version_id(&self) -> Option<MotionAssetVersionId> {
        self.source_motion_version_id
    }
    pub fn mapping_id(&self) -> BoneMappingId {
        self.mapping_id
    }
    pub fn source_skeleton_summary_id(&self) -> Option<SkeletonSummaryId> {
        self.source_skeleton_summary_id
    }
    pub fn target_skeleton_summary_id(&self) -> Option<SkeletonSummaryId> {
        self.target_skeleton_summary_id
    }

    pub fn generated_from_candidates(&self) -> bool {
        self.generated_from_candidates
    }

    pub fn user_modified(&self) -> bool {
        self.user_modified
    }

    /// Accepted review kind implied by durable workflow history. Not caller-chosen.
    pub fn derived_accepted_kind(&self) -> MappingReviewKind {
        if !self.generated_from_candidates {
            MappingReviewKind::Manual
        } else if self.user_modified {
            MappingReviewKind::UserOverride
        } else {
            MappingReviewKind::AutomaticConfirmed
        }
    }

    pub fn mark_generated_from_candidates(&mut self) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.generated_from_candidates = true;
        self.validate()
    }

    pub fn mark_user_modified(&mut self) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.user_modified = true;
        self.validate()
    }

    pub fn bind_source_motion_version(
        &mut self,
        motion_version_id: MotionAssetVersionId,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.source_motion_version_id = Some(motion_version_id);
        self.validate()
    }

    pub fn clear_source_motion_version(&mut self) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.source_motion_version_id = None;
        self.validate()
    }

    pub fn set_unmapped(
        &mut self,
        unmapped_source: Vec<UnmappedJoint>,
        unmapped_target: Vec<UnmappedJoint>,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.unmapped_source = unmapped_source;
        self.unmapped_target = unmapped_target;
        self.validate()
    }

    pub fn bind_skeleton_summaries(
        &mut self,
        source_skeleton_summary_id: SkeletonSummaryId,
        target_skeleton_summary_id: SkeletonSummaryId,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.source_skeleton_summary_id = Some(source_skeleton_summary_id);
        self.target_skeleton_summary_id = Some(target_skeleton_summary_id);
        self.validate()
    }

    pub fn replace_entries_and_review(
        &mut self,
        entries: Vec<BoneMappingEntry>,
        review: MappingReviewProvenance,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.entries = entries;
        self.review = review;
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
        let mut sources = HashSet::new();
        let mut targets = HashSet::new();
        for entry in &self.entries {
            if !sources.insert(entry.source.joint_key().as_str()) {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "BoneMappingVersion cannot map one source JointKey to multiple targets",
                ));
            }
            if !targets.insert(entry.target.joint_key().as_str()) {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "BoneMappingVersion cannot map multiple sources onto one target JointKey",
                ));
            }
        }
        for u in &self.unmapped_source {
            u.validate()?;
            if u.skeleton_side() != SkeletonSide::Source {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "unmapped_source entries must have SkeletonSide::Source",
                ));
            }
        }
        for u in &self.unmapped_target {
            u.validate()?;
            if u.skeleton_side() != SkeletonSide::Target {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "unmapped_target entries must have SkeletonSide::Target",
                ));
            }
        }
        self.review.validate()?;
        self.validate_review_history()
    }

    fn validate_review_history(&self) -> Result<(), DomainError> {
        if self.lifecycle == Lifecycle::Published {
            if !self.review.reviewed {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "Published BoneMappingVersion requires reviewed=true",
                ));
            }
            let Some(kind) = self.review.review_kind else {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "Published BoneMappingVersion requires an accepted review_kind",
                ));
            };
            if kind == MappingReviewKind::AutomaticCandidate {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    "Published BoneMappingVersion cannot retain automatic_candidate review",
                ));
            }
            let expected = self.derived_accepted_kind();
            if kind != expected {
                return Err(DomainError::new(
                    ErrorCode::MappingTooThin,
                    format!(
                        "Published review_kind {kind:?} does not match workflow history (expected {expected:?})"
                    ),
                ));
            }
        }
        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        if !self.review.reviewed {
            return Err(DomainError::new(
                ErrorCode::MappingTooThin,
                "BoneMappingVersion::publish requires explicit reviewed Mapping provenance",
            ));
        }
        if matches!(
            self.review.review_kind,
            Some(MappingReviewKind::AutomaticCandidate)
        ) {
            return Err(DomainError::new(
                ErrorCode::MappingTooThin,
                "BoneMappingVersion::publish rejects automatic_candidate review",
            ));
        }
        if self.review.review_kind.is_none() {
            self.review.review_kind = Some(self.derived_accepted_kind());
        }
        self.lifecycle = freeze(self.lifecycle)?;
        self.validate()
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
    pub fn id(&self) -> CompatibilityResultId {
        self.id
    }
    pub fn character_version_id(&self) -> CharacterAssetVersionId {
        self.character_version_id
    }
    pub fn motion_version_id(&self) -> MotionAssetVersionId {
        self.motion_version_id
    }
    pub fn mapping_version_id(&self) -> BoneMappingVersionId {
        self.mapping_version_id
    }
    pub fn policy_version_id(&self) -> RetargetPolicyVersionId {
        self.policy_version_id
    }
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// Deterministic CompatibilitySummary. UI must not choose this value.
    ///
    /// Preflight-decision-critical: mapping_completeness, structural_compatibility,
    /// method_eligibility, motion_suitability.
    /// Post-execution-only: result_acceptability (UNKNOWN does not block Ready).
    pub fn derive_summary(
        mapping_completeness: Judgment,
        structural_compatibility: Judgment,
        method_eligibility: Judgment,
        motion_suitability: Judgment,
        result_acceptability: Judgment,
    ) -> CompatibilitySummary {
        let preflight = [
            mapping_completeness,
            structural_compatibility,
            method_eligibility,
            motion_suitability,
        ];
        if preflight.iter().any(|j| matches!(j, Judgment::Fail))
            || matches!(result_acceptability, Judgment::Fail)
        {
            return CompatibilitySummary::Unsupported;
        }
        if matches!(mapping_completeness, Judgment::Unknown) {
            return CompatibilitySummary::MappingConfirmationRequired;
        }
        if preflight.iter().any(|j| matches!(j, Judgment::PassWithWarnings))
            || matches!(structural_compatibility, Judgment::Unknown)
            || matches!(method_eligibility, Judgment::Unknown)
            || matches!(motion_suitability, Judgment::Unknown)
        {
            CompatibilitySummary::ReadyWithWarnings
        } else {
            CompatibilitySummary::Ready
        }
    }

    /// Construct from dimension judgments. Summary is derived, not caller-chosen.
    #[allow(clippy::too_many_arguments)]
    pub fn from_preflight(
        character_version_id: CharacterAssetVersionId,
        motion_version_id: MotionAssetVersionId,
        mapping_version_id: BoneMappingVersionId,
        policy_version_id: RetargetPolicyVersionId,
        mapping_completeness: Judgment,
        structural_compatibility: Judgment,
        method_eligibility: Judgment,
        motion_suitability: Judgment,
        result_acceptability: Judgment,
        notes: Vec<String>,
    ) -> Result<Self, DomainError> {
        let summary = Self::derive_summary(
            mapping_completeness,
            structural_compatibility,
            method_eligibility,
            motion_suitability,
            result_acceptability,
        );
        Self::new(
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
        )
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::CompatibilityResult)?;
        let expected = Self::derive_summary(
            self.mapping_completeness,
            self.structural_compatibility,
            self.method_eligibility,
            self.motion_suitability,
            self.result_acceptability,
        );
        if self.summary != expected {
            return Err(DomainError::new(
                ErrorCode::CompatibilityContradiction,
                format!(
                    "CompatibilitySummary must equal derive_summary(...); found {:?} expected {:?}",
                    self.summary, expected
                ),
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
    pub fn quaternion_interpolation_policy(&self) -> QuaternionInterpolationPolicy {
        self.quaternion_interpolation_policy
    }
    pub fn time_bake_policy(&self) -> TimeBakePolicy {
        self.time_bake_policy
    }
    pub fn rest_alignment_policy(&self) -> RestAlignmentPolicy {
        self.rest_alignment_policy
    }
    pub fn missing_channel_policy(&self) -> &MissingChannelPolicy {
        &self.missing_channel_policy
    }
    pub fn scale_policy(&self) -> ScalePolicy {
        self.scale_policy
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
