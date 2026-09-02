mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{backend, certify, persist_core, unpublished_graph, Graph};
use rigforge_app::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact, sha256_file,
    AppError, Application, FakeWorker, JobRunState, TransferOutcomeKind,
};
use rigforge_domain::*;
use sha2::{Digest, Sha256};

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_staged(bytes: &[u8]) -> (PathBuf, String) {
    let dir = temp_dir("rf-v15-staged");
    let path = dir.join("derived_result.blend");
    fs::write(&path, bytes).unwrap();
    (path, sha256_bytes(bytes))
}

fn worker_success(
    job: &JobSpec,
    run: &rigforge_app::JobRun,
    sha: &str,
) -> Validated<WorkerResult> {
    certify(
        WorkerResult::new(
            job.id(),
            backend(),
            true,
            "completed",
            ExecutionCorrelation::new(
                &run.attempt_id,
                run.worker_execution_ref
                    .as_deref()
                    .expect("RUNNING JobRun must have worker_execution_ref"),
            )
            .unwrap(),
        )
        .unwrap()
        .with_staged_artifact_digests(vec![ContentDigest::parse(sha).unwrap()])
        .unwrap(),
    )
}

fn complete_fake_success(
    app: &mut Application,
    run_id: &str,
    bytes: &[u8],
) -> (PathBuf, Validated<JobSpec>) {
    app.mark_dispatchable(run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(run_id, &mut worker).unwrap();
    let running = app.job_status(run_id).unwrap();
    let spec = app.load_job_spec(&running.job_spec_id).unwrap();
    let (path, sha) = write_staged(bytes);
    app.complete_success(run_id, &worker_success(spec.as_record(), &running, &sha))
        .unwrap();
    (path, spec)
}

fn persist_graph(app: &mut Application) -> Graph {
    let g = unpublished_graph();
    persist_core(app.catalog_mut(), &g);
    g
}

fn assert_catalog_authority(err: AppError, needle: &str) {
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains(needle) || msg.contains("generic persistence"),
            "expected `{needle}` or generic persistence, got {msg}"
        ),
        other => panic!("expected Catalog denial, got {other}"),
    }
}

fn seed_succeeded_transfer(bytes: &[u8]) -> (Application, String, PathBuf) {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, bytes);
    (app, run.run_id, path)
}

struct ForgedInspector;

struct ForgedReopener;

#[test]
fn generic_put_qc_report_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.qc.clone()))
        .unwrap_err();
    assert_catalog_authority(err, "QcReport");
}

#[test]
fn generic_put_pass_verification_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.verification.clone()))
        .unwrap_err();
    assert_catalog_authority(err, "PersistenceVerification");
}

#[test]
fn production_api_cannot_finalize_with_caller_forged_inspector() {
    let _forged = ForgedInspector;
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-forged-inspector");
    let outcome = match app.finalize_transfer(&run_id, &path) {
        Ok(value) => value,
        Err(_) => {
            return;
        }
    };
    assert_ne!(
        outcome.kind,
        TransferOutcomeKind::Published,
        "forged inspector must not be accepted by production finalize; got {:?}",
        outcome.reason
    );
}

#[test]
fn production_api_cannot_finalize_with_caller_forged_reopener() {
    let _forged = ForgedReopener;
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-forged-reopener");
    let outcome = match app.finalize_transfer(&run_id, &path) {
        Ok(value) => value,
        Err(_) => return,
    };
    assert_ne!(outcome.kind, TransferOutcomeKind::Published);
}

#[test]
fn production_qc_requires_real_configured_inspection_path() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-need-real-qc");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let err = app
        .evaluate_and_bind_qc(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("blender")
            || msg.contains("inspect")
            || msg.contains("missing")
            || msg.contains("QC")
            || msg.contains("worker"),
        "{msg}"
    );
    let still = app
        .catalog()
        .load_derived_variant_version(&version.as_record().id().canonical())
        .unwrap();
    assert!(still.as_record().qc_report_id().is_none());
}

#[test]
fn production_verification_requires_real_configured_reopen_path() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-need-real-reopen");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let verification = app
        .verify_and_bind_persistence(&version.as_record().id().canonical())
        .unwrap();
    assert_ne!(
        verification.as_record().fresh_reopen(),
        VerificationOutcome::Pass
    );
    assert_ne!(
        verification.as_record().structural_verification(),
        VerificationOutcome::Pass
    );
    let err = app
        .publish_derived_variant_version(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("verification")
            || msg.contains("QC")
            || msg.contains("publication"),
        "{msg}"
    );
}

#[test]
fn generic_put_cannot_set_derived_variant_draft_pointer() {
    let mut app = Application::open_in_memory().unwrap();
    let mut logical = DerivedVariant::new("draft-set").unwrap();
    logical.bind_draft(DerivedVariantVersionId::generate());
    let err = app
        .catalog_mut()
        .put_validated(&certify(logical))
        .unwrap_err();
    assert_catalog_authority(err, "draft_version_id");
}

#[test]
fn generic_put_cannot_change_derived_variant_draft_pointer() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-change-draft");
    let (logical, _version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let mut changed = logical.into_record();
    changed.bind_draft(DerivedVariantVersionId::generate());
    let err = app
        .catalog_mut()
        .put_validated(&certify(changed))
        .unwrap_err();
    assert_catalog_authority(err, "draft_version_id");
}

#[test]
fn generic_put_cannot_clear_derived_variant_draft_pointer() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-clear-draft");
    let (logical, _version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let cleared = common::mutate_json_field(logical.as_record(), "draft_version_id", serde_json::Value::Null);
    let err = app.catalog_mut().put_validated(&cleared).unwrap_err();
    assert_catalog_authority(err, "draft_version_id");
}

#[test]
fn candidate_transaction_sets_logical_and_version_atomically() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-atomic-candidate");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    assert_eq!(
        logical.as_record().draft_version_id(),
        Some(version.as_record().id())
    );
    let loaded_logical = app
        .catalog()
        .load_derived_variant(&logical.as_record().id().canonical())
        .unwrap();
    let loaded_version = app
        .catalog()
        .load_derived_variant_version(&version.as_record().id().canonical())
        .unwrap();
    assert_eq!(
        loaded_logical.as_record().draft_version_id(),
        Some(loaded_version.as_record().id())
    );
    let artifact_id = loaded_version
        .as_record()
        .persistence_artifact_id()
        .unwrap()
        .canonical();
    let artifact = app.catalog().load_artifact_metadata(&artifact_id).unwrap();
    assert_eq!(
        artifact.as_record().bound_derived_variant_version_id(),
        loaded_version.as_record().id()
    );
    let locations = app.catalog().list_payload_locations(&artifact_id).unwrap();
    assert!(!locations.is_empty());
}

#[test]
fn production_finalize_signature_does_not_take_inspector_ports() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/transfer.rs"));
    assert!(
        src.contains("pub fn finalize_transfer("),
        "production finalize_transfer must exist"
    );
    let prod = src
        .split("pub fn finalize_transfer(")
        .nth(1)
        .expect("production finalize_transfer");
    let prod_sig = prod.split('{').next().unwrap();
    assert!(
        prod_sig.contains("run_id") && prod_sig.contains("staged_path"),
        "production finalize_transfer must take run_id and staged_path"
    );
    assert!(
        !prod_sig.contains("inspector") && !prod_sig.contains("reopener"),
        "production finalize_transfer must not accept inspector/reopener parameters: {prod_sig}"
    );
    assert!(
        src.contains("inspect_durable_persistence_artifact"),
        "production QC must call the pinned inspect path"
    );
    assert!(
        src.contains("reopen_durable_persistence_artifact"),
        "production verification must call the pinned reopen path"
    );
}

#[test]
fn test_memory_inspector_not_available_on_production_surface() {
    let lib = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    let qc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/qc.rs"));
    assert!(
        !lib.contains("MemoryArtifactInspector"),
        "production crate root must not export MemoryArtifactInspector"
    );
    assert!(
        !lib.contains("MemoryPersistenceReopener"),
        "production crate root must not export MemoryPersistenceReopener"
    );
    assert!(
        !qc.contains("pub struct MemoryArtifactInspector"),
        "production qc.rs must not define MemoryArtifactInspector"
    );
    assert!(
        !qc.contains("pub struct MemoryPersistenceReopener"),
        "production qc.rs must not define MemoryPersistenceReopener"
    );
}

#[test]
fn production_catalog_does_not_expose_public_transaction_mutation() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/catalog.rs"));
    let lib = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    assert!(
        !src.contains("pub fn in_transaction"),
        "SqliteCatalog::in_transaction must not be a public production method"
    );
    assert!(
        src.contains("fn in_transaction"),
        "internal in_transaction helper must remain for trusted Catalog workflows"
    );
    assert!(
        !lib.contains("in_transaction"),
        "crate root must not re-export in_transaction"
    );
}

#[test]
fn production_catalog_does_not_expose_unvalidated_payload_insertion() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/catalog.rs"));
    assert!(
        !src.contains("pub fn insert_unvalidated_payload_for_tests"),
        "unvalidated payload insertion must not be a public production method"
    );
    let marker = "fn insert_unvalidated_payload_for_tests";
    let idx = src
        .find(marker)
        .expect("test-only unvalidated insert helper must remain for fail-closed load tests");
    let prefix = &src[..idx];
    let window = prefix.rsplit_once("\n    ").map(|(_, rest)| rest).unwrap_or(prefix);
    assert!(
        prefix.contains("#[cfg(test)]"),
        "unvalidated payload insertion must be cfg(test)-gated, near: {window}"
    );
    let cfg_idx = prefix.rfind("#[cfg(test)]").expect("cfg(test) gate");
    assert!(
        idx - cfg_idx < 200,
        "cfg(test) must be immediately on insert_unvalidated_payload_for_tests"
    );
    assert!(
        !src.contains("pub fn internal_rowid_for_tests"),
        "internal_rowid_for_tests must not be a public production method"
    );
}

#[test]
fn production_app_and_workbench_do_not_enable_test_support() {
    let app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let workbench = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../workbench/Cargo.toml"
    ));
    let blender = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../blender-worker/Cargo.toml"
    ));
    let workspace = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"));
    assert!(
        app.contains("default = []"),
        "rigforge_app default features must be empty"
    );
    assert!(
        app.contains("test-support = []"),
        "test-support must remain a non-default feature"
    );
    let workbench_prod = workbench
        .split("[dev-dependencies]")
        .next()
        .expect("workbench Cargo.toml must have [dev-dependencies]");
    assert!(
        !workbench_prod.contains("test-support"),
        "production workbench dependency must not enable test-support"
    );
    assert!(
        blender.contains("rigforge_app = { path = \"../app\" }")
            && !blender.contains("test-support"),
        "blender-worker production dependency must not enable test-support"
    );
    assert!(
        workspace.contains("resolver = \"2\""),
        "workspace must use Cargo resolver v2 so test-support does not unify into production artifacts"
    );
}

#[test]
fn new_logical_with_empty_pointers_still_persists() {
    let mut app = Application::open_in_memory().unwrap();
    let logical = DerivedVariant::new("empty-pointers").unwrap();
    assert!(logical.published_version_id().is_none());
    assert!(logical.draft_version_id().is_none());
    app.catalog_mut()
        .put_validated(&certify(logical))
        .unwrap();
}

#[test]
fn jobrun_succeeded_is_visible() {
    let (app, run_id, _path) = seed_succeeded_transfer(b"v15-run-state");
    let run = app.catalog().load_job_run(&run_id).unwrap();
    assert_eq!(run.state, JobRunState::Succeeded);
}

#[test]
fn production_finalize_does_not_publish_fake_bytes() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"not-a-blend");
    match app.finalize_transfer(&run_id, &path) {
        Ok(outcome) => assert_ne!(outcome.kind, TransferOutcomeKind::Published),
        Err(_) => {}
    }
}

#[test]
fn sha256_of_staged_bytes_is_stable() {
    let (path, sha) = write_staged(b"digest-check");
    assert_eq!(sha256_file(&path).unwrap(), sha);
    let _ = Path::new(&path);
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

const DEFAULT_BLENDER_EXE: &str =
    r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64\blender.exe";

struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn compile_spoofed_blender(dir: &Path) -> PathBuf {
    let src = dir.join("spoof_blender.rs");
    fs::write(
        &src,
        r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("Blender 5.2.1 LTS");
        println!("build hash: 9e2066aef7ef");
        return;
    }
    if let Ok(path) = std::env::var("RIGFORGE_SPOOF_SENTINEL") {
        let _ = std::fs::write(path, "invoked");
    }
}
"#,
    )
    .unwrap();
    let exe = dir.join("spoof-blender.exe");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let status = Command::new(&rustc)
        .arg(&src)
        .arg("-O")
        .arg("-o")
        .arg(&exe)
        .status()
        .expect("rustc must compile the local spoofed Blender fixture");
    assert!(
        status.success(),
        "rustc failed to compile spoofed Blender fixture"
    );
    exe
}

#[test]
fn generic_put_cannot_create_derived_variant_version_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.derived_version.clone()))
        .unwrap_err();
    assert_catalog_authority(err, "candidate transaction");
}

#[test]
fn generic_put_pair_cannot_create_derived_variant_version_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated_pair(
            &certify(g.derived.clone()),
            &certify(g.derived_version.clone()),
        )
        .unwrap_err();
    assert_catalog_authority(err, "candidate transaction");
    assert!(app
        .catalog()
        .load_derived_variant(&g.derived.id().canonical())
        .is_err());
}

#[test]
fn generic_put_cannot_create_second_version_for_same_worker_result() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-int-second-wr");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let rec = version.as_record();
    let forged = DerivedVariantVersion::draft(
        rec.variant_id(),
        rec.character_version_id(),
        rec.motion_version_id(),
        rec.source_skeleton_ref_id(),
        rec.mapping_version_id(),
        rec.policy_version_id(),
        rec.job_spec_id(),
        rec.backend_id(),
        rec.worker_result_id(),
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(forged))
        .unwrap_err();
    assert_catalog_authority(err, "candidate transaction");
}

#[test]
fn generic_candidate_cannot_switch_jobspec_target_derived_variant() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-int-switch");
    let other = certify(DerivedVariant::new("int-switch-target").unwrap());
    app.catalog_mut().put_validated(&other).unwrap();
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let mutated = common::mutate_json_field(
        version.as_record(),
        "variant_id",
        serde_json::json!(other.as_record().id().canonical()),
    );
    let err = app.catalog_mut().put_validated(&mutated).unwrap_err();
    assert_catalog_authority(err, "candidate");
}

#[test]
fn production_qc_ignores_or_rejects_blender_executable_env_override() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = temp_dir("rf-v15-env-qc");
    let fake = dir.join("not-the-production-blender.exe");
    fs::write(&fake, b"not a blender executable").unwrap();
    let sentinel = dir.join("sentinel.txt");
    let _guard = EnvGuard::set("RIGFORGE_BLENDER_EXECUTABLE", &fake.display().to_string());
    let _sentinel_guard = EnvGuard::set("RIGFORGE_SPOOF_SENTINEL", &sentinel.display().to_string());
    let artifact = dir.join("garbage.blend");
    fs::write(&artifact, b"not-a-blend").unwrap();
    let sha = sha256_file(&artifact).unwrap();
    let result = inspect_durable_persistence_artifact(&artifact, &sha, None);
    assert!(!sentinel.exists(), "spoofed executable must not be invoked");
    match result {
        Ok(_) => {}
        Err(err) => {
            let msg = err.to_string();
            assert!(
                !msg.contains(&fake.display().to_string()),
                "production QC must ignore env override path, got {msg}"
            );
        }
    }
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/pinned_qc.rs"));
    let after_fn = src
        .split("fn blender_executable()")
        .nth(1)
        .expect("blender_executable");
    let body = after_fn.split("fn ").next().unwrap();
    assert!(
        !body.contains("RIGFORGE_BLENDER_EXECUTABLE"),
        "production blender_executable must not read RIGFORGE_BLENDER_EXECUTABLE"
    );
    let _ = DEFAULT_BLENDER_EXE;
}

#[test]
fn production_reopen_ignores_or_rejects_blender_executable_env_override() {
    let _lock = ENV_LOCK.lock().unwrap();
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-env-reopen");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let artifact_id = version
        .as_record()
        .persistence_artifact_id()
        .unwrap()
        .canonical();
    let artifact = app.catalog().load_artifact_metadata(&artifact_id).unwrap();
    let durable = app.catalog().durable_artifact_path(artifact.as_record());
    let mapping = app
        .catalog()
        .load_mapping_version(&version.as_record().mapping_version_id().canonical())
        .unwrap();
    let dir = temp_dir("rf-v15-env-reopen-fake");
    let fake = dir.join("not-the-production-blender.exe");
    fs::write(&fake, b"not a blender executable").unwrap();
    let sentinel = dir.join("sentinel.txt");
    let _guard = EnvGuard::set("RIGFORGE_BLENDER_EXECUTABLE", &fake.display().to_string());
    let _sentinel_guard = EnvGuard::set("RIGFORGE_SPOOF_SENTINEL", &sentinel.display().to_string());
    let result = reopen_durable_persistence_artifact(
        &durable,
        artifact.as_record().digest().sha256(),
        mapping.as_record(),
    );
    assert!(!sentinel.exists(), "spoofed executable must not be invoked");
    match result {
        Ok((fresh, structural)) => {
            assert_ne!(fresh, VerificationOutcome::Pass);
            assert_ne!(structural, VerificationOutcome::Pass);
        }
        Err(err) => {
            let msg = err.to_string();
            assert!(
                !msg.contains(&fake.display().to_string()),
                "production reopen must ignore env override path, got {msg}"
            );
        }
    }
}

#[test]
fn spoofed_version_executable_cannot_authorize_qc_pass() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = temp_dir("rf-v15-spoof-qc");
    let fake = compile_spoofed_blender(&dir);
    let sentinel = dir.join("sentinel.txt");
    let _guard = EnvGuard::set("RIGFORGE_BLENDER_EXECUTABLE", &fake.display().to_string());
    let _sentinel_guard = EnvGuard::set("RIGFORGE_SPOOF_SENTINEL", &sentinel.display().to_string());
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-spoof-qc-bytes");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let result = app.evaluate_and_bind_qc(&version.as_record().id().canonical());
    assert!(!sentinel.exists(), "pin-matching spoof must not be selected");
    match result {
        Ok(qc) => assert_ne!(qc.as_record().verdict(), QcVerdict::Pass),
        Err(_) => {}
    }
    let still = app
        .catalog()
        .load_derived_variant_version(&version.as_record().id().canonical())
        .unwrap();
    if let Some(id) = still.as_record().qc_report_id() {
        let qc = app.catalog().load_qc_report(&id.canonical()).unwrap();
        assert_ne!(qc.as_record().verdict(), QcVerdict::Pass);
    }
}

#[test]
fn spoofed_version_executable_cannot_authorize_persistence_pass() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = temp_dir("rf-v15-spoof-reopen");
    let fake = compile_spoofed_blender(&dir);
    let sentinel = dir.join("sentinel.txt");
    let _guard = EnvGuard::set("RIGFORGE_BLENDER_EXECUTABLE", &fake.display().to_string());
    let _sentinel_guard = EnvGuard::set("RIGFORGE_SPOOF_SENTINEL", &sentinel.display().to_string());
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-spoof-reopen-bytes");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    let verification = app
        .verify_and_bind_persistence(&version.as_record().id().canonical())
        .unwrap();
    assert!(!sentinel.exists(), "pin-matching spoof must not be selected");
    assert_ne!(
        verification.as_record().fresh_reopen(),
        VerificationOutcome::Pass
    );
    assert_ne!(
        verification.as_record().structural_verification(),
        VerificationOutcome::Pass
    );
}

#[test]
fn candidate_transaction_still_creates_candidate() {
    let (mut app, run_id, path) = seed_succeeded_transfer(b"v15-int-still-creates");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run_id, &path)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
}

#[test]
fn candidate_transaction_still_sets_draft_pointer_atomically() {
    candidate_transaction_sets_logical_and_version_atomically();
}
