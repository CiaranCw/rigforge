//! Relocatable V1-8 runtime resource layout. Not Product identity.
//!
//! Production resolution uses `RIGFORGE_RUNTIME_ROOT` or the directory that
//! contains the current executable. It does not use the crate compile-time
//! directory, the original source checkout, or a hard-coded developer Blender
//! tree. Pinned bytes and the Blender version/build remain verified after
//! relocate.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::qc::sha256_file;

pub const RUNTIME_ROOT_ENV: &str = "RIGFORGE_RUNTIME_ROOT";
pub const BLENDER_DISTRO_DIR_NAME: &str = "blender-5.2.1-windows-x64";

/// Package integrity pin for `worker.py`. Not Product identity.
pub const WORKER_SCRIPT_SHA256: &str =
    "7dc0aa10e1b47e35cca00bf1f85ed88cd401e4367c9e743e6d219b44b1a972b9";
/// Package integrity pin for `preview_gen.py`. Not Product identity.
pub const PREVIEW_GEN_SCRIPT_SHA256: &str =
    "c84aada5a3785dfc5a6c7114dce270616c1fe0640a4fa390dda39d4b8b3db795";

thread_local! {
    static THREAD_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Restores the previous thread runtime root when dropped.
#[must_use]
pub struct RuntimeRootGuard {
    previous: Option<PathBuf>,
}

impl Drop for RuntimeRootGuard {
    fn drop(&mut self) {
        THREAD_ROOT.with(|slot| {
            *slot.borrow_mut() = self.previous.take();
        });
    }
}

/// Bind a runtime root for this thread only. Wins over the process env and
/// the executable-relative default. Used by relocation tests.
pub fn bind_thread_runtime_root(root: impl Into<PathBuf>) -> RuntimeRootGuard {
    let root = root.into();
    let previous = THREAD_ROOT.with(|slot| slot.replace(Some(root)));
    RuntimeRootGuard { previous }
}

/// Release / development bundle layout under one runtime root:
///
/// ```text
/// <runtime root>/
///     rigforge_workbench.exe
///     blender-worker/worker.py
///     blender-worker/preview_gen.py
///     preview-viewer/.../vendor/model-viewer.min.js
///     runtime/blender-5.2.1-windows-x64/blender.exe
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLayout {
    root: PathBuf,
}

impl RuntimeLayout {
    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn resolve() -> Result<Self, AppError> {
        if let Some(root) = THREAD_ROOT.with(|slot| slot.borrow().clone()) {
            return Ok(Self { root });
        }
        if let Ok(root) = std::env::var(RUNTIME_ROOT_ENV) {
            let trimmed = root.trim();
            if !trimmed.is_empty() {
                return Ok(Self {
                    root: PathBuf::from(trimmed),
                });
            }
        }
        let exe = std::env::current_exe().map_err(|err| {
            AppError::Worker(format!("runtime root: current executable: {err}"))
        })?;
        let dir = exe.parent().ok_or_else(|| {
            AppError::Worker("runtime root: executable has no parent directory".into())
        })?;
        Ok(Self {
            root: dir.to_path_buf(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn worker_script(&self) -> PathBuf {
        self.root.join("blender-worker").join("worker.py")
    }

    pub fn preview_gen_script(&self) -> PathBuf {
        self.root.join("blender-worker").join("preview_gen.py")
    }

    pub fn viewer_root(&self) -> PathBuf {
        self.root.join("preview-viewer")
    }

    pub fn vendor_script(&self) -> PathBuf {
        self.viewer_root().join("vendor").join("model-viewer.min.js")
    }

    pub fn blender_dir(&self) -> PathBuf {
        self.root
            .join("runtime")
            .join(BLENDER_DISTRO_DIR_NAME)
    }

    pub fn blender_executable(&self) -> PathBuf {
        self.blender_dir().join("blender.exe")
    }

    /// Verify the production worker package under this runtime root.
    pub fn verify_worker_package(&self) -> Result<(), AppError> {
        verify_runtime_worker_package(&self.worker_script())
    }
}

/// Shared production worker-package integrity check.
///
/// Transfer, Preview, QC inspect, and fresh reopen must use this contract.
/// Wrong or missing `worker.py` / `preview_gen.py` bytes fail closed.
/// Digests are package evidence, not Product identity.
pub fn verify_runtime_worker_package(worker_script: &Path) -> Result<(), AppError> {
    let name = worker_script
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if name != "worker.py" {
        return Err(AppError::Worker(format!(
            "worker package integrity: expected worker.py, found {name}"
        )));
    }
    if !worker_script.is_file() {
        return Err(AppError::Worker(format!(
            "worker package integrity: worker.py is missing: {}",
            worker_script.display()
        )));
    }
    let found = sha256_file(worker_script)?;
    if found != WORKER_SCRIPT_SHA256 {
        return Err(AppError::Worker(format!(
            "worker.py sha256 mismatch: expected {WORKER_SCRIPT_SHA256} found {found}"
        )));
    }
    let preview_gen = worker_script
        .parent()
        .unwrap_or(worker_script)
        .join("preview_gen.py");
    if !preview_gen.is_file() {
        return Err(AppError::Worker(format!(
            "worker package integrity: preview_gen.py is missing: {}",
            preview_gen.display()
        )));
    }
    let found = sha256_file(&preview_gen)?;
    if found != PREVIEW_GEN_SCRIPT_SHA256 {
        return Err(AppError::Worker(format!(
            "preview_gen.py sha256 mismatch: expected {PREVIEW_GEN_SCRIPT_SHA256} found {found}"
        )));
    }
    Ok(())
}

pub fn unresolved_worker_script() -> PathBuf {
    PathBuf::from("unresolved-rigforge-runtime-root")
        .join("blender-worker")
        .join("worker.py")
}

pub fn unresolved_blender_executable() -> PathBuf {
    PathBuf::from("unresolved-rigforge-runtime-root")
        .join("runtime")
        .join(BLENDER_DISTRO_DIR_NAME)
        .join("blender.exe")
}

/// Copy pinned runtime files into `dest` in the release layout. Blender is
/// junctioned when `blender_dir` is provided; it is not copied.
pub fn materialize_runtime_bundle(
    dest: impl AsRef<Path>,
    worker_script: &Path,
    preview_gen_script: &Path,
    preview_viewer_dir: Option<&Path>,
    blender_dir: Option<&Path>,
) -> Result<RuntimeLayout, AppError> {
    let dest = dest.as_ref();
    fs::create_dir_all(dest)?;
    let worker_dir = dest.join("blender-worker");
    fs::create_dir_all(&worker_dir)?;
    let worker_dest = worker_dir.join("worker.py");
    let preview_dest = worker_dir.join("preview_gen.py");
    fs::copy(worker_script, &worker_dest).map_err(|err| {
        AppError::Worker(format!(
            "runtime bundle: copy worker.py from {}: {err}",
            worker_script.display()
        ))
    })?;
    fs::copy(preview_gen_script, &preview_dest).map_err(|err| {
        AppError::Worker(format!(
            "runtime bundle: copy preview_gen.py from {}: {err}",
            preview_gen_script.display()
        ))
    })?;
    if let Some(viewer) = preview_viewer_dir {
        copy_dir_filtered(viewer, &dest.join("preview-viewer"))?;
    }
    if let Some(blender) = blender_dir {
        if !blender.is_dir() {
            return Err(AppError::Worker(format!(
                "runtime bundle: pinned Blender directory is missing: {}",
                blender.display()
            )));
        }
        junction_dir(blender, &dest.join("runtime").join(BLENDER_DISTRO_DIR_NAME))?;
    }
    Ok(RuntimeLayout::from_root(dest.to_path_buf()))
}

fn copy_dir_filtered(src: &Path, dst: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if matches!(
            name_str.as_ref(),
            "node_modules" | ".git" | "target" | "__pycache__"
        ) {
            continue;
        }
        let to = dst.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir_filtered(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

fn junction_dir(src: &Path, dst: &Path) -> Result<(), AppError> {
    if dst.exists() {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &dst.to_string_lossy(),
            &src.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|err| AppError::Worker(format!("runtime blender junction: {err}")))?;
    if !status.success() {
        return Err(AppError::Worker(format!(
            "runtime blender junction failed: {} -> {}",
            dst.display(),
            src.display()
        )));
    }
    Ok(())
}
