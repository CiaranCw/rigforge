mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::temp_dir;
use rigforge_blender_worker::{
    default_blender_executable, enforce_pin, BlenderPin, BACKGROUND, DISABLE_AUTOEXEC,
    FACTORY_STARTUP, PYTHON, PYTHON_EXIT_CODE,
};

fn fixture_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("python")
        .join("root_scale_fixture.py")
}

fn run_blender_fixture(args: &[&str]) -> (i32, PathBuf) {
    let exe = default_blender_executable();
    let pin = BlenderPin::accepted();
    enforce_pin(&exe, &pin).unwrap();
    let out = temp_dir();
    let script = fixture_script();
    let status = Command::new(&exe)
        .arg(BACKGROUND)
        .arg(FACTORY_STARTUP)
        .arg(DISABLE_AUTOEXEC)
        .arg(PYTHON_EXIT_CODE)
        .arg("1")
        .arg(PYTHON)
        .arg(&script)
        .arg("--")
        .args(args)
        .arg(&out)
        .status()
        .unwrap();
    (status.code().unwrap_or(-1), out)
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("missing {}: {e}", path.display())
    }))
    .unwrap()
}

fn keep_scale_result() -> serde_json::Value {
    let (code, out) = run_blender_fixture(&["keep"]);
    let path = out.join("root_scale_result.json");
    if !path.is_file() {
        panic!(
            "fixture exit {code}; exception={}",
            std::fs::read_to_string(out.join("root_scale_exception.json")).unwrap_or_default()
        );
    }
    assert_eq!(code, 0, "keep fixture must exit 0");
    let mut payload = read_json(&path);
    payload["_out"] = serde_json::json!(out.display().to_string());
    payload
}

#[test]
fn animated_source_root_scale_does_not_propagate() {
    let payload = keep_scale_result();
    assert_eq!(payload["status"], "PASS");
    assert_eq!(payload["keep_target_rest_scale"], "PASS");
    assert_eq!(payload["root_scale_audit"], "PASS");
    assert_eq!(payload["source_scale_end"], 3.0);
    let pose = payload["pose_scale_end"].as_array().unwrap();
    for v in pose {
        assert!((v.as_f64().unwrap() - 1.0).abs() < 1e-4, "{payload}");
    }
    let world = payload["world_scale_end"].as_array().unwrap();
    let rest = payload["rest_scale"].as_array().unwrap();
    for i in 0..3 {
        assert!(
            (world[i].as_f64().unwrap() - rest[i].as_f64().unwrap()).abs() < 1e-4,
            "{payload}"
        );
    }
    assert!(payload["scale_fcurves"].as_array().unwrap().is_empty(), "{payload}");
    let loc = payload["world_location_end"].as_array().unwrap();
    let moved = loc.iter().any(|v| v.as_f64().unwrap().abs() > 0.1);
    assert!(moved, "root translation should transfer: {payload}");
}

#[test]
fn root_scale_audit_covers_mapped_root() {
    let payload = keep_scale_result();
    assert_eq!(payload["mapped_root_included"], true);
    assert_eq!(payload["root_scale_audit"], "PASS");
}

#[test]
fn root_scale_policy_violation_cannot_return_success() {
    let (code, out) = run_blender_fixture(&["violate"]);
    let payload = read_json(&out.join("root_scale_result.json"));
    assert_eq!(code, 0, "violate fixture reports FAIL audit with exit 0");
    assert_eq!(payload["status"], "FAIL");
    assert_eq!(payload["expected"], "FAIL");
    assert_eq!(payload["worker_success"], false);
    assert_eq!(payload["root_scale_audit"], "FAIL");
}

#[test]
fn reopen_preserves_target_root_rest_scale() {
    let payload = keep_scale_result();
    assert_eq!(payload["status"], "PASS");
    let out = PathBuf::from(payload["_out"].as_str().unwrap());
    let blend = PathBuf::from(payload["staged_blend"].as_str().unwrap());
    assert!(blend.is_file(), "{}", blend.display());
    let exe = default_blender_executable();
    enforce_pin(&exe, &BlenderPin::accepted()).unwrap();
    let reopen_out = temp_dir();
    let status = Command::new(&exe)
        .arg(BACKGROUND)
        .arg(FACTORY_STARTUP)
        .arg(DISABLE_AUTOEXEC)
        .arg(PYTHON_EXIT_CODE)
        .arg("1")
        .arg(PYTHON)
        .arg(fixture_script())
        .arg("--")
        .arg("reopen")
        .arg(&reopen_out)
        .arg(&blend)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0), "reopen exit");
    let reopen = read_json(&reopen_out.join("root_scale_reopen.json"));
    assert_eq!(reopen["root_scale_audit"], "PASS");
    assert_eq!(reopen["status"], "PASS");
    let pose = reopen["pose_scale_end"].as_array().unwrap();
    for v in pose {
        assert!((v.as_f64().unwrap() - 1.0).abs() < 1e-4, "{reopen}");
    }
    let _ = out;
}
