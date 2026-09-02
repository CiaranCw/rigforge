mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{sha256_file, temp_dir};
use rigforge_app::{
    generate_mapping_proposal, Application, MappingAssistProfile, PreviewGenerationRequest,
    PreviewSubject, TerminalOutcome, TransferOutcomeKind, WorkerCapabilityProfile,
};
use rigforge_blender_worker::{
    enforce_pin, BlenderPin, BlenderSkeletonInspector, BlenderWorker,
};
use rigforge_domain::*;
use rigforge_workbench::preview_host::PreviewHost;

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
        Some("V1-6 frozen pair".into()),
        None,
    )
    .unwrap()
}

struct Seeded {
    app: Application,
    character_version: String,
    motion_version: String,
    compatibility_id: String,
}

fn persist_frozen_character_motion(app: &mut Application) -> (String, String) {
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
    (
        character_version.id().canonical(),
        motion_version.id().canonical(),
    )
}

fn seed_sources_only() -> Seeded {
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(KNIGHT)), KNIGHT_SHA);
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);
    let dir = temp_dir();
    let mut app = Application::open(dir.join("catalog.sqlite")).unwrap();
    let (character_version, motion_version) = persist_frozen_character_motion(&mut app);
    Seeded {
        compatibility_id: String::new(),
        character_version,
        motion_version,
        app,
    }
}
fn seed_frozen() -> Seeded {
    let mut seeded = seed_sources_only();
    let character_version = seeded.character_version.clone();
    let motion_version = seeded.motion_version.clone();
    let source_skeleton = seeded
        .app
        .catalog()
        .load_motion_version(&motion_version)
        .unwrap()
        .as_record()
        .source_skeleton_ref_id()
        .canonical();
    let mut policy = RetargetPolicy::new("rest-relative-proven").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());
    seeded
        .app
        .catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy.clone()).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();
    let inspector = BlenderSkeletonInspector::production().unwrap();
    let target_summary = seeded
        .app
        .inspect_and_store_character_summary(&character_version, &inspector)
        .unwrap();
    let source_summary = seeded
        .app
        .inspect_and_store_source_summary(&source_skeleton, &inspector, &motion_version)
        .unwrap();
    let proposal = generate_mapping_proposal(
        source_summary.as_record(),
        target_summary.as_record(),
        MappingAssistProfile::OptionalHumanoid,
    )
    .unwrap();
    let (logical, draft) = seeded
        .app
        .store_mapping_draft(
            "knight-ual2-v16",
            &character_version,
            &source_skeleton,
            &proposal,
            &source_summary.as_record().id().canonical(),
            &target_summary.as_record().id().canonical(),
        )
        .unwrap();
    let published = seeded
        .app
        .accept_mapping_version(
            &logical.as_record().id().canonical(),
            &draft.as_record().id().canonical(),
        )
        .unwrap();
    let result = seeded
        .app
        .run_compatibility_preflight(
            &character_version,
            &motion_version,
            &published.as_record().id().canonical(),
            &policy_version.id().canonical(),
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )
        .unwrap();
    seeded.compatibility_id = result.as_record().id().canonical();
    seeded
}

fn publish_derived(seeded: &mut Seeded) -> String {
    let ws = temp_dir();
    let auth = seeded
        .app
        .authorize_transfer(&seeded.compatibility_id, true)
        .unwrap();
    assert!(auth.eligible);
    let (_spec, run) = seeded
        .app
        .start_transfer(&seeded.compatibility_id, true, None, "Knight Walk Carry")
        .unwrap();
    seeded.app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::production()
        .unwrap()
        .with_workspace_root(ws);
    seeded.app.dispatch(&run.run_id, &mut worker).unwrap();
    let (done, outcome) = seeded.app.collect(&run.run_id, &mut worker).unwrap();
    let TerminalOutcome::Success(_) = outcome else {
        panic!("worker did not succeed: {outcome:?} state={:?}", done.state);
    };
    let staged = worker.last_staged_blend().unwrap().to_path_buf();
    let transfer = seeded.app.finalize_transfer(&run.run_id, &staged).unwrap();
    assert_eq!(transfer.kind, TransferOutcomeKind::Published);
    transfer.derived_variant_version_id
}

fn snapshot_product(app: &Application, character_id: &str, motion_id: &str, derived_id: Option<&str>) -> String {
    let c = app.catalog().load_character_version(character_id).unwrap();
    let m = app.catalog().load_motion_version(motion_id).unwrap();
    let mut out = format!(
        "{}:{}:{}:{}",
        c.as_record().id().canonical(),
        c.as_record().source().digest().sha256(),
        m.as_record().id().canonical(),
        m.as_record().source().digest().sha256()
    );
    if let Some(id) = derived_id {
        let d = app.catalog().load_derived_variant_version(id).unwrap();
        out.push_str(&format!(
            ":{}:{:?}",
            d.as_record().id().canonical(),
            d.as_record().lifecycle()
        ));
    }
    out
}

fn measure(url: &str, kind: &str) -> serde_json::Value {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("workbench")
        .join("preview-viewer")
        .join("measure_controls.py");
    let output = Command::new("python")
        .args([script.to_str().unwrap(), url, kind])
        .output()
        .expect("python measure_controls.py");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "viewer measure failed kind={kind} stdout={stdout} stderr={stderr}"
    );
    serde_json::from_str(&stdout).unwrap_or_else(|_| serde_json::json!({ "raw": stdout }))
}

fn preview_record(
    kind: &str,
    artifact: &PreviewArtifact,
    view: &rigforge_app::PreviewView,
    measured: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "selected_product_kind": kind,
        "exact_product_version_id": artifact.bound_product_version_id(),
        "preview_artifact_id": artifact.id().canonical(),
        "producer_id": artifact.producer_id().canonical(),
        "payload_media_type": artifact.media_type(),
        "payload_sha256": artifact.digest().sha256(),
        "payload_size": artifact.size_bytes(),
        "exact_binding_result": view.is_valid(),
        "viewer_load_result": measured.get("loaded"),
        "animation_inventory": view.descriptor.as_ref().map(|d| d.animation_names.clone()),
        "declared_losses": view.descriptor.as_ref().map(|d| d.declared_losses.clone()),
        "control_measurements": {
            "camera": measured.get("camera"),
            "transport": measured.get("transport"),
            "offline_local": measured.get("offline_local"),
        }
    })
}

fn maybe_write_evidence(name: &str, value: &serde_json::Value) {
    let Ok(dir) = std::env::var("RIGFORGE_V16_EVIDENCE") else {
        return;
    };
    let path = PathBuf::from(dir).join(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serde_json::to_vec_pretty(value).unwrap());
}

#[test]
fn character_motion_derived_preview_checkpoint() {
    let mut seeded = seed_frozen();
    let character_id = seeded.character_version.clone();
    let motion_id = seeded.motion_version.clone();
    let product_before = snapshot_product(&seeded.app, &character_id, &motion_id, None);

    let character_preview = seeded
        .app
        .generate_preview(PreviewGenerationRequest::character(&character_id))
        .unwrap();
    let char_view = seeded.app.resolve_preview_for_display(PreviewSubject::Character {
        version_id: character_id.clone(),
    });
    assert!(char_view.is_valid(), "{:?}", char_view.failure);
    assert!(!char_view.descriptor.as_ref().unwrap().has_animation);

    let motion_preview = seeded
        .app
        .generate_preview(PreviewGenerationRequest::motion(&motion_id))
        .unwrap();
    let motion_view = seeded.app.resolve_preview_for_display(PreviewSubject::Motion {
        version_id: motion_id.clone(),
    });
    assert!(motion_view.is_valid(), "{:?}", motion_view.failure);
    assert!(motion_view.descriptor.as_ref().unwrap().has_animation);
    assert!(motion_preview
        .as_record()
        .bound_character_version_id()
        .is_none());

    let derived_id = publish_derived(&mut seeded);
    let derived_preview = seeded
        .app
        .generate_preview(PreviewGenerationRequest::derived_variant(&derived_id))
        .unwrap();
    let derived_view = seeded.app.resolve_preview_for_display(PreviewSubject::DerivedVariant {
        version_id: derived_id.clone(),
    });
    assert!(derived_view.is_valid(), "{:?}", derived_view.failure);
    assert!(derived_view.descriptor.as_ref().unwrap().has_animation);

    let product_after = snapshot_product(&seeded.app, &character_id, &motion_id, Some(&derived_id));
    assert!(product_after.starts_with(&product_before));

    let char_session = seeded
        .app
        .materialize_preview_session(PreviewSubject::Character {
            version_id: character_id.clone(),
        })
        .unwrap();
    let motion_session = seeded
        .app
        .materialize_preview_session(PreviewSubject::Motion {
            version_id: motion_id.clone(),
        })
        .unwrap();
    let derived_session = seeded
        .app
        .materialize_preview_session(PreviewSubject::DerivedVariant {
            version_id: derived_id.clone(),
        })
        .unwrap();
    let char_host = PreviewHost::serve(&char_session).unwrap();
    let motion_host = PreviewHost::serve(&motion_session).unwrap();
    let derived_host = PreviewHost::serve(&derived_session).unwrap();
    let char_m = measure(&char_host.url, "character");
    let motion_m = measure(&motion_host.url, "motion");
    let derived_m = measure(&derived_host.url, "derived");
    assert_eq!(char_m["loaded"], true);
    assert_eq!(char_m["camera"]["orbit_changed"], true);
    assert_eq!(char_m["camera"]["zoom_changed"], true);
    assert_eq!(motion_m["transport"]["play_advanced"], true);
    assert_eq!(motion_m["transport"]["pause_held"], true);
    assert_eq!(motion_m["transport"]["seek_ok"], true);
    assert_eq!(motion_m["transport"]["restart_near_start"], true);
    assert_eq!(derived_m["transport"]["play_advanced"], true);
    assert_eq!(derived_m["transport"]["pause_held"], true);

    maybe_write_evidence(
        "character_preview.json",
        &preview_record(
            "CharacterAssetVersion",
            character_preview.as_record(),
            &char_view,
            &char_m,
        ),
    );
    maybe_write_evidence(
        "motion_preview.json",
        &preview_record(
            "MotionAssetVersion",
            motion_preview.as_record(),
            &motion_view,
            &motion_m,
        ),
    );
    maybe_write_evidence(
        "derived_preview.json",
        &preview_record(
            "DerivedVariantVersion",
            derived_preview.as_record(),
            &derived_view,
            &derived_m,
        ),
    );
    maybe_write_evidence(
        "viewer_controls.json",
        &serde_json::json!({
            "character": char_m,
            "motion": motion_m,
            "derived": derived_m,
        }),
    );
}

#[test]
fn synthetic_non_humanoid_motion_proxy() {
    let mut seeded = seed_sources_only();
    let stored = seeded
        .app
        .generate_preview(PreviewGenerationRequest::synthetic_motion(
            &seeded.motion_version,
        ))
        .unwrap();
    let view = seeded.app.resolve_preview_for_display(PreviewSubject::Motion {
        version_id: seeded.motion_version.clone(),
    });
    assert!(view.is_valid(), "{:?}", view.failure);
    let desc = view.descriptor.as_ref().unwrap();
    assert!(desc.has_animation);
    assert!(desc
        .declared_losses
        .iter()
        .any(|l| l.contains("non-humanoid") || l.contains("proxy") || l.contains("synthetic")));
    assert!(stored.as_record().bound_character_version_id().is_none());
    maybe_write_evidence(
        "non_humanoid_preview.json",
        &serde_json::json!({
            "selected_product_kind": "MotionAssetVersion",
            "exact_product_version_id": stored.as_record().bound_product_version_id(),
            "preview_artifact_id": stored.as_record().id().canonical(),
            "producer_id": stored.as_record().producer_id().canonical(),
            "payload_media_type": stored.as_record().media_type(),
            "payload_sha256": stored.as_record().digest().sha256(),
            "payload_size": stored.as_record().size_bytes(),
            "exact_binding_result": view.is_valid(),
            "has_animation": desc.has_animation,
            "declared_losses": desc.declared_losses,
            "bound_character": stored.as_record().bound_character_version_id().map(|id| id.canonical()),
        }),
    );
}

#[test]
fn generator_failure_does_not_revoke_publication() {
    let mut seeded = seed_sources_only();
    let before = snapshot_product(
        &seeded.app,
        &seeded.character_version,
        &seeded.motion_version,
        None,
    );
    let err = seeded
        .app
        .generate_preview(PreviewGenerationRequest::character(
            &uuid::Uuid::now_v7().to_string(),
        ))
        .unwrap_err();
    assert!(
        err.to_string().contains("not found")
            || err.to_string().contains("Preview")
            || err.to_string().contains("parse")
            || err.to_string().contains("invalid")
            || err.to_string().contains("UUID")
            || err.to_string().contains("Catalog"),
        "{err}"
    );
    let after = snapshot_product(
        &seeded.app,
        &seeded.character_version,
        &seeded.motion_version,
        None,
    );
    assert_eq!(before, after);
}
