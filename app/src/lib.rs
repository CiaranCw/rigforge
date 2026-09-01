//! RigForge V1-2 application layer.
//!
//! Local catalog, orchestration, Workbench queries, and V1-3 terminal
//! collection. Not a Blender worker and not a Preview viewer.

pub mod application;
pub mod catalog;
pub mod dispatch;
pub mod error;
pub mod migrate;
pub mod orchestration;
pub mod queries;
pub mod worker;

pub use application::Application;
pub use catalog::{
    CatalogLocationEvidence, SqliteCatalog, DB_SCHEMA_NAME, DB_SCHEMA_VERSION,
};
pub use dispatch::{ResolvedSourceInput, WorkerDispatchRequest};
pub use error::AppError;
pub use orchestration::{JobRun, JobRunState};
pub use queries::AssetListItem;
pub use worker::{
    DispatchReceipt, FakeWorker, TerminalOutcome, WorkerCompletionPort, WorkerFailureClass,
    WorkerPort,
};

pub use rigforge_domain;
