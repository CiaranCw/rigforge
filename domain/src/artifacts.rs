//! Derived Variant, Persistence Artifact, Preview Artifact, optional Export Artifact,
//! and persistence verification evidence.

use crate::backend::assert_lifecycle_transition;
use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    expect_record_type, expect_schema_version, BackendExecutionContextId, BoneMappingVersionId,
    CharacterAssetVersionId, ContentDigest, DerivedVariantId, DerivedVariantVersionId,
    ExportArtifactId, JobSpecId, Lifecycle, LocationEvidence, MotionAssetVersionId,
    PersistenceArtifactId, PersistenceArtifactInstanceId, PersistenceVerificationId, PreviewArtifactId,
    QcReportId, RecordType, RetargetPolicyVersionId, SourceSkeletonReferenceId, WorkerResultId,
    SCHEMA_VERSION,
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedVariant {
    schema_version: u32,
    record_type: RecordType,
    id: DerivedVariantId,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_version_id: Option<DerivedVariantVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    draft_version_id: Option<DerivedVariantVersionId>,
}

impl DerivedVariant {
    pub fn new(display_name: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::DerivedVariant,
            id: DerivedVariantId::generate(),
            display_name: display_name.into(),
            published_version_id: None,
            draft_version_id: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> DerivedVariantId {
        self.id
    }

    pub fn bind_published(&mut self, version_id: DerivedVariantVersionId) {
        self.published_version_id = Some(version_id);
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::DerivedVariant)?;
        require_nonempty(&self.display_name, "display_name")
    }
}

impl DomainRecord for DerivedVariant {
    const RECORD_TYPE: RecordType = RecordType::DerivedVariant;
    fn validate(&self) -> Result<(), DomainError> {
        DerivedVariant::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedVariantVersion {
    schema_version: u32,
    record_type: RecordType,
    id: DerivedVariantVersionId,
    variant_id: DerivedVariantId,
    lifecycle: Lifecycle,
    character_version_id: CharacterAssetVersionId,
    motion_version_id: MotionAssetVersionId,
    source_skeleton_ref_id: SourceSkeletonReferenceId,
    mapping_version_id: BoneMappingVersionId,
    policy_version_id: RetargetPolicyVersionId,
    job_spec_id: JobSpecId,
    backend_id: BackendExecutionContextId,
    worker_result_id: WorkerResultId,
    qc_report_id: QcReportId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    persistence_artifact_id: Option<PersistenceArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    persistence_verification_id: Option<PersistenceVerificationId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    preview_artifact_ids: Vec<PreviewArtifactId>,
}

impl DerivedVariantVersion {
    #[allow(clippy::too_many_arguments)]
    pub fn draft(
        variant_id: DerivedVariantId,
        character_version_id: CharacterAssetVersionId,
        motion_version_id: MotionAssetVersionId,
        source_skeleton_ref_id: SourceSkeletonReferenceId,
        mapping_version_id: BoneMappingVersionId,
        policy_version_id: RetargetPolicyVersionId,
        job_spec_id: JobSpecId,
        backend_id: BackendExecutionContextId,
        worker_result_id: WorkerResultId,
        qc_report_id: QcReportId,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::DerivedVariantVersion,
            id: DerivedVariantVersionId::generate(),
            variant_id,
            lifecycle: Lifecycle::Draft,
            character_version_id,
            motion_version_id,
            source_skeleton_ref_id,
            mapping_version_id,
            policy_version_id,
            job_spec_id,
            backend_id,
            worker_result_id,
            qc_report_id,
            persistence_artifact_id: None,
            persistence_verification_id: None,
            preview_artifact_ids: Vec::new(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> DerivedVariantVersionId {
        self.id
    }
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn character_version_id(&self) -> CharacterAssetVersionId {
        self.character_version_id
    }
    pub fn motion_version_id(&self) -> MotionAssetVersionId {
        self.motion_version_id
    }
    pub fn source_skeleton_ref_id(&self) -> SourceSkeletonReferenceId {
        self.source_skeleton_ref_id
    }
    pub fn mapping_version_id(&self) -> BoneMappingVersionId {
        self.mapping_version_id
    }
    pub fn policy_version_id(&self) -> RetargetPolicyVersionId {
        self.policy_version_id
    }
    pub fn job_spec_id(&self) -> JobSpecId {
        self.job_spec_id
    }
    pub fn backend_id(&self) -> BackendExecutionContextId {
        self.backend_id
    }
    pub fn worker_result_id(&self) -> WorkerResultId {
        self.worker_result_id
    }
    pub fn qc_report_id(&self) -> QcReportId {
        self.qc_report_id
    }
    pub fn persistence_artifact_id(&self) -> Option<PersistenceArtifactId> {
        self.persistence_artifact_id
    }
    pub fn persistence_verification_id(&self) -> Option<PersistenceVerificationId> {
        self.persistence_verification_id
    }

    pub fn bind_qc_report(&mut self, qc_report_id: QcReportId) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.qc_report_id = qc_report_id;
        self.validate()
    }

    pub fn bind_persistence_artifact(
        &mut self,
        persistence_artifact_id: PersistenceArtifactId,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.persistence_artifact_id = Some(persistence_artifact_id);
        self.validate()
    }

    pub fn bind_persistence_verification(
        &mut self,
        persistence_verification_id: PersistenceVerificationId,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.persistence_verification_id = Some(persistence_verification_id);
        self.validate()
    }

    pub fn bind_preview_artifacts(
        &mut self,
        preview_artifact_ids: Vec<PreviewArtifactId>,
    ) -> Result<(), DomainError> {
        self.lifecycle.assert_mutable()?;
        self.preview_artifact_ids = preview_artifact_ids;
        self.validate()
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::DerivedVariantVersion)?;
        if self.lifecycle == Lifecycle::Published && self.persistence_verification_id.is_none() {
            return Err(DomainError::new(
                ErrorCode::MissingProvenance,
                "Published DerivedVariantVersion must retain the exact PersistenceVerification that authorized publication",
            ));
        }
        Ok(())
    }

    pub(crate) fn freeze_published(&mut self) -> Result<(), DomainError> {
        self.validate()?;
        self.lifecycle = freeze(self.lifecycle)?;
        self.validate()
    }
}

impl DomainRecord for DerivedVariantVersion {
    const RECORD_TYPE: RecordType = RecordType::DerivedVariantVersion;
    fn validate(&self) -> Result<(), DomainError> {
        DerivedVariantVersion::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceArtifact {
    schema_version: u32,
    record_type: RecordType,
    id: PersistenceArtifactId,
    instance_id: PersistenceArtifactInstanceId,
    bound_derived_variant_version_id: DerivedVariantVersionId,
    digest: ContentDigest,
    size_bytes: u64,
    media_type: String,
    producer_id: BackendExecutionContextId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<LocationEvidence>,
}

impl PersistenceArtifact {
    pub fn new(
        bound_derived_variant_version_id: DerivedVariantVersionId,
        digest: ContentDigest,
        size_bytes: u64,
        media_type: impl Into<String>,
        producer_id: BackendExecutionContextId,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::PersistenceArtifact,
            id: PersistenceArtifactId::generate(),
            instance_id: PersistenceArtifactInstanceId::generate(),
            bound_derived_variant_version_id,
            digest,
            size_bytes,
            media_type: media_type.into(),
            producer_id,
            location: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> PersistenceArtifactId {
        self.id
    }
    pub fn instance_id(&self) -> PersistenceArtifactInstanceId {
        self.instance_id
    }
    pub fn bound_derived_variant_version_id(&self) -> DerivedVariantVersionId {
        self.bound_derived_variant_version_id
    }
    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }
    pub fn producer_id(&self) -> BackendExecutionContextId {
        self.producer_id
    }

    pub fn regenerate(&self, digest: ContentDigest, size_bytes: u64) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: self.schema_version,
            record_type: self.record_type,
            id: self.id,
            instance_id: PersistenceArtifactInstanceId::generate(),
            bound_derived_variant_version_id: self.bound_derived_variant_version_id,
            digest,
            size_bytes,
            media_type: self.media_type.clone(),
            producer_id: self.producer_id,
            location: self.location.clone(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::PersistenceArtifact)?;
        self.digest.validate()?;
        require_nonempty(&self.media_type, "media_type")?;
        if let Some(location) = &self.location {
            location.validate()?;
        }
        if self.id.canonical() == self.digest.sha256() {
            return Err(DomainError::new(
                ErrorCode::ArtifactProductConfusion,
                "PersistenceArtifact identity must not equal its payload digest",
            ));
        }
        Ok(())
    }
}

impl DomainRecord for PersistenceArtifact {
    const RECORD_TYPE: RecordType = RecordType::PersistenceArtifact;
    fn validate(&self) -> Result<(), DomainError> {
        PersistenceArtifact::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationOutcome {
    Pass,
    Fail,
    Missing,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceVerification {
    schema_version: u32,
    record_type: RecordType,
    id: PersistenceVerificationId,
    persistence_artifact_id: PersistenceArtifactId,
    persistence_artifact_instance_id: PersistenceArtifactInstanceId,
    payload_digest: ContentDigest,
    subject_derived_variant_version_id: DerivedVariantVersionId,
    producer_id: BackendExecutionContextId,
    fresh_reopen: VerificationOutcome,
    structural_verification: VerificationOutcome,
}

impl PersistenceVerification {
    pub fn new(
        persistence_artifact_id: PersistenceArtifactId,
        persistence_artifact_instance_id: PersistenceArtifactInstanceId,
        payload_digest: ContentDigest,
        subject_derived_variant_version_id: DerivedVariantVersionId,
        producer_id: BackendExecutionContextId,
        fresh_reopen: VerificationOutcome,
        structural_verification: VerificationOutcome,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::PersistenceVerification,
            id: PersistenceVerificationId::generate(),
            persistence_artifact_id,
            persistence_artifact_instance_id,
            payload_digest,
            subject_derived_variant_version_id,
            producer_id,
            fresh_reopen,
            structural_verification,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> PersistenceVerificationId {
        self.id
    }
    pub fn persistence_artifact_id(&self) -> PersistenceArtifactId {
        self.persistence_artifact_id
    }
    pub fn persistence_artifact_instance_id(&self) -> PersistenceArtifactInstanceId {
        self.persistence_artifact_instance_id
    }
    pub fn payload_digest(&self) -> &ContentDigest {
        &self.payload_digest
    }
    pub fn subject_derived_variant_version_id(&self) -> DerivedVariantVersionId {
        self.subject_derived_variant_version_id
    }
    pub fn producer_id(&self) -> BackendExecutionContextId {
        self.producer_id
    }
    pub fn fresh_reopen(&self) -> VerificationOutcome {
        self.fresh_reopen
    }
    pub fn structural_verification(&self) -> VerificationOutcome {
        self.structural_verification
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::PersistenceVerification)?;
        self.payload_digest.validate()?;
        Ok(())
    }
}

impl DomainRecord for PersistenceVerification {
    const RECORD_TYPE: RecordType = RecordType::PersistenceVerification;
    fn validate(&self) -> Result<(), DomainError> {
        PersistenceVerification::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
    CharacterAssetVersion,
    MotionAssetVersion,
    DerivedVariantVersion,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewArtifact {
    schema_version: u32,
    record_type: RecordType,
    id: PreviewArtifactId,
    bound_product_kind: ProductKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bound_character_version_id: Option<CharacterAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bound_motion_version_id: Option<MotionAssetVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bound_derived_variant_version_id: Option<DerivedVariantVersionId>,
    digest: ContentDigest,
    size_bytes: u64,
    media_type: String,
    producer_id: BackendExecutionContextId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<LocationEvidence>,
    derived: bool,
    rebuildable: bool,
    authoritative: bool,
}

impl PreviewArtifact {
    pub fn for_derived_variant(
        bound_derived_variant_version_id: DerivedVariantVersionId,
        digest: ContentDigest,
        size_bytes: u64,
        media_type: impl Into<String>,
        producer_id: BackendExecutionContextId,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::PreviewArtifact,
            id: PreviewArtifactId::generate(),
            bound_product_kind: ProductKind::DerivedVariantVersion,
            bound_character_version_id: None,
            bound_motion_version_id: None,
            bound_derived_variant_version_id: Some(bound_derived_variant_version_id),
            digest,
            size_bytes,
            media_type: media_type.into(),
            producer_id,
            location: None,
            derived: true,
            rebuildable: true,
            authoritative: false,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> PreviewArtifactId {
        self.id
    }
    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }
    pub fn derived(&self) -> bool {
        self.derived
    }
    pub fn rebuildable(&self) -> bool {
        self.rebuildable
    }
    pub fn authoritative(&self) -> bool {
        self.authoritative
    }
    pub fn bound_derived_variant_version_id(&self) -> Option<DerivedVariantVersionId> {
        self.bound_derived_variant_version_id
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::PreviewArtifact)?;
        if !self.derived || !self.rebuildable || self.authoritative {
            return Err(DomainError::new(
                ErrorCode::PreviewBinding,
                "PreviewArtifact must be derived, rebuildable, and non-authoritative",
            ));
        }
        self.digest.validate()?;
        require_nonempty(&self.media_type, "media_type")?;
        if self.id.canonical() == self.digest.sha256() {
            return Err(DomainError::new(
                ErrorCode::ArtifactProductConfusion,
                "PreviewArtifact identity must not equal its payload digest",
            ));
        }
        match self.bound_product_kind {
            ProductKind::CharacterAssetVersion => {
                if self.bound_character_version_id.is_none()
                    || self.bound_motion_version_id.is_some()
                    || self.bound_derived_variant_version_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::PreviewBinding,
                        "character PreviewArtifact must bind exactly one CharacterAssetVersion",
                    ));
                }
            }
            ProductKind::MotionAssetVersion => {
                if self.bound_motion_version_id.is_none()
                    || self.bound_character_version_id.is_some()
                    || self.bound_derived_variant_version_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::PreviewBinding,
                        "motion PreviewArtifact must bind exactly one MotionAssetVersion",
                    ));
                }
            }
            ProductKind::DerivedVariantVersion => {
                if self.bound_derived_variant_version_id.is_none()
                    || self.bound_character_version_id.is_some()
                    || self.bound_motion_version_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::PreviewBinding,
                        "derived PreviewArtifact must bind exactly one DerivedVariantVersion",
                    ));
                }
            }
        }
        if let Some(location) = &self.location {
            location.validate()?;
        }
        Ok(())
    }

    pub fn assert_bound_derived_variant(
        &self,
        expected: DerivedVariantVersionId,
    ) -> Result<(), DomainError> {
        self.validate()?;
        if self.bound_derived_variant_version_id != Some(expected) {
            return Err(DomainError::new(
                ErrorCode::PreviewBinding,
                "PreviewArtifact is bound to the wrong DerivedVariantVersion",
            ));
        }
        Ok(())
    }
}

impl DomainRecord for PreviewArtifact {
    const RECORD_TYPE: RecordType = RecordType::PreviewArtifact;
    fn validate(&self) -> Result<(), DomainError> {
        PreviewArtifact::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportArtifact {
    schema_version: u32,
    record_type: RecordType,
    id: ExportArtifactId,
    bound_derived_variant_version_id: DerivedVariantVersionId,
    digest: ContentDigest,
    size_bytes: u64,
    media_type: String,
    producer_id: BackendExecutionContextId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<LocationEvidence>,
}

impl ExportArtifact {
    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::ExportArtifact)?;
        self.digest.validate()?;
        require_nonempty(&self.media_type, "media_type")?;
        if let Some(location) = &self.location {
            location.validate()?;
        }
        Ok(())
    }
}

impl DomainRecord for ExportArtifact {
    const RECORD_TYPE: RecordType = RecordType::ExportArtifact;
    fn validate(&self) -> Result<(), DomainError> {
        ExportArtifact::validate(self)
    }
}
