//! Catalog-crate test graph. Not a production API.

use rigforge_domain::*;

use crate::catalog::SqliteCatalog;
use crate::error::AppError;

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
    pub backend: BackendExecutionContext,
    pub verification: PersistenceVerification,
}

fn digest(n: u8) -> ContentDigest {
    ContentDigest::parse(&format!("{n:x}").repeat(64)).unwrap()
}

fn source(name: &str, n: u8) -> SourceArtifactEvidence {
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

fn mapping_entry(src: &str, dst: &str) -> BoneMappingEntry {
    BoneMappingEntry::new(
        JointRef::source(JointKey::new(src).unwrap()),
        JointRef::target(JointKey::new(dst).unwrap()),
        JointParticipation::Required,
        Some("optional.profile.root".to_string()),
        "hierarchy plus review",
    )
    .unwrap()
}

pub fn unpublished_graph() -> Graph {
    let backend = BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", source("knight", 1)).unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let time = TimeDomainProvenance::new(
        "clip:walk-carry",
        TimePoint::frames(1, 30, 1).unwrap(),
        TimePoint::frames(61, 30, 1).unwrap(),
        SamplingInterpretation::BakedEverySourceFrame,
        "unmapped target joints remain at target rest",
    )
    .unwrap();
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
        vec![mapping_entry("root", "Bone"), mapping_entry("pelvis", "Body")],
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
    .with_compatibility_authorization(compatibility.id(), false)
    .unwrap();

    let persist_digest = digest(3);
    let worker = WorkerResult::new(
        job.id(),
        backend.clone(),
        true,
        "completed",
        ExecutionCorrelation::new("app-fixture-attempt", "app-fixture-worker-ref").unwrap(),
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
        backend,
        verification,
    }
}

pub fn certify<T: DomainRecord>(record: T) -> Validated<T> {
    Validated::certify(record).unwrap()
}

pub fn persist_core(catalog: &mut SqliteCatalog, g: &Graph) -> Result<(), AppError> {
    catalog.put_validated(&certify(g.character.clone()))?;
    catalog.put_validated(&certify(g.character_version.clone()))?;
    catalog.put_validated(&certify(g.source_skeleton.clone()))?;
    catalog.put_validated(&certify(g.motion.clone()))?;
    catalog.put_validated(&certify(g.motion_version.clone()))?;
    catalog.put_validated(&certify(g.mapping.clone()))?;
    catalog.put_validated(&certify(g.mapping_version.clone()))?;
    catalog.put_validated(&certify(g.policy.clone()))?;
    catalog.put_validated(&certify(g.policy_version.clone()))?;
    catalog.put_validated(&certify(g.compatibility.clone()))?;
    Ok(())
}

pub fn persist_unpublished_without_authority(
    catalog: &mut SqliteCatalog,
    g: &Graph,
) -> Result<(), AppError> {
    persist_core(catalog, g)?;
    catalog.put_validated(&certify(g.derived.clone()))?;
    let job = g
        .job
        .clone()
        .with_target_derived_variant(g.derived.id())?;
    catalog.put_validated(&certify(job))?;
    catalog.put_validated(&certify(g.worker.clone()))?;
    let run = catalog.seed_succeeded_job_run_for_tests(
        &g.job.id().canonical(),
        &g.worker.id().canonical(),
    )?;
    let location = crate::catalog::CatalogLocationEvidence {
        product_id: g.persistence.id().canonical(),
        instance_id: g.persistence.instance_id().canonical(),
        location_kind: "filesystem_path_evidence".into(),
        location_value: "C:/research/fixture-candidate.blend".into(),
        observed_at: 1,
        note: Some("test fixture location; not Product identity".into()),
    };
    catalog.persist_candidate_graph(
        &run.run_id,
        &certify(g.derived_version.clone()),
        &certify(g.persistence.clone()),
        location,
    )?;
    Ok(())
}
