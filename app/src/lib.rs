//! RigForge V1-2 application layer.
//!
//! Local catalog, orchestration, Workbench queries, and V1-3 terminal
//! collection. Not a Blender worker and not a Preview viewer.

pub mod application;
pub mod capability;
pub mod candidates;
pub mod catalog;
pub mod dispatch;
pub mod error;
pub mod mapping_workflow;
pub mod migrate;
pub mod orchestration;
pub mod preflight;
pub mod queries;
pub mod skeleton;
pub mod worker;

pub use application::Application;
pub use capability::WorkerCapabilityProfile;
pub use candidates::{
    generate_mapping_proposal, normalize_joint_name, CandidateSignal, MappingAmbiguity,
    MappingAssistProfile, MappingProposal, ProposedMappingEntry,
};
pub use catalog::{
    CatalogLocationEvidence, SqliteCatalog, DB_SCHEMA_NAME, DB_SCHEMA_VERSION,
};
pub use dispatch::{ResolvedSourceInput, WorkerDispatchRequest};
pub use error::AppError;
pub use mapping_workflow::MappingWorkflowSnapshot;
pub use orchestration::{JobRun, JobRunState};
pub use preflight::evaluate_compatibility;
pub use queries::AssetListItem;
pub use skeleton::{MemorySkeletonInspector, SkeletonEvidenceProvider};
pub use worker::{
    DispatchReceipt, FakeWorker, TerminalOutcome, WorkerCompletionPort, WorkerFailureClass,
    WorkerPort,
};

pub use rigforge_domain;
