mod common;

use std::path::{Path, PathBuf};

use common::{sha256_file, temp_dir};
use rigforge_app::{
    generate_mapping_proposal, Application, MappingAssistProfile, TerminalOutcome,
    TransferOutcomeKind, WorkerCapabilityProfile,
};
use rigforge_blender_worker::{
    enforce_pin, BlenderPin, BlenderQcInspector, BlenderSkeletonInspector, BlenderWorker,
};
use rigforge_domain::*;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";
const KNIGHT_SHA: &str = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";

fn evidence(path: &str, sha: &str) -> SourceArtifactEvidence {
    let bytes = std::fs::metadata(path).unwrap().len();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(sha).unwrap(),
        bytes,
        "application/octet-stream",
        Some("V1-5 frozen pair".into()),
        None,
    )
    .unwrap()
}

struct SeededProduct {
    app: Application,
    compatibility_id: String,
    mapping: BoneMappingVersion,
    character_version: String,
    motion_version: String,
    catalog_path: PathBuf,
}

fn seed_frozen_product() -> SeededProduct {
    common::ensure_test_runtime();
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(KNIGHT)), KNIGHT_SHA);
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);

    let dir = temp_dir();
    let catalog_path = dir.join("catalog.sqlite");
    let mut app = Application::open(&catalog_path).unwrap();
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

    let mut policy = RetargetPolicy::new("rest-relative-proven").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());

    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character.clone()).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion.clone()).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy.clone()).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();

    let inspector = BlenderSkeletonInspector::production().unwrap();
    let target_summary = app
        .inspect_and_store_character_summary(&character_version.id().canonical(), &inspector)
        .unwrap();
    let source_summary = app
        .inspect_and_store_source_summary(
            &source_skeleton.id().canonical(),
            &inspector,
            &motion_version.id().canonical(),
        )
        .unwrap();
    let proposal = generate_mapping_proposal(
        source_summary.as_record(),
        target_summary.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = app
        .store_mapping_draft(
            "knight-ual2-v15",
            &character_version.id().canonical(),
            &source_skeleton.id().canonical(),
            &proposal,
            &source_summary.as_record().id().canonical(),
            &target_summary.as_record().id().canonical(),
        )
        .unwrap();
    let published = app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    let result = app
        .run_compatibility_preflight(
            &character_version.id().canonical(),
            &motion_version.id().canonical(),
            &published.as_record().id().canonical(),
            &policy_version.id().canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    assert_eq!(
        result.as_record().summary(),
        CompatibilitySummary::ReadyWithWarnings
    );
    SeededProduct {
        compatibility_id: result.as_record().id().canonical(),
        mapping: published.into_record(),
        character_version: character_version.id().canonical(),
        motion_version: motion_version.id().canonical(),
        catalog_path,
        app,
    }
}

fn execute_product_core(
    app: &mut Application,
    compatibility_id: &str,
    _mapping: &BoneMappingVersion,
    existing_derived: Option<&str>,
    display_name: &str,
    workspace: PathBuf,
) -> (rigforge_app::TransferOutcome, String, String) {
    let denied = app
        .authorize_transfer(compatibility_id, false)
        .unwrap();
    assert!(!denied.eligible);
    let auth = app.authorize_transfer(compatibility_id, true).unwrap();
    assert!(auth.eligible);
    let (spec, run) = app
        .start_transfer(compatibility_id, true, existing_derived, display_name)
        .unwrap();
    app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::production()
        .unwrap()
        .with_workspace_root(workspace);
    let (_running, _receipt) = app.dispatch(&run.run_id, &mut worker).unwrap();
    let (done, outcome) = app.collect(&run.run_id, &mut worker).unwrap();
    let TerminalOutcome::Success(result) = outcome else {
        panic!("worker did not succeed: {outcome:?} state={:?}", done.state);
    };
    assert!(result.as_record().worker_success());
    let staged = worker
        .last_staged_blend()
        .expect("successful collect must retain staged blend path")
        .to_path_buf();
    let before = sha256_file(&staged);
    let transfer = app
        .finalize_transfer(&run.run_id, &staged)
        .unwrap();
    let _ = before;
    (
        transfer,
        spec.as_record().id().canonical(),
        result.as_record().id().canonical(),
    )
}

fn run_snapshot(
    app: &Application,
    outcome: &rigforge_app::TransferOutcome,
) -> serde_json::Value {
    let run = app.catalog().load_job_run(&outcome.job_run_id).unwrap();
    let spec = app.load_job_spec(&outcome.job_spec_id).unwrap();
    let worker = app
        .catalog()
        .load_worker_result(run.worker_result_id.as_ref().unwrap())
        .unwrap();
    let version = app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    let qc = app
        .catalog()
        .load_qc_report(outcome.qc_report_id.as_ref().unwrap())
        .unwrap();
    let verification = app
        .catalog()
        .load_persistence_verification(outcome.persistence_verification_id.as_ref().unwrap())
        .unwrap();
    let artifact = app
        .catalog()
        .load_artifact_metadata(outcome.persistence_artifact_id.as_ref().unwrap())
        .unwrap();
    let checks: Vec<serde_json::Value> = qc
        .as_record()
        .checks()
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": format!("{:?}", c.name()),
                "outcome": format!("{:?}", c.outcome())
            })
        })
        .collect();
    let exec = worker.as_record().execution();
    serde_json::json!({
        "compatibility_result_id": spec.as_record().compatibility_result_id().map(|id| id.canonical()),
        "target_derived_variant_id": spec.as_record().target_derived_variant_id().map(|id| id.canonical()),
        "warnings_acknowledged": spec.as_record().compatibility_warnings_acknowledged(),
        "job_spec_id": outcome.job_spec_id,
        "job_run_id": outcome.job_run_id,
        "job_run_state": run.state.as_db_str(),
        "attempt_id": run.attempt_id,
        "worker_execution_ref": run.worker_execution_ref,
        "worker_result_id": worker.as_record().id().canonical(),
        "worker_success": worker.as_record().worker_success(),
        "derived_variant_id": outcome.derived_variant_id,
        "derived_variant_version_id": outcome.derived_variant_version_id,
        "lifecycle": format!("{:?}", version.as_record().lifecycle()),
        "persistence_artifact_id": artifact.as_record().id().canonical(),
        "artifact_instance_id": artifact.as_record().instance_id().canonical(),
        "artifact_digest": artifact.as_record().digest().sha256(),
        "qc_report_id": qc.as_record().id().canonical(),
        "qc_rule_set": qc.as_record().rule_set_id(),
        "qc_verdict": format!("{:?}", qc.as_record().verdict()),
        "qc_checks": checks,
        "persistence_verification_id": verification.as_record().id().canonical(),
        "fresh_reopen": format!("{:?}", verification.as_record().fresh_reopen()),
        "structural_verification": format!("{:?}", verification.as_record().structural_verification()),
        "backend_kind": exec.backend_kind(),
        "backend_version": exec.backend_version(),
        "backend_build": exec.build(),
        "adapter_version": exec.adapter_version()
    })
}

#[test]
fn two_clean_frozen_pair_product_core_runs() {
    let mut seeded = seed_frozen_product();
    let ws1 = temp_dir();
    let (first, job1, worker1) = execute_product_core(
        &mut seeded.app,
        &seeded.compatibility_id,
        &seeded.mapping,
        None,
        "Knight Walk Carry",
        ws1,
    );
    if first.kind != TransferOutcomeKind::Published {
        panic!(
            "run 1 publication denied: {:?} qc={:?} reason={:?}",
            first.kind, first.qc_verdict, first.reason
        );
    }
    let first_version = seeded
        .app
        .catalog()
        .load_derived_variant_version(&first.derived_variant_version_id)
        .unwrap();
    assert_eq!(first_version.as_record().lifecycle(), Lifecycle::Published);
    let qc1 = seeded
        .app
        .catalog()
        .load_qc_report(first.qc_report_id.as_ref().unwrap())
        .unwrap();
    let ver1 = seeded
        .app
        .catalog()
        .load_persistence_verification(first.persistence_verification_id.as_ref().unwrap())
        .unwrap();
    let artifact1 = seeded
        .app
        .catalog()
        .load_artifact_metadata(first.persistence_artifact_id.as_ref().unwrap())
        .unwrap();
    let durable = seeded.app.catalog().durable_artifact_path(artifact1.as_record());
    let sha_before = sha256_file(&durable);
    let inspector = BlenderQcInspector::production()
        .unwrap()
        .with_mapping(seeded.mapping.clone());
    inspector
        .inspect(&durable, artifact1.as_record().digest().sha256())
        .unwrap();
    let sha_after = sha256_file(&durable);
    assert_eq!(sha_before, sha_after);
    for check in qc1.as_record().checks() {
        assert_eq!(check.outcome(), QcCheckOutcome::Pass, "{:?}", check.name());
    }
    assert_eq!(qc1.as_record().verdict(), QcVerdict::Pass);
    assert_eq!(ver1.as_record().fresh_reopen(), VerificationOutcome::Pass);
    assert_eq!(
        ver1.as_record().structural_verification(),
        VerificationOutcome::Pass
    );

    let ws2 = temp_dir();
    let (second, job2, worker2) = execute_product_core(
        &mut seeded.app,
        &seeded.compatibility_id,
        &seeded.mapping,
        Some(&first.derived_variant_id),
        "Knight Walk Carry",
        ws2,
    );
    if second.kind != TransferOutcomeKind::Published {
        panic!(
            "run 2 publication denied: {:?} qc={:?} reason={:?}",
            second.kind, second.qc_verdict, second.reason
        );
    }
    assert_eq!(second.derived_variant_id, first.derived_variant_id);
    assert_ne!(second.derived_variant_version_id, first.derived_variant_version_id);
    assert_ne!(job1, job2);
    assert_ne!(worker1, worker2);
    let still = seeded
        .app
        .catalog()
        .load_derived_variant_version(&first.derived_variant_version_id)
        .unwrap();
    assert_eq!(still.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        still.as_record().job_spec_id().canonical(),
        first_version.as_record().job_spec_id().canonical()
    );

    let payload = serde_json::json!({
        "compatibility_id": seeded.compatibility_id,
        "warnings_acknowledged": true,
        "summary": "ReadyWithWarnings",
        "character_version": seeded.character_version,
        "motion_version": seeded.motion_version,
        "mapping_version": seeded.mapping.id().canonical(),
        "non_mutation_durable_sha_before": sha_before,
        "non_mutation_durable_sha_after": sha_after,
        "previous_published_immutable": true,
        "preview": "NOT AVAILABLE — V1-6",
        "run_1": run_snapshot(&seeded.app, &first),
        "run_2": run_snapshot(&seeded.app, &second)
    });
    let out_dir = PathBuf::from(r"F:\NewResearch\rigforge_temp\v1_5");
    std::fs::create_dir_all(&out_dir).unwrap();
    let encoded = serde_json::to_vec_pretty(&payload).unwrap();
    std::fs::write(out_dir.join("product_core_checkpoint.json"), &encoded).unwrap();
    std::fs::write(
        std::env::temp_dir().join("rigforge_v15_product_core_checkpoint.json"),
        &encoded,
    )
    .unwrap();
    let _ = seeded.catalog_path;
}

#[test]
fn real_pinned_product_core_finalization_still_publishes() {
    assert_real_pinned_product_core_publishes();
}

#[test]
fn real_pinned_product_core_path_still_publishes() {
    assert_real_pinned_product_core_publishes();
}

fn assert_real_pinned_product_core_publishes() {
    let mut seeded = seed_frozen_product();
    let ws = temp_dir();
    let (outcome, _job, _worker) = execute_product_core(
        &mut seeded.app,
        &seeded.compatibility_id,
        &seeded.mapping,
        None,
        "Knight Walk Carry Rev2",
        ws,
    );
    if outcome.kind != TransferOutcomeKind::Published {
        panic!(
            "production finalize must publish: {:?} qc={:?} reason={:?}",
            outcome.kind, outcome.qc_verdict, outcome.reason
        );
    }
    let version = seeded
        .app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
    let qc = seeded
        .app
        .catalog()
        .load_qc_report(outcome.qc_report_id.as_ref().unwrap())
        .unwrap();
    assert_eq!(qc.as_record().verdict(), QcVerdict::Pass);
    let verification = seeded
        .app
        .catalog()
        .load_persistence_verification(outcome.persistence_verification_id.as_ref().unwrap())
        .unwrap();
    assert_eq!(
        verification.as_record().fresh_reopen(),
        VerificationOutcome::Pass
    );
    assert_eq!(
        verification.as_record().structural_verification(),
        VerificationOutcome::Pass
    );
}
