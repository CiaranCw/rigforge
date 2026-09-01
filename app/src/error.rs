//! Application-layer errors. Catalog and orchestration failures are not Domain
//! schema changes.

use std::fmt;
use std::io;

use rigforge_domain::{DomainError, ErrorCode};

use crate::orchestration::JobRunState;

#[derive(Debug)]
pub enum AppError {
    Domain(DomainError),
    Sqlite(rusqlite::Error),
    Io(io::Error),
    MissingDirectory(String),
    Unwritable(String),
    NotFound { what: String, id: String },
    ImmutablePublished { product_id: String },
    InvalidTransition { from: JobRunState, to: JobRunState },
    UnsupportedDbSchema { found: i32, supported: i32 },
    InvalidDbState(String),
    Catalog(String),
    Orchestration(String),
    Worker(String),
}

impl AppError {
    pub fn into_domain(self) -> DomainError {
        match self {
            AppError::Domain(err) => err,
            AppError::ImmutablePublished { product_id } => DomainError::new(
                ErrorCode::ImmutableVersion,
                format!("published/frozen record cannot be replaced: {product_id}"),
            ),
            AppError::NotFound { what, id } => DomainError::new(
                ErrorCode::StoreContract,
                format!("{what} not found: {id}"),
            ),
            other => DomainError::new(ErrorCode::StoreContract, other.to_string()),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Domain(err) => write!(f, "{err}"),
            AppError::Sqlite(err) => write!(f, "sqlite: {err}"),
            AppError::Io(err) => write!(f, "io: {err}"),
            AppError::MissingDirectory(path) => {
                write!(f, "catalog parent directory does not exist: {path}")
            }
            AppError::Unwritable(path) => write!(f, "catalog is not writable: {path}"),
            AppError::NotFound { what, id } => write!(f, "{what} not found: {id}"),
            AppError::ImmutablePublished { product_id } => {
                write!(f, "frozen Product record cannot be replaced in place: {product_id}")
            }
            AppError::InvalidTransition { from, to } => {
                write!(f, "illegal job transition {from:?} → {to:?}")
            }
            AppError::UnsupportedDbSchema { found, supported } => {
                write!(f, "unsupported database schema {found}; this catalog implements {supported}")
            }
            AppError::InvalidDbState(msg) | AppError::Catalog(msg) => write!(f, "{msg}"),
            AppError::Orchestration(msg) => write!(f, "orchestration: {msg}"),
            AppError::Worker(msg) => write!(f, "worker port: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
