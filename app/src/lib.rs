//! RigForge V1-2 application layer.
//!
//! Local catalog, orchestration, Workbench queries, V1-3 terminal
//! collection, and V1-6 derived Preview generation/resolve. Not a viewer.

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
pub mod pinned_preview;
pub mod pinned_qc;
pub mod preflight;
pub mod preview;
pub mod qc;
pub mod queries;
pub mod skeleton;
pub mod transfer;
pub mod worker;

#[cfg(any(test, feature = "test-support"))]
mod test_graph;

#[cfg(feature = "test-support")]
pub use test_graph::{persist_unpublished_without_authority, unpublished_graph};

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
pub use pinned_preview::BlenderPreviewGenerator;
pub use pinned_qc::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact,
};
pub use preview::{
    sha256_bytes, synthetic_preview_glb, verify_source_before_generation, GeneratedPreview,
    MemoryPreviewGenerator, PreviewDescriptor, PreviewFailure, PreviewFailureKind,
    PreviewGenerationJob, PreviewGenerationRequest, PreviewGeneratorPort, PreviewSession,
    PreviewSessionDocument, PreviewSubject, PreviewView, PREVIEW_GENERATOR_ID,
    PREVIEW_MEDIA_TYPE, PREVIEW_RECIPE_VERSION,
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
