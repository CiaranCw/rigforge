#![allow(dead_code)]

use rigforge_domain::*;

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
        QcReportId::generate(),
    )
    .unwrap();
    let qc = QcReport::for_derived_variant(
        derived_version.id(),
        policy_version.id(),
        worker.id(),
        QcVerdict::Pass,
    )
    .unwrap();
    derived_version.bind_qc_report(qc.id()).unwrap();

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
                persistence: Some(&self.persistence),
                verification: Some(&self.verification),
            },
            &mut self.derived_version,
        )
        .unwrap();
    }
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
