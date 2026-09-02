//! V1-5 Transfer authorization, candidate lifecycle, and publication.

use std::path::Path;

use rigforge_domain::{
    publish_derived_variant, BoneMappingVersion, CompatibilityResult, CompatibilitySummary,
    DerivedVariant, DerivedVariantVersion, JobSpec, Lifecycle, PersistenceVerification,
    PublicationEvidence, QcReport, Validated, VerificationOutcome,
};

use crate::artifact::promote_staged_artifact;
use crate::error::AppError;
use crate::orchestration::{JobRun, JobRunState};
use crate::pinned_qc::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact,
};
use crate::qc::evaluate_structural_qc;
use crate::Application;

#[cfg(any(test, feature = "test-support"))]
use crate::qc::{ArtifactInspector, PersistenceReopener};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferAuthorization {
    pub compatibility_id: String,
    pub summary: CompatibilitySummary,
    pub eligible: bool,
    pub requires_acknowledgement: bool,
    pub denial_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferOutcomeKind {
    Published,
    PublicationDenied,
}

#[derive(Clone, Debug)]
pub struct TransferOutcome {
    pub kind: TransferOutcomeKind,
    pub job_spec_id: String,
    pub job_run_id: String,
    pub derived_variant_id: String,
    pub derived_variant_version_id: String,
    pub persistence_artifact_id: Option<String>,
    pub qc_report_id: Option<String>,
    pub qc_verdict: Option<String>,
    pub persistence_verification_id: Option<String>,
    pub reason: Option<String>,
}

impl Application {
    pub fn authorize_transfer(
        &self,
        compatibility_result_id: &str,
        warnings_acknowledged: bool,
    ) -> Result<TransferAuthorization, AppError> {
        let result = self
            .catalog
            .load_compatibility_result(compatibility_result_id)?
            .into_record();
        transfer_eligibility(&result, warnings_acknowledged)
    }

    pub fn start_transfer(
        &mut self,
        compatibility_result_id: &str,
        warnings_acknowledged: bool,
        existing_derived_variant_id: Option<&str>,
        display_name: impl Into<String>,
    ) -> Result<(Validated<JobSpec>, JobRun), AppError> {
        let auth = self.authorize_transfer(compatibility_result_id, warnings_acknowledged)?;
        if !auth.eligible {
            return Err(AppError::Catalog(
                auth.denial_reason
                    .unwrap_or_else(|| "Transfer is not eligible".into()),
            ));
        }
        let result = self
            .catalog
            .load_compatibility_result(compatibility_result_id)?
            .into_record();
        let character = self
            .catalog
            .load_character_version(&result.character_version_id().canonical())?
            .into_record();
        let motion = self
            .catalog
            .load_motion_version(&result.motion_version_id().canonical())?
            .into_record();
        let mapping = self
            .catalog
            .load_mapping_version(&result.mapping_version_id().canonical())?
            .into_record();
        let policy = self
            .catalog
            .load_policy_version(&result.policy_version_id().canonical())?
            .into_record();
        validate_transfer_graph(&result, &character, &motion, &mapping, &policy)?;
        let logical = match existing_derived_variant_id {
            Some(id) => self.catalog.load_derived_variant(id)?.into_record(),
            None => {
                let created = Validated::certify(DerivedVariant::new(display_name)?)?;
                self.catalog.put_validated(&created)?;
                created.into_record()
            }
        };
        let spec = JobSpec::new(
            result.character_version_id(),
            result.motion_version_id(),
            motion.source_skeleton_ref_id(),
            result.mapping_version_id(),
            result.policy_version_id(),
            "semantic-consistency; not byte-identical",
            "one isolated process; staged output unpublished on error",
        )?
        .with_compatibility_authorization(result.id(), warnings_acknowledged)?
        .with_target_derived_variant(logical.id())?;
        let spec = Validated::certify(spec)?;
        let run = self.catalog.enqueue_job(spec.clone())?;
        Ok((spec, run))
    }

    pub fn ingest_worker_success_candidate(
        &mut self,
        run_id: &str,
        staged_path: &Path,
    ) -> Result<(Validated<DerivedVariant>, Validated<DerivedVariantVersion>), AppError> {
        let run = self.catalog.load_job_run(run_id)?;
        if run.state != JobRunState::Succeeded {
            return Err(AppError::Catalog(
                "worker success candidate requires JobRun SUCCEEDED".into(),
            ));
        }
        let spec = self.catalog.load_job_spec(&run.job_spec_id)?.into_record();
        let worker_id = run.worker_result_id.clone().ok_or_else(|| {
            AppError::Catalog("SUCCEEDED JobRun is missing worker_result_id".into())
        })?;
        let worker = self.catalog.load_worker_result(&worker_id)?.into_record();
        if !worker.worker_success() {
            return Err(AppError::Catalog(
                "WorkerResult.worker_success is false; no Derived Variant publication".into(),
            ));
        }
        if let Some(existing) = self
            .catalog
            .find_derived_version_id_for_worker_result(&worker.id().canonical())?
        {
            return Err(AppError::Catalog(format!(
                "WorkerResult {} already created DerivedVariantVersion {existing}",
                worker.id().canonical()
            )));
        }
        let target_id = spec.target_derived_variant_id().ok_or_else(|| {
            AppError::Catalog(
                "candidate ingestion requires JobSpec target DerivedVariant".into(),
            )
        })?;
        let staged_digest = worker
            .staged_artifact_digests()
            .first()
            .ok_or_else(|| AppError::Catalog("WorkerResult has no staged artifact digest".into()))?;
        let logical = self
            .catalog
            .load_derived_variant(&target_id.canonical())?
            .into_record();
        let mut version = DerivedVariantVersion::draft(
            logical.id(),
            spec.character_version_id(),
            spec.motion_version_id(),
            spec.source_skeleton_ref_id(),
            spec.mapping_version_id(),
            spec.policy_version_id(),
            spec.id(),
            worker.execution().id(),
            worker.id(),
        )?;
        let (artifact, dest) = promote_staged_artifact(
            staged_path,
            staged_digest.sha256(),
            self.catalog.artifact_root(),
            version.id(),
            worker.execution().id(),
        )?;
        version.bind_persistence_artifact(artifact.as_record().id())?;
        let version = Validated::certify(version)?;
        let location = crate::catalog::CatalogLocationEvidence {
            product_id: artifact.as_record().id().canonical(),
            instance_id: artifact.as_record().instance_id().canonical(),
            location_kind: "filesystem_path_evidence".into(),
            location_value: dest.display().to_string(),
            observed_at: crate::migrate::now_ms(),
            note: Some("durable Persistence Artifact; not Product identity".into()),
        };
        self.catalog
            .persist_candidate_graph(&run.run_id, &version, &artifact, location)
    }

    pub fn evaluate_and_bind_qc(
        &mut self,
        derived_version_id: &str,
    ) -> Result<Validated<QcReport>, AppError> {
        let version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("QC requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let path = self.catalog.durable_artifact_path(&artifact);
        let mapping = self
            .catalog
            .load_mapping_version(&version.mapping_version_id().canonical())?
            .into_record();
        let evidence = inspect_durable_persistence_artifact(
            &path,
            artifact.digest().sha256(),
            Some(&mapping),
        )?;
        self.bind_qc_from_evidence(derived_version_id, evidence)
    }

    pub fn verify_and_bind_persistence(
        &mut self,
        derived_version_id: &str,
    ) -> Result<Validated<PersistenceVerification>, AppError> {
        let version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("persistence verification requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let path = self.catalog.durable_artifact_path(&artifact);
        let mapping = self
            .catalog
            .load_mapping_version(&version.mapping_version_id().canonical())?
            .into_record();
        let (fresh, structural) = reopen_durable_persistence_artifact(
            &path,
            artifact.digest().sha256(),
            &mapping,
        )?;
        self.bind_verification_from_outcomes(derived_version_id, fresh, structural)
    }

    fn bind_qc_from_evidence(
        &mut self,
        derived_version_id: &str,
        evidence: crate::qc::ArtifactInspectionEvidence,
    ) -> Result<Validated<QcReport>, AppError> {
        let mut version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("QC requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let mapping = self
            .catalog
            .load_mapping_version(&version.mapping_version_id().canonical())?
            .into_record();
        let motion = self
            .catalog
            .load_motion_version(&version.motion_version_id().canonical())?
            .into_record();
        let checks = evaluate_structural_qc(&evidence, &mapping, &motion)?;
        let qc = QcReport::for_derived_variant(
            version.id(),
            version.policy_version_id(),
            version.worker_result_id(),
            artifact.id(),
            artifact.instance_id(),
            artifact.digest().clone(),
            checks,
        )?;
        version.bind_qc_report(qc.id())?;
        let qc = Validated::certify(qc)?;
        self.catalog.persist_trusted_qc_binding(&qc)?;
        Ok(qc)
    }

    fn bind_verification_from_outcomes(
        &mut self,
        derived_version_id: &str,
        fresh: VerificationOutcome,
        structural: VerificationOutcome,
    ) -> Result<Validated<PersistenceVerification>, AppError> {
        let mut version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("persistence verification requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let verification = PersistenceVerification::new(
            artifact.id(),
            artifact.instance_id(),
            artifact.digest().clone(),
            version.id(),
            artifact.producer_id(),
            fresh,
            structural,
        )?;
        version.bind_persistence_verification(verification.id())?;
        let verification = Validated::certify(verification)?;
        self.catalog.persist_trusted_verification_binding(&verification)?;
        Ok(verification)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn evaluate_and_bind_qc_for_test<I: ArtifactInspector>(
        &mut self,
        derived_version_id: &str,
        inspector: &I,
    ) -> Result<Validated<QcReport>, AppError> {
        let version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("QC requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let path = self.catalog.durable_artifact_path(&artifact);
        let evidence = inspector.inspect(&path, artifact.digest().sha256())?;
        self.bind_qc_from_evidence(derived_version_id, evidence)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn verify_and_bind_persistence_for_test<R: PersistenceReopener>(
        &mut self,
        derived_version_id: &str,
        reopener: &R,
    ) -> Result<Validated<PersistenceVerification>, AppError> {
        let version = self
            .catalog
            .load_derived_variant_version(derived_version_id)?
            .into_record();
        let artifact_id = version.persistence_artifact_id().ok_or_else(|| {
            AppError::Catalog("persistence verification requires a bound PersistenceArtifact".into())
        })?;
        let artifact = self
            .catalog
            .load_artifact_metadata(&artifact_id.canonical())?
            .into_record();
        let path = self.catalog.durable_artifact_path(&artifact);
        let (fresh, structural) = reopener.reopen(&path, artifact.digest().sha256())?;
        self.bind_verification_from_outcomes(derived_version_id, fresh, structural)
    }

    pub fn publish_derived_variant_version(
        &mut self,
        derived_version_id: &str,
    ) -> Result<Validated<DerivedVariantVersion>, AppError> {
        self.catalog
            .publish_derived_variant_transaction(derived_version_id)
    }

    pub fn finalize_transfer(
        &mut self,
        run_id: &str,
        staged_path: &Path,
    ) -> Result<TransferOutcome, AppError> {
        let run = self.catalog.load_job_run(run_id)?;
        if run.state != JobRunState::Succeeded {
            return Ok(denied_not_succeeded(run));
        }
        let (logical, version) = self.ingest_worker_success_candidate(run_id, staged_path)?;
        let qc = self.evaluate_and_bind_qc(&version.as_record().id().canonical())?;
        let verification =
            self.verify_and_bind_persistence(&version.as_record().id().canonical())?;
        Ok(self.complete_finalize(run, logical, version, qc, verification))
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn finalize_transfer_for_test<I, R>(
        &mut self,
        run_id: &str,
        staged_path: &Path,
        inspector: &I,
        reopener: &R,
    ) -> Result<TransferOutcome, AppError>
    where
        I: ArtifactInspector,
        R: PersistenceReopener,
    {
        let run = self.catalog.load_job_run(run_id)?;
        if run.state != JobRunState::Succeeded {
            return Ok(denied_not_succeeded(run));
        }
        let (logical, version) = self.ingest_worker_success_candidate(run_id, staged_path)?;
        let qc = self.evaluate_and_bind_qc_for_test(
            &version.as_record().id().canonical(),
            inspector,
        )?;
        let verification = self.verify_and_bind_persistence_for_test(
            &version.as_record().id().canonical(),
            reopener,
        )?;
        Ok(self.complete_finalize(run, logical, version, qc, verification))
    }

    fn complete_finalize(
        &mut self,
        run: JobRun,
        logical: Validated<DerivedVariant>,
        version: Validated<DerivedVariantVersion>,
        qc: Validated<QcReport>,
        verification: Validated<PersistenceVerification>,
    ) -> TransferOutcome {
        if qc.as_record().verdict() != rigforge_domain::QcVerdict::Pass
            || verification.as_record().fresh_reopen() != VerificationOutcome::Pass
            || verification.as_record().structural_verification() != VerificationOutcome::Pass
        {
            return TransferOutcome {
                kind: TransferOutcomeKind::PublicationDenied,
                job_spec_id: run.job_spec_id,
                job_run_id: run.run_id,
                derived_variant_id: logical.as_record().id().canonical(),
                derived_variant_version_id: version.as_record().id().canonical(),
                persistence_artifact_id: version
                    .as_record()
                    .persistence_artifact_id()
                    .map(|id| id.canonical()),
                qc_report_id: Some(qc.as_record().id().canonical()),
                qc_verdict: Some(format!("{:?}", qc.as_record().verdict())),
                persistence_verification_id: Some(verification.as_record().id().canonical()),
                reason: Some("QC or persistence verification denied publication".into()),
            };
        }
        match self.publish_derived_variant_version(&version.as_record().id().canonical()) {
            Ok(published) => TransferOutcome {
                kind: TransferOutcomeKind::Published,
                job_spec_id: run.job_spec_id,
                job_run_id: run.run_id,
                derived_variant_id: logical.as_record().id().canonical(),
                derived_variant_version_id: published.as_record().id().canonical(),
                persistence_artifact_id: published
                    .as_record()
                    .persistence_artifact_id()
                    .map(|id| id.canonical()),
                qc_report_id: published
                    .as_record()
                    .qc_report_id()
                    .map(|id| id.canonical()),
                qc_verdict: Some(format!("{:?}", qc.as_record().verdict())),
                persistence_verification_id: published
                    .as_record()
                    .persistence_verification_id()
                    .map(|id| id.canonical()),
                reason: None,
            },
            Err(err) => TransferOutcome {
                kind: TransferOutcomeKind::PublicationDenied,
                job_spec_id: run.job_spec_id,
                job_run_id: run.run_id,
                derived_variant_id: logical.as_record().id().canonical(),
                derived_variant_version_id: version.as_record().id().canonical(),
                persistence_artifact_id: version
                    .as_record()
                    .persistence_artifact_id()
                    .map(|id| id.canonical()),
                qc_report_id: Some(qc.as_record().id().canonical()),
                qc_verdict: Some(format!("{:?}", qc.as_record().verdict())),
                persistence_verification_id: Some(verification.as_record().id().canonical()),
                reason: Some(err.to_string()),
            },
        }
    }
}

fn denied_not_succeeded(run: JobRun) -> TransferOutcome {
    TransferOutcome {
        kind: TransferOutcomeKind::PublicationDenied,
        job_spec_id: run.job_spec_id,
        job_run_id: run.run_id,
        derived_variant_id: String::new(),
        derived_variant_version_id: String::new(),
        persistence_artifact_id: None,
        qc_report_id: None,
        qc_verdict: None,
        persistence_verification_id: None,
        reason: Some("JobRun is not SUCCEEDED; no publication".into()),
    }
}

pub(crate) fn transfer_eligibility(
    result: &CompatibilityResult,
    warnings_acknowledged: bool,
) -> Result<TransferAuthorization, AppError> {
    let summary = result.summary();
    let requires_acknowledgement = summary == CompatibilitySummary::ReadyWithWarnings;
    let (eligible, denial_reason) = match summary {
        CompatibilitySummary::Ready => (true, None),
        CompatibilitySummary::ReadyWithWarnings if warnings_acknowledged => (true, None),
        CompatibilitySummary::ReadyWithWarnings => (
            false,
            Some("ReadyWithWarnings requires explicit durable acknowledgement".into()),
        ),
        CompatibilitySummary::MappingConfirmationRequired => (
            false,
            Some("MappingConfirmationRequired cannot authorize Transfer".into()),
        ),
        CompatibilitySummary::Unsupported => {
            (false, Some("Unsupported cannot authorize Transfer".into()))
        }
    };
    Ok(TransferAuthorization {
        compatibility_id: result.id().canonical(),
        summary,
        eligible,
        requires_acknowledgement,
        denial_reason,
    })
}

pub(crate) fn validate_transfer_graph(
    result: &CompatibilityResult,
    character: &rigforge_domain::CharacterAssetVersion,
    motion: &rigforge_domain::MotionAssetVersion,
    mapping: &BoneMappingVersion,
    policy: &rigforge_domain::RetargetPolicyVersion,
) -> Result<(), AppError> {
    if result.character_version_id() != character.id()
        || result.motion_version_id() != motion.id()
        || result.mapping_version_id() != mapping.id()
        || result.policy_version_id() != policy.id()
    {
        return Err(AppError::Catalog(
            "CompatibilityResult is not bound to the exact Character/Motion/Mapping/Policy".into(),
        ));
    }
    if mapping.target_character_version_id() != character.id() {
        return Err(AppError::Catalog(
            "Mapping.target_character_version_id must equal CharacterAssetVersion".into(),
        ));
    }
    if mapping.source_skeleton_ref_id() != motion.source_skeleton_ref_id() {
        return Err(AppError::Catalog(
            "Mapping.source_skeleton_ref_id must equal Motion.source_skeleton_ref_id".into(),
        ));
    }
    if mapping.lifecycle() != Lifecycle::Published {
        return Err(AppError::Catalog(
            "Transfer requires Published BoneMappingVersion".into(),
        ));
    }
    let _ = policy;
    Ok(())
}

pub(crate) fn publication_evidence<'a>(
    character: &'a rigforge_domain::CharacterAssetVersion,
    motion: &'a rigforge_domain::MotionAssetVersion,
    source: &'a rigforge_domain::SourceSkeletonReference,
    mapping: &'a BoneMappingVersion,
    policy: &'a rigforge_domain::RetargetPolicyVersion,
    job: &'a JobSpec,
    worker: &'a rigforge_domain::WorkerResult,
    qc: &'a QcReport,
    backend: &'a rigforge_domain::BackendExecutionContext,
    compatibility: &'a CompatibilityResult,
    persistence: &'a rigforge_domain::PersistenceArtifact,
    verification: &'a PersistenceVerification,
) -> PublicationEvidence<'a> {
    PublicationEvidence {
        character_version: character,
        motion_version: motion,
        source_skeleton: source,
        mapping_version: mapping,
        policy_version: policy,
        job,
        worker,
        qc,
        backend,
        compatibility,
        persistence: Some(persistence),
        verification: Some(verification),
    }
}

pub(crate) fn domain_publish(
    evidence: PublicationEvidence<'_>,
    derived: &mut DerivedVariantVersion,
) -> Result<(), AppError> {
    publish_derived_variant(evidence, derived).map_err(AppError::from)
}
