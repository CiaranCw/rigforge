//! Backend-neutral SkeletonSummary acquisition.
//! Implementations may use a Blender inspect process; callers receive Domain records.

use rigforge_domain::{
    BackendExecutionContext, CharacterAssetVersion, JointKey, JointObservation, SkeletonSubjectKind,
    SkeletonSummary, SourceArtifactEvidence, SourceSkeletonReference,
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

/// Registration/test inspector that rebinds template joints onto the inspected subject.
/// Does not launch Blender. Subject IDs need not be known in advance.
#[derive(Clone, Debug)]
pub struct FixtureSkeletonInspector {
    joints: Vec<JointObservation>,
    producer: BackendExecutionContext,
    fail: bool,
}

impl FixtureSkeletonInspector {
    pub fn usable() -> Self {
        Self {
            joints: vec![JointObservation::new(
                JointKey::new("root").unwrap(),
                "root",
                None,
                true,
                Some("deforming".into()),
                Some("rest".into()),
            )
            .unwrap()],
            producer: BackendExecutionContext::new(
                "isolated-worker",
                "1.0.0",
                "build-test",
                "adapter-1",
                "exec-policy-1",
            )
            .unwrap(),
            fail: false,
        }
    }

    pub fn empty_joints() -> Self {
        let mut inspector = Self::usable();
        inspector.joints.clear();
        inspector
    }

    pub fn failing() -> Self {
        let mut inspector = Self::usable();
        inspector.fail = true;
        inspector
    }
}

impl SkeletonEvidenceProvider for FixtureSkeletonInspector {
    fn inspect_character(
        &self,
        character: &CharacterAssetVersion,
    ) -> Result<SkeletonSummary, AppError> {
        if self.fail {
            return Err(AppError::Catalog(
                "registration: Character Skeleton inspection failed".into(),
            ));
        }
        SkeletonSummary::new(
            SkeletonSubjectKind::CharacterAssetVersion,
            Some(character.id()),
            None,
            self.producer.clone(),
            self.joints.clone(),
            Vec::new(),
        )
        .map_err(AppError::from)
    }

    fn inspect_source_skeleton(
        &self,
        source: &SourceSkeletonReference,
        inspect_from: &SourceArtifactEvidence,
    ) -> Result<SkeletonSummary, AppError> {
        let _ = inspect_from;
        if self.fail {
            return Err(AppError::Catalog(
                "registration: source Skeleton inspection failed".into(),
            ));
        }
        SkeletonSummary::new(
            SkeletonSubjectKind::SourceSkeletonReference,
            None,
            Some(source.id()),
            self.producer.clone(),
            self.joints.clone(),
            Vec::new(),
        )
        .map_err(AppError::from)
    }
}
