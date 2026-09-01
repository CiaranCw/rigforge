//! SQLite Asset Catalog. Consumes `Validated<T>` only.

use std::path::{Path, PathBuf};

use rigforge_domain::{
    ingest_validated, to_json, validate_job_inputs, BoneMappingVersion, CharacterAsset,
    CharacterAssetVersion, DerivedVariant, DomainRecord, JobSpec, MotionAsset, MotionAssetVersion,
    PersistenceArtifact, PersistenceVerification, ProductVersionStore, RecordType,
    RetargetPolicyVersion, SourceSkeletonReference, Validated, WorkerResult,
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde_json::Value;
use std::fmt;

use crate::error::AppError;
use crate::migrate::{apply_migrations, now_ms};
use crate::orchestration::{JobRun, JobRunState};
use crate::queries::AssetListItem;
use crate::worker::{DispatchReceipt, WorkerPort};

pub use crate::migrate::{DB_SCHEMA_NAME, DB_SCHEMA_VERSION};

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
        let conn = Connection::open(path)?;
        let mut catalog = Self {
            conn,
            path: Some(path.to_path_buf()),
        };
        catalog.configure()?;
        Ok(catalog)
    }

    pub fn open_in_memory() -> Result<Self, AppError> {
        let mut catalog = Self {
            conn: Connection::open_in_memory()?,
            path: None,
        };
        catalog.configure()?;
        Ok(catalog)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
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

    pub fn in_transaction<F, T>(&mut self, f: F) -> Result<T, AppError>
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
        self.in_transaction(|tx| put_validated_on(&*tx, record))
    }

    pub fn put_validated_pair<A: DomainRecord, B: DomainRecord>(
        &mut self,
        first: &Validated<A>,
        second: &Validated<B>,
    ) -> Result<(), AppError> {
        self.in_transaction(|tx| {
            put_validated_on(&*tx, first)?;
            put_validated_on(&*tx, second)?;
            Ok(())
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
                    'derived_variant_version'
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
        let exists: Option<String> = self
            .conn
            .query_row(
                "SELECT product_id FROM records WHERE product_id = ?1 LIMIT 1",
                [&evidence.product_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(AppError::NotFound {
                what: "catalog record".into(),
                id: evidence.product_id,
            });
        }
        self.conn.execute(
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

    /// Internal SQLite rowid. Never a Product ID. Exposed only to prove independence.
    pub fn internal_rowid_for_tests(&self, product_id: &str) -> Result<i64, AppError> {
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
    pub fn insert_unvalidated_payload_for_tests(
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
        match worker.dispatch(&spec, &run.attempt_id) {
            Ok(receipt) => {
                if receipt.attempt_id != run.attempt_id {
                    let reason =
                        "worker receipt attempt_id does not match orchestrator attempt_id";
                    self.fail_dispatch(run_id, reason)?;
                    return Err(AppError::Worker(reason.into()));
                }
                let now = now_ms();
                self.conn.execute(
                    "UPDATE job_runs
                     SET state = ?1, worker_execution_ref = ?2, updated_at = ?3
                     WHERE run_id = ?4",
                    rusqlite::params![
                        JobRunState::Running.as_db_str(),
                        receipt.worker_execution_ref,
                        now,
                        run_id
                    ],
                )?;
                Ok((self.load_job_run(run_id)?, receipt))
            }
            Err(err) => {
                self.fail_dispatch(run_id, &err.to_string())?;
                Err(err)
            }
        }
    }

    pub fn complete_success(
        &mut self,
        run_id: &str,
        worker_result: &Validated<WorkerResult>,
    ) -> Result<JobRun, AppError> {
        self.in_transaction(|tx| {
            let run = load_job_run_on(&*tx, run_id)?;
            let result = worker_result.as_record();
            if result.job_spec_id().canonical() != run.job_spec_id {
                return Err(AppError::Orchestration(
                    "WorkerResult.job_spec_id must equal JobRun.job_spec_id".into(),
                ));
            }
            if !result.worker_success() {
                return Err(AppError::Orchestration(
                    "SUCCEEDED requires WorkerResult.worker_success == true".into(),
                ));
            }
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
        let reason = reason.into();
        self.in_transaction(|tx| {
            transition_on(&*tx, run_id, JobRunState::Failed, Some(reason), None)
        })
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
    .map_err(AppError::from)
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
