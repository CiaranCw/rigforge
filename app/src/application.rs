//! Workbench-facing application services. Product invariants stay out of GUI code.

use std::fmt;
use std::path::Path;

use rigforge_domain::{
    BoneMappingVersion, CharacterAsset, CharacterAssetVersion, DerivedVariant, JobSpec,
    MotionAsset, MotionAssetVersion, PersistenceArtifact, RetargetPolicyVersion,
    SourceSkeletonReference, Validated,
};

use crate::catalog::SqliteCatalog;
use crate::error::AppError;
use crate::orchestration::JobRun;
use crate::queries::AssetListItem;
use crate::worker::{DispatchReceipt, WorkerPort};

pub struct Application {
    pub catalog: SqliteCatalog,
}

impl fmt::Debug for Application {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Application").finish_non_exhaustive()
    }
}

impl Application {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        Ok(Self {
            catalog: SqliteCatalog::open(path)?,
        })
    }

    pub fn open_in_memory() -> Result<Self, AppError> {
        Ok(Self {
            catalog: SqliteCatalog::open_in_memory()?,
        })
    }

    pub fn from_catalog(catalog: SqliteCatalog) -> Self {
        Self { catalog }
    }

    pub fn catalog(&self) -> &SqliteCatalog {
        &self.catalog
    }

    pub fn catalog_mut(&mut self) -> &mut SqliteCatalog {
        &mut self.catalog
    }

    pub fn requires_network() -> bool {
        false
    }

    pub fn requires_blender() -> bool {
        false
    }

    pub fn list_characters(&self) -> Result<Vec<AssetListItem>, AppError> {
        self.catalog
            .list_assets(rigforge_domain::RecordType::CharacterAsset)
    }

    pub fn list_motions(&self) -> Result<Vec<AssetListItem>, AppError> {
        self.catalog
            .list_assets(rigforge_domain::RecordType::MotionAsset)
    }

    pub fn list_derived_variants(&self) -> Result<Vec<AssetListItem>, AppError> {
        self.catalog
            .list_assets(rigforge_domain::RecordType::DerivedVariant)
    }

    pub fn load_logical_character(
        &self,
        id: &str,
    ) -> Result<Validated<CharacterAsset>, AppError> {
        self.catalog.load_character_asset(id)
    }

    pub fn load_logical_motion(&self, id: &str) -> Result<Validated<MotionAsset>, AppError> {
        self.catalog.load_motion_asset(id)
    }

    pub fn load_logical_derived(&self, id: &str) -> Result<Validated<DerivedVariant>, AppError> {
        self.catalog.load_derived_variant(id)
    }

    pub fn resolve_exact_character_version(
        &self,
        version_id: &str,
    ) -> Result<Validated<CharacterAssetVersion>, AppError> {
        self.catalog.load_character_version(version_id)
    }

    pub fn resolve_exact_motion_version(
        &self,
        version_id: &str,
    ) -> Result<Validated<MotionAssetVersion>, AppError> {
        self.catalog.load_motion_version(version_id)
    }

    pub fn list_versions(&self, logical_id: &str) -> Result<Vec<String>, AppError> {
        self.catalog.list_version_ids(logical_id)
    }

    pub fn lookup_source_skeleton(
        &self,
        id: &str,
    ) -> Result<Validated<SourceSkeletonReference>, AppError> {
        self.catalog.load_source_skeleton(id)
    }

    pub fn lookup_mapping_version(
        &self,
        id: &str,
    ) -> Result<Validated<BoneMappingVersion>, AppError> {
        self.catalog.load_mapping_version(id)
    }

    pub fn lookup_policy_version(
        &self,
        id: &str,
    ) -> Result<Validated<RetargetPolicyVersion>, AppError> {
        self.catalog.load_policy_version(id)
    }

    pub fn load_job_spec(&self, id: &str) -> Result<Validated<JobSpec>, AppError> {
        self.catalog.load_job_spec(id)
    }

    pub fn load_artifact_metadata(
        &self,
        id: &str,
    ) -> Result<Validated<PersistenceArtifact>, AppError> {
        self.catalog.load_artifact_metadata(id)
    }

    pub fn enqueue_job(&mut self, spec: Validated<JobSpec>) -> Result<JobRun, AppError> {
        self.catalog.enqueue_job(spec)
    }

    pub fn job_status(&self, run_id: &str) -> Result<JobRun, AppError> {
        self.catalog.load_job_run(run_id)
    }

    pub fn mark_dispatchable(&mut self, run_id: &str) -> Result<JobRun, AppError> {
        self.catalog.mark_dispatchable(run_id)
    }

    pub fn dispatch<W: WorkerPort + ?Sized>(
        &mut self,
        run_id: &str,
        worker: &mut W,
    ) -> Result<(JobRun, DispatchReceipt), AppError> {
        self.catalog.dispatch(run_id, worker)
    }
}
