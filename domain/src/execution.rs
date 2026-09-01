//! JobSpec, WorkerResult, and QCReport. Worker success is not publication.

use crate::backend::{BackendExecutionContext, RequestedCapability};
use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    assert_backend_neutral_text, expect_record_type, expect_schema_version, BoneMappingVersionId,
    CharacterAssetVersionId, ContentDigest, DerivedVariantVersionId, JobSpecId, MotionAssetVersionId,
    PersistenceArtifactId, QcReportId, RecordType, RetargetPolicyVersionId, SkeletonSummaryId,
    SourceSkeletonReferenceId, WorkerResultId, SCHEMA_VERSION,
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedMeasurement {
    name: String,
    value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}

impl NamedMeasurement {
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        unit: Option<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            name: name.into(),
            value: value.into(),
            unit,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_nonempty(&self.name, "measurement name")?;
        require_nonempty(&self.value, "measurement value")?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    schema_version: u32,
    record_type: RecordType,
    id: JobSpecId,
    character_version_id: CharacterAssetVersionId,
    motion_version_id: MotionAssetVersionId,
    source_skeleton_ref_id: SourceSkeletonReferenceId,
    mapping_version_id: BoneMappingVersionId,
    policy_version_id: RetargetPolicyVersionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    character_skeleton_summary_id: Option<SkeletonSummaryId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    motion_skeleton_summary_id: Option<SkeletonSummaryId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    requested_capabilities: Vec<RequestedCapability>,
    determinism_context: String,
    isolation_limits: String,
}

impl JobSpec {
    pub fn new(
        character_version_id: CharacterAssetVersionId,
        motion_version_id: MotionAssetVersionId,
        source_skeleton_ref_id: SourceSkeletonReferenceId,
        mapping_version_id: BoneMappingVersionId,
        policy_version_id: RetargetPolicyVersionId,
        determinism_context: impl Into<String>,
        isolation_limits: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::JobSpec,
            id: JobSpecId::generate(),
            character_version_id,
            motion_version_id,
            source_skeleton_ref_id,
            mapping_version_id,
            policy_version_id,
            character_skeleton_summary_id: None,
            motion_skeleton_summary_id: None,
            requested_capabilities: Vec::new(),
            determinism_context: determinism_context.into(),
            isolation_limits: isolation_limits.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> JobSpecId {
        self.id
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
    pub fn requested_capabilities(&self) -> &[RequestedCapability] {
        &self.requested_capabilities
    }

    pub fn with_requested_capabilities(
        mut self,
        requested_capabilities: Vec<RequestedCapability>,
    ) -> Result<Self, DomainError> {
        self.requested_capabilities = requested_capabilities;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::JobSpec)?;
        require_nonempty(&self.determinism_context, "determinism_context")?;
        require_nonempty(&self.isolation_limits, "isolation_limits")?;
        assert_backend_neutral_text(&self.determinism_context, "determinism_context")?;
        assert_backend_neutral_text(&self.isolation_limits, "isolation_limits")?;
        Ok(())
    }
}

impl DomainRecord for JobSpec {
    const RECORD_TYPE: RecordType = RecordType::JobSpec;
    fn validate(&self) -> Result<(), DomainError> {
        JobSpec::validate(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerResult {
    schema_version: u32,
    record_type: RecordType,
    id: WorkerResultId,
    job_spec_id: JobSpecId,
    execution: BackendExecutionContext,
    worker_success: bool,
    terminal_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    measurements: Vec<NamedMeasurement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    declared_losses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    staged_artifact_digests: Vec<ContentDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    applied_mapping_projection_note: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<String>,
}

impl WorkerResult {
    pub fn new(
        job_spec_id: JobSpecId,
        execution: BackendExecutionContext,
        worker_success: bool,
        terminal_status: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::WorkerResult,
            id: WorkerResultId::generate(),
            job_spec_id,
            execution,
            worker_success,
            terminal_status: terminal_status.into(),
            measurements: Vec::new(),
            declared_losses: Vec::new(),
            staged_artifact_digests: Vec::new(),
            applied_mapping_projection_note: None,
            diagnostics: Vec::new(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> WorkerResultId {
        self.id
    }
    pub fn job_spec_id(&self) -> JobSpecId {
        self.job_spec_id
    }
    pub fn execution(&self) -> &BackendExecutionContext {
        &self.execution
    }
    pub fn worker_success(&self) -> bool {
        self.worker_success
    }
    pub fn staged_artifact_digests(&self) -> &[ContentDigest] {
        &self.staged_artifact_digests
    }
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn with_staged_artifact_digests(
        mut self,
        staged_artifact_digests: Vec<ContentDigest>,
    ) -> Result<Self, DomainError> {
        self.staged_artifact_digests = staged_artifact_digests;
        self.validate()?;
        Ok(self)
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<String>) -> Result<Self, DomainError> {
        self.diagnostics = diagnostics;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::WorkerResult)?;
        require_nonempty(&self.terminal_status, "terminal_status")?;
        self.execution.validate()?;
        for m in &self.measurements {
            m.validate()?;
        }
        for digest in &self.staged_artifact_digests {
            digest.validate()?;
        }
        Ok(())
    }

    pub fn implies_qc_pass(&self) -> bool {
        false
    }

    pub fn implies_publication(&self) -> bool {
        false
    }
}

impl DomainRecord for WorkerResult {
    const RECORD_TYPE: RecordType = RecordType::WorkerResult;
    fn validate(&self) -> Result<(), DomainError> {
        WorkerResult::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QcVerdict {
    Pass,
    Fail,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QcSubjectKind {
    DerivedVariantVersion,
    PersistenceArtifact,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QcReport {
    schema_version: u32,
    record_type: RecordType,
    id: QcReportId,
    subject_kind: QcSubjectKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject_derived_variant_version_id: Option<DerivedVariantVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject_persistence_artifact_id: Option<PersistenceArtifactId>,
    policy_version_id: RetargetPolicyVersionId,
    verdict: QcVerdict,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    measurements: Vec<NamedMeasurement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_result_id: Option<WorkerResultId>,
}

impl QcReport {
    pub fn for_derived_variant(
        subject_derived_variant_version_id: DerivedVariantVersionId,
        policy_version_id: RetargetPolicyVersionId,
        worker_result_id: WorkerResultId,
        verdict: QcVerdict,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::QcReport,
            id: QcReportId::generate(),
            subject_kind: QcSubjectKind::DerivedVariantVersion,
            subject_derived_variant_version_id: Some(subject_derived_variant_version_id),
            subject_persistence_artifact_id: None,
            policy_version_id,
            verdict,
            diagnostics: Vec::new(),
            measurements: Vec::new(),
            worker_result_id: Some(worker_result_id),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> QcReportId {
        self.id
    }
    pub fn subject_kind(&self) -> QcSubjectKind {
        self.subject_kind
    }
    pub fn subject_derived_variant_version_id(&self) -> Option<DerivedVariantVersionId> {
        self.subject_derived_variant_version_id
    }
    pub fn policy_version_id(&self) -> RetargetPolicyVersionId {
        self.policy_version_id
    }
    pub fn verdict(&self) -> QcVerdict {
        self.verdict
    }
    pub fn worker_result_id(&self) -> Option<WorkerResultId> {
        self.worker_result_id
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::QcReport)?;
        match self.subject_kind {
            QcSubjectKind::DerivedVariantVersion => {
                if self.subject_derived_variant_version_id.is_none()
                    || self.subject_persistence_artifact_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::SubjectUnion,
                        "QC subject DerivedVariantVersion must bind exactly that subject",
                    ));
                }
                if self.worker_result_id.is_none() {
                    return Err(DomainError::new(
                        ErrorCode::QcSubjectMismatch,
                        "worker-backed QC requires worker_result_id",
                    ));
                }
            }
            QcSubjectKind::PersistenceArtifact => {
                if self.subject_persistence_artifact_id.is_none()
                    || self.subject_derived_variant_version_id.is_some()
                {
                    return Err(DomainError::new(
                        ErrorCode::SubjectUnion,
                        "QC subject PersistenceArtifact must bind exactly that subject",
                    ));
                }
            }
        }
        for m in &self.measurements {
            m.validate()?;
        }
        Ok(())
    }
}

impl DomainRecord for QcReport {
    const RECORD_TYPE: RecordType = RecordType::QcReport;
    fn validate(&self) -> Result<(), DomainError> {
        QcReport::validate(self)
    }
}

pub fn assert_not_used_as_publication(worker: &WorkerResult) -> Result<(), DomainError> {
    let _ = worker;
    Err(DomainError::new(
        ErrorCode::WorkerNotPublicationAuthority,
        "WorkerResult success is not QC PASS and is not Derived Variant publication",
    ))
}
