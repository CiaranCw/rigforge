//! Source-file evidence. Paths and filenames are not Product identity.

use crate::error::{DomainError, ErrorCode};
use crate::identity::{ContentDigest, LocationEvidence};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactEvidence {
    location: LocationEvidence,
    digest: ContentDigest,
    size_bytes: u64,
    observed_media_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ingested_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    adapter_evidence: Option<String>,
}

impl SourceArtifactEvidence {
    pub fn new(
        location: LocationEvidence,
        digest: ContentDigest,
        size_bytes: u64,
        observed_media_type: impl Into<String>,
        ingested_at: Option<String>,
        adapter_evidence: Option<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            location,
            digest,
            size_bytes,
            observed_media_type: observed_media_type.into(),
            ingested_at,
            adapter_evidence,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        self.location.validate()?;
        self.digest.validate()?;
        if self.observed_media_type.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::MissingRequiredField,
                "observed_media_type is required",
            ));
        }
        Ok(())
    }
}
