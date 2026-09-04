mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{sha256_file, temp_dir};
use rigforge_app::{
    generate_mapping_proposal, Application, MappingAssistProfile, TerminalOutcome,
    TransferOutcomeKind, WorkerCapabilityProfile,
};
use rigforge_blender_worker::{
    default_blender_executable, enforce_pin, BlenderPin, BlenderSkeletonInspector, BlenderWorker,
    BACKGROUND, DISABLE_AUTOEXEC, FACTORY_STARTUP, PYTHON, PYTHON_EXIT_CODE,
};
use rigforge_domain::*;
use serde_json::{json, Value};

const WANDERER: &str = r"F:\NewResearch\Project-City\assets\workspace\rigforge\characters\SM_Chr_Wanderer_Male_01\8fffeab700475a75\character.fbx";
const ZOMBIE: &str = r"F:\NewResearch\Project-City\assets\workspace\rigforge\characters\SM_Chr_Zombie_Male_01\8fffeab700475a75\character.fbx";
const UAL2: &str = r"F:\NewResearch\Project-City\assets\vendor\quaternius\universal-animation-library-2\standard\expanded\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";
const UAL2_RM: &str = r"F:\NewResearch\Project-City\assets\vendor\quaternius\universal-animation-library-2\standard\expanded\Universal Animation Library 2[Standard]\Unity\UAL2_Standard_RM.fbx";
const WANDERER_SHA: &str = "de2f4daa5bdc4fc202a3c98338f7cccd9eda2e7713fedf4a46dffb950bfa04b8";
const ZOMBIE_SHA: &str = "11c3952fe6bc1b950a1177ec0710033768659fefee37b978aba1c4db7a254c49";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";

fn evidence(path: &str, sha: &str) -> SourceArtifactEvidence {
    let bytes = std::fs::metadata(path).unwrap().len();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(sha).unwrap(),
        bytes,
        "application/octet-stream",
        Some("Phase II body-frame frozen pair".into()),
        None,
    )
    .unwrap()
}

fn mapping_projection(mapping: &BoneMappingVersion) -> Value {
    Value::Array(
        mapping
            .entries()
            .iter()
            .map(|entry| {
                json!({
                    "source": entry.source().joint_key().as_str(),
                    "target": entry.target().joint_key().as_str(),
                    "role": entry.role_profile().unwrap_or("mapped"),
                    "required": entry.participation() == JointParticipation::Required
                })
            })
            .collect(),
    )
}

struct Seeded {
    app: Application,
    compatibility_id: String,
    mapping: BoneMappingVersion,
}

fn seed_character(name: &str, path: &str, sha: &str) -> Seeded {
    common::ensure_test_runtime();
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(path)), sha);
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);

    let dir = temp_dir();
    let mut app = Application::open(dir.join("catalog.sqlite")).unwrap();
    let mut character = CharacterAsset::new(name).unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), format!("{name} isolated"), evidence(path, sha))
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
            &format!("{name}-ual2"),
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
    Seeded {
        compatibility_id: result.as_record().id().canonical(),
        mapping: published.into_record(),
        app,
    }
}

fn publish_transfer(seeded: &mut Seeded, display: &str) -> (String, PathBuf) {
    let denied = seeded
        .app
        .authorize_transfer(&seeded.compatibility_id, false)
        .unwrap();
    assert!(!denied.eligible);
    let auth = seeded
        .app
        .authorize_transfer(&seeded.compatibility_id, true)
        .unwrap();
    assert!(auth.eligible);
    let workspace = temp_dir();
    let (_spec, run) = seeded
        .app
        .start_transfer(&seeded.compatibility_id, true, None, display)
        .unwrap();
    seeded.app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::production()
        .unwrap()
        .with_workspace_root(workspace);
    let (_running, _receipt) = seeded.app.dispatch(&run.run_id, &mut worker).unwrap();
    let (done, outcome) = seeded.app.collect(&run.run_id, &mut worker).unwrap();
    let TerminalOutcome::Success(result) = outcome else {
        panic!("worker did not succeed: {outcome:?} state={:?}", done.state);
    };
    assert!(result.as_record().worker_success());
    let adapter = result.as_record().execution().adapter_version();
    assert_eq!(adapter, "rigforge-blender-worker/0.1.2");
    let staged = worker
        .last_staged_blend()
        .expect("staged blend")
        .to_path_buf();
    let transfer = seeded.app.finalize_transfer(&run.run_id, &staged).unwrap();
    assert_eq!(transfer.kind, TransferOutcomeKind::Published);
    let version = seeded
        .app
        .catalog()
        .load_derived_variant_version(&transfer.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
    let artifact = seeded
        .app
        .catalog()
        .load_artifact_metadata(transfer.persistence_artifact_id.as_ref().unwrap())
        .unwrap();
    let durable = seeded.app.catalog().durable_artifact_path(artifact.as_record());
    (transfer.derived_variant_version_id, durable)
}

fn run_semantic_fixture(
    character: &str,
    derived: &Path,
    mapping: &BoneMappingVersion,
    out: &Path,
) -> Value {
    let map_path = out.join("mapping.json");
    std::fs::write(
        &map_path,
        serde_json::to_vec_pretty(&mapping_projection(mapping)).unwrap(),
    )
    .unwrap();
    let exe = default_blender_executable();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("python")
        .join("semantic_acceptance_fixture.py");
    let status = Command::new(&exe)
        .arg(BACKGROUND)
        .arg(FACTORY_STARTUP)
        .arg(DISABLE_AUTOEXEC)
        .arg(PYTHON_EXIT_CODE)
        .arg("1")
        .arg(PYTHON)
        .arg(&script)
        .arg("--")
        .arg(out)
        .arg(character)
        .arg(UAL2)
        .arg(derived)
        .arg(&map_path)
        .arg(CLIP)
        .arg(UAL2_RM)
        .status()
        .unwrap();
    let path = out.join("semantic_acceptance.json");
    if !path.is_file() {
        panic!(
            "semantic fixture exit {:?}; {}",
            status.code(),
            std::fs::read_to_string(out.join("semantic_exception.json")).unwrap_or_default()
        );
    }
    let payload: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(status.code(), Some(0), "{payload}");
    payload
}

#[test]
fn wanderer_standard_body_aligned_product_semantics() {
    let mut seeded = seed_character("WandererMale01", WANDERER, WANDERER_SHA);
    let (derived_id, durable) = publish_transfer(&mut seeded, "Wanderer Walk Carry");
    let out = temp_dir();
    let payload = run_semantic_fixture(WANDERER, &durable, &seeded.mapping, &out);
    assert_eq!(payload["status"], "PASS", "{payload}");
    let metrics = &payload["metrics"];
    for key in [
        "HAND_FORWARDNESS",
        "HAND_VERTICAL_SEMANTICS",
        "TORSO_LEAN_SIGN",
        "STEP_DIRECTION_SIGN",
        "KNEE_BEND_DIRECTION",
        "LEFT_RIGHT_PRESERVATION",
        "FOOT_SWING_AMPLITUDE",
        "SEMANTIC_FROZEN_PAIR",
    ] {
        assert_eq!(metrics[key], "PASS", "{key} {payload}");
    }
    assert_eq!(payload["rm_root"]["status"], "PASS", "{payload}");
    assert_eq!(metrics["motion_used_for_body_frame"], false);
    assert_eq!(metrics["name_heuristics_used"], false);
    let angle = metrics["alignment_angle_deg"].as_f64().unwrap();
    assert!((angle - 180.0).abs() < 15.0, "expected ~180 yaw, got {angle}");
    let _ = derived_id;
}

#[test]
fn zombie_standard_body_frame_is_not_wanderer_specific() {
    let mut seeded = seed_character("ZombieMale01", ZOMBIE, ZOMBIE_SHA);
    let (_derived_id, durable) = publish_transfer(&mut seeded, "Zombie Walk Carry");
    let out = temp_dir();
    let payload = run_semantic_fixture(ZOMBIE, &durable, &seeded.mapping, &out);
    assert_eq!(payload["status"], "PASS", "{payload}");
    assert_eq!(payload["metrics"]["SEMANTIC_FROZEN_PAIR"], "PASS", "{payload}");
}
