//! Character, Motion, Source Skeleton, and derived Skeleton Summary.

use crate::backend::{assert_lifecycle_transition, BackendExecutionContext};
use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    expect_record_type, expect_schema_version, CharacterAssetId, CharacterAssetVersionId, JointKey,
    Lifecycle, MotionAssetId, MotionAssetVersionId, RecordType, SkeletonSummaryId,
    SourceSkeletonReferenceId, SCHEMA_VERSION,
};
use crate::record::DomainRecord;
use crate::source::SourceArtifactEvidence;
use crate::time::TimeDomainProvenance;
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterAsset {
    schema_version: u32,
    record_type: RecordType,
    id: CharacterAssetId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_version_id: Option<CharacterAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    draft_version_id: Option<CharacterAssetVersionId>,
}

impl CharacterAsset {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::CharacterAsset,
            id: CharacterAssetId::generate(),
            display_name: display_name.into(),
            published_version_id: None,
            draft_version_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> CharacterAssetId {
        self.id
    }

    pub fn bind_published(&mut self, version_id: CharacterAssetVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::CharacterAsset)?;
        require_nonempty(&self.display_name, "display_name")
    }
}

impl DomainRecord for CharacterAsset {
    const RECORD_TYPE: RecordType = RecordType::CharacterAsset;
    fn validate(&self) -> Result<(), DomainError> {
        CharacterAsset::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterAssetVersion {
    schema_version: u32,
    record_type: RecordType,
    id: CharacterAssetVersionId,
    asset_id: CharacterAssetId,
    lifecycle: Lifecycle,
    display_name: String,
    source: SourceArtifactEvidence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skeleton_summary_id: Option<SkeletonSummaryId>,
}

impl CharacterAssetVersion {
    pub fn draft(
        asset_id: CharacterAssetId,
        display_name: impl Into<String>,
        source: SourceArtifactEvidence,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::CharacterAssetVersion,
            id: CharacterAssetVersionId::generate(),
            asset_id,
            lifecycle: Lifecycle::Draft,
            display_name: display_name.into(),
            source,
            skeleton_summary_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> CharacterAssetVersionId {
        self.id
    }
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub fn source(&self) -> &SourceArtifactEvidence {
        &self.source
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::CharacterAssetVersion)?;
        require_nonempty(&self.display_name, "display_name")?;
        self.source.validate()
    }

    pub fn try_set_display_name(&mut self, display_name: impl Into<String>) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.display_name = display_name.into();
        self.validate()
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

impl DomainRecord for CharacterAssetVersion {
    const RECORD_TYPE: RecordType = RecordType::CharacterAssetVersion;
    fn validate(&self) -> Result<(), DomainError> {
        CharacterAssetVersion::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionAsset {
    schema_version: u32,
    record_type: RecordType,
    id: MotionAssetId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_version_id: Option<MotionAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    draft_version_id: Option<MotionAssetVersionId>,
}

impl MotionAsset {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::MotionAsset,
            id: MotionAssetId::generate(),
            display_name: display_name.into(),
            published_version_id: None,
            draft_version_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> MotionAssetId {
        self.id
    }

    pub fn bind_published(&mut self, version_id: MotionAssetVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::MotionAsset)?;
        require_nonempty(&self.display_name, "display_name")
    }
}

impl DomainRecord for MotionAsset {
    const RECORD_TYPE: RecordType = RecordType::MotionAsset;
    fn validate(&self) -> Result<(), DomainError> {
        MotionAsset::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionAssetVersion {
    schema_version: u32,
    record_type: RecordType,
    id: MotionAssetVersionId,
    asset_id: MotionAssetId,
    lifecycle: Lifecycle,
    display_name: String,
    source_skeleton_ref_id: SourceSkeletonReferenceId,
    time: TimeDomainProvenance,
    source: SourceArtifactEvidence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skeleton_summary_id: Option<SkeletonSummaryId>,
}

impl MotionAssetVersion {
    pub fn draft(
        asset_id: MotionAssetId,
        display_name: impl Into<String>,
        source_skeleton_ref_id: SourceSkeletonReferenceId,
        time: TimeDomainProvenance,
        source: SourceArtifactEvidence,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::MotionAssetVersion,
            id: MotionAssetVersionId::generate(),
            asset_id,
            lifecycle: Lifecycle::Draft,
            display_name: display_name.into(),
            source_skeleton_ref_id,
            time,
            source,
            skeleton_summary_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> MotionAssetVersionId {
        self.id
    }
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn source_skeleton_ref_id(&self) -> SourceSkeletonReferenceId {
        self.source_skeleton_ref_id
    }
    pub fn source(&self) -> &SourceArtifactEvidence {
        &self.source
    }
    pub fn time(&self) -> &TimeDomainProvenance {
        &self.time
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::MotionAssetVersion)?;
        require_nonempty(&self.display_name, "display_name")?;
        self.time.validate()?;
        self.source.validate()
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

impl DomainRecord for MotionAssetVersion {
    const RECORD_TYPE: RecordType = RecordType::MotionAssetVersion;
    fn validate(&self) -> Result<(), DomainError> {
        MotionAssetVersion::validate(self)
    }
}

/// Immutable Source Skeleton binding record (V1-1 option A).
/// Once constructed, historical meaning cannot be mutated in place.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSkeletonReference {
    schema_version: u32,
    record_type: RecordType,
    id: SourceSkeletonReferenceId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    originating_character_version_id: Option<CharacterAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<SourceArtifactEvidence>,
}

impl SourceSkeletonReference {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::SourceSkeletonReference,
            id: SourceSkeletonReferenceId::generate(),
            display_name: display_name.into(),
            originating_character_version_id: None,
            source: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> SourceSkeletonReferenceId {
        self.id
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::SourceSkeletonReference)?;
        require_nonempty(&self.display_name, "display_name")?;
        if let Some(source) = &self.source {
            source.validate()?;
        }
        Ok(())
    }
}

impl DomainRecord for SourceSkeletonReference {
    const RECORD_TYPE: RecordType = RecordType::SourceSkeletonReference;
    fn validate(&self) -> Result<(), DomainError> {
        SourceSkeletonReference::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonSubjectKind {
    CharacterAssetVersion,
    SourceSkeletonReference,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointObservation {
    joint_key: JointKey,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_key: Option<JointKey>,
    is_root: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deform_observation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rest_evidence: Option<String>,
}

impl JointObservation {
    pub fn new(
        joint_key: JointKey,
        display_name: impl Into<String>,
        parent_key: Option<JointKey>,
        is_root: bool,
        deform_observation: Option<String>,
        rest_evidence: Option<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            joint_key,
            display_name: display_name.into(),
            parent_key,
            is_root,
            deform_observation,
            rest_evidence,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_nonempty(&self.display_name, "joint display_name")
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonSummary {
    schema_version: u32,
    record_type: RecordType,
    id: SkeletonSummaryId,
    subject_kind: SkeletonSubjectKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject_character_version_id: Option<CharacterAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject_source_skeleton_ref_id: Option<SourceSkeletonReferenceId>,
    producer: BackendExecutionContext,
    joints: Vec<JointObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<String>,
}

impl SkeletonSummary {
    pub fn new(
        subject_kind: SkeletonSubjectKind,
        subject_character_version_id: Option<CharacterAssetVersionId>,
        subject_source_skeleton_ref_id: Option<SourceSkeletonReferenceId>,
        producer: BackendExecutionContext,
        joints: Vec<JointObservation>,
        diagnostics: Vec<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::SkeletonSummary,
            id: SkeletonSummaryId::generate(),
            subject_kind,
            subject_character_version_id,
            subject_source_skeleton_ref_id,
            producer,
            joints,
            diagnostics,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::SkeletonSummary)?;
        self.producer.validate()?;
        match self.subject_kind {
            SkeletonSubjectKind::CharacterAssetVersion => {
                if self.subject_character_version_id.is_none()
                    || self.subject_source_skeleton_ref_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::SubjectUnion,
                        "character SkeletonSummary must bind exactly one CharacterAssetVersion",
                    ));
                }
            }
            SkeletonSubjectKind::SourceSkeletonReference => {
                if self.subject_source_skeleton_ref_id.is_none()
                    || self.subject_character_version_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::SubjectUnion,
                        "source SkeletonSummary must bind exactly one SourceSkeletonReference",
                    ));
                }
            }
        }
        for joint in &self.joints {
            joint.validate()?;
        }
        Ok(())
    }
}

impl DomainRecord for SkeletonSummary {
    const RECORD_TYPE: RecordType = RecordType::SkeletonSummary;
    fn validate(&self) -> Result<(), DomainError> {
        SkeletonSummary::validate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ContentDigest;
    use crate::json::deserialize_untrusted;
    use crate::source::SourceArtifactEvidence;
    use crate::time::{SamplingInterpretation, TimeDomainProvenance, TimePoint};

    fn source() -> SourceArtifactEvidence {
        SourceArtifactEvidence::new(
            crate::identity::LocationEvidence::filesystem_path("C:/research/knight.bin").unwrap(),
            ContentDigest::parse(&"1".repeat(64)).unwrap(),
            1024,
            "application/octet-stream",
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn invalid_draft_cannot_publish() {
        let character = CharacterAsset::new("Knight").unwrap();
        let valid = CharacterAssetVersion::draft(character.id(), "Knight v1", source()).unwrap();
        let mut value = serde_json::to_value(&valid).unwrap();
        value["display_name"] = serde_json::json!("");
        let text = serde_json::to_string(&value).unwrap();
        let mut invalid: CharacterAssetVersion = deserialize_untrusted(&text).unwrap();
        assert_eq!(invalid.lifecycle(), Lifecycle::Draft);
        assert!(invalid.validate().is_err());
        let err = invalid.publish().unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingRequiredField);
        assert_eq!(invalid.lifecycle(), Lifecycle::Draft);
    }

    #[test]
    fn published_cannot_return_to_draft() {
        let character = CharacterAsset::new("Knight").unwrap();
        let mut version = CharacterAssetVersion::draft(character.id(), "Knight v1", source()).unwrap();
        version.publish().unwrap();
        let err = crate::backend::assert_lifecycle_transition(version.lifecycle(), Lifecycle::Draft)
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidLifecycle);
    }

    #[test]
    fn motion_binds_immutable_source_skeleton_id() {
        let skeleton = SourceSkeletonReference::new("UAL2").unwrap();
        let motion = MotionAsset::new("Walk").unwrap();
        let time = TimeDomainProvenance::new(
            "clip:walk",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(10, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap();
        let version = MotionAssetVersion::draft(
            motion.id(),
            "Walk v1",
            skeleton.id(),
            time,
            source(),
        )
        .unwrap();
        assert_eq!(version.source_skeleton_ref_id(), skeleton.id());
        assert_eq!(skeleton.display_name(), "UAL2");
    }
}
