//! Map exact Domain Mapping + Policy onto the worker JSON job. No auto-map.

use rigforge_app::WorkerDispatchRequest;
use rigforge_domain::{to_json, JointParticipation};
use serde::Serialize;
use serde_json::{json, Value};

use crate::envelope::JOB_SCHEMA;
use crate::pin::{ADAPTER_VERSION, BACKEND_KIND, BLENDER_BUILD, BLENDER_VERSION};
use crate::policy::project_supported_policy;

pub const SOURCE_REST_ACTION_CANDIDATES: &[&str] =
    &["Armature|Armature|A_TPose", "A_TPose"];

#[derive(Clone, Debug, Serialize)]
pub struct MappingEntryProjection {
    pub source: String,
    pub target: String,
    pub role: String,
    pub required: bool,
}

pub fn project_mapping_version(
    mapping: &rigforge_domain::BoneMappingVersion,
) -> Vec<MappingEntryProjection> {
    mapping
        .entries()
        .iter()
        .map(|entry| MappingEntryProjection {
            source: entry.source().joint_key().as_str().to_string(),
            target: entry.target().joint_key().as_str().to_string(),
            role: entry.role_profile().unwrap_or("mapped").to_string(),
            required: entry.participation() == JointParticipation::Required,
        })
        .collect()
}

pub fn project_mapping(
    request: &WorkerDispatchRequest,
) -> Result<Vec<MappingEntryProjection>, rigforge_app::AppError> {
    let entries = project_mapping_version(request.mapping().as_record());
    if entries.is_empty() {
        return Err(rigforge_app::AppError::Worker(
            "mapping projection is empty; fail before blender mutation".into(),
        ));
    }
    Ok(entries)
}

pub fn job_document(
    request: &WorkerDispatchRequest,
    workspace: &std::path::Path,
    staged_blend: &std::path::Path,
    result_path: &std::path::Path,
    reopen_path: &std::path::Path,
    test_behavior: Option<&str>,
) -> Result<Value, rigforge_app::AppError> {
    let spec = request.job_spec().as_record();
    let policy = project_supported_policy(request.policy().as_record())?;
    let mapping = project_mapping(request)?;
    let mapping_json = to_json(request.mapping().as_record()).map_err(rigforge_app::AppError::from)?;
    let policy_json = to_json(request.policy().as_record()).map_err(rigforge_app::AppError::from)?;
    let mut job = json!({
        "schema": JOB_SCHEMA,
        "job_spec_id": spec.id().canonical(),
        "attempt_id": request.attempt_id(),
        "source_skeleton_id": request.source_skeleton_id(),
        "determinism_context": spec.determinism_context(),
        "isolation_limits": spec.isolation_limits(),
        "expected_backend": {
            "kind": BACKEND_KIND,
            "version": BLENDER_VERSION,
            "build": BLENDER_BUILD,
            "adapter_version": ADAPTER_VERSION
        },
        "character": {
            "version_id": request.character().version_id,
            "path": request.character().location,
            "sha256": request.character().digest_sha256,
            "size_bytes": request.character().size_bytes
        },
        "motion": {
            "version_id": request.motion().version_id,
            "path": request.motion().location,
            "sha256": request.motion().digest_sha256,
            "size_bytes": request.motion().size_bytes,
            "clip_id": request.motion_clip_id(),
            "frame_start": request.frame_start(),
            "frame_end": request.frame_end(),
            "fps_num": request.fps_num(),
            "fps_den": request.fps_den()
        },
        "source_rest_action_candidates": SOURCE_REST_ACTION_CANDIDATES,
        "mapping": {
            "version_id": request.mapping().as_record().id().canonical(),
            "entries": mapping
        },
        "policy": {
            "version_id": request.policy().as_record().id().canonical(),
            "root_policy": policy.root_policy,
            "channel_policy": policy.channel_policy,
            "quaternion_normalization_policy": policy.quaternion_normalization_policy,
            "quaternion_continuity_policy": policy.quaternion_continuity_policy,
            "quaternion_interpolation_policy": policy.quaternion_interpolation_policy,
            "time_bake_policy": policy.time_bake_policy,
            "rest_alignment_policy": policy.rest_alignment_policy,
            "unmapped_target_joints": policy.unmapped_target_joints,
            "missing_source_joint": policy.missing_source_joint,
            "scale_policy": policy.scale_policy,
            "ik_policy": policy.ik_policy
        },
        "outputs": {
            "workspace": workspace,
            "result_envelope": result_path,
            "staged_blend": staged_blend,
            "reopen_verification": reopen_path
        },
        "domain_record_sha256": {
            "mapping_json": crate::pin::sha256_bytes(mapping_json.as_bytes()),
            "policy_json": crate::pin::sha256_bytes(policy_json.as_bytes())
        }
    });
    if let Some(behavior) = test_behavior {
        job.as_object_mut()
            .expect("job object")
            .insert("test_behavior".into(), Value::String(behavior.to_string()));
    }
    Ok(job)
}
