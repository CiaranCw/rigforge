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
        .join("body_frame_fixture.py")
}

fn read_json(path: &PathBuf) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("missing {}: {e}", path.display())
    }))
    .unwrap()
}

fn run_fixture() -> serde_json::Value {
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
    let path = out.join("body_frame_result.json");
    if !path.is_file() {
        panic!(
            "fixture exit {:?}; exception={}",
            status.code(),
            std::fs::read_to_string(out.join("body_frame_exception.json")).unwrap_or_default()
        );
    }
    let payload = read_json(&path);
    assert_eq!(status.code(), Some(0), "{payload}");
    payload
}

#[test]
fn body_frame_alignment_cases() {
    let payload = run_fixture();
    assert_eq!(payload["status"], "PASS", "{payload}");
    assert_eq!(payload["zero"]["status"], "PASS", "{payload}");
    assert_eq!(payload["yaw90"]["status"], "PASS", "{payload}");
    assert_eq!(payload["yaw180"]["status"], "PASS", "{payload}");
    assert_eq!(payload["roll_and_body"]["status"], "PASS", "{payload}");
    assert_eq!(payload["degenerate"]["status"], "PASS", "{payload}");
    assert_eq!(payload["reflection"]["status"], "PASS", "{payload}");
    assert_eq!(payload["root_travel"]["status"], "PASS", "{payload}");
    assert_eq!(payload["root_inplace"]["status"], "PASS", "{payload}");
    assert_eq!(payload["root_inplace"]["synthetic_forward_motion"], false);
    assert!(payload["zero"]["cross_order_invariant"].as_bool().unwrap());
    assert!(payload["yaw180"]["cross_order_invariant"].as_bool().unwrap());
    let zero_angle = payload["zero"]["alignment_angle_deg"].as_f64().unwrap();
    assert!(zero_angle < 8.0, "{payload}");
    let yaw180 = payload["yaw180"]["alignment_angle_deg"].as_f64().unwrap();
    assert!((yaw180 - 180.0).abs() < 8.0, "{payload}");
    let yaw90 = payload["yaw90"]["alignment_angle_deg"].as_f64().unwrap();
    assert!((yaw90 - 90.0).abs() < 8.0, "{payload}");
    assert_eq!(payload["adapter_version"], "rigforge-blender-worker/0.1.2");
}
