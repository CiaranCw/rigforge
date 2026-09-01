//! V1-2 persistence interface. No database is implemented in V1-1.
//!
//! V1-2 must store `Validated<T>` Domain records. Raw deserialized `T` values
//! and JSON strings are not a legal store/load contract.

use crate::error::DomainError;
use crate::record::{DomainRecord, Validated};

pub trait ProductVersionStore {
    fn store_immutable_version<T: DomainRecord>(
        &mut self,
        record: Validated<T>,
    ) -> Result<(), DomainError>;
    fn load_exact_version<T: DomainRecord>(
        &self,
        version_id: &str,
    ) -> Result<Validated<T>, DomainError>;
    fn resolve_logical_object_versions(&self, logical_id: &str) -> Result<Vec<String>, DomainError>;
    fn store_artifact_metadata<T: DomainRecord>(
        &mut self,
        record: Validated<T>,
    ) -> Result<(), DomainError>;
}
