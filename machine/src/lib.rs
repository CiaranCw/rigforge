//! Backend-neutral machine interface for local hosts.
//!
//! Hosts talk to [`rigforge_app::Application`] over a typed JSON/stdio
//! protocol. This crate does not introduce Product identity, does not replace
//! Mapping acceptance, Compatibility, QC, or PersistenceVerification, and
//! must not contain host-specific catalog or UI concepts.

pub mod protocol;
pub mod review;
pub mod session;

pub use protocol::{Envelope, MachineEvent, MachineMessage, MachineResult};
pub use review::{core_semantic_pairs, mapping_review_from_version, CorePair, MappingReview};
pub use session::{handle_line, MachineSession};
