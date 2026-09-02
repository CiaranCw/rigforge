use std::path::Path;

use rigforge_app::rigforge_domain::VerificationOutcome;
use rigforge_app::{
    sha256_file, ArtifactInspectionEvidence, ArtifactInspector, PersistenceReopener,
};

/// Test-only inspector. Not part of the production Application surface.
#[derive(Clone, Debug)]
pub struct MemoryArtifactInspector {
    pub evidence: ArtifactInspectionEvidence,
}

impl MemoryArtifactInspector {
    pub fn passing(expected_duration_s: Option<f64>) -> Self {
        let duration = expected_duration_s.or(Some(2.0));
        Self {
            evidence: ArtifactInspectionEvidence {
                digest_before: String::new(),
                digest_after: String::new(),
                structurally_readable: true,
                finite_transforms: true,
                present_joint_keys: vec!["Bone".into(), "Body".into()],
                baked_animation_present: true,
                duration_s: duration,
                expected_duration_s: duration,
                gross_scale_sane: true,
                root_trajectory_sane: true,
            },
        }
    }

    pub fn with_joints(mut self, joints: Vec<String>) -> Self {
        self.evidence.present_joint_keys = joints;
        self
    }

    pub fn failing_finite_transforms(expected_duration_s: Option<f64>) -> Self {
        let mut inspector = Self::passing(expected_duration_s);
        inspector.evidence.finite_transforms = false;
        inspector
    }
}

impl ArtifactInspector for MemoryArtifactInspector {
    fn inspect(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<ArtifactInspectionEvidence, rigforge_app::AppError> {
        if !artifact_path.is_file() {
            return Err(rigforge_app::AppError::Worker(
                "QC inspect: artifact file is missing".into(),
            ));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Err(rigforge_app::AppError::Worker(format!(
                "QC inspect digest mismatch: expected {expected_sha256} found {found}"
            )));
        }
        let after = sha256_file(artifact_path)?;
        let mut evidence = self.evidence.clone();
        evidence.digest_before = found;
        evidence.digest_after = after;
        Ok(evidence)
    }
}

/// Test-only reopener. Not part of the production Application surface.
#[derive(Clone, Debug)]
pub struct MemoryPersistenceReopener {
    pub fresh_reopen: VerificationOutcome,
    pub structural: VerificationOutcome,
}

impl Default for MemoryPersistenceReopener {
    fn default() -> Self {
        Self {
            fresh_reopen: VerificationOutcome::Pass,
            structural: VerificationOutcome::Pass,
        }
    }
}

impl PersistenceReopener for MemoryPersistenceReopener {
    fn reopen(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<(VerificationOutcome, VerificationOutcome), rigforge_app::AppError> {
        if !artifact_path.is_file() {
            return Ok((VerificationOutcome::Missing, VerificationOutcome::Missing));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Ok((VerificationOutcome::Fail, VerificationOutcome::Fail));
        }
        Ok((self.fresh_reopen, self.structural))
    }
}
