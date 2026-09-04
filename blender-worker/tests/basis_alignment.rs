mod common;

use std::path::PathBuf;
use std::process::Command;

use common::temp_dir;
use rigforge_blender_worker::{
    default_blender_executable, enforce_pin, BlenderPin, BACKGROUND, DISABLE_AUTOEXEC,
    FACTORY_STARTUP, PYTHON, PYTHON_EXIT_CODE,
};

fn fixture_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("python")
        .join("basis_alignment_fixture.py")
}

fn read_json(path: &PathBuf) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("missing {}: {e}", path.display())
    }))
    .unwrap()
}

#[test]
fn rest_relative_world_delta_survives_mismatched_bone_roll() {
    common::ensure_test_runtime();
    let exe = default_blender_executable();
    enforce_pin(&exe, &BlenderPin::accepted()).unwrap();
    let out = temp_dir();
    let status = Command::new(&exe)
        .arg(BACKGROUND)
        .arg(FACTORY_STARTUP)
        .arg(DISABLE_AUTOEXEC)
        .arg(PYTHON_EXIT_CODE)
        .arg("1")
        .arg(PYTHON)
        .arg(fixture_script())
        .arg("--")
        .arg(&out)
        .status()
        .unwrap();
    let path = out.join("basis_alignment_result.json");
    if !path.is_file() {
        panic!(
            "fixture exit {:?}; exception={}",
            status.code(),
            std::fs::read_to_string(out.join("basis_alignment_exception.json")).unwrap_or_default()
        );
    }
    let payload = read_json(&path);
    assert_eq!(
        payload["legacy_status"], "FAIL",
        "source-rest-local delta must fail under a 90-degree roll mismatch: {payload}"
    );
    assert_eq!(
        payload["world_status"], "PASS",
        "world-space rest-relative delta must restore semantic swing: {payload}"
    );
    assert_eq!(payload["status"], "PASS", "{payload}");
    let basis = &payload["basis"];
    let limb_err = basis["semantic_limb_dir_err_deg"].as_f64().unwrap();
    let axis_err = basis["local_x_err_deg"].as_f64().unwrap();
    assert!(
        limb_err < 1.0,
        "semantic rest limb directions must match: {payload}"
    );
    assert!(
        axis_err > 45.0,
        "local bone X axes must differ (roll mismatch): {payload}"
    );
    let world = &payload["world"];
    assert_eq!(world["keep_target_rest_scale"], "PASS", "{payload}");
    assert_eq!(world["quaternion_policy"], "PASS", "{payload}");
    assert_eq!(world["nan_count"], 0, "{payload}");
    assert_eq!(world["laterality_ok"], true, "{payload}");
    assert_eq!(world["apply_passes"], 1, "{payload}");
    let endpoint_err = world["endpoint_err_deg"].as_f64().unwrap();
    assert!(endpoint_err < 8.0, "child endpoint world direction: {payload}");
    assert_eq!(status.code(), Some(0), "{payload}");
}
