//! Backend-neutral resolved execution request.
//!
//! Assembled from exact Catalog records before the worker boundary.
//! This is not a Product identity record and is not a JobSpec mutation.

use std::path::PathBuf;

use rigforge_domain::{
    BoneMappingVersion, JobSpec, LocationKind, MotionAssetVersion, TimeKind, Validated,
};

use crate::error::AppError;

/// Exact source file binding for one Product version. Path is location evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSourceInput {
    pub version_id: String,
    pub location: PathBuf,
    pub digest_sha256: String,
    pub size_bytes: u64,
}

/// Frozen execution input projected from exact Catalog records.
#[derive(Clone, Debug)]
pub struct WorkerDispatchRequest {
    job_spec: Validated<JobSpec>,
    attempt_id: String,
    character: ResolvedSourceInput,
    motion: ResolvedSourceInput,
    motion_clip_id: String,
    frame_start: i64,
    frame_end: i64,
    fps_num: u32,
    fps_den: u32,
    source_skeleton_id: String,
    mapping: Validated<BoneMappingVersion>,
    policy: Validated<rigforge_domain::RetargetPolicyVersion>,
}

impl WorkerDispatchRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_spec: Validated<JobSpec>,
        attempt_id: impl Into<String>,
        character: ResolvedSourceInput,
        motion: ResolvedSourceInput,
        motion_version: &MotionAssetVersion,
        source_skeleton_id: impl Into<String>,
        mapping: Validated<BoneMappingVersion>,
        policy: Validated<rigforge_domain::RetargetPolicyVersion>,
    ) -> Result<Self, AppError> {
        let spec = job_spec.as_record();
        if character.version_id != spec.character_version_id().canonical() {
            return Err(AppError::Orchestration(
                "resolved character version does not match JobSpec".into(),
            ));
        }
        if motion.version_id != spec.motion_version_id().canonical() {
            return Err(AppError::Orchestration(
                "resolved motion version does not match JobSpec".into(),
            ));
        }
        if mapping.as_record().id().canonical() != spec.mapping_version_id().canonical() {
            return Err(AppError::Orchestration(
                "resolved mapping version does not match JobSpec".into(),
            ));
        }
        if policy.as_record().id().canonical() != spec.policy_version_id().canonical() {
            return Err(AppError::Orchestration(
                "resolved policy version does not match JobSpec".into(),
            ));
        }
        let time = motion_version.time();
        if time.start().kind() != TimeKind::Frames || time.end().kind() != TimeKind::Frames {
            return Err(AppError::Orchestration(
                "motion time domain is not frames; fail before blender mutation".into(),
            ));
        }
        let fps_num = time.start().fps_num().ok_or_else(|| {
            AppError::Orchestration("motion frame time is missing fps_num".into())
        })?;
        let fps_den = time.start().fps_den().ok_or_else(|| {
            AppError::Orchestration("motion frame time is missing fps_den".into())
        })?;
        if !time.start().is_integral_frame() || !time.end().is_integral_frame() {
            return Err(AppError::Orchestration(
                "unsupported execution-time provenance: V1-3 Blender path requires integral frame points (value_den == 1); fail before blender mutation".into(),
            ));
        }
        Ok(Self {
            job_spec,
            attempt_id: attempt_id.into(),
            character,
            motion,
            motion_clip_id: time.clip_identity_evidence().to_string(),
            frame_start: time.start().value_num(),
            frame_end: time.end().value_num(),
            fps_num,
            fps_den,
            source_skeleton_id: source_skeleton_id.into(),
            mapping,
            policy,
        })
    }

    pub fn job_spec(&self) -> &Validated<JobSpec> {
        &self.job_spec
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn character(&self) -> &ResolvedSourceInput {
        &self.character
    }

    pub fn motion(&self) -> &ResolvedSourceInput {
        &self.motion
    }

    pub fn motion_clip_id(&self) -> &str {
        &self.motion_clip_id
    }

    pub fn frame_start(&self) -> i64 {
        self.frame_start
    }

    pub fn frame_end(&self) -> i64 {
        self.frame_end
    }

    pub fn fps_num(&self) -> u32 {
        self.fps_num
    }

    pub fn fps_den(&self) -> u32 {
        self.fps_den
    }

    pub fn source_skeleton_id(&self) -> &str {
        &self.source_skeleton_id
    }

    pub fn mapping(&self) -> &Validated<BoneMappingVersion> {
        &self.mapping
    }

    pub fn policy(&self) -> &Validated<rigforge_domain::RetargetPolicyVersion> {
        &self.policy
    }
}

pub fn filesystem_location(evidence: &rigforge_domain::SourceArtifactEvidence) -> Result<PathBuf, AppError> {
    if evidence.location().kind() != LocationKind::FilesystemPathEvidence {
        return Err(AppError::Orchestration(
            "source location is not filesystem path evidence; fail before blender mutation".into(),
        ));
    }
    Ok(PathBuf::from(evidence.location().value()))
}
