//! JSON serialization. Validated ingress is the public Domain boundary.

use crate::error::{DomainError, ErrorCode};
use crate::record::{DomainRecord, Validated};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn to_json<T: Serialize>(value: &T) -> Result<String, DomainError> {
    serde_json::to_string_pretty(value).map_err(|e| {
        DomainError::new(ErrorCode::InvalidJson, format!("serialize failed: {e}"))
    })
}

pub(crate) fn deserialize_untrusted<T: DeserializeOwned>(text: &str) -> Result<T, DomainError> {
    serde_json::from_str::<T>(text).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown field") {
            DomainError::new(ErrorCode::UnknownField, msg)
        } else if msg.contains("unknown variant") {
            DomainError::new(ErrorCode::UnknownField, msg)
        } else if msg.contains("must be RFC 9562 UUIDv7") {
            DomainError::new(ErrorCode::UuidVersion, msg)
        } else {
            DomainError::new(ErrorCode::InvalidJson, msg)
        }
    })
}

/// Public Domain ingress: deserialize, then semantic validate.
pub fn from_json_validated<T: DomainRecord>(text: &str) -> Result<T, DomainError> {
    let value = deserialize_untrusted::<T>(text)?;
    value.validate()?;
    Ok(value)
}

/// Validated Domain ingress: deserialize + semantic validate → `Validated<T>`.
pub fn ingest_validated<T: DomainRecord>(text: &str) -> Result<Validated<T>, DomainError> {
    Validated::certify(from_json_validated(text)?)
}

pub fn round_trip<T: DomainRecord>(value: &T) -> Result<T, DomainError> {
    value.validate()?;
    let json = to_json(value)?;
    from_json_validated(&json)
}
