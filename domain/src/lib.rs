//! RigForge V1-1 thin Workflow Domain.
//!
//! This crate is Product Domain code. It is not a Blender worker, catalog
//! database, GUI, Mapping algorithm, or Preview generator.

pub mod artifacts;
pub mod assets;
pub mod backend;
pub mod error;
pub mod execution;
pub mod graph;
pub mod identity;
pub mod json;
pub mod mapping;
pub mod publication;
pub mod record;
pub mod source;
pub mod store;
pub mod time;

pub use artifacts::*;
pub use assets::*;
pub use backend::*;
pub use error::*;
pub use execution::*;
pub use graph::{
    validate_execution_lineage, validate_job_inputs, validate_publication_lineage, PublicationEvidence,
};
pub use identity::*;
pub use json::{from_json_validated, ingest_validated, round_trip, to_json};
pub use mapping::*;
pub use publication::publish_derived_variant;
pub use record::{DomainRecord, Validated};
pub use source::*;
pub use store::ProductVersionStore;
pub use time::*;
