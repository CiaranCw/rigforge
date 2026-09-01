//! Isolated Blender inspect. Observation envelope → Domain SkeletonSummary.
//! Does not execute retarget or produce WorkerResult.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use rigforge_app::{AppError, SkeletonEvidenceProvider};
use rigforge_domain::{
    BackendExecutionContext, CharacterAssetVersion, CharacterAssetVersionId, JointKey,
    JointObservation, SkeletonSubjectKind, SkeletonSummary, SourceArtifactEvidence,
    SourceSkeletonReference, SourceSkeletonReferenceId, Validated,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::command::{assert_safety_flags, blender_argv, blender_command};
use crate::isolation::{attempt_workspace_root, AttemptWorkspace};
use crate::pin::{
    enforce_pin, sha256_file, ADAPTER_VERSION, BACKEND_KIND, BLENDER_BUILD, BLENDER_VERSION,
    BlenderPin, EXECUTION_POLICY_VERSION,
};

use crate::adapter::production_worker_script;

#[derive(Debug, Deserialize)]
struct InspectEnvelope {
    status: String,
    joints: Vec<InspectJoint>,
    #[serde(default)]
    diagnostics: Vec<String>,
    #[serde(default)]
    source_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InspectJoint {
    joint_key: String,
    display_name: String,
    parent_key: Option<String>,
    is_root: bool,
    deform_observation: Option<String>,
    rest_evidence: Option<String>,
}

pub struct BlenderSkeletonInspector {
    pin: BlenderPin,
    script: PathBuf,
    workspace_root: PathBuf,
}

impl BlenderSkeletonInspector {
    pub fn production() -> Result<Self, AppError> {
        let pin = BlenderPin::accepted();
        Ok(Self {
            pin,
            script: production_worker_script(),
            workspace_root: std::env::temp_dir().join("rigforge-v14-inspect"),
        })
    }

    pub fn inspect_path(
        &self,
        source_path: &Path,
        expected_sha256: &str,
        subject_kind: SkeletonSubjectKind,
        subject_character_version_id: Option<CharacterAssetVersionId>,
        subject_source_skeleton_ref_id: Option<SourceSkeletonReferenceId>,
    ) -> Result<Validated<SkeletonSummary>, AppError> {
        enforce_pin(&self.pin.executable, &self.pin)?;
        let found = sha256_file(source_path)?;
        if found != expected_sha256 {
            return Err(AppError::Worker(format!(
                "inspect source digest mismatch: expected {expected_sha256} found {found}"
            )));
        }
        let attempt = format!("inspect-{}", Uuid::now_v7());
        let root = attempt_workspace_root(&self.workspace_root, &attempt);
        let workspace = AttemptWorkspace::create(&root)?;
        let job_json = workspace.root.join("inspect_job.json");
        let envelope_path = workspace.root.join("inspect_envelope.json");
        let job = serde_json::json!({
            "source_path": source_path.to_string_lossy(),
            "expected_digest": expected_sha256,
            "outputs": { "inspect_envelope": envelope_path }
        });
        fs::write(&job_json, serde_json::to_vec_pretty(&job).unwrap())?;
        let argv = blender_argv(&self.pin.executable, &self.script, "inspect", &job_json);
        if !assert_safety_flags(&argv) {
            return Err(AppError::Worker("inspect command missing safety flags".into()));
        }
        let mut cmd = blender_command(&self.pin.executable, &self.script, "inspect", &job_json);
        for (k, v) in workspace.isolated_env() {
            cmd.env(k, v);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(AppError::Worker(format!(
                "inspect blender exited {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let text = fs::read_to_string(&envelope_path)?;
        let envelope: InspectEnvelope = serde_json::from_str(&text)
            .map_err(|e| AppError::Worker(format!("inspect envelope: {e}")))?;
        if !envelope.status.eq_ignore_ascii_case("SUCCESS") {
            return Err(AppError::Worker("inspect envelope was not SUCCESS".into()));
        }
        if let Some(digest) = &envelope.source_digest {
            if digest != expected_sha256 {
                return Err(AppError::Worker(
                    "inspect envelope digest does not match subject source".into(),
                ));
            }
        }
        let mut joints = Vec::new();
        for joint in envelope.joints {
            joints.push(JointObservation::new(
                JointKey::new(joint.joint_key)?,
                joint.display_name,
                joint.parent_key.map(JointKey::new).transpose()?,
                joint.is_root,
                joint.deform_observation,
                joint.rest_evidence,
            )?);
        }
        let producer = BackendExecutionContext::new(
            BACKEND_KIND,
            BLENDER_VERSION,
            BLENDER_BUILD,
            ADAPTER_VERSION,
            EXECUTION_POLICY_VERSION,
        )?;
        let mut diagnostics = envelope.diagnostics;
        diagnostics.push(format!("source_digest={expected_sha256}"));
        let summary = SkeletonSummary::new(
            subject_kind,
            subject_character_version_id,
            subject_source_skeleton_ref_id,
            producer,
            joints,
            diagnostics,
        )?;
        Validated::certify(summary).map_err(AppError::from)
    }
}

impl SkeletonEvidenceProvider for BlenderSkeletonInspector {
    fn inspect_character(
        &self,
        character: &CharacterAssetVersion,
    ) -> Result<SkeletonSummary, AppError> {
        let path = Path::new(character.source().location().value());
        Ok(self
            .inspect_path(
                path,
                character.source().digest().sha256(),
                SkeletonSubjectKind::CharacterAssetVersion,
                Some(character.id()),
                None,
            )?
            .into_record())
    }

    fn inspect_source_skeleton(
        &self,
        source: &SourceSkeletonReference,
        inspect_from: &SourceArtifactEvidence,
    ) -> Result<SkeletonSummary, AppError> {
        let path = Path::new(inspect_from.location().value());
        Ok(self
            .inspect_path(
                path,
                inspect_from.digest().sha256(),
                SkeletonSubjectKind::SourceSkeletonReference,
                None,
                Some(source.id()),
            )?
            .into_record())
    }
}
