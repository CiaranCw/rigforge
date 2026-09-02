//! Read-only Blender inspection of durable Persistence Artifact bytes.
//! Does not execute retarget, does not save, and is not Product QC authority.
//!
//! Production Application finalization calls the same pinned inspect/reopen
//! functions in `rigforge_app`. These types remain a Blender-worker convenience
//! wrapper for adapter tests. They are not Application publication ports.

use std::path::Path;

use rigforge_app::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact, AppError,
    ArtifactInspectionEvidence,
};
use rigforge_domain::{BoneMappingVersion, VerificationOutcome};

pub struct BlenderQcInspector {
    mapping: Option<BoneMappingVersion>,
}

impl BlenderQcInspector {
    pub fn production() -> Result<Self, AppError> {
        Ok(Self { mapping: None })
    }

    pub fn with_mapping(mut self, mapping: BoneMappingVersion) -> Self {
        self.mapping = Some(mapping);
        self
    }

    pub fn inspect(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<ArtifactInspectionEvidence, AppError> {
        inspect_durable_persistence_artifact(
            artifact_path,
            expected_sha256,
            self.mapping.as_ref(),
        )
    }
}

pub struct BlenderPersistenceReopener {
    mapping: BoneMappingVersion,
}

impl BlenderPersistenceReopener {
    pub fn production(mapping: BoneMappingVersion) -> Result<Self, AppError> {
        Ok(Self { mapping })
    }

    pub fn reopen(
        &self,
        artifact_path: &Path,
        expected_sha256: &str,
    ) -> Result<(VerificationOutcome, VerificationOutcome), AppError> {
        reopen_durable_persistence_artifact(artifact_path, expected_sha256, &self.mapping)
    }
}
