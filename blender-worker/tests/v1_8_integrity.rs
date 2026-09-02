mod common;

use std::path::PathBuf;

use common::{crate_worker_script, ensure_test_runtime};
use rigforge_blender_worker::{
    production_worker_script, sha256_file, verify_worker_package_integrity, BlenderPin,
    PREVIEW_GEN_SCRIPT_SHA256, WORKER_SCRIPT_SHA256,
};

#[test]
fn bundled_worker_scripts_match_pinned_digests() {
    let worker = crate_worker_script();
    verify_worker_package_integrity(&worker).unwrap();
    assert_eq!(sha256_file(&worker).unwrap(), WORKER_SCRIPT_SHA256);
    let preview = worker.parent().unwrap().join("preview_gen.py");
    assert_eq!(sha256_file(&preview).unwrap(), PREVIEW_GEN_SCRIPT_SHA256);
}

#[test]
fn production_runtime_worker_scripts_match_pinned_digests() {
    ensure_test_runtime();
    let worker = production_worker_script();
    verify_worker_package_integrity(&worker).unwrap();
    let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !worker.starts_with(&checkout),
        "production resolution must use the runtime root, not the crate checkout"
    );
}

#[test]
fn replaced_worker_script_is_rejected() {
    let dir = std::env::temp_dir().join(format!(
        "rigforge-v18-integrity-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fake = dir.join("worker.py");
    std::fs::write(&fake, b"print('not the accepted worker')\n").unwrap();
    std::fs::write(dir.join("preview_gen.py"), b"print('also fake')\n").unwrap();
    let err = verify_worker_package_integrity(&fake).unwrap_err().to_string();
    assert!(err.contains("sha256 mismatch"), "{err}");
}

#[test]
fn replaced_preview_gen_script_is_rejected() {
    let dir = std::env::temp_dir().join(format!(
        "rigforge-v18-integrity-preview-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let worker = crate_worker_script();
    std::fs::copy(&worker, dir.join("worker.py")).unwrap();
    std::fs::write(dir.join("preview_gen.py"), b"print('not the accepted preview gen')\n").unwrap();
    let err = verify_worker_package_integrity(&dir.join("worker.py"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("preview_gen.py sha256 mismatch"), "{err}");
}

#[test]
fn accepted_blender_pin_constants_are_stable() {
    let pin = BlenderPin::accepted();
    assert_eq!(pin.version, "5.2.1 LTS");
    assert_eq!(pin.build, "9e2066aef7ef");
    assert_eq!(
        pin.archive_sha256,
        "0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c"
    );
}
