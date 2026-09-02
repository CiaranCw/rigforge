//! Production evidence acquisition for publication-critical QC / reopen.
//!
//! This is the concrete pinned Blender inspect/reopen path owned by Application.
//! It is not a caller-replaceable port. Worker execute/collect remains in
//! `blender-worker`. Product QC verdicts remain backend-neutral.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rigforge_domain::{BoneMappingVersion, JointParticipation, VerificationOutcome};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::qc::{interpret_qc_inspect, sha256_file, ArtifactInspectionEvidence, QcInspectEnvelope};

const BLENDER_VERSION: &str = "5.2.1 LTS";
const BLENDER_BUILD: &str = "9e2066aef7ef";
const DEFAULT_BLENDER_DIR: &str =
    r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64";

const BACKGROUND: &str = "--background";
const FACTORY_STARTUP: &str = "--factory-startup";
const DISABLE_AUTOEXEC: &str = "--disable-autoexec";
const PYTHON_EXIT_CODE: &str = "--python-exit-code";
const PYTHON: &str = "--python";

/// Sealed production executable for publication-critical inspect/reopen.
///
/// Worker execute/collect may still honor `RIGFORGE_BLENDER_EXECUTABLE` (V1-3).
/// This path must not. Release installer integrity remains V1-8.
fn blender_executable() -> PathBuf {
    PathBuf::from(DEFAULT_BLENDER_DIR).join("blender.exe")
}

fn production_worker_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("blender-worker")
        .join("python")
        .join("worker.py")
}

fn blender_argv(blender: &Path, script: &Path, mode: &str, job_json: &Path) -> Vec<OsString> {
    vec![
        blender.as_os_str().to_os_string(),
        BACKGROUND.into(),
        FACTORY_STARTUP.into(),
        DISABLE_AUTOEXEC.into(),
        PYTHON_EXIT_CODE.into(),
        "1".into(),
        PYTHON.into(),
        script.as_os_str().to_os_string(),
        "--".into(),
        mode.into(),
        job_json.as_os_str().to_os_string(),
    ]
}

fn blender_command(blender: &Path, script: &Path, mode: &str, job_json: &Path) -> Command {
    let argv = blender_argv(blender, script, mode, job_json);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd
}

fn assert_safety_flags(argv: &[OsString]) -> bool {
    let text: Vec<String> = argv
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    text.windows(2).any(|w| w[0] == BACKGROUND)
        && text.iter().any(|a| a == FACTORY_STARTUP)
        && text.iter().any(|a| a == DISABLE_AUTOEXEC)
        && text
            .windows(2)
            .any(|w| w[0] == PYTHON_EXIT_CODE && w[1] == "1")
        && text.iter().any(|a| a == PYTHON)
}

fn parse_blender_version_output(stdout: &str) -> Result<(String, String), AppError> {
    let mut version = None;
    let mut build = None;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Blender ") {
            version = Some(line.trim_start_matches("Blender ").trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("build hash: ") {
            build = Some(rest.trim().to_string());
        }
    }
    match (version, build) {
        (Some(version), Some(build)) => Ok((version, build)),
        _ => Err(AppError::Worker(
            "blender --version did not report version and build hash".into(),
        )),
    }
}

fn enforce_pin(executable: &Path) -> Result<(), AppError> {
    if !executable.is_file() {
        return Err(AppError::Worker(format!(
            "launch failure: blender executable is missing: {}",
            executable.display()
        )));
    }
    let output = Command::new(executable)
        .arg("--version")
        .output()
        .map_err(|err| AppError::Worker(format!("launch failure: blender --version: {err}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (version, build) = parse_blender_version_output(&stdout)?;
    if !version.contains("5.2.1") {
        return Err(AppError::Worker(format!(
            "blender version mismatch: expected {BLENDER_VERSION} found {version}"
        )));
    }
    if build != BLENDER_BUILD {
        return Err(AppError::Worker(format!(
            "blender build mismatch: expected {BLENDER_BUILD} found {build}"
        )));
    }
    Ok(())
}

struct AttemptWorkspace {
    root: PathBuf,
    tmp: PathBuf,
    user_config: PathBuf,
    user_scripts: PathBuf,
    user_datafiles: PathBuf,
}

impl AttemptWorkspace {
    fn create(root: PathBuf) -> Result<Self, AppError> {
        let ws = Self {
            tmp: root.join("tmp"),
            user_config: root.join("blender_user").join("config"),
            user_scripts: root.join("blender_user").join("scripts"),
            user_datafiles: root.join("blender_user").join("datafiles"),
            root,
        };
        for dir in [
            &ws.root,
            &ws.tmp,
            &ws.user_config,
            &ws.user_scripts,
            &ws.user_datafiles,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(ws)
    }

    fn isolated_env(&self) -> HashMap<String, String> {
        let mut env: HashMap<String, String> = std::env::vars().collect();
        env.remove("RIGFORGE_BLENDER_EXECUTABLE");
        let tmp = self.tmp.display().to_string();
        env.insert("TEMP".into(), tmp.clone());
        env.insert("TMP".into(), tmp.clone());
        env.insert("TMPDIR".into(), tmp);
        env.insert(
            "BLENDER_USER_CONFIG".into(),
            self.user_config.display().to_string(),
        );
        env.insert(
            "BLENDER_USER_SCRIPTS".into(),
            self.user_scripts.display().to_string(),
        );
        env.insert(
            "BLENDER_USER_DATAFILES".into(),
            self.user_datafiles.display().to_string(),
        );
        env
    }
}

fn project_mapping_entries(mapping: Option<&BoneMappingVersion>) -> serde_json::Value {
    let entries = mapping
        .map(|mapping| {
            mapping
                .entries()
                .iter()
                .map(|entry| {
                    serde_json::json!({
                        "source": entry.source().joint_key().as_str(),
                        "target": entry.target().joint_key().as_str(),
                        "role": entry.role_profile().unwrap_or("mapped"),
                        "required": entry.participation() == JointParticipation::Required,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::json!({ "entries": entries })
}

/// Read-only inspect of durable Persistence Artifact bytes via the pinned worker.
pub fn inspect_durable_persistence_artifact(
    artifact_path: &Path,
    expected_sha256: &str,
    mapping: Option<&BoneMappingVersion>,
) -> Result<ArtifactInspectionEvidence, AppError> {
    let blender = blender_executable();
    let script = production_worker_script();
    enforce_pin(&blender)?;
    if !artifact_path.is_file() {
        return Err(AppError::Worker(
            "QC inspect: artifact file is missing".into(),
        ));
    }
    let digest_before = sha256_file(artifact_path)?;
    if digest_before != expected_sha256 {
        return Err(AppError::Worker(format!(
            "QC inspect digest mismatch: expected {expected_sha256} found {digest_before}"
        )));
    }
    let attempt = format!("qc-inspect-{}", Uuid::now_v7());
    let root = std::env::temp_dir()
        .join("rigforge-v15-qc-inspect")
        .join("attempts")
        .join(attempt);
    let workspace = AttemptWorkspace::create(root)?;
    let job_json = workspace.root.join("inspect_qc_job.json");
    let envelope_path = workspace.root.join("inspect_qc_envelope.json");
    let job = serde_json::json!({
        "artifact_path": artifact_path.to_string_lossy(),
        "expected_digest": expected_sha256,
        "mapping": project_mapping_entries(mapping),
        "outputs": {
            "inspect_envelope": envelope_path,
            "workspace": workspace.root
        }
    });
    fs::write(&job_json, serde_json::to_vec_pretty(&job).unwrap())?;
    let argv = blender_argv(&blender, &script, "inspect_qc", &job_json);
    if !assert_safety_flags(&argv) {
        return Err(AppError::Worker(
            "inspect_qc command missing safety flags".into(),
        ));
    }
    let mut cmd = blender_command(&blender, &script, "inspect_qc", &job_json);
    for (k, v) in workspace.isolated_env() {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output()?;
    let digest_after = sha256_file(artifact_path)?;
    if digest_after != digest_before {
        return Err(AppError::Worker(
            "QC inspect mutated persisted bytes; publication is denied".into(),
        ));
    }
    if !envelope_path.is_file() {
        return Err(AppError::Worker(format!(
            "inspect_qc envelope missing; blender exited {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let text = fs::read_to_string(&envelope_path)?;
    let envelope: QcInspectEnvelope = serde_json::from_str(&text)
        .map_err(|e| AppError::Worker(format!("inspect_qc envelope: {e}")))?;
    interpret_qc_inspect(
        output.status.success(),
        &envelope,
        expected_sha256,
        &digest_before,
        &digest_after,
    )
}

/// Fresh-process reopen of durable Persistence Artifact bytes via the pinned worker.
pub fn reopen_durable_persistence_artifact(
    artifact_path: &Path,
    expected_sha256: &str,
    mapping: &BoneMappingVersion,
) -> Result<(VerificationOutcome, VerificationOutcome), AppError> {
    if !artifact_path.is_file() {
        return Ok((VerificationOutcome::Missing, VerificationOutcome::Missing));
    }
    let blender = blender_executable();
    let script = production_worker_script();
    enforce_pin(&blender)?;
    let found = sha256_file(artifact_path)?;
    if found != expected_sha256 {
        return Ok((VerificationOutcome::Fail, VerificationOutcome::Fail));
    }
    let attempt = format!("persist-reopen-{}", Uuid::now_v7());
    let root = std::env::temp_dir()
        .join("rigforge-v15-persist-reopen")
        .join("attempts")
        .join(attempt);
    let workspace = AttemptWorkspace::create(root)?;
    let job_json = workspace.root.join("persist_reopen_job.json");
    let reopen_path = workspace.root.join("persist_reopen.json");
    let job = serde_json::json!({
        "mapping": project_mapping_entries(Some(mapping)),
        "outputs": {
            "staged_blend": artifact_path,
            "reopen_verification": reopen_path,
            "workspace": workspace.root
        }
    });
    fs::write(&job_json, serde_json::to_vec_pretty(&job).unwrap())?;
    let argv = blender_argv(&blender, &script, "reopen", &job_json);
    if !assert_safety_flags(&argv) {
        return Err(AppError::Worker(
            "persistence reopen command missing safety flags".into(),
        ));
    }
    let mut cmd = blender_command(&blender, &script, "reopen", &job_json);
    for (k, v) in workspace.isolated_env() {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output()?;
    let digest_after = sha256_file(artifact_path)?;
    if digest_after != found {
        return Ok((VerificationOutcome::Fail, VerificationOutcome::Fail));
    }
    if !reopen_path.is_file() {
        return Ok((VerificationOutcome::Fail, VerificationOutcome::Missing));
    }
    let text = fs::read_to_string(&reopen_path)?;
    #[derive(Deserialize)]
    struct ReopenEnvelopeLite {
        status: String,
        #[serde(default)]
        target_present: bool,
        #[serde(default)]
        baked_action: Option<String>,
    }
    let envelope: ReopenEnvelopeLite = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return Ok((VerificationOutcome::Fail, VerificationOutcome::Fail)),
    };
    let fresh = if output.status.success() && envelope.status.eq_ignore_ascii_case("SUCCESS") {
        VerificationOutcome::Pass
    } else {
        VerificationOutcome::Fail
    };
    let structural = if envelope.target_present
        && envelope
            .baked_action
            .as_deref()
            .map(|name| !name.is_empty())
            .unwrap_or(false)
        && envelope.status.eq_ignore_ascii_case("SUCCESS")
    {
        VerificationOutcome::Pass
    } else {
        VerificationOutcome::Fail
    };
    Ok((fresh, structural))
}
