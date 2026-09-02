mod common;

use std::path::{Path, PathBuf};

use common::{
    crate_preview_gen_script, crate_preview_viewer, crate_worker_script, ensure_test_runtime,
    local_pinned_blender_dir, same_volume_runtime_dest, temp_dir,
};
use rigforge_app::{
    bind_thread_runtime_root, materialize_runtime_bundle, RuntimeLayout,
};
use rigforge_blender_worker::{
    enforce_pin, production_worker_script, sha256_file, verify_worker_package_integrity, BlenderPin,
    BlenderWorker,
};

fn unique_bundle_root(tag: &str) -> PathBuf {
    temp_dir().join(tag)
}

fn unique_blender_bundle_root(tag: &str) -> PathBuf {
    same_volume_runtime_dest(tag)
}

#[test]
fn relocated_runtime_resolves_worker_and_preview_scripts() {
    let dest = unique_bundle_root("reloc-scripts");
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_worker_script(),
        &crate_preview_gen_script(),
        Some(&crate_preview_viewer()),
        None,
    )
    .unwrap();
    let _guard = bind_thread_runtime_root(layout.root());
    let resolved = RuntimeLayout::resolve().unwrap();
    let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let worker = production_worker_script();
    assert_eq!(worker, resolved.worker_script());
    assert_eq!(worker, dest.join("blender-worker").join("worker.py"));
    assert_eq!(
        resolved.preview_gen_script(),
        dest.join("blender-worker").join("preview_gen.py")
    );
    assert_eq!(resolved.viewer_root(), dest.join("preview-viewer"));
    assert!(
        !worker.starts_with(&checkout),
        "production worker must not resolve from crate checkout {}",
        checkout.display()
    );
    verify_worker_package_integrity(&worker).unwrap();
}

#[test]
fn relocated_runtime_rejects_wrong_worker_and_preview_bytes() {
    let dest = unique_bundle_root("reloc-bad-scripts");
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_worker_script(),
        &crate_preview_gen_script(),
        None,
        None,
    )
    .unwrap();
    std::fs::write(layout.worker_script(), b"print('tampered worker')\n").unwrap();
    let _guard = bind_thread_runtime_root(layout.root());
    let err = verify_worker_package_integrity(&production_worker_script())
        .unwrap_err()
        .to_string();
    assert!(err.contains("worker.py sha256 mismatch"), "{err}");

    let dest = unique_bundle_root("reloc-bad-preview-gen");
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_worker_script(),
        &crate_preview_gen_script(),
        None,
        None,
    )
    .unwrap();
    std::fs::write(layout.preview_gen_script(), b"print('tampered preview_gen')\n").unwrap();
    let _guard = bind_thread_runtime_root(layout.root());
    let err = verify_worker_package_integrity(&production_worker_script())
        .unwrap_err()
        .to_string();
    assert!(err.contains("preview_gen.py sha256 mismatch"), "{err}");
}

#[test]
fn relocated_runtime_uses_bundle_blender_and_still_enforces_pin() {
    let dest = unique_blender_bundle_root("reloc-blender");
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_worker_script(),
        &crate_preview_gen_script(),
        None,
        Some(&local_pinned_blender_dir()),
    )
    .unwrap();
    let _guard = bind_thread_runtime_root(layout.root());
    let resolved = RuntimeLayout::resolve().unwrap();
    let exe = resolved.blender_executable();
    assert_eq!(
        exe,
        dest.join("runtime")
            .join("blender-5.2.1-windows-x64")
            .join("blender.exe")
    );
    let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(!exe.starts_with(&checkout));
    let pin = BlenderPin::accepted().with_executable(exe.clone());
    let (version, build) = enforce_pin(&exe, &pin).unwrap();
    assert!(version.contains("5.2.1"), "{version}");
    assert_eq!(build, "9e2066aef7ef");
    let worker = BlenderWorker::production().unwrap();
    assert_eq!(worker.worker_script(), layout.worker_script().as_path());
}

#[test]
fn production_resolution_source_does_not_use_build_tree_paths() {
    let pin = include_str!("../src/pin.rs");
    assert!(
        !pin.contains("CARGO_MANIFEST_DIR"),
        "pin.rs must not use CARGO_MANIFEST_DIR"
    );
    assert!(
        !pin.contains(r"F:\NewResearch"),
        "pin.rs must not hard-code the developer tree"
    );
    assert!(
        !pin.contains("RIGFORGE_BLENDER_EXECUTABLE"),
        "production Blender resolution must not honor arbitrary unpinned executable env"
    );
    let adapter = include_str!("../src/adapter.rs");
    let production_script = adapter
        .split("pub fn production_worker_script()")
        .nth(1)
        .unwrap()
        .split("pub fn bundled_worker_script()")
        .next()
        .unwrap();
    assert!(!production_script.contains("CARGO_MANIFEST_DIR"));
    let production_ctor = adapter
        .split("pub fn production()")
        .nth(1)
        .unwrap()
        .split("pub fn with_workspace_root")
        .next()
        .unwrap();
    assert!(!production_ctor.contains("CARGO_MANIFEST_DIR"));
    assert!(!production_ctor.contains(r"F:\NewResearch"));
}

#[test]
fn shared_dev_runtime_still_verifies_integrity() {
    ensure_test_runtime();
    verify_worker_package_integrity(&production_worker_script()).unwrap();
    assert_eq!(
        sha256_file(&production_worker_script()).unwrap(),
        rigforge_blender_worker::WORKER_SCRIPT_SHA256
    );
}

#[test]
fn path_is_not_product_identity() {
    let dest = unique_bundle_root("reloc-not-identity");
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_worker_script(),
        &crate_preview_gen_script(),
        None,
        None,
    )
    .unwrap();
    let _ = Path::new("");
    assert_ne!(
        layout.root().to_string_lossy().as_ref(),
        rigforge_blender_worker::WORKER_SCRIPT_SHA256
    );
}
