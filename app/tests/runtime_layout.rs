use std::path::PathBuf;

#[test]
fn production_preview_and_qc_do_not_use_build_tree_paths() {
    let preview = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/pinned_preview.rs"));
    let qc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/pinned_qc.rs"));
    let runtime = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime.rs"));
    for (name, src) in [
        ("pinned_preview.rs", preview),
        ("pinned_qc.rs", qc),
        ("runtime.rs", runtime),
    ] {
        assert!(
            !src.contains("CARGO_MANIFEST_DIR"),
            "{name} must not use CARGO_MANIFEST_DIR"
        );
        assert!(
            !src.contains(r"F:\NewResearch"),
            "{name} must not hard-code the developer tree"
        );
    }
    let preview_exe = preview
        .split("fn blender_executable()")
        .nth(1)
        .unwrap()
        .split("fn ")
        .next()
        .unwrap();
    assert!(!preview_exe.contains("RIGFORGE_BLENDER_EXECUTABLE"));
    assert!(
        qc.contains("verify_runtime_worker_package"),
        "QC inspect/reopen must use the shared runtime package verifier"
    );
    assert!(
        preview.contains("verify_runtime_worker_package"),
        "Preview generation must use the shared runtime package verifier"
    );
    let _ = PathBuf::from("runtime-root");
}
