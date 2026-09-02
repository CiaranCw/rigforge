//! SQLite Asset Catalog. Consumes `Validated<T>` only.

use std::path::{Path, PathBuf};

use rigforge_domain::{
    ingest_validated, to_json, validate_job_inputs, validate_publication_lineage, BackendExecutionContext,
    BoneMappingVersion, CharacterAsset, CharacterAssetVersion, CompatibilityResult, CompatibilitySummary,
    DerivedVariant, DomainRecord, JobSpec, Lifecycle, MotionAsset, MotionAssetVersion,
    PersistenceArtifact, PersistenceVerification, PreviewArtifact, ProductKind, ProductVersionStore,
    RecordType, RetargetPolicyVersion, SourceArtifactEvidence, SourceSkeletonReference, Validated,
    WorkerResult,
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde_json::Value;
use std::fmt;

use crate::dispatch::{filesystem_location, ResolvedSourceInput, WorkerDispatchRequest};
use crate::error::AppError;
use crate::migrate::{apply_migrations, now_ms};
use crate::orchestration::{JobRun, JobRunState};
use crate::queries::AssetListItem;
use crate::worker::{
    DispatchReceipt, TerminalOutcome, WorkerCompletionPort, WorkerFailureClass, WorkerPort,
};

pub use crate::migrate::{DB_SCHEMA_NAME, DB_SCHEMA_VERSION};

/// Catalog-adjacent dispatch intent files. Not Product identity.
pub const DISPATCH_INTENT_DIRNAME: &str = "rigforge-dispatch-intents";

/// Catalog-side location evidence. Paths here are not Product identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogLocationEvidence {
    pub product_id: String,
    pub instance_id: String,
    pub location_kind: String,
    pub location_value: String,
    pub observed_at: i64,
    pub note: Option<String>,
}

struct RecordMeta {
    product_id: String,
    instance_id: String,
    record_type: String,
    logical_id: Option<String>,
    lifecycle: Option<String>,
    display_name: Option<String>,
    domain_schema_version: i64,
}

pub struct SqliteCatalog {
    conn: Connection,
    path: Option<PathBuf>,
    artifact_root: PathBuf,
    preview_root: PathBuf,
    #[cfg(test)]
    fail_candidate_after_writes: bool,
}

impl fmt::Debug for SqliteCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteCatalog")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl SqliteCatalog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(AppError::MissingDirectory(parent.display().to_string()));
            }
        }
        let artifact_root = match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join("rigforge-artifacts"),
            _ => PathBuf::from("rigforge-artifacts"),
        };
        let preview_root = match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join("rigforge-previews"),
            _ => PathBuf::from("rigforge-previews"),
        };
        std::fs::create_dir_all(&artifact_root)?;
        std::fs::create_dir_all(&preview_root)?;
        let conn = Connection::open(path)?;
        let mut catalog = Self {
            conn,
            path: Some(path.to_path_buf()),
            artifact_root,
            preview_root,
            #[cfg(test)]
            fail_candidate_after_writes: false,
        };
        catalog.configure()?;
        catalog.reconcile_interrupted_runtime()?;
        Ok(catalog)
    }

    pub fn open_in_memory() -> Result<Self, AppError> {
        let artifact_root = std::env::temp_dir().join(format!(
            "rigforge-artifacts-{}",
            uuid::Uuid::now_v7()
        ));
        let preview_root = std::env::temp_dir().join(format!(
            "rigforge-previews-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&artifact_root)?;
        std::fs::create_dir_all(&preview_root)?;
        let mut catalog = Self {
            conn: Connection::open_in_memory()?,
            path: None,
            artifact_root,
            preview_root,
            #[cfg(test)]
            fail_candidate_after_writes: false,
        };
        catalog.configure()?;
        catalog.reconcile_interrupted_runtime()?;
        Ok(catalog)
    }

    pub fn artifact_root(&self) -> &Path {
        &self.artifact_root
    }

    pub fn preview_root(&self) -> &Path {
        &self.preview_root
    }

    pub fn durable_artifact_path(
        &self,
        artifact: &PersistenceArtifact,
    ) -> PathBuf {
        self.artifact_root
            .join(artifact.id().canonical())
            .join(artifact.instance_id().canonical())
            .join("derived_result.blend")
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn sidecar_dir(&self) -> PathBuf {
        match &self.path {
            Some(path) => match path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
                _ => PathBuf::from("."),
            },
            None => self.artifact_root.clone(),
        }
    }

    pub fn dispatch_intent_dir(&self) -> PathBuf {
        self.sidecar_dir().join(DISPATCH_INTENT_DIRNAME)
    }

    fn dispatch_intent_path(&self, run_id: &str) -> PathBuf {
        self.dispatch_intent_dir()
            .join(format!("{run_id}.intent.json"))
    }

    fn write_dispatch_intent(&self, run_id: &str, attempt_id: &str) -> Result<(), AppError> {
        std::fs::create_dir_all(self.dispatch_intent_dir())?;
        let body = serde_json::json!({
            "run_id": run_id,
            "attempt_id": attempt_id,
            "written_at_ms": now_ms(),
        });
        let bytes = serde_json::to_vec_pretty(&body).map_err(|err| {
            AppError::Catalog(format!("dispatch intent serialize failed: {err}"))
        })?;
        std::fs::write(self.dispatch_intent_path(run_id), bytes)?;
        Ok(())
    }

    fn clear_dispatch_intent(&self, run_id: &str) {
        let _ = std::fs::remove_file(self.dispatch_intent_path(run_id));
    }

    /// Fail closed leftover DISPATCHABLE+intent and RUNNING-without-handle state.
    /// Does not publish and does not attach to orphan processes.
    pub fn reconcile_interrupted_runtime(&mut self) -> Result<Vec<JobRun>, AppError> {
        let mut changed = Vec::new();
        for run in self.list_job_runs()? {
            let has_intent = self.dispatch_intent_path(&run.run_id).is_file();
            match run.state {
                JobRunState::Dispatchable if has_intent => {
                    let failed = self.fail_dispatch(
                        &run.run_id,
                        "interrupted after dispatch intent; RUNNING was not durable; spawn-to-RUNNING crash window failed closed",
                    )?;
                    self.clear_dispatch_intent(&run.run_id);
                    changed.push(failed);
                }
                JobRunState::Running => {
                    let failed = self.complete_terminal_failure(
                        &run.run_id,
                        "application restarted while JobRun was RUNNING; in-process worker handle was not retained; Product publication remains blocked",
                        None,
                    )?;
                    self.clear_dispatch_intent(&run.run_id);
                    changed.push(failed);
                }
                _ if has_intent => self.clear_dispatch_intent(&run.run_id),
                _ => {}
            }
        }
        Ok(changed)
    }

    pub fn db_schema_version(&self) -> Result<i32, AppError> {
        let version: i32 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(version)
    }

    pub fn schema_name(&self) -> Result<String, AppError> {
        let name: Option<String> = self
            .conn
            .query_row(
                "SELECT name FROM schema_migrations WHERE version = ?1",
                [DB_SCHEMA_VERSION],
                |row| row.get(0),
            )
            .optional()?;
        Ok(name.unwrap_or_else(|| crate::migrate::DB_SCHEMA_NAME.to_string()))
    }

    fn configure(&mut self) -> Result<(), AppError> {
        apply_migrations(&mut self.conn)
    }

    /// Internal SQLite transaction helper. Not part of the public Catalog API.
    /// Ordinary production callers must not receive a mutable `rusqlite::Transaction`.
    fn in_transaction<F, T>(&mut self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Transaction<'_>) -> Result<T, AppError>,
    {
        let tx = self.conn.transaction()?;
        match f(&tx) {
            Ok(value) => {
                tx.commit()?;
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub fn put_validated<T: DomainRecord>(
        &mut self,
        record: &Validated<T>,
    ) -> Result<(), AppError> {
        self.in_transaction(|tx| {
            reject_public_authority_bypass(&*tx, record)?;
            put_validated_on(&*tx, record)
        })
    }

    pub fn put_validated_pair<A: DomainRecord, B: DomainRecord>(
        &mut self,
        first: &Validated<A>,
        second: &Validated<B>,
    ) -> Result<(), AppError> {
        self.in_transaction(|tx| {
            reject_public_authority_bypass(&*tx, first)?;
            reject_public_authority_bypass(&*tx, second)?;
            put_validated_on(&*tx, first)?;
            put_validated_on(&*tx, second)?;
            Ok(())
        })
    }

    #[cfg(test)]
    pub(crate) fn put_trusted_qc_report(
        &mut self,
        report: &Validated<rigforge_domain::QcReport>,
    ) -> Result<(), AppError> {
        self.in_transaction(|tx| put_validated_on(&*tx, report))
    }

    #[cfg(test)]
    pub(crate) fn put_trusted_persistence_verification(
        &mut self,
        verification: &Validated<PersistenceVerification>,
    ) -> Result<(), AppError> {
        self.in_transaction(|tx| put_validated_on(&*tx, verification))
    }

    pub(crate) fn persist_trusted_qc_binding(
        &mut self,
        report: &Validated<rigforge_domain::QcReport>,
    ) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
        self.in_transaction(|tx| persist_trusted_qc_on(&*tx, report))
    }

    pub(crate) fn persist_trusted_verification_binding(
        &mut self,
        verification: &Validated<PersistenceVerification>,
    ) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
        self.in_transaction(|tx| persist_trusted_verification_on(&*tx, verification))
    }

    pub(crate) fn find_derived_version_id_for_worker_result(
        &self,
        worker_result_id: &str,
    ) -> Result<Option<String>, AppError> {
        find_derived_version_id_for_worker_result_on(&self.conn, worker_result_id)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn seed_succeeded_job_run_for_tests(
        &mut self,
        job_spec_id: &str,
        worker_result_id: &str,
    ) -> Result<JobRun, AppError> {
        let now = now_ms();
        let run = JobRun {
            run_id: uuid::Uuid::now_v7().hyphenated().to_string(),
            job_spec_id: job_spec_id.to_string(),
            state: JobRunState::Succeeded,
            attempt_id: "test-seed-attempt".into(),
            worker_execution_ref: Some("test-seed-ref".into()),
            failure_reason: None,
            worker_result_id: Some(worker_result_id.to_string()),
            created_at: now,
            updated_at: now,
        };
        self.conn.execute(
            "INSERT INTO job_runs (
                run_id, job_spec_id, state, attempt_id, worker_execution_ref,
                failure_reason, worker_result_id, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?7)",
            rusqlite::params![
                run.run_id,
                run.job_spec_id,
                run.state.as_db_str(),
                run.attempt_id,
                run.worker_execution_ref,
                run.worker_result_id,
                now,
            ],
        )?;
        Ok(run)
    }

    #[cfg(test)]
    pub(crate) fn replace_record_payload_for_tests(
        &mut self,
        product_id: &str,
        payload_json: &str,
    ) -> Result<(), AppError> {
        let n = self.conn.execute(
            "UPDATE records SET payload_json = ?1, updated_at = ?2 WHERE product_id = ?3",
            rusqlite::params![payload_json, now_ms(), product_id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound {
                what: "catalog record".into(),
                id: product_id.to_string(),
            });
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fail_next_candidate_transaction(&mut self) {
        self.fail_candidate_after_writes = true;
    }

    pub(crate) fn persist_candidate_graph(
        &mut self,
        run_id: &str,
        version: &Validated<rigforge_domain::DerivedVariantVersion>,
        artifact: &Validated<PersistenceArtifact>,
        location: CatalogLocationEvidence,
    ) -> Result<
        (
            Validated<DerivedVariant>,
            Validated<rigforge_domain::DerivedVariantVersion>,
        ),
        AppError,
    > {
        #[cfg(test)]
        let fail_after_writes = {
            let fail = self.fail_candidate_after_writes;
            self.fail_candidate_after_writes = false;
            fail
        };
        self.in_transaction(|tx| {
            let bound = persist_candidate_on(
                &*tx,
                run_id,
                version,
                artifact,
                &location,
            )?;
            #[cfg(test)]
            if fail_after_writes {
                return Err(AppError::Catalog(
                    "forced candidate transaction failure".into(),
                ));
            }
            Ok(bound)
        })
    }

    pub fn load_validated<T: DomainRecord>(
        &self,
        product_id: &str,
    ) -> Result<Validated<T>, AppError> {
        load_validated_on(&self.conn, product_id)
    }

    pub fn list_version_ids(&self, logical_id: &str) -> Result<Vec<String>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT product_id FROM records
             WHERE logical_id = ?1
               AND record_type IN (
                    'character_asset_version',
                    'motion_asset_version',
                    'bone_mapping_version',
                    'retarget_policy_version',
                    'derived_variant_version',
                    'skeleton_summary',
                    'compatibility_result'
               )
             ORDER BY created_at ASC, product_id ASC",
        )?;
        let ids = stmt
            .query_map([logical_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn list_assets(&self, record_type: RecordType) -> Result<Vec<AssetListItem>, AppError> {
        let key = record_type_key(record_type);
        let mut stmt = self.conn.prepare(
            "SELECT product_id, display_name, payload_json, record_type
             FROM records
             WHERE record_type = ?1
             ORDER BY display_name ASC, product_id ASC",
        )?;
        let rows = stmt.query_map([key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (product_id, display_name, payload, record_type) = row?;
            let value: Value = serde_json::from_str(&payload).unwrap_or(Value::Null);
            items.push(AssetListItem {
                logical_id: product_id,
                display_name: display_name.unwrap_or_default(),
                record_type,
                published_version_id: json_string(&value, "published_version_id"),
                draft_version_id: json_string(&value, "draft_version_id"),
            });
        }
        Ok(items)
    }

    pub fn load_character_asset(&self, id: &str) -> Result<Validated<CharacterAsset>, AppError> {
        self.load_validated(id)
    }
    pub fn load_character_version(
        &self,
        id: &str,
    ) -> Result<Validated<CharacterAssetVersion>, AppError> {
        self.load_validated(id)
    }
    pub fn load_motion_asset(&self, id: &str) -> Result<Validated<MotionAsset>, AppError> {
        self.load_validated(id)
    }
    pub fn load_motion_version(&self, id: &str) -> Result<Validated<MotionAssetVersion>, AppError> {
        self.load_validated(id)
    }
    pub fn load_source_skeleton(
        &self,
        id: &str,
    ) -> Result<Validated<SourceSkeletonReference>, AppError> {
        self.load_validated(id)
    }
    pub fn load_mapping_version(
        &self,
        id: &str,
    ) -> Result<Validated<BoneMappingVersion>, AppError> {
        self.load_validated(id)
    }
    pub fn load_policy_version(
        &self,
        id: &str,
    ) -> Result<Validated<RetargetPolicyVersion>, AppError> {
        self.load_validated(id)
    }
    pub fn load_job_spec(&self, id: &str) -> Result<Validated<JobSpec>, AppError> {
        self.load_validated(id)
    }
    pub fn load_worker_result(&self, id: &str) -> Result<Validated<WorkerResult>, AppError> {
        self.load_validated(id)
    }
    pub fn load_derived_variant(&self, id: &str) -> Result<Validated<DerivedVariant>, AppError> {
        self.load_validated(id)
    }
    pub fn load_artifact_metadata(
        &self,
        id: &str,
    ) -> Result<Validated<PersistenceArtifact>, AppError> {
        self.load_validated(id)
    }
    pub fn load_persistence_verification(
        &self,
        id: &str,
    ) -> Result<Validated<PersistenceVerification>, AppError> {
        self.load_validated(id)
    }

    pub fn load_skeleton_summary(
        &self,
        id: &str,
    ) -> Result<Validated<rigforge_domain::SkeletonSummary>, AppError> {
        self.load_validated(id)
    }

    pub fn load_compatibility_result(
        &self,
        id: &str,
    ) -> Result<Validated<rigforge_domain::CompatibilityResult>, AppError> {
        self.load_validated(id)
    }

    pub fn load_qc_report(
        &self,
        id: &str,
    ) -> Result<Validated<rigforge_domain::QcReport>, AppError> {
        self.load_validated(id)
    }

    pub fn load_derived_variant_version(
        &self,
        id: &str,
    ) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
        self.load_validated(id)
    }

    pub fn load_preview_artifact(&self, id: &str) -> Result<Validated<PreviewArtifact>, AppError> {
        self.load_validated(id)
    }

    pub fn persist_preview_artifact(
        &mut self,
        artifact: &PreviewArtifact,
        payload_path: &Path,
    ) -> Result<Validated<PreviewArtifact>, AppError> {
        self.in_transaction(|tx| persist_preview_on(&*tx, artifact, payload_path))
    }

    pub fn latest_preview_for_exact_version(
        &self,
        kind: ProductKind,
        version_id: &str,
    ) -> Result<Option<Validated<PreviewArtifact>>, AppError> {
        let field = match kind {
            ProductKind::CharacterAssetVersion => "bound_character_version_id",
            ProductKind::MotionAssetVersion => "bound_motion_version_id",
            ProductKind::DerivedVariantVersion => "bound_derived_variant_version_id",
        };
        let sql = format!(
            "SELECT product_id FROM records
             WHERE record_type = 'preview_artifact'
               AND json_extract(payload_json, '$.{field}') = ?1
             ORDER BY created_at DESC, product_id DESC
             LIMIT 1"
        );
        let id: Option<String> = self
            .conn
            .query_row(&sql, [version_id], |row| row.get(0))
            .optional()?;
        match id {
            Some(id) => Ok(Some(self.load_preview_artifact(&id)?)),
            None => Ok(None),
        }
    }

    pub fn delete_preview_artifact(&mut self, preview_id: &str) -> Result<(), AppError> {
        let loaded = self.load_preview_artifact(preview_id)?;
        let location = loaded
            .as_record()
            .location()
            .map(|l| PathBuf::from(l.value()));
        self.conn.execute(
            "DELETE FROM records WHERE product_id = ?1 AND record_type = 'preview_artifact'",
            [preview_id],
        )?;
        self.conn.execute(
            "DELETE FROM payload_locations WHERE product_id = ?1",
            [preview_id],
        )?;
        if let Some(path) = location {
            let _ = std::fs::remove_file(&path);
            if let Some(parent) = path.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        }
        Ok(())
    }

    pub fn publish_derived_variant_transaction(
        &mut self,
        derived_version_id: &str,
    ) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
        self.in_transaction(|tx| publish_derived_on(&*tx, derived_version_id))
    }

    pub fn load_bone_mapping(
        &self,
        id: &str,
    ) -> Result<Validated<rigforge_domain::BoneMapping>, AppError> {
        self.load_validated(id)
    }

    pub fn list_skeleton_summaries_for_character(
        &self,
        character_version_id: &str,
    ) -> Result<Vec<String>, AppError> {
        self.list_summaries_by_json_field(
            "subject_character_version_id",
            character_version_id,
        )
    }

    pub fn list_skeleton_summaries_for_source_skeleton(
        &self,
        source_skeleton_id: &str,
    ) -> Result<Vec<String>, AppError> {
        self.list_summaries_by_json_field("subject_source_skeleton_ref_id", source_skeleton_id)
    }

    fn list_summaries_by_json_field(
        &self,
        field: &str,
        value: &str,
    ) -> Result<Vec<String>, AppError> {
        let sql = format!(
            "SELECT product_id FROM records
             WHERE record_type = 'skeleton_summary'
               AND json_extract(payload_json, '$.{field}') = ?1
             ORDER BY created_at DESC, product_id DESC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let ids = stmt
            .query_map([value], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn latest_compatibility_for_exact_set(
        &self,
        character_version_id: &str,
        motion_version_id: &str,
        mapping_version_id: &str,
        policy_version_id: &str,
    ) -> Result<Option<Validated<rigforge_domain::CompatibilityResult>>, AppError> {
        let row: Option<String> = self
            .conn
            .query_row(
                "SELECT product_id FROM records
                 WHERE record_type = 'compatibility_result'
                   AND json_extract(payload_json, '$.character_version_id') = ?1
                   AND json_extract(payload_json, '$.motion_version_id') = ?2
                   AND json_extract(payload_json, '$.mapping_version_id') = ?3
                   AND json_extract(payload_json, '$.policy_version_id') = ?4
                 ORDER BY created_at DESC, product_id DESC
                 LIMIT 1",
                rusqlite::params![
                    character_version_id,
                    motion_version_id,
                    mapping_version_id,
                    policy_version_id
                ],
                |row| row.get(0),
            )
            .optional()?;
        match row {
            Some(id) => Ok(Some(self.load_compatibility_result(&id)?)),
            None => Ok(None),
        }
    }

    pub fn list_artifact_instances(
        &self,
        artifact_id: &str,
    ) -> Result<Vec<Validated<PersistenceArtifact>>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT payload_json FROM records
             WHERE product_id = ?1 AND record_type = 'persistence_artifact'
             ORDER BY created_at ASC, instance_id ASC",
        )?;
        let payloads = stmt
            .query_map([artifact_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        payloads
            .iter()
            .map(|json| ingest_validated::<PersistenceArtifact>(json).map_err(AppError::from))
            .collect()
    }

    pub fn record_payload_location(
        &mut self,
        evidence: CatalogLocationEvidence,
    ) -> Result<(), AppError> {
        record_payload_location_on(&self.conn, &evidence)
    }

    pub fn list_payload_locations(
        &self,
        product_id: &str,
    ) -> Result<Vec<CatalogLocationEvidence>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT product_id, instance_id, location_kind, location_value, observed_at, note
             FROM payload_locations
             WHERE product_id = ?1
             ORDER BY observed_at ASC, location_rowid ASC",
        )?;
        let rows = stmt.query_map([product_id], |row| {
            Ok(CatalogLocationEvidence {
                product_id: row.get(0)?,
                instance_id: row.get(1)?,
                location_kind: row.get(2)?,
                location_value: row.get(3)?,
                observed_at: row.get(4)?,
                note: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    /// Internal SQLite rowid. Never a Product ID. Test-only; not a production API.
    #[cfg(test)]
    pub(crate) fn internal_rowid_for_tests(&self, product_id: &str) -> Result<i64, AppError> {
        self.conn
            .query_row(
                "SELECT rowid FROM records WHERE product_id = ?1 LIMIT 1",
                [product_id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::NotFound {
                what: "rowid".into(),
                id: product_id.to_string(),
            })
    }

    /// Insert Domain-shaped JSON without validation. Used to prove fail-closed load.
    /// Test-only: must not exist on the ordinary production Catalog surface.
    #[cfg(test)]
    pub(crate) fn insert_unvalidated_payload_for_tests(
        &mut self,
        product_id: &str,
        record_type: &str,
        payload_json: &str,
        domain_schema_version: i64,
    ) -> Result<(), AppError> {
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO records (
                product_id, instance_id, record_type, logical_id, lifecycle, display_name,
                domain_schema_version, payload_json, created_at, updated_at
             ) VALUES (?1, '', ?2, NULL, NULL, NULL, ?3, ?4, ?5, ?5)",
            rusqlite::params![
                product_id,
                record_type,
                domain_schema_version,
                payload_json,
                now
            ],
        )?;
        Ok(())
    }

    pub fn enqueue_job(&mut self, spec: Validated<JobSpec>) -> Result<JobRun, AppError> {
        let spec_id = spec.as_record().id().canonical();
        self.in_transaction(|tx| {
            validate_job_graph_on(&*tx, spec.as_record())?;
            put_validated_on(&*tx, &spec)?;
            insert_job_run_on(&*tx, &spec_id, JobRunState::Queued, None)
        })
    }

    pub fn load_job_run(&self, run_id: &str) -> Result<JobRun, AppError> {
        load_job_run_on(&self.conn, run_id)
    }

    pub fn list_job_runs(&self) -> Result<Vec<JobRun>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT run_id, job_spec_id, state, attempt_id, worker_execution_ref,
                    failure_reason, worker_result_id, created_at, updated_at
             FROM job_runs
             ORDER BY created_at ASC, run_id ASC",
        )?;
        let rows = stmt.query_map([], job_run_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn mark_dispatchable(&mut self, run_id: &str) -> Result<JobRun, AppError> {
        self.transition(run_id, JobRunState::Dispatchable)
    }

    pub fn cancel(&mut self, run_id: &str) -> Result<JobRun, AppError> {
        self.transition(run_id, JobRunState::Cancelled)
    }

    pub fn dispatch<W: WorkerPort + ?Sized>(
        &mut self,
        run_id: &str,
        worker: &mut W,
    ) -> Result<(JobRun, DispatchReceipt), AppError> {
        let run = self.load_job_run(run_id)?;
        if run.state != JobRunState::Dispatchable {
            return Err(AppError::InvalidTransition {
                from: run.state,
                to: JobRunState::Running,
            });
        }
        let spec = self.load_job_spec(&run.job_spec_id)?;
        if let Err(err) = validate_job_graph_on(&self.conn, spec.as_record()) {
            self.fail_dispatch(run_id, &err.to_string())?;
            return Err(err);
        }
        let request = match self.assemble_worker_dispatch_request(&run) {
            Ok(request) => request,
            Err(err) => {
                self.fail_dispatch(run_id, &err.to_string())?;
                return Err(err);
            }
        };
        self.write_dispatch_intent(run_id, &run.attempt_id)?;
        match worker.dispatch_resolved(&request) {
            Ok(receipt) => {
                if receipt.attempt_id != run.attempt_id {
                    let reason =
                        "worker receipt attempt_id does not match orchestrator attempt_id";
                    self.clear_dispatch_intent(run_id);
                    self.fail_dispatch(run_id, reason)?;
                    return Err(AppError::Worker(reason.into()));
                }
                let now = now_ms();
                let updated = self.conn.execute(
                    "UPDATE job_runs
                     SET state = ?1, worker_execution_ref = ?2, updated_at = ?3
                     WHERE run_id = ?4",
                    rusqlite::params![
                        JobRunState::Running.as_db_str(),
                        receipt.worker_execution_ref,
                        now,
                        run_id
                    ],
                );
                match updated {
                    Ok(_) => {
                        self.clear_dispatch_intent(run_id);
                        Ok((self.load_job_run(run_id)?, receipt))
                    }
                    Err(err) => {
                        self.fail_dispatch(run_id, &err.to_string())?;
                        self.clear_dispatch_intent(run_id);
                        Err(err.into())
                    }
                }
            }
            Err(err) => {
                self.clear_dispatch_intent(run_id);
                self.fail_dispatch(run_id, &err.to_string())?;
                Err(err)
            }
        }
    }

    /// Test and recovery helper. Ordinary runtime completion is [`Self::collect`].
    /// Successful completion still requires exact JobSpec, attempt_id, and
    /// worker_execution_ref correlation, and one WorkerResult ID cannot
    /// authorize two JobRuns.
    pub fn complete_success(
        &mut self,
        run_id: &str,
        worker_result: &Validated<WorkerResult>,
    ) -> Result<JobRun, AppError> {
        self.in_transaction(|tx| {
            let run = load_job_run_on(&*tx, run_id)?;
            let result = worker_result.as_record();
            bind_successful_worker_result(&*tx, &run, result)?;
            put_validated_on(&*tx, worker_result)?;
            transition_on(
                &*tx,
                run_id,
                JobRunState::Succeeded,
                None,
                Some(result.id().canonical()),
            )
        })
    }

    pub fn complete_failure(
        &mut self,
        run_id: &str,
        reason: impl Into<String>,
    ) -> Result<JobRun, AppError> {
        self.complete_terminal_failure(run_id, reason, None)
    }

    /// Persist a launched terminal failure. Optional failed `WorkerResult` is
    /// stored before the JobRun becomes FAILED. Launch failures omit it.
    pub fn complete_terminal_failure(
        &mut self,
        run_id: &str,
        reason: impl Into<String>,
        worker_result: Option<&Validated<WorkerResult>>,
    ) -> Result<JobRun, AppError> {
        let reason = reason.into();
        self.in_transaction(|tx| {
            let run = load_job_run_on(&*tx, run_id)?;
            if let Some(result) = worker_result {
                match bind_failed_worker_result(&*tx, &run, result.as_record()) {
                    Ok(()) => {
                        put_validated_on(&*tx, result)?;
                        transition_on(
                            &*tx,
                            run_id,
                            JobRunState::Failed,
                            Some(reason),
                            Some(result.as_record().id().canonical()),
                        )
                    }
                    Err(reject) => {
                        let combined = format!(
                            "{reason}; evidence rejected: {reject}"
                        );
                        transition_on(
                            &*tx,
                            run_id,
                            JobRunState::Failed,
                            Some(combined),
                            None,
                        )
                    }
                }
            } else {
                transition_on(&*tx, run_id, JobRunState::Failed, Some(reason), None)
            }
        })
    }

    /// Collect a launched attempt and write the matching terminal JobRun state.
    pub fn collect<C: WorkerCompletionPort + ?Sized>(
        &mut self,
        run_id: &str,
        worker: &mut C,
    ) -> Result<(JobRun, TerminalOutcome), AppError> {
        let run = self.load_job_run(run_id)?;
        if run.state != JobRunState::Running {
            return Err(AppError::InvalidTransition {
                from: run.state,
                to: JobRunState::Succeeded,
            });
        }
        let worker_execution_ref = run.worker_execution_ref.clone().ok_or_else(|| {
            AppError::Orchestration("RUNNING JobRun is missing worker_execution_ref".into())
        })?;
        let receipt = DispatchReceipt {
            attempt_id: run.attempt_id.clone(),
            worker_execution_ref,
        };
        let outcome = match worker.collect(&receipt) {
            Ok(outcome) => outcome,
            Err(err) => {
                let reason = format!("terminal collection failure: {err}");
                self.complete_terminal_failure(run_id, reason.clone(), None)?;
                return Ok((
                    self.load_job_run(run_id)?,
                    TerminalOutcome::Failed {
                        class: WorkerFailureClass::Other("collection_failure".into()),
                        reason,
                        worker_result: None,
                    },
                ));
            }
        };
        match &outcome {
            TerminalOutcome::Success(result) => {
                if let Err(err) = self.complete_success(run_id, result) {
                    let reason = format!("terminal collection failure: {err}");
                    self.complete_terminal_failure(run_id, reason, None)?;
                    return Err(err);
                }
            }
            TerminalOutcome::Failed {
                reason,
                worker_result,
                ..
            } => {
                self.complete_terminal_failure(run_id, reason.clone(), worker_result.as_ref())?;
            }
        }
        Ok((self.load_job_run(run_id)?, outcome))
    }

    /// Exact Catalog projection used at dispatch. Never resolves latest/current.
    pub fn assemble_worker_dispatch_request(
        &self,
        run: &JobRun,
    ) -> Result<WorkerDispatchRequest, AppError> {
        let spec = self.load_job_spec(&run.job_spec_id)?;
        let record = spec.as_record();
        let character = self.load_character_version(&record.character_version_id().canonical())?;
        let motion = self.load_motion_version(&record.motion_version_id().canonical())?;
        let mapping = self.load_mapping_version(&record.mapping_version_id().canonical())?;
        let policy = self.load_policy_version(&record.policy_version_id().canonical())?;
        let skeleton = self.load_source_skeleton(&record.source_skeleton_ref_id().canonical())?;
        let character_src = resolve_source_input(
            self,
            &record.character_version_id().canonical(),
            character.as_record().source(),
        )?;
        let motion_src = resolve_source_input(
            self,
            &record.motion_version_id().canonical(),
            motion.as_record().source(),
        )?;
        WorkerDispatchRequest::new(
            spec,
            run.attempt_id.clone(),
            character_src,
            motion_src,
            motion.as_record(),
            skeleton.as_record().id().canonical(),
            mapping,
            policy,
        )
    }

    pub fn transition(&mut self, run_id: &str, next: JobRunState) -> Result<JobRun, AppError> {
        if next == JobRunState::Succeeded {
            return Err(AppError::Orchestration(
                "SUCCEEDED requires complete_success with a matching successful WorkerResult"
                    .into(),
            ));
        }
        self.in_transaction(|tx| transition_on(&*tx, run_id, next, None, None))
    }

    fn fail_dispatch(&mut self, run_id: &str, reason: &str) -> Result<JobRun, AppError> {
        self.in_transaction(|tx| {
            transition_on(
                &*tx,
                run_id,
                JobRunState::Failed,
                Some(reason.to_string()),
                None,
            )
        })
    }
}

impl ProductVersionStore for SqliteCatalog {
    fn store_immutable_version<T: DomainRecord>(
        &mut self,
        record: Validated<T>,
    ) -> Result<(), rigforge_domain::DomainError> {
        self.put_validated(&record).map_err(AppError::into_domain)
    }

    fn load_exact_version<T: DomainRecord>(
        &self,
        version_id: &str,
    ) -> Result<Validated<T>, rigforge_domain::DomainError> {
        self.load_validated(version_id).map_err(AppError::into_domain)
    }

    fn resolve_logical_object_versions(
        &self,
        logical_id: &str,
    ) -> Result<Vec<String>, rigforge_domain::DomainError> {
        self.list_version_ids(logical_id)
            .map_err(AppError::into_domain)
    }

    fn store_artifact_metadata<T: DomainRecord>(
        &mut self,
        record: Validated<T>,
    ) -> Result<(), rigforge_domain::DomainError> {
        self.put_validated(&record).map_err(AppError::into_domain)
    }
}

fn validate_compatibility_graph_on(
    conn: &Connection,
    result: &CompatibilityResult,
) -> Result<(), AppError> {
    let character = load_validated_on::<CharacterAssetVersion>(
        conn,
        &result.character_version_id().canonical(),
    )?;
    let motion =
        load_validated_on::<MotionAssetVersion>(conn, &result.motion_version_id().canonical())?;
    let mapping =
        load_validated_on::<BoneMappingVersion>(conn, &result.mapping_version_id().canonical())?;
    let policy =
        load_validated_on::<RetargetPolicyVersion>(conn, &result.policy_version_id().canonical())?;

    if character.as_record().id() != result.character_version_id() {
        return Err(AppError::Catalog(
            "CompatibilityResult.character_version_id must equal the referenced CharacterAssetVersion.id".into(),
        ));
    }
    if motion.as_record().id() != result.motion_version_id() {
        return Err(AppError::Catalog(
            "CompatibilityResult.motion_version_id must equal the referenced MotionAssetVersion.id"
                .into(),
        ));
    }
    if mapping.as_record().id() != result.mapping_version_id() {
        return Err(AppError::Catalog(
            "CompatibilityResult.mapping_version_id must equal the referenced BoneMappingVersion.id"
                .into(),
        ));
    }
    if policy.as_record().id() != result.policy_version_id() {
        return Err(AppError::Catalog(
            "CompatibilityResult.policy_version_id must equal the referenced RetargetPolicyVersion.id".into(),
        ));
    }
    if mapping.as_record().target_character_version_id() != result.character_version_id() {
        return Err(AppError::Catalog(
            "CompatibilityResult character does not match BoneMappingVersion.target_character_version_id".into(),
        ));
    }
    if matches!(
        result.summary(),
        CompatibilitySummary::Ready | CompatibilitySummary::ReadyWithWarnings
    ) {
        if mapping.as_record().lifecycle() != Lifecycle::Published {
            return Err(AppError::Catalog(
                "Ready/ReadyWithWarnings CompatibilityResult requires Published BoneMappingVersion"
                    .into(),
            ));
        }
        if mapping.as_record().source_skeleton_ref_id() != motion.as_record().source_skeleton_ref_id()
        {
            return Err(AppError::Catalog(
                "Ready/ReadyWithWarnings CompatibilityResult requires Mapping and Motion to share SourceSkeletonReference".into(),
            ));
        }
    }
    Ok(())
}

fn validate_job_graph_on(conn: &Connection, spec: &JobSpec) -> Result<(), AppError> {
    let character = load_validated_on::<CharacterAssetVersion>(
        conn,
        &spec.character_version_id().canonical(),
    )?;
    let motion =
        load_validated_on::<MotionAssetVersion>(conn, &spec.motion_version_id().canonical())?;
    let source_skeleton = load_validated_on::<SourceSkeletonReference>(
        conn,
        &spec.source_skeleton_ref_id().canonical(),
    )?;
    let mapping =
        load_validated_on::<BoneMappingVersion>(conn, &spec.mapping_version_id().canonical())?;
    let policy =
        load_validated_on::<RetargetPolicyVersion>(conn, &spec.policy_version_id().canonical())?;
    validate_job_inputs(
        character.as_record(),
        motion.as_record(),
        source_skeleton.as_record(),
        mapping.as_record(),
        policy.as_record(),
        spec,
    )
    .map_err(AppError::from)?;
    if let Some(compat_id) = spec.compatibility_result_id() {
        let result = load_validated_on::<CompatibilityResult>(conn, &compat_id.canonical())?;
        let rec = result.as_record();
        crate::transfer::validate_transfer_graph(
            rec,
            character.as_record(),
            motion.as_record(),
            mapping.as_record(),
            policy.as_record(),
        )?;
        let auth = crate::transfer::transfer_eligibility(
            rec,
            spec.compatibility_warnings_acknowledged(),
        )?;
        if !auth.eligible {
            return Err(AppError::Catalog(
                auth.denial_reason
                    .unwrap_or_else(|| "JobSpec CompatibilityResult cannot authorize Transfer".into()),
            ));
        }
        if rec.id() != compat_id {
            return Err(AppError::Catalog(
                "JobSpec.compatibility_result_id must equal the referenced CompatibilityResult".into(),
            ));
        }
    }
    if let Some(target_id) = spec.target_derived_variant_id() {
        let logical = load_validated_on::<DerivedVariant>(conn, &target_id.canonical())?;
        if logical.as_record().id() != target_id {
            return Err(AppError::Catalog(
                "JobSpec.target_derived_variant_id must equal the stored DerivedVariant".into(),
            ));
        }
    }
    Ok(())
}

fn bind_successful_worker_result(
    conn: &Connection,
    run: &JobRun,
    result: &WorkerResult,
) -> Result<(), AppError> {
    if result.job_spec_id().canonical() != run.job_spec_id {
        return Err(AppError::Orchestration(
            "WorkerResult.job_spec_id must equal JobRun.job_spec_id".into(),
        ));
    }
    if result.attempt_id() != run.attempt_id {
        return Err(AppError::Orchestration(
            "WorkerResult.attempt_id must equal JobRun.attempt_id".into(),
        ));
    }
    let expected_ref = run.worker_execution_ref.as_deref().ok_or_else(|| {
        AppError::Orchestration("RUNNING JobRun is missing worker_execution_ref".into())
    })?;
    if result.worker_execution_ref() != expected_ref {
        return Err(AppError::Orchestration(
            "WorkerResult.worker_execution_ref must equal JobRun.worker_execution_ref".into(),
        ));
    }
    if !result.worker_success() {
        return Err(AppError::Orchestration(
            "SUCCEEDED requires WorkerResult.worker_success == true".into(),
        ));
    }
    reject_worker_result_reuse(conn, &run.run_id, &result.id().canonical())
}

fn bind_failed_worker_result(
    conn: &Connection,
    run: &JobRun,
    result: &WorkerResult,
) -> Result<(), String> {
    if result.job_spec_id().canonical() != run.job_spec_id {
        return Err("WorkerResult.job_spec_id must equal JobRun.job_spec_id".into());
    }
    if result.attempt_id() != run.attempt_id {
        return Err("WorkerResult.attempt_id must equal JobRun.attempt_id".into());
    }
    match run.worker_execution_ref.as_deref() {
        Some(expected) if result.worker_execution_ref() == expected => {}
        Some(_) => {
            return Err(
                "WorkerResult.worker_execution_ref must equal JobRun.worker_execution_ref".into(),
            )
        }
        None => return Err("RUNNING JobRun is missing worker_execution_ref".into()),
    }
    if result.worker_success() {
        return Err(
            "terminal failure cannot persist worker_success == true; use complete_success".into(),
        );
    }
    if let Err(err) = reject_worker_result_reuse(conn, &run.run_id, &result.id().canonical()) {
        return Err(err.to_string());
    }
    Ok(())
}

fn reject_worker_result_reuse(
    conn: &Connection,
    run_id: &str,
    worker_result_id: &str,
) -> Result<(), AppError> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT run_id FROM job_runs WHERE worker_result_id = ?1 LIMIT 1",
            [worker_result_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(other) = existing {
        if other != run_id {
            return Err(AppError::Orchestration(format!(
                "WorkerResult {worker_result_id} already authorizes JobRun {other} and cannot complete {run_id}"
            )));
        }
    }
    Ok(())
}

fn validate_qc_graph_on(
    conn: &Connection,
    report: &rigforge_domain::QcReport,
) -> Result<(), AppError> {
    let Some(dv_id) = report.subject_derived_variant_version_id() else {
        return Ok(());
    };
    let version = load_validated_on::<rigforge_domain::DerivedVariantVersion>(
        conn,
        &dv_id.canonical(),
    )?;
    let worker_id = report.worker_result_id().ok_or_else(|| {
        AppError::Catalog("QcReport must bind an exact WorkerResult".into())
    })?;
    let worker = load_validated_on::<WorkerResult>(conn, &worker_id.canonical())?;
    let _policy = load_validated_on::<RetargetPolicyVersion>(
        conn,
        &report.policy_version_id().canonical(),
    )?;
    if version.as_record().id() != dv_id {
        return Err(AppError::Catalog(
            "QcReport subject is not the stored DerivedVariantVersion".into(),
        ));
    }
    if worker.as_record().id() != worker_id
        || version.as_record().worker_result_id() != worker_id
    {
        return Err(AppError::Catalog(
            "QcReport WorkerResult is not bound to this DerivedVariantVersion".into(),
        ));
    }
    if report.policy_version_id() != version.as_record().policy_version_id() {
        return Err(AppError::Catalog(
            "QcReport policy does not match DerivedVariantVersion".into(),
        ));
    }
    let Some(artifact_id) = report.evaluated_persistence_artifact_id() else {
        return Err(AppError::Catalog(
            "QcReport must evaluate an exact PersistenceArtifact".into(),
        ));
    };
    if version.as_record().persistence_artifact_id() != Some(artifact_id) {
        return Err(AppError::Catalog(
            "QcReport evaluated PersistenceArtifact is not bound to this DerivedVariantVersion"
                .into(),
        ));
    }
    let artifact = load_validated_on::<PersistenceArtifact>(conn, &artifact_id.canonical())?;
    if artifact.as_record().id() != artifact_id
        || report.evaluated_persistence_artifact_instance_id()
            != Some(artifact.as_record().instance_id())
        || report.evaluated_payload_digest() != Some(artifact.as_record().digest())
    {
        return Err(AppError::Catalog(
            "QcReport evaluated artifact instance/digest must match PersistenceArtifact".into(),
        ));
    }
    Ok(())
}

fn validate_persistence_artifact_graph_on(
    conn: &Connection,
    artifact: &PersistenceArtifact,
) -> Result<(), AppError> {
    let version = load_validated_on::<rigforge_domain::DerivedVariantVersion>(
        conn,
        &artifact.bound_derived_variant_version_id().canonical(),
    )?;
    if version.as_record().id() != artifact.bound_derived_variant_version_id() {
        return Err(AppError::Catalog(
            "PersistenceArtifact must bind an exact DerivedVariantVersion".into(),
        ));
    }
    Ok(())
}

fn validate_persistence_verification_graph_on(
    conn: &Connection,
    verification: &PersistenceVerification,
) -> Result<(), AppError> {
    let artifact = load_validated_on::<PersistenceArtifact>(
        conn,
        &verification.persistence_artifact_id().canonical(),
    )?;
    if artifact.as_record().instance_id() != verification.persistence_artifact_instance_id()
        || artifact.as_record().digest() != verification.payload_digest()
    {
        return Err(AppError::Catalog(
            "PersistenceVerification must bind the exact PersistenceArtifact instance and digest"
                .into(),
        ));
    }
    let version = load_validated_on::<rigforge_domain::DerivedVariantVersion>(
        conn,
        &verification.subject_derived_variant_version_id().canonical(),
    )?;
    if version.as_record().id() != verification.subject_derived_variant_version_id() {
        return Err(AppError::Catalog(
            "PersistenceVerification subject must be the exact DerivedVariantVersion".into(),
        ));
    }
    if artifact.as_record().bound_derived_variant_version_id()
        != verification.subject_derived_variant_version_id()
    {
        return Err(AppError::Catalog(
            "PersistenceArtifact.bound_derived_variant_version_id must equal PersistenceVerification.subject_derived_variant_version_id".into(),
        ));
    }
    if verification.producer_id() != artifact.as_record().producer_id() {
        return Err(AppError::Catalog(
            "PersistenceVerification.producer_id must equal PersistenceArtifact.producer_id".into(),
        ));
    }
    if artifact.as_record().producer_id() != version.as_record().backend_id() {
        return Err(AppError::Catalog(
            "PersistenceArtifact.producer_id must equal DerivedVariantVersion.backend_id".into(),
        ));
    }
    Ok(())
}

fn validate_published_derived_on(
    conn: &Connection,
    derived: &rigforge_domain::DerivedVariantVersion,
) -> Result<(), AppError> {
    rebuild_publication_evidence(conn, derived)?;
    Ok(())
}

fn succeeded_job_run_on(
    conn: &Connection,
    job_spec_id: &str,
    worker_result_id: &str,
) -> Result<JobRun, AppError> {
    let run_id: Option<String> = conn
        .query_row(
            "SELECT run_id FROM job_runs
             WHERE job_spec_id = ?1 AND worker_result_id = ?2 AND state = ?3
             LIMIT 1",
            rusqlite::params![job_spec_id, worker_result_id, JobRunState::Succeeded.as_db_str()],
            |row| row.get(0),
        )
        .optional()?;
    let run_id = run_id.ok_or_else(|| {
        AppError::Catalog(
            "publication requires JobRun SUCCEEDED bound to this JobSpec and WorkerResult".into(),
        )
    })?;
    load_job_run_on(conn, &run_id)
}

fn rebuild_publication_evidence<'a>(
    conn: &'a Connection,
    derived: &rigforge_domain::DerivedVariantVersion,
) -> Result<(), AppError> {
    let run = succeeded_job_run_on(
        conn,
        &derived.job_spec_id().canonical(),
        &derived.worker_result_id().canonical(),
    )?;
    if run.job_spec_id != derived.job_spec_id().canonical()
        || run.worker_result_id.as_deref() != Some(&derived.worker_result_id().canonical())
    {
        return Err(AppError::Catalog(
            "JobRun does not match DerivedVariantVersion JobSpec/WorkerResult".into(),
        ));
    }
    let character = load_validated_on::<CharacterAssetVersion>(
        conn,
        &derived.character_version_id().canonical(),
    )?;
    let motion =
        load_validated_on::<MotionAssetVersion>(conn, &derived.motion_version_id().canonical())?;
    let source = load_validated_on::<SourceSkeletonReference>(
        conn,
        &derived.source_skeleton_ref_id().canonical(),
    )?;
    let mapping =
        load_validated_on::<BoneMappingVersion>(conn, &derived.mapping_version_id().canonical())?;
    let policy =
        load_validated_on::<RetargetPolicyVersion>(conn, &derived.policy_version_id().canonical())?;
    let job = load_validated_on::<JobSpec>(conn, &derived.job_spec_id().canonical())?;
    let worker =
        load_validated_on::<WorkerResult>(conn, &derived.worker_result_id().canonical())?;
    let qc_id = derived.qc_report_id().ok_or_else(|| {
        AppError::Catalog("Published DerivedVariantVersion requires QCReport".into())
    })?;
    let qc = load_validated_on::<rigforge_domain::QcReport>(conn, &qc_id.canonical())?;
    let artifact_id = derived.persistence_artifact_id().ok_or_else(|| {
        AppError::Catalog("Published DerivedVariantVersion requires PersistenceArtifact".into())
    })?;
    let persistence =
        load_validated_on::<PersistenceArtifact>(conn, &artifact_id.canonical())?;
    let verification_id = derived.persistence_verification_id().ok_or_else(|| {
        AppError::Catalog("Published DerivedVariantVersion requires PersistenceVerification".into())
    })?;
    let verification =
        load_validated_on::<PersistenceVerification>(conn, &verification_id.canonical())?;
    let compat_id = job.as_record().compatibility_result_id().ok_or_else(|| {
        AppError::Catalog("publication requires JobSpec CompatibilityResult authorization".into())
    })?;
    let compatibility =
        load_validated_on::<CompatibilityResult>(conn, &compat_id.canonical())?;
    let backend = worker.as_record().execution().clone();
    validate_publication_lineage(
        &crate::transfer::publication_evidence(
            character.as_record(),
            motion.as_record(),
            source.as_record(),
            mapping.as_record(),
            policy.as_record(),
            job.as_record(),
            worker.as_record(),
            qc.as_record(),
            &backend,
            compatibility.as_record(),
            persistence.as_record(),
            verification.as_record(),
        ),
        derived,
    )
    .map_err(AppError::from)?;
    Ok(())
}

fn publish_derived_on(
    conn: &Connection,
    derived_version_id: &str,
) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
    let mut derived = load_validated_on::<rigforge_domain::DerivedVariantVersion>(
        conn,
        derived_version_id,
    )?
    .into_record();
    let character = load_validated_on::<CharacterAssetVersion>(
        conn,
        &derived.character_version_id().canonical(),
    )?;
    let motion =
        load_validated_on::<MotionAssetVersion>(conn, &derived.motion_version_id().canonical())?;
    let source = load_validated_on::<SourceSkeletonReference>(
        conn,
        &derived.source_skeleton_ref_id().canonical(),
    )?;
    let mapping =
        load_validated_on::<BoneMappingVersion>(conn, &derived.mapping_version_id().canonical())?;
    let policy =
        load_validated_on::<RetargetPolicyVersion>(conn, &derived.policy_version_id().canonical())?;
    let job = load_validated_on::<JobSpec>(conn, &derived.job_spec_id().canonical())?;
    let worker =
        load_validated_on::<WorkerResult>(conn, &derived.worker_result_id().canonical())?;
    let qc_id = derived.qc_report_id().ok_or_else(|| {
        AppError::Catalog("publication requires bound QCReport".into())
    })?;
    let qc = load_validated_on::<rigforge_domain::QcReport>(conn, &qc_id.canonical())?;
    let artifact_id = derived.persistence_artifact_id().ok_or_else(|| {
        AppError::Catalog("publication requires bound PersistenceArtifact".into())
    })?;
    let persistence =
        load_validated_on::<PersistenceArtifact>(conn, &artifact_id.canonical())?;
    let verification_id = derived.persistence_verification_id().ok_or_else(|| {
        AppError::Catalog("publication requires bound PersistenceVerification".into())
    })?;
    let verification =
        load_validated_on::<PersistenceVerification>(conn, &verification_id.canonical())?;
    let compat_id = job.as_record().compatibility_result_id().ok_or_else(|| {
        AppError::Catalog("publication requires JobSpec CompatibilityResult authorization".into())
    })?;
    let compatibility =
        load_validated_on::<CompatibilityResult>(conn, &compat_id.canonical())?;
    let _run = succeeded_job_run_on(
        conn,
        &derived.job_spec_id().canonical(),
        &derived.worker_result_id().canonical(),
    )?;
    let backend = worker.as_record().execution().clone();
    if job.as_record().target_derived_variant_id() != Some(derived.variant_id()) {
        return Err(AppError::Catalog(
            "publication requires JobSpec.target_derived_variant_id to equal DerivedVariantVersion.variant_id".into(),
        ));
    }
    if let Some(other) = find_other_derived_version_id_for_worker_result_on(
        conn,
        &derived.worker_result_id().canonical(),
        &derived.id().canonical(),
    )? {
        return Err(AppError::Catalog(format!(
            "WorkerResult {} already created DerivedVariantVersion {other}",
            derived.worker_result_id().canonical()
        )));
    }
    crate::transfer::domain_publish(
        crate::transfer::publication_evidence(
            character.as_record(),
            motion.as_record(),
            source.as_record(),
            mapping.as_record(),
            policy.as_record(),
            job.as_record(),
            worker.as_record(),
            qc.as_record(),
            &backend,
            compatibility.as_record(),
            persistence.as_record(),
            verification.as_record(),
        ),
        &mut derived,
    )?;
    let logical_id = derived.variant_id().canonical();
    let mut logical =
        load_validated_on::<DerivedVariant>(conn, &logical_id)?.into_record();
    if logical.draft_version_id() != Some(derived.id()) {
        return Err(AppError::Catalog(
            "publication requires DerivedVariant.draft_version_id to equal the exact DerivedVariantVersion".into(),
        ));
    }
    logical.bind_published(derived.id());
    if derived.lifecycle() != Lifecycle::Published {
        return Err(AppError::Catalog(
            "publication transaction requires the DerivedVariantVersion to be Published".into(),
        ));
    }
    if derived.variant_id() != logical.id() {
        return Err(AppError::Catalog(
            "published_version_id must refer to a version of this DerivedVariant".into(),
        ));
    }
    let derived = Validated::certify(derived)?;
    let logical = Validated::certify(logical)?;
    put_validated_on(conn, &derived)?;
    put_validated_on(conn, &logical)?;
    Ok(derived)
}

fn reject_public_authority_bypass<T: DomainRecord>(
    conn: &Connection,
    record: &Validated<T>,
) -> Result<(), AppError> {
    match T::RECORD_TYPE {
        RecordType::QcReport => Err(AppError::Catalog(
            "generic persistence cannot author publication-critical QcReport; use the Application QC workflow".into(),
        )),
        RecordType::PersistenceVerification => Err(AppError::Catalog(
            "generic persistence cannot author publication-critical PersistenceVerification; use the Application reopen workflow".into(),
        )),
        RecordType::PreviewArtifact => reject_preview_graph(conn, record),
        RecordType::DerivedVariant => reject_public_logical_pointer_change(conn, record),
        RecordType::DerivedVariantVersion => reject_public_derived_variant_version(conn, record),
        _ => Ok(()),
    }
}

fn reject_preview_graph<T: DomainRecord>(
    conn: &Connection,
    record: &Validated<T>,
) -> Result<(), AppError> {
    let payload = to_json(record)?;
    let preview = ingest_validated::<PreviewArtifact>(&payload)?;
    let preview = preview.as_record();
    if preview.location().is_none() {
        return Ok(());
    }
    require_location_bearing_preview(
        conn,
        preview,
        Path::new(preview.location().unwrap().value()),
    )
}

fn require_bound_preview_product(
    conn: &Connection,
    preview: &PreviewArtifact,
) -> Result<(), AppError> {
    let version_id = preview.bound_product_version_id().ok_or_else(|| {
        AppError::Catalog("PreviewArtifact is missing a bound Product version".into())
    })?;
    match preview.bound_product_kind() {
        ProductKind::CharacterAssetVersion => {
            load_validated_on::<CharacterAssetVersion>(conn, &version_id)?;
        }
        ProductKind::MotionAssetVersion => {
            load_validated_on::<MotionAssetVersion>(conn, &version_id)?;
        }
        ProductKind::DerivedVariantVersion => {
            load_validated_on::<rigforge_domain::DerivedVariantVersion>(conn, &version_id)?;
        }
    }
    Ok(())
}

fn require_preview_producer(
    conn: &Connection,
    preview: &PreviewArtifact,
) -> Result<(), AppError> {
    match load_validated_on::<BackendExecutionContext>(
        conn,
        &preview.producer_id().canonical(),
    ) {
        Ok(_) => Ok(()),
        Err(AppError::NotFound { .. }) => Err(AppError::Catalog(format!(
            "PreviewArtifact.producer_id {} does not resolve to an existing BackendExecutionContext",
            preview.producer_id().canonical()
        ))),
        Err(err) => Err(err),
    }
}

fn require_location_bearing_preview(
    conn: &Connection,
    preview: &PreviewArtifact,
    payload_path: &Path,
) -> Result<(), AppError> {
    require_bound_preview_product(conn, preview)?;
    require_preview_producer(conn, preview)?;
    verify_preview_payload_file(preview, payload_path)
}

fn verify_preview_payload_file(
    preview: &PreviewArtifact,
    payload_path: &Path,
) -> Result<(), AppError> {
    if preview.media_type() != "model/gltf-binary" {
        return Err(AppError::Catalog(format!(
            "unsupported Preview media type: {}",
            preview.media_type()
        )));
    }
    if !payload_path.is_file() {
        return Err(AppError::Catalog(format!(
            "Preview payload missing: {}",
            payload_path.display()
        )));
    }
    let size = std::fs::metadata(payload_path)?.len();
    if size != preview.size_bytes() {
        return Err(AppError::Catalog(format!(
            "Preview payload size mismatch: expected {} found {size}",
            preview.size_bytes()
        )));
    }
    let digest = crate::qc::sha256_file(payload_path)?;
    if digest != preview.digest().sha256() {
        return Err(AppError::Catalog(
            "Preview payload digest mismatch".into(),
        ));
    }
    Ok(())
}

fn persist_preview_on(
    conn: &Connection,
    artifact: &PreviewArtifact,
    payload_path: &Path,
) -> Result<Validated<PreviewArtifact>, AppError> {
    require_location_bearing_preview(conn, artifact, payload_path)?;
    let validated = Validated::certify(artifact.clone())?;
    put_validated_on(conn, &validated)?;
    Ok(validated)
}

fn reject_public_logical_pointer_change<T: DomainRecord>(
    conn: &Connection,
    record: &Validated<T>,
) -> Result<(), AppError> {
    let payload = to_json(record)?;
    let incoming = ingest_validated::<DerivedVariant>(&payload)?;
    let incoming = incoming.as_record();
    let existing = match load_validated_on::<DerivedVariant>(conn, &incoming.id().canonical()) {
        Ok(value) => Some(value),
        Err(AppError::NotFound { .. }) => None,
        Err(err) => return Err(err),
    };
    match existing {
        None if incoming.published_version_id().is_some() => Err(AppError::Catalog(
            "generic persistence cannot set DerivedVariant.published_version_id; use the publication transaction".into(),
        )),
        Some(old)
            if old.as_record().published_version_id() != incoming.published_version_id() =>
        {
            Err(AppError::Catalog(
                "generic persistence cannot change DerivedVariant.published_version_id; use the publication transaction".into(),
            ))
        }
        None if incoming.draft_version_id().is_some() => Err(AppError::Catalog(
            "generic persistence cannot set DerivedVariant.draft_version_id; use the candidate transaction".into(),
        )),
        Some(old) if old.as_record().draft_version_id() != incoming.draft_version_id() => {
            Err(AppError::Catalog(
                "generic persistence cannot change DerivedVariant.draft_version_id; use the candidate transaction".into(),
            ))
        }
        _ => Ok(()),
    }
}

fn reject_public_derived_variant_version<T: DomainRecord>(
    conn: &Connection,
    record: &Validated<T>,
) -> Result<(), AppError> {
    let payload = to_json(record)?;
    let incoming = ingest_validated::<rigforge_domain::DerivedVariantVersion>(&payload)?;
    let id = incoming.as_record().id().canonical();
    match load_validated_on::<rigforge_domain::DerivedVariantVersion>(conn, &id) {
        Ok(existing) => {
            let existing_payload = to_json(&existing)?;
            if existing_payload == payload {
                Ok(())
            } else {
                Err(AppError::Catalog(
                    "generic persistence cannot change DerivedVariantVersion; use the candidate transaction or trusted QC/verification binding".into(),
                ))
            }
        }
        Err(AppError::NotFound { .. }) => Err(AppError::Catalog(
            "generic persistence cannot create DerivedVariantVersion; use the candidate transaction".into(),
        )),
        Err(err) => Err(err),
    }
}

fn find_derived_version_id_for_worker_result_on(
    conn: &Connection,
    worker_result_id: &str,
) -> Result<Option<String>, AppError> {
    find_other_derived_version_id_for_worker_result_on(conn, worker_result_id, "")
}

fn find_other_derived_version_id_for_worker_result_on(
    conn: &Connection,
    worker_result_id: &str,
    except_version_id: &str,
) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT product_id FROM records
         WHERE record_type = 'derived_variant_version'
           AND json_extract(payload_json, '$.worker_result_id') = ?1
           AND product_id != ?2
         LIMIT 1",
    )?;
    stmt.query_row([worker_result_id, except_version_id], |row| row.get(0))
        .optional()
        .map_err(AppError::from)
}

fn record_payload_location_on(
    conn: &Connection,
    evidence: &CatalogLocationEvidence,
) -> Result<(), AppError> {
    let exists: Option<String> = conn
        .query_row(
            "SELECT product_id FROM records WHERE product_id = ?1 LIMIT 1",
            [&evidence.product_id],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound {
            what: "catalog record".into(),
            id: evidence.product_id.clone(),
        });
    }
    conn.execute(
        "INSERT INTO payload_locations
            (product_id, instance_id, location_kind, location_value, observed_at, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            evidence.product_id,
            evidence.instance_id,
            evidence.location_kind,
            evidence.location_value,
            evidence.observed_at,
            evidence.note,
        ],
    )?;
    Ok(())
}

fn persist_candidate_on(
    conn: &Connection,
    run_id: &str,
    version: &Validated<rigforge_domain::DerivedVariantVersion>,
    artifact: &Validated<PersistenceArtifact>,
    location: &CatalogLocationEvidence,
) -> Result<
    (
        Validated<DerivedVariant>,
        Validated<rigforge_domain::DerivedVariantVersion>,
    ),
    AppError,
> {
    let version_rec = version.as_record();
    let artifact_rec = artifact.as_record();
    let run = load_job_run_on(conn, run_id)?;
    if run.state != JobRunState::Succeeded {
        return Err(AppError::Catalog(
            "worker success candidate requires JobRun SUCCEEDED".into(),
        ));
    }
    let spec = load_validated_on::<JobSpec>(conn, &run.job_spec_id)?;
    let spec_rec = spec.as_record();
    let worker_id = run.worker_result_id.clone().ok_or_else(|| {
        AppError::Catalog("SUCCEEDED JobRun is missing worker_result_id".into())
    })?;
    let worker = load_validated_on::<WorkerResult>(conn, &worker_id)?;
    if !worker.as_record().worker_success() {
        return Err(AppError::Catalog(
            "WorkerResult.worker_success is false; no Derived Variant publication".into(),
        ));
    }
    let target_id = spec_rec.target_derived_variant_id().ok_or_else(|| {
        AppError::Catalog("candidate ingestion requires JobSpec target DerivedVariant".into())
    })?;
    if let Some(existing) = find_other_derived_version_id_for_worker_result_on(
        conn,
        &worker_id,
        &version_rec.id().canonical(),
    )? {
        return Err(AppError::Catalog(format!(
            "WorkerResult {worker_id} already created DerivedVariantVersion {existing}"
        )));
    }
    let mut logical = load_validated_on::<DerivedVariant>(conn, &target_id.canonical())?.into_record();
    if version_rec.variant_id() != logical.id() {
        return Err(AppError::Catalog(
            "candidate DerivedVariantVersion.variant_id must equal the JobSpec target DerivedVariant".into(),
        ));
    }
    if spec_rec.target_derived_variant_id() != Some(logical.id()) {
        return Err(AppError::Catalog(
            "JobSpec.target_derived_variant_id must equal the candidate logical DerivedVariant".into(),
        ));
    }
    if version_rec.job_spec_id().canonical() != run.job_spec_id
        || spec_rec.id() != version_rec.job_spec_id()
    {
        return Err(AppError::Catalog(
            "candidate version.job_spec_id must equal JobRun.job_spec_id".into(),
        ));
    }
    if version_rec.worker_result_id().canonical() != worker_id {
        return Err(AppError::Catalog(
            "candidate version.worker_result_id must equal JobRun.worker_result_id".into(),
        ));
    }
    if artifact_rec.bound_derived_variant_version_id() != version_rec.id() {
        return Err(AppError::Catalog(
            "PersistenceArtifact.bound_derived_variant_version_id must equal the candidate version".into(),
        ));
    }
    if version_rec.persistence_artifact_id() != Some(artifact_rec.id()) {
        return Err(AppError::Catalog(
            "DerivedVariantVersion.persistence_artifact_id must equal the candidate PersistenceArtifact".into(),
        ));
    }
    if version_rec.character_version_id() != spec_rec.character_version_id()
        || version_rec.motion_version_id() != spec_rec.motion_version_id()
        || version_rec.source_skeleton_ref_id() != spec_rec.source_skeleton_ref_id()
        || version_rec.mapping_version_id() != spec_rec.mapping_version_id()
        || version_rec.policy_version_id() != spec_rec.policy_version_id()
        || version_rec.backend_id() != worker.as_record().execution().id()
    {
        return Err(AppError::Catalog(
            "candidate version Character/Motion/Mapping/Policy/backend must match JobSpec and WorkerResult".into(),
        ));
    }
    if location.product_id != artifact_rec.id().canonical()
        || location.instance_id != artifact_rec.instance_id().canonical()
    {
        return Err(AppError::Catalog(
            "candidate payload location must bind the exact PersistenceArtifact instance".into(),
        ));
    }
    logical.bind_draft(version_rec.id());
    let logical = Validated::certify(logical)?;
    put_validated_on(conn, version)?;
    put_validated_on(conn, artifact)?;
    record_payload_location_on(conn, location)?;
    put_validated_on(conn, &logical)?;
    Ok((logical, version.clone()))
}

fn persist_trusted_qc_on(
    conn: &Connection,
    report: &Validated<rigforge_domain::QcReport>,
) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
    let subject = report.as_record().subject_derived_variant_version_id().ok_or_else(|| {
        AppError::Catalog("trusted QC binding requires a DerivedVariantVersion subject".into())
    })?;
    let mut version =
        load_validated_on::<rigforge_domain::DerivedVariantVersion>(conn, &subject.canonical())?
            .into_record();
    if version.lifecycle() != Lifecycle::Draft {
        return Err(AppError::Catalog(
            "trusted QC binding requires a Draft DerivedVariantVersion".into(),
        ));
    }
    put_validated_on(conn, report)?;
    version.bind_qc_report(report.as_record().id())?;
    let version = Validated::certify(version)?;
    put_validated_on(conn, &version)?;
    Ok(version)
}

fn persist_trusted_verification_on(
    conn: &Connection,
    verification: &Validated<PersistenceVerification>,
) -> Result<Validated<rigforge_domain::DerivedVariantVersion>, AppError> {
    let subject = verification
        .as_record()
        .subject_derived_variant_version_id();
    let mut version = load_validated_on::<rigforge_domain::DerivedVariantVersion>(
        conn,
        &subject.canonical(),
    )?
    .into_record();
    if version.lifecycle() != Lifecycle::Draft {
        return Err(AppError::Catalog(
            "trusted persistence verification binding requires a Draft DerivedVariantVersion".into(),
        ));
    }
    put_validated_on(conn, verification)?;
    version.bind_persistence_verification(verification.as_record().id())?;
    let version = Validated::certify(version)?;
    put_validated_on(conn, &version)?;
    Ok(version)
}

fn put_validated_on<T: DomainRecord>(
    conn: &Connection,
    record: &Validated<T>,
) -> Result<(), AppError> {
    let payload = to_json(record)?;
    let value: Value = serde_json::from_str(&payload)
        .map_err(|e| AppError::Catalog(format!("catalog payload is not JSON: {e}")))?;
    let expected_type = record_type_key(T::RECORD_TYPE);
    let stored_type = json_string(&value, "record_type").unwrap_or_default();
    if stored_type != expected_type {
        return Err(AppError::Catalog(format!(
            "payload record_type {stored_type} does not match {}",
            expected_type
        )));
    }
    if T::RECORD_TYPE == RecordType::JobSpec {
        let spec = ingest_validated::<JobSpec>(&payload)?;
        validate_job_graph_on(conn, spec.as_record())?;
    }
    if T::RECORD_TYPE == RecordType::CompatibilityResult {
        let result = ingest_validated::<CompatibilityResult>(&payload)?;
        validate_compatibility_graph_on(conn, result.as_record())?;
    }
    if T::RECORD_TYPE == RecordType::QcReport {
        let report = ingest_validated::<rigforge_domain::QcReport>(&payload)?;
        validate_qc_graph_on(conn, report.as_record())?;
    }
    if T::RECORD_TYPE == RecordType::PersistenceArtifact {
        let artifact = ingest_validated::<PersistenceArtifact>(&payload)?;
        validate_persistence_artifact_graph_on(conn, artifact.as_record())?;
    }
    if T::RECORD_TYPE == RecordType::PersistenceVerification {
        let verification = ingest_validated::<PersistenceVerification>(&payload)?;
        validate_persistence_verification_graph_on(conn, verification.as_record())?;
    }
    if T::RECORD_TYPE == RecordType::DerivedVariantVersion {
        let version = ingest_validated::<rigforge_domain::DerivedVariantVersion>(&payload)?;
        if version.as_record().lifecycle() == Lifecycle::Published {
            validate_published_derived_on(conn, version.as_record())?;
        }
    }
    if T::RECORD_TYPE == RecordType::PreviewArtifact {
        let preview = ingest_validated::<PreviewArtifact>(&payload)?;
        if preview.as_record().location().is_some() {
            require_location_bearing_preview(
                conn,
                preview.as_record(),
                Path::new(preview.as_record().location().unwrap().value()),
            )?;
        }
    }
    let meta = meta_from_json(&value)?;
    let now = now_ms();
    let existing: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT payload_json, lifecycle FROM records
             WHERE product_id = ?1 AND instance_id = ?2",
            rusqlite::params![meta.product_id, meta.instance_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((old_json, old_lifecycle)) = existing {
        if old_json == payload {
            return Ok(());
        }
        if !may_replace(old_lifecycle.as_deref(), &meta.record_type) {
            return Err(AppError::ImmutablePublished {
                product_id: meta.product_id,
            });
        }
        conn.execute(
            "UPDATE records SET
                record_type = ?1,
                logical_id = ?2,
                lifecycle = ?3,
                display_name = ?4,
                domain_schema_version = ?5,
                payload_json = ?6,
                updated_at = ?7
             WHERE product_id = ?8 AND instance_id = ?9",
            rusqlite::params![
                meta.record_type,
                meta.logical_id,
                meta.lifecycle,
                meta.display_name,
                meta.domain_schema_version,
                payload,
                now,
                meta.product_id,
                meta.instance_id,
            ],
        )?;
        return Ok(());
    }
    conn.execute(
        "INSERT INTO records (
            product_id, instance_id, record_type, logical_id, lifecycle, display_name,
            domain_schema_version, payload_json, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        rusqlite::params![
            meta.product_id,
            meta.instance_id,
            meta.record_type,
            meta.logical_id,
            meta.lifecycle,
            meta.display_name,
            meta.domain_schema_version,
            payload,
            now,
        ],
    )?;
    Ok(())
}

fn load_validated_on<T: DomainRecord>(
    conn: &Connection,
    product_id: &str,
) -> Result<Validated<T>, AppError> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT payload_json, record_type FROM records
             WHERE product_id = ?1
             ORDER BY created_at DESC, instance_id DESC
             LIMIT 1",
            [product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((payload, stored_type)) = row else {
        return Err(AppError::NotFound {
            what: record_type_key(T::RECORD_TYPE),
            id: product_id.to_string(),
        });
    };
    let expected = record_type_key(T::RECORD_TYPE);
    if stored_type != expected {
        return Err(AppError::Catalog(format!(
            "record {product_id} has type {stored_type}, expected {expected}"
        )));
    }
    ingest_validated::<T>(&payload).map_err(AppError::from)
}

fn insert_job_run_on(
    conn: &Connection,
    job_spec_id: &str,
    state: JobRunState,
    attempt_id: Option<String>,
) -> Result<JobRun, AppError> {
    let spec_exists: Option<String> = conn
        .query_row(
            "SELECT product_id FROM records
             WHERE product_id = ?1 AND record_type = 'job_spec'
             LIMIT 1",
            [job_spec_id],
            |row| row.get(0),
        )
        .optional()?;
    if spec_exists.is_none() {
        return Err(AppError::Orchestration(format!(
            "cannot enqueue JobRun; JobSpec {job_spec_id} is not in the catalog"
        )));
    }
    let now = now_ms();
    let run = JobRun {
        run_id: uuid::Uuid::now_v7().hyphenated().to_string(),
        job_spec_id: job_spec_id.to_string(),
        state,
        attempt_id: attempt_id
            .unwrap_or_else(|| uuid::Uuid::now_v7().hyphenated().to_string()),
        worker_execution_ref: None,
        failure_reason: None,
        worker_result_id: None,
        created_at: now,
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO job_runs (
            run_id, job_spec_id, state, attempt_id, worker_execution_ref,
            failure_reason, worker_result_id, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, ?5, ?5)",
        rusqlite::params![
            run.run_id,
            run.job_spec_id,
            run.state.as_db_str(),
            run.attempt_id,
            now
        ],
    )?;
    Ok(run)
}

fn transition_on(
    conn: &Connection,
    run_id: &str,
    next: JobRunState,
    failure_reason: Option<String>,
    worker_result_id: Option<String>,
) -> Result<JobRun, AppError> {
    let current = load_job_run_on(conn, run_id)?;
    if !current.state.can_transition(next) {
        return Err(AppError::InvalidTransition {
            from: current.state,
            to: next,
        });
    }
    let now = now_ms();
    conn.execute(
        "UPDATE job_runs
         SET state = ?1, failure_reason = COALESCE(?2, failure_reason),
             worker_result_id = COALESCE(?3, worker_result_id), updated_at = ?4
         WHERE run_id = ?5",
        rusqlite::params![
            next.as_db_str(),
            failure_reason,
            worker_result_id,
            now,
            run_id
        ],
    )?;
    load_job_run_on(conn, run_id)
}

fn load_job_run_on(conn: &Connection, run_id: &str) -> Result<JobRun, AppError> {
    conn.query_row(
        "SELECT run_id, job_spec_id, state, attempt_id, worker_execution_ref,
                failure_reason, worker_result_id, created_at, updated_at
         FROM job_runs WHERE run_id = ?1",
        [run_id],
        job_run_from_row,
    )
    .map_err(|_| AppError::NotFound {
        what: "job_run".into(),
        id: run_id.to_string(),
    })
}

fn job_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobRun> {
    let state_raw: String = row.get(2)?;
    Ok(JobRun {
        run_id: row.get(0)?,
        job_spec_id: row.get(1)?,
        state: JobRunState::parse(&state_raw).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
            )
        })?,
        attempt_id: row.get(3)?,
        worker_execution_ref: row.get(4)?,
        failure_reason: row.get(5)?,
        worker_result_id: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn meta_from_json(value: &Value) -> Result<RecordMeta, AppError> {
    let product_id = json_string(value, "id").ok_or_else(|| {
        AppError::Catalog("Domain payload is missing id".into())
    })?;
    let record_type = json_string(value, "record_type").ok_or_else(|| {
        AppError::Catalog("Domain payload is missing record_type".into())
    })?;
    let domain_schema_version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::Catalog("Domain payload is missing schema_version".into()))?
        as i64;
    Ok(RecordMeta {
        instance_id: json_string(value, "instance_id").unwrap_or_default(),
        logical_id: logical_id_of(&record_type, value),
        lifecycle: json_string(value, "lifecycle"),
        display_name: json_string(value, "display_name"),
        product_id,
        record_type,
        domain_schema_version,
    })
}

fn logical_id_of(record_type: &str, value: &Value) -> Option<String> {
    match record_type {
        "character_asset_version" | "motion_asset_version" => json_string(value, "asset_id"),
        "bone_mapping_version" => json_string(value, "mapping_id"),
        "retarget_policy_version" => json_string(value, "policy_id"),
        "derived_variant_version" => json_string(value, "variant_id"),
        "character_asset"
                | "motion_asset"
                | "bone_mapping"
                | "retarget_policy"
                | "derived_variant"
                | "source_skeleton_reference" => json_string(value, "id"),
        "skeleton_summary" | "compatibility_result" => json_string(value, "id"),
        _ => None,
    }
}

fn may_replace(existing_lifecycle: Option<&str>, record_type: &str) -> bool {
    match existing_lifecycle {
        Some("draft") => true,
        Some("published") | Some("ready") | Some("invalidated") => false,
        None => matches!(
            record_type,
            "character_asset"
                | "motion_asset"
                | "bone_mapping"
                | "retarget_policy"
                | "derived_variant"
        ),
        Some(_) => false,
    }
}

fn record_type_key(record_type: RecordType) -> String {
    serde_json::to_value(record_type)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{record_type:?}"))
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_owned)
}

fn resolve_source_input(
    catalog: &SqliteCatalog,
    version_id: &str,
    evidence: &SourceArtifactEvidence,
) -> Result<ResolvedSourceInput, AppError> {
    let mut location = filesystem_location(evidence)?;
    let overlays = catalog.list_payload_locations(version_id)?;
    if let Some(last) = overlays.last() {
        if last.location_kind == "filesystem_path_evidence" && !last.location_value.trim().is_empty()
        {
            location = PathBuf::from(&last.location_value);
        }
    }
    Ok(ResolvedSourceInput {
        version_id: version_id.to_string(),
        location,
        digest_sha256: evidence.digest().sha256().to_string(),
        size_bytes: evidence.size_bytes(),
    })
}

#[cfg(test)]
mod authority_tests {
    use super::*;
    use crate::test_graph::{
        certify, persist_unpublished_without_authority, unpublished_graph,
    };
    use rigforge_domain::{
        BackendExecutionContext, PersistenceVerification, VerificationOutcome,
    };

    #[test]
    fn verification_subject_must_match_artifact_bound_version() {
        let mut catalog = SqliteCatalog::open_in_memory().unwrap();
        let a = unpublished_graph();
        let b = unpublished_graph();
        persist_unpublished_without_authority(&mut catalog, &a).unwrap();
        persist_unpublished_without_authority(&mut catalog, &b).unwrap();
        let forged = PersistenceVerification::new(
            a.persistence.id(),
            a.persistence.instance_id(),
            a.persistence.digest().clone(),
            b.derived_version.id(),
            a.backend.id(),
            VerificationOutcome::Pass,
            VerificationOutcome::Pass,
        )
        .unwrap();
        let err = catalog
            .put_trusted_persistence_verification(&certify(forged))
            .unwrap_err();
        match err {
            AppError::Catalog(msg) => assert!(
                msg.contains("bound_derived_variant_version_id")
                    || msg.contains("subject"),
                "{msg}"
            ),
            other => panic!("expected subject/bound-version mismatch, got {other}"),
        }
    }

    #[test]
    fn verification_producer_must_match_artifact_and_version() {
        let mut catalog = SqliteCatalog::open_in_memory().unwrap();
        let a = unpublished_graph();
        persist_unpublished_without_authority(&mut catalog, &a).unwrap();
        let other = BackendExecutionContext::new(
            "isolated-worker",
            "1.0.0",
            "build-other",
            "adapter-1",
            "exec-policy-1",
        )
        .unwrap();
        let forged = PersistenceVerification::new(
            a.persistence.id(),
            a.persistence.instance_id(),
            a.persistence.digest().clone(),
            a.derived_version.id(),
            other.id(),
            VerificationOutcome::Pass,
            VerificationOutcome::Pass,
        )
        .unwrap();
        let err = catalog
            .put_trusted_persistence_verification(&certify(forged))
            .unwrap_err();
        match err {
            AppError::Catalog(msg) => assert!(msg.contains("producer_id"), "{msg}"),
            other => panic!("expected producer mismatch, got {other}"),
        }
    }

    #[test]
    fn trusted_matching_verification_persists_through_put_validated_on() {
        let mut catalog = SqliteCatalog::open_in_memory().unwrap();
        let a = unpublished_graph();
        persist_unpublished_without_authority(&mut catalog, &a).unwrap();
        catalog
            .put_trusted_qc_report(&certify(a.qc.clone()))
            .unwrap();
        catalog
            .put_trusted_persistence_verification(&certify(a.verification.clone()))
            .unwrap();
        let loaded = catalog
            .load_persistence_verification(&a.verification.id().canonical())
            .unwrap();
        assert_eq!(loaded.as_record(), &a.verification);
    }
}

#[cfg(test)]
mod catalog_surface_tests {
    use super::*;
    use crate::test_graph::{certify, unpublished_graph};
    use rigforge_domain::{to_json, ErrorCode};

    #[test]
    fn product_ids_are_not_sqlite_rowids() {
        let mut catalog = SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        catalog
            .put_validated(&certify(g.character.clone()))
            .unwrap();
        let product_id = g.character.id().canonical();
        let rowid = catalog.internal_rowid_for_tests(&product_id).unwrap();
        assert_ne!(product_id, rowid.to_string());
        assert!(product_id.contains('-'), "UUIDv7 is hyphenated");
        assert!(!product_id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn unsupported_domain_schema_fails_closed_on_load() {
        let mut catalog = SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        let mut payload: serde_json::Value =
            serde_json::from_str(&to_json(&g.character_version).unwrap()).unwrap();
        payload["schema_version"] = serde_json::json!(99);
        catalog
            .insert_unvalidated_payload_for_tests(
                &g.character_version.id().canonical(),
                "character_asset_version",
                &payload.to_string(),
                99,
            )
            .unwrap();
        let err = catalog
            .load_character_version(&g.character_version.id().canonical())
            .unwrap_err();
        match err {
            AppError::Domain(domain) => {
                assert_eq!(domain.code, ErrorCode::UnsupportedSchemaVersion);
            }
            other => panic!("expected Domain UnsupportedSchemaVersion, got {other}"),
        }
    }
}

