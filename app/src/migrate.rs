//! Database schema / migration version. Distinct from Domain schema_version.

use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::error::AppError;

/// Catalog SQL schema version. Not Domain `schema_version` and not a Product version.
pub const DB_SCHEMA_VERSION: i32 = 1;
pub const DB_SCHEMA_NAME: &str = "v1_2_catalog";

const REQUIRED_TABLES: [&str; 4] = [
    "schema_migrations",
    "records",
    "payload_locations",
    "job_runs",
];

const REQUIRED_INDEXES: [&str; 2] = ["idx_records_logical", "idx_records_type"];

const V1_DDL: &str = r#"
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at INTEGER NOT NULL
);

CREATE TABLE records (
    product_id TEXT NOT NULL,
    instance_id TEXT NOT NULL DEFAULT '',
    record_type TEXT NOT NULL,
    logical_id TEXT,
    lifecycle TEXT,
    display_name TEXT,
    domain_schema_version INTEGER NOT NULL,
    payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (product_id, instance_id)
);

CREATE INDEX idx_records_logical ON records(logical_id);
CREATE INDEX idx_records_type ON records(record_type);

CREATE TABLE payload_locations (
    location_rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id TEXT NOT NULL,
    instance_id TEXT NOT NULL DEFAULT '',
    location_kind TEXT NOT NULL,
    location_value TEXT NOT NULL,
    observed_at INTEGER NOT NULL,
    note TEXT
);

CREATE TABLE job_runs (
    run_id TEXT PRIMARY KEY,
    job_spec_id TEXT NOT NULL,
    state TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    worker_execution_ref TEXT,
    failure_reason TEXT,
    worker_result_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
"#;

pub fn apply_migrations(conn: &mut Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")?;
    let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");

    let user_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if user_version > DB_SCHEMA_VERSION {
        return Err(AppError::UnsupportedDbSchema {
            found: user_version,
            supported: DB_SCHEMA_VERSION,
        });
    }
    if user_version == DB_SCHEMA_VERSION {
        verify_current_schema(conn)?;
        return Ok(());
    }
    if user_version == 0 {
        migrate_to_v1(conn)?;
        return Ok(());
    }
    Err(AppError::InvalidDbState(format!(
        "no migration path from database schema {user_version} to {DB_SCHEMA_VERSION}"
    )))
}

fn migrate_to_v1(conn: &mut Connection) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    apply_v1_schema(&tx)?;
    tx.commit()?;
    Ok(())
}

fn apply_v1_schema(tx: &Transaction<'_>) -> Result<(), AppError> {
    tx.execute_batch(V1_DDL)?;
    tx.execute(
        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![DB_SCHEMA_VERSION, DB_SCHEMA_NAME, now_ms()],
    )?;
    tx.pragma_update(None, "user_version", DB_SCHEMA_VERSION)?;
    Ok(())
}

#[cfg(test)]
fn migrate_to_v1_forced_failure(conn: &mut Connection) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    apply_v1_schema(&tx)?;
    Err(AppError::InvalidDbState(
        "forced migration failure after applying v1 objects".into(),
    ))
}

pub fn verify_current_schema(conn: &Connection) -> Result<(), AppError> {
    for table in REQUIRED_TABLES {
        if !object_exists(conn, "table", table)? {
            return Err(AppError::InvalidDbState(format!(
                "database user_version is current but required table `{table}` is missing"
            )));
        }
    }
    for index in REQUIRED_INDEXES {
        if !object_exists(conn, "index", index)? {
            return Err(AppError::InvalidDbState(format!(
                "database user_version is current but required index `{index}` is missing"
            )));
        }
    }
    let row: Option<(i32, String)> = conn
        .query_row(
            "SELECT version, name FROM schema_migrations WHERE version = ?1",
            [DB_SCHEMA_VERSION],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match row {
        Some((version, name)) if version == DB_SCHEMA_VERSION && name == DB_SCHEMA_NAME => Ok(()),
        Some((version, name)) => Err(AppError::InvalidDbState(format!(
            "schema_migrations row is ({version}, {name}); expected ({DB_SCHEMA_VERSION}, {DB_SCHEMA_NAME})"
        ))),
        None => Err(AppError::InvalidDbState(
            "schema_migrations is missing the v1 catalog row".into(),
        )),
    }
}

fn object_exists(conn: &Connection, kind: &str, name: &str) -> Result<bool, AppError> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
        rusqlite::params![kind, name],
        |row| row.get(0),
    )?;
    Ok(count == 1)
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::SqliteCatalog;
    use rusqlite::Connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rigforge-v1-2-migrate-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("catalog.sqlite")
    }

    fn open_raw(path: &std::path::Path) -> Connection {
        Connection::open(path).unwrap()
    }

    #[test]
    fn future_database_schema_fails_closed() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        let err = apply_migrations(&mut conn).unwrap_err();
        match err {
            AppError::UnsupportedDbSchema {
                found: 99,
                supported: DB_SCHEMA_VERSION,
            } => {}
            other => panic!("expected UnsupportedDbSchema, got {other}"),
        }
    }

    #[test]
    fn empty_memory_migrates_to_v1() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_migrations(&mut conn).unwrap();
        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, DB_SCHEMA_VERSION);
        verify_current_schema(&conn).unwrap();
    }

    #[test]
    fn current_user_version_with_missing_job_runs_fails_open() {
        let path = temp_db();
        SqliteCatalog::open(&path).unwrap();
        {
            let conn = open_raw(&path);
            conn.execute("DROP TABLE job_runs", []).unwrap();
        }
        let err = SqliteCatalog::open(&path).unwrap_err();
        match err {
            AppError::InvalidDbState(msg) => assert!(msg.contains("job_runs"), "{msg}"),
            other => panic!("expected InvalidDbState, got {other}"),
        }
    }

    #[test]
    fn current_user_version_with_missing_payload_locations_fails_open() {
        let path = temp_db();
        SqliteCatalog::open(&path).unwrap();
        {
            let conn = open_raw(&path);
            conn.execute("DROP TABLE payload_locations", []).unwrap();
        }
        let err = SqliteCatalog::open(&path).unwrap_err();
        match err {
            AppError::InvalidDbState(msg) => {
                assert!(msg.contains("payload_locations"), "{msg}")
            }
            other => panic!("expected InvalidDbState, got {other}"),
        }
    }

    #[test]
    fn current_user_version_with_missing_schema_migrations_fails_open() {
        let path = temp_db();
        SqliteCatalog::open(&path).unwrap();
        {
            let conn = open_raw(&path);
            conn.execute("DROP TABLE schema_migrations", []).unwrap();
        }
        let err = SqliteCatalog::open(&path).unwrap_err();
        match err {
            AppError::InvalidDbState(msg) => {
                assert!(msg.contains("schema_migrations"), "{msg}")
            }
            other => panic!("expected InvalidDbState, got {other}"),
        }
    }

    #[test]
    fn wrong_or_missing_v1_migration_record_fails_open() {
        let path = temp_db();
        SqliteCatalog::open(&path).unwrap();
        {
            let conn = open_raw(&path);
            conn.execute("DELETE FROM schema_migrations", []).unwrap();
        }
        let err = SqliteCatalog::open(&path).unwrap_err();
        match err {
            AppError::InvalidDbState(msg) => {
                assert!(msg.contains("schema_migrations"), "{msg}")
            }
            other => panic!("expected InvalidDbState, got {other}"),
        }

        let path2 = temp_db();
        SqliteCatalog::open(&path2).unwrap();
        {
            let conn = open_raw(&path2);
            conn.execute(
                "UPDATE schema_migrations SET name = 'not-v1-2-catalog' WHERE version = 1",
                [],
            )
            .unwrap();
        }
        let err = SqliteCatalog::open(&path2).unwrap_err();
        match err {
            AppError::InvalidDbState(msg) => {
                assert!(
                    msg.contains("not-v1-2-catalog") || msg.contains("schema_migrations"),
                    "{msg}"
                )
            }
            other => panic!("expected InvalidDbState, got {other}"),
        }
    }

    #[test]
    fn failed_migration_does_not_leave_accepted_partial_schema() {
        let path = temp_db();
        {
            let mut conn = Connection::open(&path).unwrap();
            let err = migrate_to_v1_forced_failure(&mut conn).unwrap_err();
            match err {
                AppError::InvalidDbState(msg) => assert!(msg.contains("forced")),
                other => panic!("expected forced failure, got {other}"),
            }
            let version: i32 = conn
                .query_row("PRAGMA user_version", [], |row| row.get(0))
                .unwrap();
            assert_eq!(version, 0);
            let records: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'records'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(records, 0);
            let jobs: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'job_runs'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(jobs, 0);
        }
        let catalog = SqliteCatalog::open(&path).unwrap();
        assert_eq!(catalog.db_schema_version().unwrap(), DB_SCHEMA_VERSION);
    }
}
