//! Pinned official Blender 5.2.1 LTS build. Not Product identity.

use std::path::{Path, PathBuf};
use std::process::Command;

use rigforge_app::AppError;
use sha2::{Digest, Sha256};

pub const BLENDER_VERSION: &str = "5.2.1 LTS";
pub const BLENDER_BUILD: &str = "9e2066aef7ef";
pub const BLENDER_ARCHIVE_NAME: &str = "blender-5.2.1-windows-x64.zip";
pub const BLENDER_ARCHIVE_SHA256: &str =
    "0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c";
pub const DEFAULT_BLENDER_DIR: &str =
    r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64";
pub const ADAPTER_VERSION: &str = "rigforge-blender-worker/0.1.0";
pub const EXECUTION_POLICY_VERSION: &str = "v1-1-proven-unexecuted";
pub const BACKEND_KIND: &str = "Blender";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlenderPin {
    pub executable: PathBuf,
    pub version: String,
    pub build: String,
    pub archive_sha256: String,
}

impl BlenderPin {
    pub fn accepted() -> Self {
        Self {
            executable: default_blender_executable(),
            version: BLENDER_VERSION.to_string(),
            build: BLENDER_BUILD.to_string(),
            archive_sha256: BLENDER_ARCHIVE_SHA256.to_string(),
        }
    }

    pub fn with_executable(mut self, executable: PathBuf) -> Self {
        self.executable = executable;
        self
    }
}

pub fn default_blender_executable() -> PathBuf {
    if let Ok(path) = std::env::var("RIGFORGE_BLENDER_EXECUTABLE") {
        return PathBuf::from(path);
    }
    PathBuf::from(DEFAULT_BLENDER_DIR).join("blender.exe")
}

pub fn default_archive_path() -> PathBuf {
    PathBuf::from(r"F:\NewResearch\rigforge_w0p_work\toolchains").join(BLENDER_ARCHIVE_NAME)
}

pub fn sha256_file(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn verify_archive_sha256(archive: &Path) -> Result<(), AppError> {
    let found = sha256_file(archive)?;
    if found != BLENDER_ARCHIVE_SHA256 {
        return Err(AppError::Worker(format!(
            "blender archive sha256 mismatch: expected {BLENDER_ARCHIVE_SHA256} found {found}"
        )));
    }
    Ok(())
}

pub fn parse_blender_version_output(stdout: &str) -> Result<(String, String), AppError> {
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

pub fn probe_blender_pin(executable: &Path) -> Result<(String, String), AppError> {
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
    parse_blender_version_output(&stdout)
}

pub fn enforce_pin(executable: &Path, expected: &BlenderPin) -> Result<(String, String), AppError> {
    let (version, build) = probe_blender_pin(executable)?;
    if !version.contains("5.2.1") {
        return Err(AppError::Worker(format!(
            "blender version mismatch: expected {} found {version}",
            expected.version
        )));
    }
    if build != expected.build {
        return Err(AppError::Worker(format!(
            "blender build mismatch: expected {} found {build}",
            expected.build
        )));
    }
    Ok((version, build))
}
