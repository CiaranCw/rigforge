//! Per-attempt isolated environment. User Blender config/add-ons must not apply.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rigforge_app::AppError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptWorkspace {
    pub root: PathBuf,
    pub tmp: PathBuf,
    pub user_config: PathBuf,
    pub user_scripts: PathBuf,
    pub user_datafiles: PathBuf,
    pub staged: PathBuf,
}

impl AttemptWorkspace {
    pub fn create(root: impl Into<PathBuf>) -> Result<Self, AppError> {
        let root = root.into();
        let ws = Self {
            tmp: root.join("tmp"),
            user_config: root.join("blender_user").join("config"),
            user_scripts: root.join("blender_user").join("scripts"),
            user_datafiles: root.join("blender_user").join("datafiles"),
            staged: root.join("staged"),
            root,
        };
        for dir in [
            &ws.root,
            &ws.tmp,
            &ws.user_config,
            &ws.user_scripts,
            &ws.user_datafiles,
            &ws.staged,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(ws)
    }

    pub fn isolated_env(&self) -> HashMap<String, String> {
        let mut env: HashMap<String, String> = std::env::vars().collect();
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

pub fn attempt_workspace_root(base: &Path, attempt_id: &str) -> PathBuf {
    base.join("attempts").join(attempt_id)
}
