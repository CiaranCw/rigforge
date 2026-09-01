//! Shared Domain record validation contract.

use crate::error::DomainError;
use crate::identity::RecordType;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait DomainRecord: Sized + Serialize + DeserializeOwned {
    const RECORD_TYPE: RecordType;
    fn validate(&self) -> Result<(), DomainError>;
}

/// Type-level Domain boundary: a record that has passed semantic validation.
///
/// Raw structs may still implement `Deserialize`. They cannot enter the normal
/// store / application persistence flow without conversion through `certify`
/// or `from_json`. `Validated<T>` does not implement `Deserialize`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Validated<T: DomainRecord> {
    inner: T,
}

impl<T: DomainRecord> Validated<T> {
    pub fn certify(record: T) -> Result<Self, DomainError> {
        record.validate()?;
        Ok(Self { inner: record })
    }

    pub fn as_record(&self) -> &T {
        &self.inner
    }

    pub fn into_record(self) -> T {
        self.inner
    }
}

impl<T: DomainRecord> Serialize for Validated<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

pub fn usable_as_job_input(lifecycle: crate::identity::Lifecycle) -> Result<(), DomainError> {
    match lifecycle {
        crate::identity::Lifecycle::Invalidated => Err(DomainError::new(
            crate::error::ErrorCode::InvalidatedInput,
            "Invalidated versions cannot be used as a new exact Job input",
        )),
        crate::identity::Lifecycle::Draft
        | crate::identity::Lifecycle::Ready
        | crate::identity::Lifecycle::Published => Ok(()),
    }
}
