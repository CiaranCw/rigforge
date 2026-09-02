mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use common::unpublished_graph;
use rigforge_app::{
    bind_thread_runtime_root, inspect_durable_persistence_artifact, materialize_runtime_bundle,
    reopen_durable_persistence_artifact, sha256_file, verify_runtime_worker_package, RuntimeLayout,
    RuntimeRootGuard,
};
use rigforge_domain::VerificationOutcome;

fn crate_python(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("blender-worker")
        .join("python")
        .join(name)
}

fn scratch(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rigforge-v18-major004-{label}-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn bind_scripts(label: &str) -> (RuntimeLayout, RuntimeRootGuard, PathBuf) {
    let dest = scratch(label);
    let layout = materialize_runtime_bundle(
        &dest,
        &crate_python("worker.py"),
        &crate_python("preview_gen.py"),
        None,
        None,
    )
    .unwrap();
    let guard = bind_thread_runtime_root(layout.root());
    (layout, guard, dest)
}

fn dummy_artifact(dir: &Path) -> (PathBuf, String) {
    let artifact = dir.join("dummy.blend");
    fs::write(&artifact, b"not-a-blend").unwrap();
    let digest = sha256_file(&artifact).unwrap();
    (artifact, digest)
}

#[test]
fn inspect_durable_persistence_artifact_rejects_tampered_worker_py() {
    let (layout, _guard, dest) = bind_scripts("inspect-worker");
    fs::write(layout.worker_script(), b"# tampered worker.py\nprint('nope')\n").unwrap();
    let (artifact, digest) = dummy_artifact(&dest);
    let err = inspect_durable_persistence_artifact(&artifact, &digest, None)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("worker.py sha256 mismatch"),
        "QC inspect must reject a tampered worker before trusting evidence, got {err}"
    );
}

#[test]
fn reopen_durable_persistence_artifact_rejects_tampered_worker_py() {
    let (layout, _guard, dest) = bind_scripts("reopen-worker");
    fs::write(layout.worker_script(), b"# tampered worker.py\nprint('nope')\n").unwrap();
    let (artifact, digest) = dummy_artifact(&dest);
    let mapping = unpublished_graph().mapping_version;
    let err = reopen_durable_persistence_artifact(&artifact, &digest, &mapping)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("worker.py sha256 mismatch"),
        "fresh reopen must fail closed on a tampered worker, got {err}"
    );
}

#[test]
fn inspect_durable_persistence_artifact_rejects_tampered_preview_gen_py() {
    let (layout, _guard, dest) = bind_scripts("inspect-preview-gen");
    fs::write(
        layout.preview_gen_script(),
        b"# tampered preview_gen.py\nprint('nope')\n",
    )
    .unwrap();
    let (artifact, digest) = dummy_artifact(&dest);
    let err = inspect_durable_persistence_artifact(&artifact, &digest, None)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("preview_gen.py sha256 mismatch"),
        "package integrity must reject tampered preview_gen.py; this is not QC semantics, got {err}"
    );
}

#[test]
fn verify_runtime_worker_package_rejects_tampered_preview_gen_py() {
    let (layout, _guard, _) = bind_scripts("verify-preview-gen");
    fs::write(
        layout.preview_gen_script(),
        b"# tampered preview_gen.py\nprint('nope')\n",
    )
    .unwrap();
    let err = verify_runtime_worker_package(&layout.worker_script())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("preview_gen.py sha256 mismatch"),
        "shared package verifier must reject tampered preview_gen.py, got {err}"
    );
}

#[test]
fn valid_runtime_package_passes_shared_verifier_and_does_not_fail_qc_on_integrity() {
    let (layout, _guard, dest) = bind_scripts("valid-package");
    verify_runtime_worker_package(&layout.worker_script()).unwrap();
    layout.verify_worker_package().unwrap();
    let (artifact, digest) = dummy_artifact(&dest);
    let inspect_err = inspect_durable_persistence_artifact(&artifact, &digest, None)
        .unwrap_err()
        .to_string();
    assert!(
        !inspect_err.contains("sha256 mismatch"),
        "valid package must not fail QC on worker integrity, got {inspect_err}"
    );
    let mapping = unpublished_graph().mapping_version;
    let reopen = reopen_durable_persistence_artifact(&artifact, &digest, &mapping);
    match reopen {
        Ok((fresh, structural)) => {
            assert_ne!(fresh, VerificationOutcome::Pass);
            assert_ne!(structural, VerificationOutcome::Pass);
        }
        Err(err) => {
            let msg = err.to_string();
            assert!(
                !msg.contains("sha256 mismatch"),
                "valid package must not fail reopen on worker integrity, got {msg}"
            );
        }
    }
}
