//! Backend/build provenance. Not a Product identity component.

use crate::error::{DomainError, ErrorCode};
use crate::identity::{
    assert_backend_neutral_text, expect_record_type, expect_schema_version, BackendExecutionContextId,
    Lifecycle, RecordType, SCHEMA_VERSION,
};
use crate::record::DomainRecord;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendExecutionContext {
    schema_version: u32,
    record_type: RecordType,
    id: BackendExecutionContextId,
    backend_kind: String,
    backend_version: String,
    build: String,
    adapter_version: String,
    execution_policy_version: String,
}

impl BackendExecutionContext {
    pub fn new(
        backend_kind: impl Into<String>,
        backend_version: impl Into<String>,
        build: impl Into<String>,
        adapter_version: impl Into<String>,
        execution_policy_version: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            schema_version: SCHEMA_VERSION,
            record_type: RecordType::BackendExecutionContext,
            id: BackendExecutionContextId::generate(),
            backend_kind: backend_kind.into(),
            backend_version: backend_version.into(),
            build: build.into(),
            adapter_version: adapter_version.into(),
            execution_policy_version: execution_policy_version.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn id(&self) -> BackendExecutionContextId {
        self.id
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        expect_schema_version(self.schema_version)?;
        expect_record_type(self.record_type, RecordType::BackendExecutionContext)?;
        for (field, value) in [
            ("backend_kind", self.backend_kind.as_str()),
            ("backend_version", self.backend_version.as_str()),
            ("build", self.build.as_str()),
            ("adapter_version", self.adapter_version.as_str()),
            (
                "execution_policy_version",
                self.execution_policy_version.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                return Err(DomainError::new(
                    ErrorCode::MissingRequiredField,
                    format!("{field} is required"),
                ));
            }
            assert_backend_neutral_text(value, field)?;
        }
        Ok(())
    }
}

impl DomainRecord for BackendExecutionContext {
    const RECORD_TYPE: RecordType = RecordType::BackendExecutionContext;
    fn validate(&self) -> Result<(), DomainError> {
        BackendExecutionContext::validate(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedCapability {
    PersistenceArtifact,
    PreviewPayload,
    ExportArtifact,
}

pub fn assert_lifecycle_transition(from: Lifecycle, to: Lifecycle) -> Result<(), DomainError> {
    match (from, to) {
        (Lifecycle::Draft, Lifecycle::Ready)
        | (Lifecycle::Draft, Lifecycle::Published)
        | (Lifecycle::Ready, Lifecycle::Published)
        | (Lifecycle::Draft, Lifecycle::Invalidated)
        | (Lifecycle::Ready, Lifecycle::Invalidated)
        | (Lifecycle::Published, Lifecycle::Invalidated) => Ok(()),
        (a, b) if a == b => Ok(()),
        (a, b) => Err(DomainError::new(
            ErrorCode::InvalidLifecycle,
            format!("illegal lifecycle transition {a:?} -> {b:?}"),
        )),
    }
}
