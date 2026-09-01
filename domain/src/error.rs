//! Domain validation and contract errors.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidId,
    PathUsedAsId,
    DigestUsedAsId,
    InvalidDigest,
    InvalidLocation,
    InvalidSchemaVersion,
    UnsupportedSchemaVersion,
    InvalidRecordType,
    MissingRequiredField,
    MissingProvenance,
    ImmutableVersion,
    InvalidLifecycle,
    BackendNeutralViolation,
    WorkerNotPublicationAuthority,
    QcSubjectMismatch,
    PreviewBinding,
    ArtifactProductConfusion,
    TimeDomainInvalid,
    CompatibilityCollapsed,
    MappingTooThin,
    UnknownField,
    InvalidJson,
    StoreContract,
    UuidVersion,
    GraphMismatch,
    PublicationEvidenceMissing,
    VerificationFailed,
    SubjectUnion,
    CompatibilityContradiction,
    InvalidatedInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}
