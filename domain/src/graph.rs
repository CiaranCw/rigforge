//! Exact-version provenance graph validation.

use crate::artifacts::{
    DerivedVariantVersion, PersistenceArtifact, PersistenceVerification, VerificationOutcome,
};
use crate::assets::{CharacterAssetVersion, MotionAssetVersion, SourceSkeletonReference};
use crate::backend::BackendExecutionContext;
use crate::error::{DomainError, ErrorCode};
use crate::execution::{JobSpec, QcReport, QcSubjectKind, QcVerdict, WorkerResult};
use crate::identity::ContentDigest;
use crate::mapping::{BoneMappingVersion, RetargetPolicyVersion};
use crate::record::usable_as_job_input;

pub struct PublicationEvidence<'a> {
    pub character_version: &'a CharacterAssetVersion,
    pub motion_version: &'a MotionAssetVersion,
    pub source_skeleton: &'a SourceSkeletonReference,
    pub mapping_version: &'a BoneMappingVersion,
    pub policy_version: &'a RetargetPolicyVersion,
    pub job: &'a JobSpec,
    pub worker: &'a WorkerResult,
    pub qc: &'a QcReport,
    pub backend: &'a BackendExecutionContext,
    pub persistence: Option<&'a PersistenceArtifact>,
    pub verification: Option<&'a PersistenceVerification>,
}

fn mismatch(message: impl Into<String>) -> DomainError {
    DomainError::new(ErrorCode::GraphMismatch, message)
}

fn missing(message: impl Into<String>) -> DomainError {
    DomainError::new(ErrorCode::PublicationEvidenceMissing, message)
}

pub fn validate_job_inputs(
    character: &CharacterAssetVersion,
    motion: &MotionAssetVersion,
    source_skeleton: &SourceSkeletonReference,
    mapping: &BoneMappingVersion,
    policy: &RetargetPolicyVersion,
    job: &JobSpec,
) -> Result<(), DomainError> {
    character.validate()?;
    motion.validate()?;
    source_skeleton.validate()?;
    mapping.validate()?;
    policy.validate()?;
    job.validate()?;
    usable_as_job_input(character.lifecycle())?;
    usable_as_job_input(motion.lifecycle())?;
    usable_as_job_input(mapping.lifecycle())?;
    usable_as_job_input(policy.lifecycle())?;
    if job.character_version_id() != character.id() {
        return Err(mismatch(
            "JobSpec.character_version_id must equal CharacterAssetVersion.id",
        ));
    }
    if job.motion_version_id() != motion.id() {
        return Err(mismatch(
            "JobSpec.motion_version_id must equal MotionAssetVersion.id",
        ));
    }
    if job.source_skeleton_ref_id() != source_skeleton.id() {
        return Err(mismatch(
            "JobSpec.source_skeleton_ref_id must equal SourceSkeletonReference.id",
        ));
    }
    if job.mapping_version_id() != mapping.id() {
        return Err(mismatch(
            "JobSpec.mapping_version_id must equal BoneMappingVersion.id",
        ));
    }
    if job.policy_version_id() != policy.id() {
        return Err(mismatch(
            "JobSpec.policy_version_id must equal RetargetPolicyVersion.id",
        ));
    }
    if mapping.target_character_version_id() != character.id() {
        return Err(mismatch(
            "BoneMappingVersion.target_character_version_id must equal CharacterAssetVersion.id",
        ));
    }
    if mapping.source_skeleton_ref_id() != source_skeleton.id() {
        return Err(mismatch(
            "BoneMappingVersion.source_skeleton_ref_id must equal SourceSkeletonReference.id",
        ));
    }
    if motion.source_skeleton_ref_id() != source_skeleton.id() {
        return Err(mismatch(
            "MotionAssetVersion.source_skeleton_ref_id must equal SourceSkeletonReference.id",
        ));
    }
    Ok(())
}

pub fn validate_execution_lineage(
    job: &JobSpec,
    worker: &WorkerResult,
    backend: &BackendExecutionContext,
    derived: &DerivedVariantVersion,
) -> Result<(), DomainError> {
    worker.validate()?;
    backend.validate()?;
    derived.validate()?;
    if worker.job_spec_id() != job.id() {
        return Err(mismatch("WorkerResult.job_spec_id must equal JobSpec.id"));
    }
    if derived.job_spec_id() != job.id() {
        return Err(mismatch(
            "DerivedVariantVersion.job_spec_id must equal JobSpec.id",
        ));
    }
    if worker.execution().id() != backend.id() {
        return Err(mismatch(
            "WorkerResult execution context must equal the supplied BackendExecutionContext",
        ));
    }
    if derived.backend_id() != backend.id() {
        return Err(mismatch(
            "DerivedVariantVersion.backend_id must equal the WorkerResult execution context",
        ));
    }
    if derived.worker_result_id() != worker.id() {
        return Err(mismatch(
            "DerivedVariantVersion.worker_result_id must equal WorkerResult.id",
        ));
    }
    if job.character_version_id() != derived.character_version_id() {
        return Err(mismatch(
            "JobSpec.character_version_id must equal DerivedVariantVersion.character_version_id",
        ));
    }
    if job.motion_version_id() != derived.motion_version_id() {
        return Err(mismatch(
            "JobSpec.motion_version_id must equal DerivedVariantVersion.motion_version_id",
        ));
    }
    if job.source_skeleton_ref_id() != derived.source_skeleton_ref_id() {
        return Err(mismatch(
            "JobSpec.source_skeleton_reference must equal DerivedVariantVersion.source_skeleton_reference",
        ));
    }
    if job.mapping_version_id() != derived.mapping_version_id() {
        return Err(mismatch(
            "JobSpec.mapping_version_id must equal DerivedVariantVersion.mapping_version_id",
        ));
    }
    if job.policy_version_id() != derived.policy_version_id() {
        return Err(mismatch(
            "JobSpec.policy_version_id must equal DerivedVariantVersion.policy_version_id",
        ));
    }
    Ok(())
}

fn staged_contains(worker: &WorkerResult, digest: &ContentDigest) -> bool {
    worker
        .staged_artifact_digests()
        .iter()
        .any(|staged| staged == digest)
}

pub fn validate_publication_lineage(
    evidence: &PublicationEvidence<'_>,
    derived: &DerivedVariantVersion,
) -> Result<(), DomainError> {
    validate_job_inputs(
        evidence.character_version,
        evidence.motion_version,
        evidence.source_skeleton,
        evidence.mapping_version,
        evidence.policy_version,
        evidence.job,
    )?;
    validate_execution_lineage(evidence.job, evidence.worker, evidence.backend, derived)?;
    evidence.qc.validate()?;
    if !evidence.worker.worker_success() {
        return Err(DomainError::new(
            ErrorCode::WorkerNotPublicationAuthority,
            "worker failure cannot publish a Derived Variant",
        ));
    }
    if evidence.qc.id() != derived.qc_report_id() {
        return Err(DomainError::new(
            ErrorCode::MissingProvenance,
            "publication requires the QCReport bound on DerivedVariantVersion",
        ));
    }
    if evidence.qc.verdict() != QcVerdict::Pass {
        return Err(DomainError::new(
            ErrorCode::WorkerNotPublicationAuthority,
            "QC FAIL cannot publish a Derived Variant",
        ));
    }
    if evidence.qc.subject_kind() != QcSubjectKind::DerivedVariantVersion
        || evidence.qc.subject_derived_variant_version_id() != Some(derived.id())
    {
        return Err(DomainError::new(
            ErrorCode::QcSubjectMismatch,
            "QC report is not bound to this DerivedVariantVersion",
        ));
    }
    if evidence.qc.worker_result_id() != Some(evidence.worker.id()) {
        return Err(mismatch(
            "QCReport.worker_result_id must equal WorkerResult.id",
        ));
    }
    if evidence.qc.policy_version_id() != derived.policy_version_id() {
        return Err(mismatch(
            "QCReport.policy_version_id must equal DerivedVariantVersion.policy_version_id",
        ));
    }
    let persistence = evidence
        .persistence
        .ok_or_else(|| missing("publication requires a Persistence Artifact instance"))?;
    persistence.validate()?;
    if derived.persistence_artifact_id() != Some(persistence.id()) {
        return Err(missing(
            "DerivedVariantVersion must bind the exact Persistence Artifact",
        ));
    }
    if persistence.bound_derived_variant_version_id() != derived.id() {
        return Err(mismatch(
            "PersistenceArtifact must bind this DerivedVariantVersion",
        ));
    }
    if persistence.producer_id() != evidence.backend.id() {
        return Err(mismatch(
            "PersistenceArtifact producer must equal the execution context",
        ));
    }
    if !staged_contains(evidence.worker, persistence.digest()) {
        return Err(missing(
            "WorkerResult must stage the Persistence Artifact payload digest",
        ));
    }
    let verification = evidence.verification.ok_or_else(|| {
        missing("publication requires persistence fresh-reopen and structural verification")
    })?;
    verification.validate()?;
    if verification.persistence_artifact_id() != persistence.id() {
        return Err(mismatch(
            "PersistenceVerification must bind the exact Persistence Artifact id",
        ));
    }
    if verification.persistence_artifact_instance_id() != persistence.instance_id() {
        return Err(mismatch(
            "PersistenceVerification must bind the exact Persistence Artifact instance",
        ));
    }
    if verification.payload_digest() != persistence.digest() {
        return Err(mismatch(
            "PersistenceVerification must bind the exact Persistence Artifact digest",
        ));
    }
    if verification.subject_derived_variant_version_id() != derived.id() {
        return Err(mismatch(
            "PersistenceVerification must bind this DerivedVariantVersion",
        ));
    }
    if verification.producer_id() != evidence.backend.id() {
        return Err(mismatch(
            "PersistenceVerification producer must equal the execution context",
        ));
    }
    match verification.fresh_reopen() {
        VerificationOutcome::Pass => {}
        VerificationOutcome::Missing => {
            return Err(missing("publication requires fresh-process reopen verification"));
        }
        VerificationOutcome::Fail => {
            return Err(DomainError::new(
                ErrorCode::VerificationFailed,
                "fresh-process reopen verification failed",
            ));
        }
    }
    match verification.structural_verification() {
        VerificationOutcome::Pass => {}
        VerificationOutcome::Missing => {
            return Err(missing("publication requires structural verification"));
        }
        VerificationOutcome::Fail => {
            return Err(DomainError::new(
                ErrorCode::VerificationFailed,
                "structural verification failed",
            ));
        }
    }
    Ok(())
}
