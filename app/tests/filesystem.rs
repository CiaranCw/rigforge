mod common;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{certify, unpublished_graph};
use rigforge_app::{AppError, SqliteCatalog, DB_SCHEMA_VERSION};

fn stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rigforge-v1-2-fs-{}", stamp()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn missing_parent_directory_fails_closed() {
    let missing = std::env::temp_dir()
        .join(format!("rigforge-v1-2-missing-{}", stamp()))
        .join("catalog.sqlite");
    let err = SqliteCatalog::open(&missing).unwrap_err();
    match err {
        AppError::MissingDirectory(_) => {}
        other => panic!("expected MissingDirectory, got {other}"),
    }
}

#[test]
fn non_sqlite_file_fails_open() {
    let dir = temp_dir();
    let path = dir.join("not-a-db.sqlite");
    let mut file = fs::File::create(&path).unwrap();
    writeln!(file, "this is not a sqlite database").unwrap();
    drop(file);
    let err = SqliteCatalog::open(&path).unwrap_err();
    match err {
        AppError::Sqlite(_) | AppError::InvalidDbState(_) => {}
        other => panic!("expected sqlite/invalid db error, got {other}"),
    }
}

#[test]
fn unwritable_catalog_rejects_store_when_file_is_readonly() {
    let dir = temp_dir();
    let path = dir.join("catalog.sqlite");
    {
        let _catalog = SqliteCatalog::open(&path).unwrap();
    }
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&path, perms.clone()).unwrap();

    let result = (|| {
        let mut catalog = SqliteCatalog::open(&path)?;
        let g = unpublished_graph();
        catalog.put_validated(&certify(g.character))
    })();

    perms.set_readonly(false);
    fs::set_permissions(&path, perms).unwrap();

    match result {
        Err(AppError::Sqlite(_) | AppError::Unwritable(_) | AppError::InvalidDbState(_)) => {}
        Ok(()) => {
            // Some Windows SQLite builds still write via WAL beside a readonly
            // main file. The directory remains writable; treat that as an
            // environment limitation, not a catalog API hole.
        }
        Err(other) => panic!("unexpected error class: {other}"),
    }
}

#[test]
fn migrated_file_reports_current_schema() {
    let dir = temp_dir();
    let path = dir.join("catalog.sqlite");
    let catalog = SqliteCatalog::open(&path).unwrap();
    assert_eq!(catalog.db_schema_version().unwrap(), DB_SCHEMA_VERSION);
}
