//! Promote staged worker output to durable Persistence Artifact bytes.

use std::fs;
use std::path::{Path, PathBuf};

use rigforge_domain::{
    BackendExecutionContextId, ContentDigest, DerivedVariantVersionId, PersistenceArtifact,
    Validated,
};

use crate::error::AppError;
use crate::qc::sha256_file;

pub fn promote_staged_artifact(
    staged_path: &Path,
    expected_staged_sha256: &str,
    durable_dir: &Path,
    bound_derived_variant_version_id: DerivedVariantVersionId,
    producer_id: BackendExecutionContextId,
) -> Result<(Validated<PersistenceArtifact>, PathBuf), AppError> {
    if !staged_path.is_file() {
        return Err(AppError::Catalog(
            "staged candidate artifact is missing; file existence is not success".into(),
        ));
    }
    let staged_sha = sha256_file(staged_path)?;
    if staged_sha != expected_staged_sha256 {
        return Err(AppError::Catalog(format!(
            "staged SHA {staged_sha} does not equal WorkerResult staged digest {expected_staged_sha256}"
        )));
    }
    let artifact = PersistenceArtifact::new(
        bound_derived_variant_version_id,
        ContentDigest::parse(&staged_sha)?,
        fs::metadata(staged_path)?.len(),
        "application/x-blender",
        producer_id,
    )?;
    let dest_dir = durable_dir
        .join(artifact.id().canonical())
        .join(artifact.instance_id().canonical());
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("derived_result.blend");
    fs::copy(staged_path, &dest)?;
    let durable_sha = sha256_file(&dest)?;
    if durable_sha != staged_sha {
        let _ = fs::remove_file(&dest);
        return Err(AppError::Catalog(format!(
            "durable persisted SHA {durable_sha} does not equal staged SHA {staged_sha}"
        )));
    }
    Ok((Validated::certify(artifact)?, dest))
}
