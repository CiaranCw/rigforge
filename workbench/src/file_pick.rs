//! Native file selection helpers. OS dialog acquisition is separate from
//! applying a PathBuf so tests never open a modal dialog.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilePickState {
    pub path: Option<PathBuf>,
    pub filename: String,
    pub display_name: String,
    pub last_auto_name: String,
}

pub fn derive_display_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let replaced = stem.replace(['_', '-'], " ");
    replaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn filename_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

/// Apply a Browse/Change result. `None` is cancel and must preserve the
/// previous selection. Returns whether a new path was stored.
pub fn apply_picked_path(state: &mut FilePickState, picked: Option<PathBuf>) -> bool {
    let Some(path) = picked else {
        return false;
    };
    let derived = derive_display_name(&path);
    let filename = filename_label(&path);
    if state.display_name.trim().is_empty() || state.display_name == state.last_auto_name {
        state.display_name = derived.clone();
    }
    state.last_auto_name = derived;
    state.filename = filename;
    state.path = Some(path);
    true
}

pub fn pick_fbx_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("FBX", &["fbx"])
        .pick_file()
}
