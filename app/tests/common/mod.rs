#![allow(dead_code)]

use rigforge_domain::*;

#[cfg(feature = "test-support")]
use rigforge_app::{sha256_file, ArtifactInspector, PersistenceReopener};
#[cfg(feature = "test-support")]
use std::path::Path;

pub fn digest(n: u8) -> ContentDigest {
    ContentDigest::parse(&format!("{n:x}").repeat(64)).unwrap()
}

pub fn source(name: &str, n: u8) -> SourceArtifactEvidence {
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(format!("C:/research/{name}.bin")).unwrap(),
        digest(n),
        1024,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".to_string()),
        None,
    )
    .unwrap()
}

pub fn fixture_correlation() -> ExecutionCorrelation {
    ExecutionCorrelation::new("app-fixture-attempt", "app-fixture-worker-ref").unwrap()
}

pub fn time_domain() -> TimeDomainProvenance {
    TimeDomainProvenance::new(
        "clip:walk-carry",
        TimePoint::frames(1, 30, 1).unwrap(),
        TimePoint::frames(61, 30, 1).unwrap(),
        SamplingInterpretation::BakedEverySourceFrame,
        "unmapped target joints remain at target rest",
    )
    .unwrap()
}

pub fn backend() -> BackendExecutionContext {
    BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap()
}

pub fn mapping_entry(src: &str, dst: &str) -> BoneMappingEntry {
    BoneMappingEntry::new(
        JointRef::source(JointKey::new(src).unwrap()),
        JointRef::target(JointKey::new(dst).unwrap()),
        JointParticipation::Required,
        Some("optional.profile.root".to_string()),
        "hierarchy plus review",
    )
    .unwrap()
}

pub struct Graph {
    pub character: CharacterAsset,
    pub character_version: CharacterAssetVersion,
    pub motion: MotionAsset,
    pub motion_version: MotionAssetVersion,
    pub source_skeleton: SourceSkeletonReference,
    pub mapping: BoneMapping,
    pub mapping_version: BoneMappingVersion,
    pub policy: RetargetPolicy,
    pub policy_version: RetargetPolicyVersion,
    pub job: JobSpec,
    pub worker: WorkerResult,
    pub compatibility: CompatibilityResult,
    pub derived: DerivedVariant,
    pub derived_version: DerivedVariantVersion,
    pub qc: QcReport,
    pub persistence: PersistenceArtifact,
    pub preview: PreviewArtifact,
    pub backend: BackendExecutionContext,
    pub verification: PersistenceVerification,
}

pub fn unpublished_graph() -> Graph {
    unpublished_graph_with_time(time_domain())
}

pub fn unpublished_graph_with_time(time: TimeDomainProvenance) -> Graph {
    let backend = backend();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", source("knight", 1)).unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        time,
        source("motion", 2),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());

    let mut mapping = BoneMapping::new("knight-ual2").unwrap();
    let mut mapping_version = BoneMappingVersion::draft(
        mapping.id(),
        character_version.id(),
        source_skeleton.id(),
        vec![
            mapping_entry("root", "Bone"),
            mapping_entry("pelvis", "Body"),
        ],
        MappingReviewProvenance::with_kind(
            true,
            Some("manual review".to_string()),
            vec!["foot parent chain".to_string()],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    mapping_version
        .bind_source_motion_version(motion_version.id())
        .unwrap();
    mapping_version.publish().unwrap();
    mapping.bind_published(mapping_version.id());

    let mut policy = RetargetPolicy::new("rest-relative").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());

    let compatibility = CompatibilityResult::from_preflight(
        character_version.id(),
        motion_version.id(),
        mapping_version.id(),
        policy_version.id(),
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["fixture Ready preflight".to_string()],
    )
    .unwrap();

    let job = JobSpec::new(
        character_version.id(),
        motion_version.id(),
        source_skeleton.id(),
        mapping_version.id(),
        policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap()
    .with_requested_capabilities(vec![
        RequestedCapability::PersistenceArtifact,
        RequestedCapability::PreviewPayload,
    ])
    .unwrap()
    .with_compatibility_authorization(compatibility.id(), false)
    .unwrap();

    let persist_digest = digest(3);
    let worker = WorkerResult::new(
        job.id(),
        backend.clone(),
        true,
        "completed",
        fixture_correlation(),
    )
        .unwrap()
        .with_staged_artifact_digests(vec![persist_digest.clone()])
        .unwrap();

    let derived = DerivedVariant::new("Knight Walk Carry").unwrap();
    let mut derived_version = DerivedVariantVersion::draft(
        derived.id(),
        character_version.id(),
        motion_version.id(),
        source_skeleton.id(),
        mapping_version.id(),
        policy_version.id(),
        job.id(),
        backend.id(),
        worker.id(),
    )
    .unwrap();

    let persistence = PersistenceArtifact::new(
        derived_version.id(),
        persist_digest,
        4096,
        "application/octet-stream",
        backend.id(),
    )
    .unwrap();
    derived_version
        .bind_persistence_artifact(persistence.id())
        .unwrap();
    let qc = QcReport::for_derived_variant(
        derived_version.id(),
        policy_version.id(),
        worker.id(),
        persistence.id(),
        persistence.instance_id(),
        persistence.digest().clone(),
        passing_structural_qc_checks(),
    )
    .unwrap();
    derived_version.bind_qc_report(qc.id()).unwrap();
    let preview = PreviewArtifact::for_derived_variant(
        derived_version.id(),
        digest(4),
        2048,
        "application/octet-stream",
        backend.id(),
    )
    .unwrap();
    derived_version
        .bind_preview_artifacts(vec![preview.id()])
        .unwrap();
    let verification = PersistenceVerification::new(
        persistence.id(),
        persistence.instance_id(),
        persistence.digest().clone(),
        derived_version.id(),
        backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();

    Graph {
        character,
        character_version,
        motion,
        motion_version,
        source_skeleton,
        mapping,
        mapping_version,
        policy,
        policy_version,
        job,
        worker,
        compatibility,
        derived,
        derived_version,
        qc,
        persistence,
        preview,
        backend,
        verification,
    }
}

pub fn valid_graph() -> Graph {
    let mut g = unpublished_graph();
    g.try_publish();
    g.derived.bind_published(g.derived_version.id());
    g
}

impl Graph {
    pub fn try_publish(&mut self) {
        publish_derived_variant(
            PublicationEvidence {
                character_version: &self.character_version,
                motion_version: &self.motion_version,
                source_skeleton: &self.source_skeleton,
                mapping_version: &self.mapping_version,
                policy_version: &self.policy_version,
                job: &self.job,
                worker: &self.worker,
                qc: &self.qc,
                backend: &self.backend,
                compatibility: &self.compatibility,
                persistence: Some(&self.persistence),
                verification: Some(&self.verification),
            },
            &mut self.derived_version,
        )
        .unwrap();
    }
}

pub fn persist_core(catalog: &mut rigforge_app::SqliteCatalog, g: &Graph) {
    catalog
        .put_validated_pair(&certify(g.character.clone()), &certify(g.character_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(&certify(g.motion.clone()), &certify(g.motion_version.clone()))
        .unwrap();
    catalog
        .put_validated_pair(&certify(g.mapping.clone()), &certify(g.mapping_version.clone()))
        .unwrap();
    catalog
        .put_validated_pair(&certify(g.policy.clone()), &certify(g.policy_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.compatibility.clone()))
        .unwrap();
}

pub fn certify<T: DomainRecord>(record: T) -> Validated<T> {
    Validated::certify(record).unwrap()
}

pub fn matching_success(job: &JobSpec, run: &rigforge_app::JobRun) -> Validated<WorkerResult> {
    successful_worker_for(
        job,
        &run.attempt_id,
        run.worker_execution_ref
            .as_deref()
            .expect("RUNNING JobRun must have worker_execution_ref"),
    )
}

pub fn matching_failure(job: &JobSpec, run: &rigforge_app::JobRun) -> Validated<WorkerResult> {
    failed_worker_for(
        job,
        &run.attempt_id,
        run.worker_execution_ref
            .as_deref()
            .expect("RUNNING JobRun must have worker_execution_ref"),
    )
}

pub fn successful_worker_for(
    job: &JobSpec,
    attempt_id: &str,
    worker_execution_ref: &str,
) -> Validated<WorkerResult> {
    certify(
        WorkerResult::new(
            job.id(),
            backend(),
            true,
            "completed",
            ExecutionCorrelation::new(attempt_id, worker_execution_ref).unwrap(),
        )
        .unwrap(),
    )
}

pub fn failed_worker_for(
    job: &JobSpec,
    attempt_id: &str,
    worker_execution_ref: &str,
) -> Validated<WorkerResult> {
    certify(
        WorkerResult::new(
            job.id(),
            backend(),
            false,
            "failed",
            ExecutionCorrelation::new(attempt_id, worker_execution_ref).unwrap(),
        )
        .unwrap(),
    )
}

pub fn mutate_json_field<T: DomainRecord>(record: &T, field: &str, value: serde_json::Value) -> Validated<T> {
    let json = to_json(record).unwrap();
    let mut parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    parsed
        .as_object_mut()
        .unwrap()
        .insert(field.to_string(), value);
    ingest_validated(&parsed.to_string()).unwrap()
}

/// Test-only inspector. Not part of the production Application surface.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug)]
pub struct MemoryArtifactInspector {
    pub evidence: rigforge_app::ArtifactInspectionEvidence,
}

#[cfg(feature = "test-support")]
impl MemoryArtifactInspector {
    pub fn passing(expected_duration_s: Option<f64>) -> Self {
        let duration = expected_duration_s.or(Some(2.0));
        Self {
            evidence: rigforge_app::ArtifactInspectionEvidence {
                digest_before: String::new(),
                digest_after: String::new(),
                structurally_readable: true,
                finite_transforms: true,
                present_joint_keys: vec!["Bone".into(), "Body".into()],
                baked_animation_present: true,
                duration_s: duration,
                expected_duration_s: duration,
                gross_scale_sane: true,
                root_trajectory_sane: true,
            },
        }
    }

    pub fn with_joints(mut self, joints: Vec<String>) -> Self {
        self.evidence.present_joint_keys = joints;
        self
    }

    pub fn failing_finite_transforms(expected_duration_s: Option<f64>) -> Self {
        let mut inspector = Self::passing(expected_duration_s);
        inspector.evidence.finite_transforms = false;
        inspector
    }
}

#[cfg(feature = "test-support")]
impl ArtifactInspector for MemoryArtifactInspector {
    fn inspect(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<rigforge_app::ArtifactInspectionEvidence, rigforge_app::AppError> {
        if !artifact_path.is_file() {
            return Err(rigforge_app::AppError::Worker(
                "QC inspect: artifact file is missing".into(),
            ));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Err(rigforge_app::AppError::Worker(format!(
                "QC inspect digest mismatch: expected {expected_sha256} found {found}"
            )));
        }
        let after = sha256_file(artifact_path)?;
        let mut evidence = self.evidence.clone();
        evidence.digest_before = found;
        evidence.digest_after = after;
        Ok(evidence)
    }
}

/// Test-only reopener. Not part of the production Application surface.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug)]
pub struct MemoryPersistenceReopener {
    pub fresh_reopen: VerificationOutcome,
    pub structural: VerificationOutcome,
}

#[cfg(feature = "test-support")]
impl Default for MemoryPersistenceReopener {
    fn default() -> Self {
        Self {
            fresh_reopen: VerificationOutcome::Pass,
            structural: VerificationOutcome::Pass,
        }
    }
}

#[cfg(feature = "test-support")]
impl PersistenceReopener for MemoryPersistenceReopener {
    fn reopen(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<(VerificationOutcome, VerificationOutcome), rigforge_app::AppError> {
        if !artifact_path.is_file() {
            return Ok((VerificationOutcome::Missing, VerificationOutcome::Missing));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Ok((VerificationOutcome::Fail, VerificationOutcome::Fail));
        }
        Ok((self.fresh_reopen, self.structural))
    }
}
