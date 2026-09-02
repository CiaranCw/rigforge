//! Product-owned QC evaluation. Inspection is read-only evidence, not authority.

use std::path::Path;

use rigforge_domain::{
    passing_structural_qc_checks, BoneMappingVersion, ContentDigest, JointParticipation,
    MotionAssetVersion, QcCheck, QcCheckName, QcCheckOutcome,
};
use serde::Deserialize;

use crate::error::AppError;

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactInspectionEvidence {
    pub digest_before: String,
    pub digest_after: String,
    pub structurally_readable: bool,
    pub finite_transforms: bool,
    pub present_joint_keys: Vec<String>,
    pub baked_animation_present: bool,
    pub duration_s: Option<f64>,
    pub expected_duration_s: Option<f64>,
    pub gross_scale_sane: bool,
    pub root_trajectory_sane: bool,
}

/// inspect_qc worker envelope. Not a Product record and not QC authority.
#[derive(Debug, Deserialize)]
pub struct QcInspectEnvelope {
    pub status: String,
    #[serde(default)]
    pub digest_before: Option<String>,
    #[serde(default)]
    pub digest_after: Option<String>,
    #[serde(default)]
    pub structurally_readable: bool,
    #[serde(default)]
    pub finite_transforms: bool,
    #[serde(default)]
    pub present_joint_keys: Vec<String>,
    #[serde(default)]
    pub baked_animation_present: bool,
    #[serde(default)]
    pub duration_s: Option<f64>,
    #[serde(default)]
    pub gross_scale_sane: bool,
    #[serde(default)]
    pub root_trajectory_sane: bool,
    #[serde(default)]
    pub saved: bool,
}

/// Fail-closed gate before typed Product QC checks. Requires process SUCCESS
/// and envelope SUCCESS. Not a caller-replaceable verdict.
pub fn interpret_qc_inspect(
    process_success: bool,
    envelope: &QcInspectEnvelope,
    expected_sha256: &str,
    digest_before: &str,
    digest_after: &str,
) -> Result<ArtifactInspectionEvidence, AppError> {
    if envelope.saved {
        return Err(AppError::Worker(
            "inspect_qc envelope claims the scene was saved".into(),
        ));
    }
    if digest_before != expected_sha256
        || envelope.digest_before.as_deref() != Some(expected_sha256)
        || digest_after != digest_before
        || envelope.digest_after.as_deref() != Some(digest_after)
    {
        return Err(AppError::Worker(
            "inspect_qc envelope digest does not match persisted artifact bytes".into(),
        ));
    }
    if !process_success {
        return Err(AppError::Worker(
            "inspect_qc process exit was not SUCCESS; QC fails closed".into(),
        ));
    }
    if !envelope.status.eq_ignore_ascii_case("SUCCESS") {
        return Err(AppError::Worker(format!(
            "inspect_qc envelope status '{}' is not SUCCESS; QC fails closed",
            envelope.status
        )));
    }
    Ok(ArtifactInspectionEvidence {
        digest_before: digest_before.to_string(),
        digest_after: digest_after.to_string(),
        structurally_readable: envelope.structurally_readable,
        finite_transforms: envelope.finite_transforms,
        present_joint_keys: envelope.present_joint_keys.clone(),
        baked_animation_present: envelope.baked_animation_present,
        duration_s: envelope.duration_s,
        expected_duration_s: None,
        gross_scale_sane: envelope.gross_scale_sane,
        root_trajectory_sane: envelope.root_trajectory_sane,
    })
}

/// Test-only evidence ports. Not part of the production Application surface.
#[cfg(any(test, feature = "test-support"))]
pub trait ArtifactInspector {
    fn inspect(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<ArtifactInspectionEvidence, AppError>;
}

/// Test-only evidence ports. Not part of the production Application surface.
#[cfg(any(test, feature = "test-support"))]
pub trait PersistenceReopener {
    fn reopen(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<
        (
            rigforge_domain::VerificationOutcome,
            rigforge_domain::VerificationOutcome,
        ),
        AppError,
    >;
}

pub fn sha256_file(path: &Path) -> Result<String, AppError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn evaluate_structural_qc(
    evidence: &ArtifactInspectionEvidence,
    mapping: &BoneMappingVersion,
    motion: &MotionAssetVersion,
) -> Result<Vec<QcCheck>, AppError> {
    let _ = ContentDigest::parse(&evidence.digest_before)?;
    let mut checks = passing_structural_qc_checks();
    set_check(
        &mut checks,
        QcCheckName::PersistenceDigestStable,
        if evidence.digest_before == evidence.digest_after
            && evidence.digest_before.len() == 64
        {
            QcCheckOutcome::Pass
        } else {
            QcCheckOutcome::Fail
        },
    );
    set_check(
        &mut checks,
        QcCheckName::FiniteTransforms,
        bool_outcome(evidence.structurally_readable && evidence.finite_transforms),
    );
    let required_targets: Vec<_> = mapping
        .entries()
        .iter()
        .filter(|e| e.participation() == JointParticipation::Required)
        .map(|e| e.target().joint_key().as_str().to_string())
        .collect();
    let joints_ok = required_targets
        .iter()
        .all(|key| evidence.present_joint_keys.iter().any(|k| k == key));
    set_check(
        &mut checks,
        QcCheckName::RequiredMappedJointsPresent,
        bool_outcome(joints_ok),
    );
    set_check(
        &mut checks,
        QcCheckName::ExpectedBakedAnimationPresent,
        bool_outcome(evidence.baked_animation_present),
    );
    let expected = evidence.expected_duration_s.or_else(|| {
        let start = motion.time().start();
        let end = motion.time().end();
        let fps = start.fps_num()? as f64 / start.fps_den()? as f64;
        if fps <= 0.0 {
            return None;
        }
        Some((end.value_num() - start.value_num()) as f64 / fps)
    });
    let duration_ok = match (evidence.duration_s, expected) {
        (Some(actual), Some(want)) => (actual - want).abs() <= want.abs().max(0.1) * 0.25 + 0.25,
        (Some(_), None) => true,
        (None, _) => false,
    };
    set_check(
        &mut checks,
        QcCheckName::TimeRangeDurationSane,
        bool_outcome(duration_ok),
    );
    set_check(
        &mut checks,
        QcCheckName::GrossScaleTransformSane,
        bool_outcome(evidence.gross_scale_sane),
    );
    set_check(
        &mut checks,
        QcCheckName::RootTrajectorySane,
        bool_outcome(evidence.root_trajectory_sane),
    );
    Ok(checks)
}

fn bool_outcome(ok: bool) -> QcCheckOutcome {
    if ok {
        QcCheckOutcome::Pass
    } else {
        QcCheckOutcome::Fail
    }
}

fn set_check(checks: &mut [QcCheck], name: QcCheckName, outcome: QcCheckOutcome) {
    if let Some(check) = checks.iter_mut().find(|c| c.name() == name) {
        *check = QcCheck::new(name, outcome);
    }
}

#[cfg(test)]
pub(crate) struct MemoryArtifactInspector {
    pub evidence: ArtifactInspectionEvidence,
}

#[cfg(test)]
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

    #[allow(dead_code)]
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

#[cfg(test)]
impl ArtifactInspector for MemoryArtifactInspector {
    fn inspect(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<ArtifactInspectionEvidence, AppError> {
        if !artifact_path.is_file() {
            return Err(AppError::Worker(
                "QC inspect: artifact file is missing".into(),
            ));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Err(AppError::Worker(format!(
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

#[cfg(test)]
pub(crate) struct MemoryPersistenceReopener {
    pub fresh_reopen: rigforge_domain::VerificationOutcome,
    pub structural: rigforge_domain::VerificationOutcome,
}

#[cfg(test)]
impl Default for MemoryPersistenceReopener {
    fn default() -> Self {
        Self {
            fresh_reopen: rigforge_domain::VerificationOutcome::Pass,
            structural: rigforge_domain::VerificationOutcome::Pass,
        }
    }
}

#[cfg(test)]
impl PersistenceReopener for MemoryPersistenceReopener {
    fn reopen(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<
        (
            rigforge_domain::VerificationOutcome,
            rigforge_domain::VerificationOutcome,
        ),
        AppError,
    > {
        if !artifact_path.is_file() {
            return Ok((
                rigforge_domain::VerificationOutcome::Missing,
                rigforge_domain::VerificationOutcome::Missing,
            ));
        }
        let found = sha256_file(artifact_path)?;
        if found != expected_sha256 {
            return Ok((
                rigforge_domain::VerificationOutcome::Fail,
                rigforge_domain::VerificationOutcome::Fail,
            ));
        }
        Ok((self.fresh_reopen, self.structural))
    }
}

#[cfg(test)]
mod interpret_tests {
    use super::*;

    fn all_pass_envelope(status: &str) -> QcInspectEnvelope {
        QcInspectEnvelope {
            status: status.into(),
            digest_before: Some("abc".into()),
            digest_after: Some("abc".into()),
            structurally_readable: true,
            finite_transforms: true,
            present_joint_keys: vec!["Hips".into()],
            baked_animation_present: true,
            duration_s: Some(1.0),
            gross_scale_sane: true,
            root_trajectory_sane: true,
            saved: false,
        }
    }

    #[test]
    fn qc_process_nonzero_with_all_pass_fields_fails() {
        let err = interpret_qc_inspect(false, &all_pass_envelope("SUCCESS"), "abc", "abc", "abc")
            .expect_err("nonzero process must fail closed");
        assert!(err.to_string().contains("process exit"));
    }

    #[test]
    fn qc_envelope_fail_with_all_pass_fields_fails() {
        let err = interpret_qc_inspect(true, &all_pass_envelope("FAIL"), "abc", "abc", "abc")
            .expect_err("FAIL envelope must fail closed");
        assert!(err.to_string().contains("not SUCCESS"));
    }

    #[test]
    fn qc_envelope_unknown_status_fails() {
        let err = interpret_qc_inspect(true, &all_pass_envelope("UNKNOWN"), "abc", "abc", "abc")
            .expect_err("unknown envelope status must fail closed");
        assert!(err.to_string().contains("not SUCCESS"));
    }

    #[test]
    fn qc_process_success_and_envelope_success_can_continue() {
        let evidence =
            interpret_qc_inspect(true, &all_pass_envelope("SUCCESS"), "abc", "abc", "abc")
                .expect("process SUCCESS + envelope SUCCESS may continue");
        assert!(evidence.structurally_readable);
        assert!(evidence.finite_transforms);
    }

    #[test]
    fn qc_fail_cannot_create_qc_pass() {
        assert!(
            interpret_qc_inspect(false, &all_pass_envelope("SUCCESS"), "abc", "abc", "abc")
                .is_err()
        );
        assert!(interpret_qc_inspect(true, &all_pass_envelope("FAIL"), "abc", "abc", "abc").is_err());
    }
}
