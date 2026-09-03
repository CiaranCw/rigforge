//! Product-owned local Character / Motion registration.
//!
//! Workbench calls Application. Application builds Domain records and
//! persists them through Catalog transactions. Paths and digests are
//! evidence only; they are not Product identity. FBX is the currently
//! supported V1 ingest format, not Product authority.

use std::path::Path;

use rigforge_domain::{
    CharacterAsset, CharacterAssetVersion, ContentDigest, LocationEvidence, MotionAsset,
    MotionAssetVersion, SamplingInterpretation, SourceArtifactEvidence, SourceSkeletonReference,
    TimeDomainProvenance, TimePoint, Validated,
};

use crate::error::AppError;
use crate::skeleton::SkeletonEvidenceProvider;
use crate::Application;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalCharacterRegistration {
    pub asset_id: String,
    pub version_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalMotionRegistration {
    pub asset_id: String,
    pub version_id: String,
    pub source_skeleton_id: String,
}

fn registration_err(message: impl Into<String>) -> AppError {
    AppError::Catalog(message.into())
}

fn require_nonempty_field(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(registration_err(format!(
            "registration: {field} is required"
        )));
    }
    Ok(())
}

fn source_evidence_from_local_path(path: &Path) -> Result<SourceArtifactEvidence, AppError> {
    if !path.exists() {
        return Err(registration_err(format!(
            "registration: file is missing: {}",
            path.display()
        )));
    }
    if path.is_dir() {
        return Err(registration_err(format!(
            "registration: path is a directory, not a file: {}",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(registration_err(format!(
            "registration: not a regular file: {}",
            path.display()
        )));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "fbx" {
        return Err(registration_err(format!(
            "unsupported source format '{ext}'; V1 registration supports FBX only (FBX is not Product authority)"
        )));
    }
    let size = std::fs::metadata(path)?.len();
    let digest = crate::qc::sha256_file(path)?;
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path.to_string_lossy().into_owned())?,
        ContentDigest::parse(&digest)?,
        size,
        "application/octet-stream",
        None,
        Some("v1-local-fbx-registration".into()),
    )
    .map_err(AppError::from)
}

fn require_usable_skeleton(joint_count: usize, kind: &str) -> Result<(), AppError> {
    if joint_count == 0 {
        return Err(registration_err(format!(
            "registration: {kind} has no usable Skeleton evidence (zero joints / uninspectable as a V1 rigged Character or Motion source)"
        )));
    }
    Ok(())
}

impl Application {
    pub fn register_local_character(
        &mut self,
        display_name: &str,
        source_path: impl AsRef<Path>,
        inspector: &impl SkeletonEvidenceProvider,
    ) -> Result<LocalCharacterRegistration, AppError> {
        require_nonempty_field(display_name, "display name")?;
        let source = source_evidence_from_local_path(source_path.as_ref())?;
        let mut asset = CharacterAsset::new(display_name.trim())?;
        let mut version =
            CharacterAssetVersion::draft(asset.id(), display_name.trim(), source)?;
        let summary = inspector.inspect_character(&version)?;
        require_usable_skeleton(summary.joints().len(), "Character")?;
        version.publish()?;
        asset.bind_published(version.id());
        let asset_id = asset.id().canonical();
        let version_id = version.id().canonical();
        let certified_asset = Validated::certify(asset)?;
        let certified_version = Validated::certify(version)?;
        self.catalog
            .persist_character_registration(&certified_asset, &certified_version)?;
        Ok(LocalCharacterRegistration {
            asset_id,
            version_id,
        })
    }

    pub fn register_local_motion(
        &mut self,
        display_name: &str,
        source_path: impl AsRef<Path>,
        source_skeleton_display_name: &str,
        clip_identity: &str,
        start_frame: i64,
        end_frame: i64,
        fps_num: u32,
        fps_den: u32,
        inspector: &impl SkeletonEvidenceProvider,
    ) -> Result<LocalMotionRegistration, AppError> {
        require_nonempty_field(display_name, "display name")?;
        require_nonempty_field(source_skeleton_display_name, "Source Skeleton name")?;
        require_nonempty_field(clip_identity, "clip identifier")?;
        if fps_num == 0 || fps_den == 0 {
            return Err(registration_err(
                "registration: FPS numerator and denominator must be positive integers",
            ));
        }
        let source = source_evidence_from_local_path(source_path.as_ref())?;
        let source_skeleton =
            SourceSkeletonReference::new(source_skeleton_display_name.trim())?;
        let time = TimeDomainProvenance::new(
            clip_identity.trim(),
            TimePoint::frames(start_frame, fps_num, fps_den)?,
            TimePoint::frames(end_frame, fps_num, fps_den)?,
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )?;
        let mut asset = MotionAsset::new(display_name.trim())?;
        let mut version = MotionAssetVersion::draft(
            asset.id(),
            display_name.trim(),
            source_skeleton.id(),
            time,
            source.clone(),
        )?;
        let summary = inspector.inspect_source_skeleton(&source_skeleton, &source)?;
        require_usable_skeleton(summary.joints().len(), "Motion source Skeleton")?;
        version.publish()?;
        asset.bind_published(version.id());
        let asset_id = asset.id().canonical();
        let version_id = version.id().canonical();
        let source_skeleton_id = source_skeleton.id().canonical();
        let certified_source = Validated::certify(source_skeleton)?;
        let certified_asset = Validated::certify(asset)?;
        let certified_version = Validated::certify(version)?;
        self.catalog.persist_motion_registration(
            &certified_source,
            &certified_asset,
            &certified_version,
        )?;
        Ok(LocalMotionRegistration {
            asset_id,
            version_id,
            source_skeleton_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::FixtureSkeletonInspector;
    use rigforge_domain::RecordType;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(name: &str, bytes: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rf-reg-{}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn usable() -> FixtureSkeletonInspector {
        FixtureSkeletonInspector::usable()
    }

    #[test]
    fn character_registration_transaction_failure_leaves_no_partial_records() {
        let fbx = temp_path("hero.fbx", b"fake-fbx-bytes");
        let mut app = Application::open_in_memory().unwrap();
        app.catalog_mut().fail_next_registration_transaction();
        let err = app
            .register_local_character("Hero", &fbx, &usable())
            .unwrap_err()
            .to_string();
        assert!(err.contains("forced registration"), "{err}");
        assert!(app.list_characters().unwrap().is_empty());
        assert!(app
            .catalog()
            .list_assets(RecordType::CharacterAssetVersion)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn motion_registration_transaction_failure_leaves_no_partial_records() {
        let fbx = temp_path("walk.fbx", b"fake-motion-fbx");
        let mut app = Application::open_in_memory().unwrap();
        app.catalog_mut().fail_next_registration_transaction();
        let err = app
            .register_local_motion(
                "Walk",
                &fbx,
                "UAL2",
                "clip:walk",
                1,
                10,
                30,
                1,
                &usable(),
            )
            .unwrap_err()
            .to_string();
        assert!(err.contains("forced registration"), "{err}");
        assert!(app.list_motions().unwrap().is_empty());
        assert!(app
            .catalog()
            .list_assets(RecordType::MotionAssetVersion)
            .unwrap()
            .is_empty());
        assert!(app
            .catalog()
            .list_assets(RecordType::SourceSkeletonReference)
            .unwrap()
            .is_empty());
    }
}
