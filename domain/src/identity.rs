//! Product identity, integrity digests, location evidence, lifecycle, and schema.

use crate::error::{DomainError, ErrorCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current Domain contract / serialization schema. Distinct from Product entity versions.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Draft,
    Ready,
    Published,
    Invalidated,
}

impl Lifecycle {
    pub fn allows_in_place_replace(self) -> bool {
        matches!(self, Lifecycle::Draft)
    }

    pub fn is_frozen(self) -> bool {
        !self.allows_in_place_replace()
    }

    pub fn assert_mutable(self) -> Result<(), DomainError> {
        if self.allows_in_place_replace() {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::ImmutableVersion,
                format!("{self:?} versions cannot be replaced in-place"),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    CharacterAsset,
    CharacterAssetVersion,
    MotionAsset,
    MotionAssetVersion,
    SourceSkeletonReference,
    SkeletonSummary,
    BoneMapping,
    BoneMappingVersion,
    CompatibilityResult,
    RetargetPolicy,
    RetargetPolicyVersion,
    JobSpec,
    WorkerResult,
    QcReport,
    DerivedVariant,
    DerivedVariantVersion,
    PersistenceArtifact,
    PreviewArtifact,
    ExportArtifact,
    BackendExecutionContext,
    PersistenceVerification,
}

pub fn expect_schema_version(version: u32) -> Result<(), DomainError> {
    match version {
        SCHEMA_VERSION => Ok(()),
        0 => Err(DomainError::new(
            ErrorCode::InvalidSchemaVersion,
            "schema_version 0 is not a valid Domain schema",
        )),
        other => Err(DomainError::new(
            ErrorCode::UnsupportedSchemaVersion,
            format!("unsupported schema_version {other}; this Domain implements {SCHEMA_VERSION}"),
        )),
    }
}

pub fn expect_record_type(actual: RecordType, expected: RecordType) -> Result<(), DomainError> {
    if actual == expected {
        Ok(())
    } else {
        Err(DomainError::new(
            ErrorCode::InvalidRecordType,
            format!("expected {expected:?}, got {actual:?}"),
        ))
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/')
        || s.contains('\\')
        || (s.len() >= 2 && s.as_bytes()[0].is_ascii_alphabetic() && s.as_bytes()[1] == b':')
}

fn looks_like_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) && !s.contains('-')
}

pub fn parse_product_uuid(s: &str, label: &str) -> Result<Uuid, DomainError> {
    if looks_like_path(s) {
        return Err(DomainError::new(
            ErrorCode::PathUsedAsId,
            format!("{label} must not be a filesystem path: {s}"),
        ));
    }
    if looks_like_sha256_hex(s) {
        return Err(DomainError::new(
            ErrorCode::DigestUsedAsId,
            format!("{label} must not be a SHA-256 digest: {s}"),
        ));
    }
    let uuid = Uuid::parse_str(s).map_err(|_| {
        DomainError::new(
            ErrorCode::InvalidId,
            format!("{label} is not a UUID: {s}"),
        )
    })?;
    if uuid.is_nil() {
        return Err(DomainError::new(
            ErrorCode::InvalidId,
            format!("{label} must not be the nil UUID"),
        ));
    }
    require_uuid_v7(uuid, label)?;
    Ok(uuid)
}

fn require_uuid_v7(uuid: Uuid, label: &str) -> Result<(), DomainError> {
    if uuid.get_version() != Some(uuid::Version::SortRand) {
        return Err(DomainError::new(
            ErrorCode::UuidVersion,
            format!("{label} must be RFC 9562 UUIDv7, not {uuid}"),
        ));
    }
    Ok(())
}

macro_rules! typed_id {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(Uuid);

        impl $name {
            pub const LABEL: &'static str = $label;

            pub fn generate() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(uuid: Uuid) -> Result<Self, DomainError> {
                if uuid.is_nil() {
                    return Err(DomainError::new(
                        ErrorCode::InvalidId,
                        format!("{} must not be the nil UUID", $label),
                    ));
                }
                require_uuid_v7(uuid, $label)?;
                Ok(Self(uuid))
            }

            pub fn parse(s: &str) -> Result<Self, DomainError> {
                Ok(Self(parse_product_uuid(s, $label)?))
            }

            pub fn uuid(self) -> Uuid {
                self.0
            }

            pub fn canonical(self) -> String {
                self.0.hyphenated().to_string()
            }
        }

        impl TryFrom<String> for $name {
            type Error = DomainError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(&value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.canonical()
            }
        }
    };
}

typed_id!(CharacterAssetId, "CharacterAssetId");
typed_id!(CharacterAssetVersionId, "CharacterAssetVersionId");
typed_id!(MotionAssetId, "MotionAssetId");
typed_id!(MotionAssetVersionId, "MotionAssetVersionId");
typed_id!(SourceSkeletonReferenceId, "SourceSkeletonReferenceId");
typed_id!(SkeletonSummaryId, "SkeletonSummaryId");
typed_id!(BoneMappingId, "BoneMappingId");
typed_id!(BoneMappingVersionId, "BoneMappingVersionId");
typed_id!(CompatibilityResultId, "CompatibilityResultId");
typed_id!(RetargetPolicyId, "RetargetPolicyId");
typed_id!(RetargetPolicyVersionId, "RetargetPolicyVersionId");
typed_id!(JobSpecId, "JobSpecId");
typed_id!(WorkerResultId, "WorkerResultId");
typed_id!(QcReportId, "QcReportId");
typed_id!(DerivedVariantId, "DerivedVariantId");
typed_id!(DerivedVariantVersionId, "DerivedVariantVersionId");
typed_id!(PersistenceArtifactId, "PersistenceArtifactId");
typed_id!(PersistenceArtifactInstanceId, "PersistenceArtifactInstanceId");
typed_id!(PersistenceVerificationId, "PersistenceVerificationId");
typed_id!(PreviewArtifactId, "PreviewArtifactId");
typed_id!(ExportArtifactId, "ExportArtifactId");
typed_id!(BackendExecutionContextId, "BackendExecutionContextId");

/// SHA-256 integrity digest. Never a Product ID.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentDigest {
    sha256: String,
}

impl ContentDigest {
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn parse(hex: &str) -> Result<Self, DomainError> {
        let hex = hex.trim();
        if looks_like_path(hex) {
            return Err(DomainError::new(
                ErrorCode::InvalidDigest,
                "content digest must not be a filesystem path",
            ));
        }
        if hex.len() != 64 || !hex.bytes().all(|b| (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b))
        {
            return Err(DomainError::new(
                ErrorCode::InvalidDigest,
                "content digest must be 64 lowercase hex SHA-256 characters",
            ));
        }
        Ok(Self {
            sha256: hex.to_string(),
        })
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        Self::parse(&self.sha256).map(|_| ())
    }
}

/// Filesystem or other location evidence. Never Product identity.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationEvidence {
    kind: LocationKind,
    value: String,
}

impl LocationEvidence {
    pub fn kind(&self) -> LocationKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationKind {
    FilesystemPathEvidence,
    UriEvidence,
}

impl LocationEvidence {
    pub fn filesystem_path(path: impl Into<String>) -> Result<Self, DomainError> {
        let value = path.into();
        if value.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::InvalidLocation,
                "location evidence must not be empty",
            ));
        }
        Ok(Self {
            kind: LocationKind::FilesystemPathEvidence,
            value,
        })
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.value.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::InvalidLocation,
                "location evidence must not be empty",
            ));
        }
        Ok(())
    }
}

/// Source-local joint key. Evidence, not Product identity.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JointKey(String);

impl JointKey {
    pub fn new(key: impl Into<String>) -> Result<Self, DomainError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::MissingRequiredField,
                "joint key must not be empty",
            ));
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn assert_backend_neutral_text(s: &str, field: &str) -> Result<(), DomainError> {
    let lower = s.to_ascii_lowercase();
    let forbidden = [
        "bpy.",
        "bpy/",
        "bpy.types",
        "bpy.ops",
        "posebone",
        "id_data",
        "datablock",
    ];
    for needle in forbidden {
        if lower.contains(needle) {
            return Err(DomainError::new(
                ErrorCode::BackendNeutralViolation,
                format!("{field} must not carry Blender/Python API identity ({needle})"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_not_an_id() {
        let err = CharacterAssetId::parse(r"C:\assets\hero.fbx").unwrap_err();
        assert_eq!(err.code, ErrorCode::PathUsedAsId);
    }

    #[test]
    fn digest_is_not_an_id() {
        let hex = "a".repeat(64);
        let err = CharacterAssetVersionId::parse(&hex).unwrap_err();
        assert_eq!(err.code, ErrorCode::DigestUsedAsId);
    }

    #[test]
    fn uuid_round_trip() {
        let id = MotionAssetId::generate();
        let parsed = MotionAssetId::parse(&id.canonical()).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn uuid_v4_is_rejected_on_parse() {
        let err = CharacterAssetId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap_err();
        assert_eq!(err.code, ErrorCode::UuidVersion);
    }

    #[test]
    fn uuid_v4_is_rejected_on_from_uuid() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid.get_version(), Some(uuid::Version::Random));
        let err = CharacterAssetId::from_uuid(uuid).unwrap_err();
        assert_eq!(err.code, ErrorCode::UuidVersion);
    }
}
