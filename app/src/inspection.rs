//! Transient source-inspection evidence for R1 first-use ingest.
//!
//! These structures are not Domain Product IDs, Catalog rows, Character
//! identity, Motion identity, or SourceSkeletonReference identity.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::qc::sha256_file;

pub const INGEST_NO_SKELETON: &str = "No skeleton was found in this FBX.";
pub const INGEST_MULTIPLE_SKELETONS: &str = "Multiple usable skeletons were found. R1 currently requires one unambiguous skeleton per source file.";
pub const INGEST_NO_CLIPS: &str = "No usable animation clips were found.";
pub const INGEST_FRACTIONAL_FRAMES: &str =
    "This animation uses fractional frame endpoints that are not currently supported.";
pub const INGEST_FILE_CHANGED: &str =
    "This file changed after it was inspected. Inspect it again.";
pub const INGEST_INSPECT_FAILED: &str =
    "RigForge could not inspect this FBX with the configured Blender runtime.";
pub const INGEST_UNUSABLE_TIMING: &str =
    "This Motion source has timing that RigForge cannot represent exactly.";
pub const INGEST_MISSING_FILE: &str = "The selected file is missing.";
pub const INGEST_UNSUPPORTED_FORMAT: &str = "This file is not a supported FBX source.";

pub fn ingest_message_for_code(code: &str) -> &'static str {
    match code {
        "no_skeleton" => INGEST_NO_SKELETON,
        "multiple_skeletons" => INGEST_MULTIPLE_SKELETONS,
        "no_clips" => INGEST_NO_CLIPS,
        "fractional_frames" => INGEST_FRACTIONAL_FRAMES,
        "file_changed" | "digest_mismatch" | "size_mismatch" => INGEST_FILE_CHANGED,
        "unusable_timing" => INGEST_UNUSABLE_TIMING,
        "missing_file" => INGEST_MISSING_FILE,
        "unsupported_format" => INGEST_UNSUPPORTED_FORMAT,
        _ => INGEST_INSPECT_FAILED,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedTimingContext {
    pub fps_num: u32,
    pub fps_den: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectedJoint {
    pub joint_key: String,
    pub display_name: String,
    pub parent_key: Option<String>,
    pub is_root: bool,
    pub deform_observation: Option<String>,
    pub rest_evidence: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletonCandidate {
    pub source_local_key: String,
    pub display_name: String,
    pub joint_count: usize,
    pub joints: Vec<InspectedJoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssociationEvidence {
    #[serde(default)]
    pub kinds_present: Vec<String>,
    pub assigned_slot_identifier: Option<String>,
    pub nla_track_name: Option<String>,
    pub nla_strip_name: Option<String>,
    pub nla_slot_identifier: Option<String>,
    pub pose_slot_identifier: Option<String>,
    #[serde(default)]
    pub pose_data_path_samples: Vec<String>,
    #[serde(default)]
    pub resolved_pose_bones: Vec<String>,
    #[serde(default)]
    pub unresolved_pose_bones: Vec<String>,
    #[serde(default)]
    pub weak_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationCandidate {
    pub clip_identity: String,
    pub display_label: String,
    pub source_skeleton_local_key: String,
    pub association_kind: String,
    #[serde(default)]
    pub association_evidence: AssociationEvidence,
    pub start_frame: Option<i64>,
    pub end_frame: Option<i64>,
    pub fps_num: Option<u32>,
    pub fps_den: Option<u32>,
    pub usable: bool,
    pub unusable_reason: Option<String>,
}

impl AnimationCandidate {
    pub fn association_is_strong(&self) -> bool {
        matches!(
            self.association_kind.as_str(),
            "direct_action" | "nla_strip" | "pose_channels"
        )
    }

    pub fn presentation_line(&self) -> String {
        let frames = match (self.start_frame, self.end_frame) {
            (Some(start), Some(end)) => format!("Frames {start}–{end}"),
            _ => "Frames unavailable".to_string(),
        };
        let fps = match (self.fps_num, self.fps_den) {
            (Some(num), Some(1)) => format!("{num} FPS"),
            (Some(num), Some(den)) => format!("{num}/{den} FPS"),
            _ => "FPS unavailable".to_string(),
        };
        format!("{} · {}", frames, fps)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSourceInspection {
    pub source_path: PathBuf,
    pub source_digest: String,
    pub size_bytes: u64,
    pub observed_media_type: String,
    pub usable_armature_count: usize,
    pub skeleton_candidates: Vec<SkeletonCandidate>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionSourceInspection {
    pub source_path: PathBuf,
    pub source_digest: String,
    pub size_bytes: u64,
    pub observed_media_type: String,
    pub usable_armature_count: usize,
    pub skeleton_candidates: Vec<SkeletonCandidate>,
    pub animation_candidates: Vec<AnimationCandidate>,
    pub timing_context: Option<ObservedTimingContext>,
    pub diagnostics: Vec<String>,
}

impl MotionSourceInspection {
    pub fn usable_clips(&self) -> Vec<&AnimationCandidate> {
        self.animation_candidates
            .iter()
            .filter(|clip| clip.usable && clip.association_is_strong())
            .collect()
    }

    pub fn usable_clip(&self, clip_identity: &str) -> Option<&AnimationCandidate> {
        self.usable_clips()
            .into_iter()
            .find(|clip| clip.clip_identity == clip_identity)
    }
}

pub trait SourceInspectionProvider: Send {
    fn inspect_character_source(
        &self,
        path: &Path,
    ) -> Result<CharacterSourceInspection, AppError>;

    fn inspect_motion_source(&self, path: &Path) -> Result<MotionSourceInspection, AppError>;
}

pub fn require_live_source_matches(
    path: &Path,
    expected_digest: &str,
    expected_size: u64,
) -> Result<(String, u64), AppError> {
    if !path.is_file() {
        return Err(AppError::Catalog(INGEST_FILE_CHANGED.into()));
    }
    let size = std::fs::metadata(path)?.len();
    let digest = sha256_file(path)?;
    if digest != expected_digest || size != expected_size {
        return Err(AppError::Catalog(INGEST_FILE_CHANGED.into()));
    }
    Ok((digest, size))
}

pub fn unique_skeleton(inspection_count: usize) -> Result<(), AppError> {
    match inspection_count {
        0 => Err(AppError::Catalog(INGEST_NO_SKELETON.into())),
        1 => Ok(()),
        _ => Err(AppError::Catalog(INGEST_MULTIPLE_SKELETONS.into())),
    }
}

pub fn motion_ingest_failure_message(inspection: &MotionSourceInspection) -> Option<&'static str> {
    if !inspection.usable_clips().is_empty() {
        return None;
    }
    let strong: Vec<&AnimationCandidate> = inspection
        .animation_candidates
        .iter()
        .filter(|clip| clip.association_is_strong())
        .collect();
    if !strong.is_empty()
        && strong
            .iter()
            .all(|clip| clip.unusable_reason.as_deref() == Some("fractional_frames"))
    {
        return Some(INGEST_FRACTIONAL_FRAMES);
    }
    if !strong.is_empty()
        && strong
            .iter()
            .all(|clip| clip.unusable_reason.as_deref() == Some("unusable_timing"))
    {
        return Some(INGEST_UNUSABLE_TIMING);
    }
    Some(INGEST_NO_CLIPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_skeleton_zero_one_many() {
        assert!(unique_skeleton(0)
            .unwrap_err()
            .to_string()
            .contains("No skeleton"));
        assert!(unique_skeleton(1).is_ok());
        assert!(unique_skeleton(2)
            .unwrap_err()
            .to_string()
            .contains("Multiple usable skeletons"));
    }
}
