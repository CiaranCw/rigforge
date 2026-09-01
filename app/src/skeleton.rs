//! Backend-neutral SkeletonSummary acquisition.
//! Implementations may use a Blender inspect process; callers receive Domain records.

use rigforge_domain::{
    CharacterAssetVersion, SkeletonSummary, SourceArtifactEvidence, SourceSkeletonReference,
};

use crate::error::AppError;

pub trait SkeletonEvidenceProvider {
    fn inspect_character(
        &self,
        character: &CharacterAssetVersion,
    ) -> Result<SkeletonSummary, AppError>;

    fn inspect_source_skeleton(
        &self,
        source: &SourceSkeletonReference,
        inspect_from: &SourceArtifactEvidence,
    ) -> Result<SkeletonSummary, AppError>;
}

/// Test/fixture inspector. Does not launch Blender.
#[derive(Clone, Debug)]
pub struct MemorySkeletonInspector {
    pub character: SkeletonSummary,
    pub source: SkeletonSummary,
}

impl SkeletonEvidenceProvider for MemorySkeletonInspector {
    fn inspect_character(
        &self,
        character: &CharacterAssetVersion,
    ) -> Result<SkeletonSummary, AppError> {
        if self.character.subject_character_version_id() != Some(character.id()) {
            return Err(AppError::Catalog(
                "character SkeletonSummary subject does not match CharacterAssetVersion".into(),
            ));
        }
        Ok(self.character.clone())
    }

    fn inspect_source_skeleton(
        &self,
        source: &SourceSkeletonReference,
        inspect_from: &SourceArtifactEvidence,
    ) -> Result<SkeletonSummary, AppError> {
        if self.source.subject_source_skeleton_ref_id() != Some(source.id()) {
            return Err(AppError::Catalog(
                "source SkeletonSummary subject does not match SourceSkeletonReference".into(),
            ));
        }
        let _ = inspect_from;
        Ok(self.source.clone())
    }
}
