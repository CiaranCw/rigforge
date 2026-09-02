mod common;

use std::fs;
use std::path::Path;

use common::temp_dir;
use rigforge_app::sha256_file;
use rigforge_blender_worker::{enforce_pin, BlenderPin, BlenderQcInspector};

#[test]
fn qc_inspect_missing_artifact_fails() {
    let inspector = BlenderQcInspector::production().unwrap();
    let missing = temp_dir().join("missing.blend");
    let err = inspector
        .inspect(&missing, "a".repeat(64).as_str())
        .unwrap_err();
    assert!(err.to_string().contains("missing"));
}

#[test]
fn qc_inspect_invalid_open_fails_without_mutating_bytes() {
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    let dir = temp_dir();
    let path = dir.join("not_a_blend.blend");
    fs::write(&path, b"not a blender file").unwrap();
    let sha = sha256_file(&path).unwrap();
    let inspector = BlenderQcInspector::production().unwrap();
    let before = sha256_file(&path).unwrap();
    let err = inspector.inspect(&path, &sha).unwrap_err();
    let after = sha256_file(&path).unwrap();
    assert_eq!(before, after);
    assert_eq!(before, sha);
    assert!(
        err.to_string().contains("inspect_qc")
            || err.to_string().contains("open")
            || err.to_string().contains("failed"),
        "{err}"
    );
    let _ = Path::new(&path);
}
