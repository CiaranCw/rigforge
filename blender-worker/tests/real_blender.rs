mod common;

use std::path::{Path, PathBuf};

use common::{local_blender_archive, mapping_entry, sha256_file, temp_dir};
use rigforge_app::{JobRunState, SqliteCatalog, TerminalOutcome};
use rigforge_blender_worker::{
    default_blender_executable, enforce_pin, sha256_file as worker_sha,
    verify_archive_sha256, BlenderPin, BlenderWorker, BLENDER_ARCHIVE_SHA256, BLENDER_BUILD,
};
use rigforge_domain::*;
use serde::Serialize;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";
const KNIGHT_SHA: &str = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";
const POC_MAPPING: &str = r"F:\NewResearch\rigforge\experiments\w0p\poc_blender_e2e_01\config\mapping_frozen.json";
const POC_POLICY: &str = r"F:\NewResearch\rigforge\experiments\w0p\poc_blender_e2e_01\config\retarget_policy.json";
const POC_MAPPING_SHA: &str =
    "683603627bbe431a8438c8b6bacca4d6779046e4ef09ede7fa3218c41e8a602b";
const POC_POLICY_SHA: &str =
    "43eee4b181ffad617b270eeb8b076e650e39c4e2d7ed8425918d825c9cab1b8b";

#[derive(Serialize, Clone)]
struct RealRunEvidence {
    blender_version: String,
    blender_build: String,
    character_sha256: String,
    motion_sha256: String,
    job_spec_id: String,
    attempt_id: String,
    worker_execution_ref: String,
    mapping_version_id: String,
    policy_version_id: String,
    poc_mapping_sha256: String,
    poc_policy_sha256: String,
    terminal_status: String,
    worker_success: bool,
    staged_digest: Option<String>,
    reopen: String,
    rotation_only_audit: Option<String>,
    quaternion_policy: Option<String>,
    keep_target_rest_scale: Option<String>,
    root_scale_audit: Option<String>,
    loop_closure: Option<String>,
}

fn evidence(path: &str, expected: &str) -> SourceArtifactEvidence {
    let hex = sha256_file(Path::new(path));
    assert_eq!(hex, expected, "frozen source hash mismatch for {path}");
    let size = std::fs::metadata(path).unwrap().len();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(&hex).unwrap(),
        size,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".into()),
        Some("V1-3 production checkpoint; bytes were hashed not parsed as Product truth".into()),
    )
    .unwrap()
}

fn frozen_mapping_entries() -> Vec<BoneMappingEntry> {
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(POC_MAPPING).unwrap()).unwrap();
    raw["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            mapping_entry(
                entry["source"].as_str().unwrap(),
                entry["target"].as_str().unwrap(),
                entry["role"].as_str().unwrap(),
                entry["required"].as_bool().unwrap(),
            )
        })
        .collect()
}

fn seed_frozen(catalog: &mut SqliteCatalog) -> (JobSpec, BoneMappingVersionId, RetargetPolicyVersionId) {
    let mut character = CharacterAsset::new("Knight_Male").unwrap();
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        "Knight_Male frozen FBX",
        evidence(KNIGHT, KNIGHT_SHA),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2_Standard").unwrap();
    let mut motion = MotionAsset::new("UAL2_Standard").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "UAL2 Walk_Carry_Loop",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            CLIP,
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence(UAL2, UAL2_SHA),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());

    let mut mapping = BoneMapping::new("knight-ual2-frozen").unwrap();
    let mut mapping_version = BoneMappingVersion::draft(
        mapping.id(),
        character_version.id(),
        source_skeleton.id(),
        frozen_mapping_entries(),
        MappingReviewProvenance::with_kind(
            true,
            Some("independent name and hierarchy review; frozen before worker execution".into()),
            vec!["Knight Foot parents to Bone; mapped as foot role".into()],
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

    let mut policy = RetargetPolicy::new("rest-relative-proven").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());

    catalog
        .put_validated(&Validated::certify(character_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(motion_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(mapping_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(policy_version.clone()).unwrap())
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
    .unwrap();
    (job, mapping_version.id(), policy_version.id())
}

fn measurement(result: &WorkerResult, name: &str) -> Option<String> {
    result
        .measurements()
        .iter()
        .find(|m| m.name() == name)
        .map(|m| m.value().to_string())
}

fn run_production(
    catalog: &mut SqliteCatalog,
    spec: JobSpec,
    workspace: PathBuf,
) -> RealRunEvidence {
    common::ensure_test_runtime();
    let spec = Validated::certify(spec).unwrap();
    let run = catalog.enqueue_job(spec.clone()).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::production()
        .unwrap()
        .with_workspace_root(workspace);
    let (running, receipt) = catalog.dispatch(&run.run_id, &mut worker).unwrap();
    assert_eq!(running.state, JobRunState::Running);
    assert_eq!(receipt.attempt_id, running.attempt_id);
    let (done, outcome) = catalog.collect(&run.run_id, &mut worker).unwrap();
    let TerminalOutcome::Success(result) = outcome else {
        panic!("expected success, got {outcome:?} state={:?}", done.state);
    };
    let rec = result.as_record();
    assert!(rec.worker_success());
    assert_eq!(rec.job_spec_id(), spec.as_record().id());
    assert_eq!(rec.attempt_id(), done.attempt_id.as_str());
    assert_eq!(rec.worker_execution_ref(), receipt.worker_execution_ref.as_str());
    assert_eq!(rec.execution().build(), BLENDER_BUILD);
    assert!(!rec.staged_artifact_digests().is_empty());
    assert!(catalog
        .list_assets(RecordType::DerivedVariant)
        .unwrap()
        .is_empty());
    let joined = rec.diagnostics().join("\n");
    assert!(joined.contains("reopen_status=PASS"), "{joined}");
    RealRunEvidence {
        blender_version: rec.execution().backend_version().to_string(),
        blender_build: rec.execution().build().to_string(),
        character_sha256: KNIGHT_SHA.into(),
        motion_sha256: UAL2_SHA.into(),
        job_spec_id: spec.as_record().id().canonical(),
        attempt_id: done.attempt_id,
        worker_execution_ref: receipt.worker_execution_ref,
        mapping_version_id: spec.as_record().mapping_version_id().canonical(),
        policy_version_id: spec.as_record().policy_version_id().canonical(),
        poc_mapping_sha256: sha256_file(Path::new(POC_MAPPING)),
        poc_policy_sha256: sha256_file(Path::new(POC_POLICY)),
        terminal_status: rec.terminal_status().to_string(),
        worker_success: rec.worker_success(),
        staged_digest: rec
            .staged_artifact_digests()
            .first()
            .map(|d| d.sha256().to_string()),
        reopen: "PASS".into(),
        rotation_only_audit: measurement(rec, "rotation_only_audit"),
        quaternion_policy: measurement(rec, "quaternion_policy"),
        keep_target_rest_scale: measurement(rec, "keep_target_rest_scale"),
        root_scale_audit: measurement(rec, "root_scale_audit"),
        loop_closure: measurement(rec, "loop_closure"),
    }
}

#[test]
fn pinned_blender_version_and_archive() {
    common::ensure_test_runtime();
    assert!(Path::new(KNIGHT).is_file(), "Knight_Male.fbx missing");
    assert!(Path::new(UAL2).is_file(), "UAL2_Standard.fbx missing");
    let exe = default_blender_executable();
    let pin = BlenderPin::accepted();
    let (version, build) = enforce_pin(&exe, &pin).unwrap();
    assert!(version.contains("5.2.1"), "{version}");
    assert_eq!(build, BLENDER_BUILD);
    let archive = local_blender_archive();
    if archive.is_file() {
        verify_archive_sha256(&archive).unwrap();
        assert_eq!(worker_sha(&archive).unwrap(), BLENDER_ARCHIVE_SHA256);
    }
    assert_eq!(sha256_file(Path::new(POC_MAPPING)), POC_MAPPING_SHA);
    assert_eq!(sha256_file(Path::new(POC_POLICY)), POC_POLICY_SHA);
}

#[test]
fn production_path_two_clean_runs_and_missing_source_fail_closed() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (job, mapping_id, policy_id) = seed_frozen(&mut catalog);
    let run_a = run_production(
        &mut catalog,
        job.clone(),
        temp_dir().join("run_a"),
    );
    let job_b = JobSpec::new(
        job.character_version_id(),
        job.motion_version_id(),
        job.source_skeleton_ref_id(),
        job.mapping_version_id(),
        job.policy_version_id(),
        job.determinism_context(),
        job.isolation_limits(),
    )
    .unwrap();
    let run_b = run_production(&mut catalog, job_b, temp_dir().join("run_b"));

    assert_eq!(run_a.character_sha256, run_b.character_sha256);
    assert_eq!(run_a.motion_sha256, run_b.motion_sha256);
    assert_eq!(run_a.mapping_version_id, mapping_id.canonical());
    assert_eq!(run_b.mapping_version_id, mapping_id.canonical());
    assert_eq!(run_a.policy_version_id, policy_id.canonical());
    assert_eq!(run_b.policy_version_id, policy_id.canonical());
    assert_eq!(run_a.blender_build, BLENDER_BUILD);
    assert_eq!(run_b.blender_build, BLENDER_BUILD);
    assert_eq!(run_a.terminal_status, "completed");
    assert_eq!(run_b.terminal_status, "completed");
    assert_eq!(run_a.reopen, "PASS");
    assert_eq!(run_b.reopen, "PASS");
    assert_eq!(run_a.rotation_only_audit.as_deref(), Some("PASS"));
    assert_eq!(run_b.rotation_only_audit.as_deref(), Some("PASS"));
    assert_eq!(run_a.quaternion_policy.as_deref(), Some("PASS"));
    assert_eq!(run_b.quaternion_policy.as_deref(), Some("PASS"));
    assert_eq!(run_a.keep_target_rest_scale.as_deref(), Some("PASS"));
    assert_eq!(run_b.keep_target_rest_scale.as_deref(), Some("PASS"));
    assert_eq!(run_a.root_scale_audit.as_deref(), Some("PASS"));
    assert_eq!(run_b.root_scale_audit.as_deref(), Some("PASS"));
    assert!(run_a.staged_digest.is_some());
    assert!(run_b.staged_digest.is_some());
    assert_ne!(run_a.attempt_id, run_b.attempt_id);
    assert_ne!(run_a.job_spec_id, run_b.job_spec_id);
    assert_eq!(
        run_a.worker_execution_ref,
        format!("blender-worker:{}", run_a.attempt_id)
    );
    assert_eq!(
        run_b.worker_execution_ref,
        format!("blender-worker:{}", run_b.attempt_id)
    );

    let out = temp_dir().join("real_e2e_result.json");
    std::fs::write(
        &out,
        serde_json::to_string_pretty(&[&run_a, &run_b]).unwrap(),
    )
    .unwrap();
    eprintln!("wrote {}", out.display());

    let mut fail_catalog = SqliteCatalog::open_in_memory().unwrap();
    let mut character = CharacterAsset::new("missing").unwrap();
    let missing = r"F:\NewResearch\rigforge_w0p_assets\MISSING_Knight_Male.fbx";
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        "missing character",
        SourceArtifactEvidence::new(
            LocationEvidence::filesystem_path(missing).unwrap(),
            ContentDigest::parse(KNIGHT_SHA).unwrap(),
            1,
            "application/octet-stream",
            Some("2026-09-01T00:00:00Z".into()),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let (job_ok, _, _) = seed_frozen(&mut fail_catalog);
    // Replace only by enqueueing a job bound to missing character: rebuild job against frozen motion.
    let motion_id = job_ok.motion_version_id();
    let skeleton_id = job_ok.source_skeleton_ref_id();
    fail_catalog
        .put_validated(&Validated::certify(character_version.clone()).unwrap())
        .unwrap();
    let missing_job = JobSpec::new(
        character_version.id(),
        motion_id,
        skeleton_id,
        job_ok.mapping_version_id(),
        job_ok.policy_version_id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    );
    // mapping targets the frozen character, not this missing one — enqueue should fail graph.
    // Use a dedicated mapping against the missing character so dispatch reaches the worker.
    let mut mapping = BoneMapping::new("missing-pair").unwrap();
    let mut mapping_version = BoneMappingVersion::draft(
        mapping.id(),
        character_version.id(),
        skeleton_id,
        frozen_mapping_entries(),
        MappingReviewProvenance::with_kind(
            true,
            Some("manual review".into()),
            vec![],
            MappingReviewKind::Manual,
        )
        .unwrap(),
    )
    .unwrap();
    mapping_version
        .bind_source_motion_version(motion_id)
        .unwrap();
    mapping_version.publish().unwrap();
    mapping.bind_published(mapping_version.id());
    fail_catalog
        .put_validated(&Validated::certify(mapping_version.clone()).unwrap())
        .unwrap();
    let fail_job = JobSpec::new(
        character_version.id(),
        motion_id,
        skeleton_id,
        mapping_version.id(),
        job_ok.policy_version_id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let run = fail_catalog
        .enqueue_job(Validated::certify(fail_job).unwrap())
        .unwrap();
    fail_catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::production()
        .unwrap()
        .with_workspace_root(temp_dir().join("fail"));
    let err = fail_catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    assert!(
        err.to_string().contains("missing") || err.to_string().contains("digest mismatch"),
        "{err}"
    );
    assert_eq!(
        fail_catalog.load_job_run(&run.run_id).unwrap().state,
        JobRunState::Failed
    );
    assert!(fail_catalog
        .list_assets(RecordType::DerivedVariant)
        .unwrap()
        .is_empty());
    let _ = missing_job;
}

#[test]
fn real_blender_worker_result_contains_exact_attempt_correlation() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let (job, _, _) = seed_frozen(&mut catalog);
    let evidence = run_production(
        &mut catalog,
        job,
        temp_dir().join("attempt_correlation"),
    );
    assert!(!evidence.attempt_id.is_empty());
    assert_eq!(
        evidence.worker_execution_ref,
        format!("blender-worker:{}", evidence.attempt_id)
    );
    assert!(evidence.worker_success);
    assert_eq!(evidence.keep_target_rest_scale.as_deref(), Some("PASS"));
    assert_eq!(evidence.root_scale_audit.as_deref(), Some("PASS"));
}
