//! Production Preview generation via the pinned Blender executable.
//!
//! Blender is generation-time only. It is not Preview Product authority,
//! not a viewer requirement, and not a user-facing DCC dependency.
//! Transfer and Preview both resolve Blender and worker scripts from the
//! same relocatable runtime root. Neither honors an arbitrary unpinned
//! `RIGFORGE_BLENDER_EXECUTABLE` fallback.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::preview::{
    verify_source_before_generation, GeneratedPreview, PreviewDescriptor, PreviewGenerationJob,
    PreviewGeneratorPort, PREVIEW_GENERATOR_ID, PREVIEW_RECIPE_VERSION,
};
use crate::qc::sha256_file;
use crate::runtime::RuntimeLayout;
use rigforge_domain::ProductKind;

const BLENDER_VERSION: &str = "5.2.1 LTS";
const BLENDER_BUILD: &str = "9e2066aef7ef";

const BACKGROUND: &str = "--background";
const FACTORY_STARTUP: &str = "--factory-startup";
const DISABLE_AUTOEXEC: &str = "--disable-autoexec";
const PYTHON_EXIT_CODE: &str = "--python-exit-code";
const PYTHON: &str = "--python";

/// Sealed production executable for Preview generation.
fn blender_executable() -> Result<PathBuf, AppError> {
    Ok(RuntimeLayout::resolve()?.blender_executable())
}

fn production_worker_script() -> Result<PathBuf, AppError> {
    Ok(RuntimeLayout::resolve()?.worker_script())
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

fn preview_mode(job: &PreviewGenerationJob) -> &'static str {
    if job.synthetic {
        return "preview_motion_synthetic";
    }
    match job.kind {
        ProductKind::CharacterAssetVersion => "preview_character",
        ProductKind::MotionAssetVersion => "preview_motion",
        ProductKind::DerivedVariantVersion => "preview_derived",
    }
}

#[derive(Deserialize)]
struct PreviewGenerationEnvelope {
    status: String,
    #[serde(default)]
    #[allow(dead_code)]
    kind: Option<String>,
    #[serde(default)]
    animation_inventory: Vec<PreviewAnimationItem>,
    #[serde(default)]
    default_animation: Option<String>,
    #[serde(default)]
    duration_s: Option<f64>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    blender_version: Option<String>,
    #[serde(default)]
    blender_build_hash: Option<String>,
    #[serde(default)]
    stripped_leftover_action_count: Option<u32>,
    #[serde(default)]
    proxy: Option<PreviewProxyMeta>,
    #[serde(default)]
    traceback: Option<String>,
}

#[derive(Deserialize)]
struct PreviewAnimationItem {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct PreviewProxyMeta {
    #[serde(default)]
    humanoid_role_table: Option<bool>,
}

pub struct BlenderPreviewGenerator;

impl PreviewGeneratorPort for BlenderPreviewGenerator {
    fn generate(&self, job: &PreviewGenerationJob) -> Result<GeneratedPreview, AppError> {
        verify_source_before_generation(job)?;
        let blender = blender_executable()?;
        let script = production_worker_script()?;
        crate::runtime::verify_runtime_worker_package(&script)?;
        enforce_pin(&blender)?;
        let source_digest_before = if job.synthetic {
            None
        } else {
            Some(sha256_file(&job.source_path)?)
        };
        let attempt = format!("preview-{}", Uuid::now_v7());
        let root = std::env::temp_dir()
            .join("rigforge-v16-preview")
            .join("attempts")
            .join(attempt);
        let workspace = AttemptWorkspace::create(root)?;
        if let Some(parent) = job.output_glb.parent() {
            fs::create_dir_all(parent)?;
        }
        let job_json = workspace.root.join("preview_job.json");
        let envelope_path = workspace.root.join("preview_generation.json");
        let mode = preview_mode(job);
        let document = serde_json::json!({
            "source_path": job.source_path,
            "expected_digest": job.expected_source_sha256,
            "expected_size": job.expected_source_size,
            "clip_id": job.clip_id,
            "frame_start": job.frame_start,
            "frame_end": job.frame_end,
            "fps": job.fps.unwrap_or(30),
            "synthetic": job.synthetic,
            "product_version_id": job.product_version_id,
            "outputs": {
                "payload": job.output_glb,
                "generation_json": envelope_path,
                "workspace": workspace.root
            }
        });
        fs::write(&job_json, serde_json::to_vec_pretty(&document).unwrap())?;
        let argv = blender_argv(&blender, &script, mode, &job_json);
        if !assert_safety_flags(&argv) {
            return Err(AppError::Worker(
                "preview generation command missing safety flags".into(),
            ));
        }
        let mut cmd = blender_command(&blender, &script, mode, &job_json);
        for (k, v) in workspace.isolated_env() {
            cmd.env(k, v);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = cmd.output()?;
        if let Some(before) = source_digest_before {
            let after = sha256_file(&job.source_path)?;
            if after != before {
                return Err(AppError::Worker(
                    "Preview generation mutated source/persistence bytes; generation is denied".into(),
                ));
            }
        }
        if !envelope_path.is_file() {
            return Err(AppError::Worker(format!(
                "preview generation envelope missing; blender exited {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let text = fs::read_to_string(&envelope_path)?;
        let envelope: PreviewGenerationEnvelope = serde_json::from_str(&text)
            .map_err(|e| AppError::Worker(format!("preview generation envelope: {e}")))?;
        if !envelope.status.eq_ignore_ascii_case("SUCCESS") || !output.status.success() {
            return Err(AppError::Worker(format!(
                "Preview generation failed: {}",
                envelope.traceback.unwrap_or(envelope.status)
            )));
        }
        if !job.output_glb.is_file() {
            return Err(AppError::Worker(
                "Preview generation did not write a payload".into(),
            ));
        }
        if job.kind == ProductKind::CharacterAssetVersion {
            if envelope
                .stripped_leftover_action_count
                .map(|n| n > 0)
                .unwrap_or(false)
                && envelope
                    .animation_inventory
                    .iter()
                    .any(|a| !a.name.is_empty())
            {
                return Err(AppError::Worker(
                    "Character Preview retained source animation after strip".into(),
                ));
            }
        }
        if job.synthetic {
            if envelope
                .proxy
                .as_ref()
                .and_then(|p| p.humanoid_role_table)
                .unwrap_or(true)
            {
                return Err(AppError::Worker(
                    "synthetic Motion proxy must not use a humanoid role table".into(),
                ));
            }
        }
        let names: Vec<String> = envelope
            .animation_inventory
            .iter()
            .map(|a| a.name.clone())
            .filter(|n| !n.is_empty())
            .collect();
        let has_animation = job.kind != ProductKind::CharacterAssetVersion;
        if has_animation && names.is_empty() && envelope.default_animation.is_none() {
            return Err(AppError::Worker(
                "Motion/Derived Preview generation produced no animation".into(),
            ));
        }
        let mut losses = envelope.warnings;
        losses.push("Preview is a derived internal viewing representation, not an export contract".into());
        Ok(GeneratedPreview {
            payload_path: job.output_glb.clone(),
            descriptor: PreviewDescriptor {
                has_animation,
                animation_names: names.clone(),
                default_animation: envelope.default_animation.or_else(|| names.first().cloned()),
                duration_s: envelope.duration_s,
                declared_losses: losses,
                generation_recipe: PREVIEW_RECIPE_VERSION.into(),
                generator_id: PREVIEW_GENERATOR_ID.into(),
                backend_kind: Some("blender".into()),
                backend_version: envelope.blender_version.or_else(|| Some(BLENDER_VERSION.into())),
                backend_build: envelope.blender_build_hash.or_else(|| Some(BLENDER_BUILD.into())),
            },
        })
    }
}
