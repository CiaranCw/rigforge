//! RigForge V1-2 application layer.
//!
//! Local catalog, orchestration, Workbench queries, and V1-3 terminal
//! collection. Not a Blender worker and not a Preview viewer.

extern crate self as rigforge_app;

pub mod application;
pub mod artifact;
pub mod capability;
pub mod candidates;
pub mod catalog;
pub mod dispatch;
pub mod error;
pub mod mapping_workflow;
pub mod migrate;
pub mod orchestration;
pub mod pinned_qc;
pub mod preflight;
pub mod qc;
pub mod queries;
pub mod skeleton;
pub mod transfer;
pub mod worker;

#[cfg(test)]
mod test_graph;

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod transfer_tests;

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
pub use pinned_qc::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact,
};
pub use preflight::evaluate_compatibility;
pub use qc::{
    evaluate_structural_qc, interpret_qc_inspect, sha256_file, ArtifactInspectionEvidence,
    QcInspectEnvelope,
};

#[cfg(any(test, feature = "test-support"))]
pub use qc::{ArtifactInspector, PersistenceReopener};
pub use queries::AssetListItem;
pub use skeleton::{MemorySkeletonInspector, SkeletonEvidenceProvider};
pub use transfer::{TransferAuthorization, TransferOutcome, TransferOutcomeKind};
pub use worker::{
    DispatchReceipt, FakeWorker, TerminalOutcome, WorkerCompletionPort, WorkerFailureClass,
    WorkerPort,
};

pub use rigforge_domain;
