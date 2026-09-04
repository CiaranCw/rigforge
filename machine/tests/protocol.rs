use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rigforge_machine::{handle_line, MachineMessage, MachineSession};
use serde_json::{json, Value};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_fbx(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rf-machine-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

fn temp_catalog() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rf-machine-cat-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir.join("catalog.sqlite")
}

fn drive(session: &mut MachineSession, id: &str, op: &str, params: Value) -> Value {
    let line = serde_json::to_string(&json!({ "id": id, "op": op, "params": params })).unwrap();
    let mut messages = Vec::new();
    let mut emit = |message: MachineMessage| messages.push(message);
    handle_line(session, &line, &mut emit).unwrap();
    match messages.last() {
        Some(MachineMessage::Result {
            ok: true, result, ..
        }) => result.clone(),
        Some(MachineMessage::Result {
            ok: false, error, ..
        }) => panic!("op {op} failed: {error:?}"),
        other => panic!("unexpected machine messages for {op}: {other:?}"),
    }
}

fn drive_err(session: &mut MachineSession, id: &str, op: &str, params: Value) -> String {
    let line = serde_json::to_string(&json!({ "id": id, "op": op, "params": params })).unwrap();
    let mut messages = Vec::new();
    let mut emit = |message: MachineMessage| messages.push(message);
    handle_line(session, &line, &mut emit).unwrap();
    match messages.last() {
        Some(MachineMessage::Result {
            ok: false, error, ..
        }) => error.clone().unwrap_or_default(),
        other => panic!("expected failure for {op}, got {other:?}"),
    }
}

#[test]
fn ping_and_unknown_op() {
    let mut session = MachineSession::for_tests();
    let result = drive(&mut session, "1", "ping", json!({}));
    assert_eq!(result["ok"], true);
    let err = drive_err(&mut session, "2", "not_an_op", json!({}));
    assert!(err.contains("unknown machine op"), "{err}");
}

#[test]
fn runtime_status_reports_generic_machine_identity() {
    let mut session = MachineSession::for_tests();
    let result = drive(&mut session, "status", "runtime_status", json!({}));
    assert_eq!(result["machine_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        result["adapter_version"],
        rigforge_blender_worker::ADAPTER_VERSION
    );
    assert_eq!(result["backend_kind"], rigforge_blender_worker::BACKEND_KIND);
    assert_eq!(
        result["backend_version"],
        rigforge_blender_worker::BLENDER_VERSION
    );
    assert_eq!(
        result["execution_policy_version"],
        rigforge_blender_worker::EXECUTION_POLICY_VERSION
    );
    assert!(
        result.get("adapter_version").and_then(Value::as_str).is_some(),
        "adapter_version must be present even when runtime layout is unresolved"
    );
}

#[test]
fn proposal_is_not_accepted_mapping() {
    let mut session = MachineSession::for_tests();
    let catalog = temp_catalog();
    drive(
        &mut session,
        "open",
        "open_catalog",
        json!({ "path": catalog.display().to_string() }),
    );
    let character = temp_fbx("hero.fbx", b"character-bytes");
    let motion = temp_fbx("walk.fbx", b"motion-bytes");
    let inspected_motion = drive(
        &mut session,
        "im",
        "inspect_motion_source",
        json!({ "source_path": motion.display().to_string() }),
    );
    let clips = inspected_motion["usable_clips"].as_array().unwrap();
    assert!(clips.len() >= 2, "test inspector must not collapse to one clip");
    let walk = clips
        .iter()
        .find(|clip| clip["clip_identity"] == "Walk")
        .expect("Walk clip");
    assert_eq!(walk["clip_identity"], "Walk");

    let first_clip = clips[0]["clip_identity"].as_str().unwrap();
    let other = clips
        .iter()
        .find(|clip| clip["clip_identity"].as_str() != Some(first_clip))
        .unwrap()["clip_identity"]
        .as_str()
        .unwrap()
        .to_string();

    let character_reg = drive(
        &mut session,
        "rc",
        "register_character",
        json!({
            "display_name": "Hero",
            "source_path": character.display().to_string()
        }),
    );
    let wrong_clip = drive_err(
        &mut session,
        "bad-clip",
        "register_motion_clip",
        json!({
            "display_name": "Unused",
            "source_path": motion.display().to_string(),
            "clip_identity": "NotARealClip"
        }),
    );
    assert!(wrong_clip.contains("NotARealClip"), "{wrong_clip}");

    let motion_reg = drive(
        &mut session,
        "rm",
        "register_motion_clip",
        json!({
            "display_name": "Walk clip",
            "source_path": motion.display().to_string(),
            "clip_identity": other
        }),
    );
    assert_eq!(motion_reg["clip_identity"], other);

    let proposal = drive(
        &mut session,
        "prop",
        "propose_mapping",
        json!({
            "character_version_id": character_reg["version_id"],
            "motion_version_id": motion_reg["version_id"],
            "display_name": "test proposal"
        }),
    );
    assert_eq!(proposal["accepted"], false);
    assert_eq!(proposal["automatic_proposal"], true);
    assert_eq!(proposal["lifecycle"], "Draft");
    assert_eq!(proposal["review"]["accepted"], false);
    assert_eq!(proposal["review"]["confirmation_required"], true);

    let accepted = drive(
        &mut session,
        "acc",
        "accept_mapping",
        json!({
            "mapping_id": proposal["mapping_id"],
            "mapping_version_id": proposal["mapping_version_id"]
        }),
    );
    assert_eq!(accepted["accepted"], true);
    assert_eq!(accepted["lifecycle"], "Published");
}

#[test]
fn protocol_progress_then_result_lines_are_typed() {
    let encoded = MachineMessage::progress("1", "quality_check")
        .to_line()
        .unwrap();
    assert!(encoded.contains("\"type\":\"progress\""));
    assert!(encoded.contains("quality_check"));
    let result = MachineMessage::ok("1", json!({ "ok": true })).to_line().unwrap();
    assert!(result.contains("\"ok\":true"));
}
