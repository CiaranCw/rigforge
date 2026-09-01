//! Publication gate: worker success ≠ QC PASS ≠ persistence verification ≠ publication.

use crate::artifacts::DerivedVariantVersion;
use crate::error::{DomainError, ErrorCode};
use crate::graph::{validate_publication_lineage, PublicationEvidence};

pub fn publish_derived_variant(
    evidence: PublicationEvidence<'_>,
    derived: &mut DerivedVariantVersion,
) -> Result<(), DomainError> {
    validate_publication_lineage(&evidence, derived)?;
    let verification = evidence.verification.ok_or_else(|| {
        DomainError::new(
            ErrorCode::PublicationEvidenceMissing,
            "publication requires persistence fresh-reopen and structural verification",
        )
    })?;
    match derived.persistence_verification_id() {
        Some(existing) if existing == verification.id() => {}
        Some(_) => {
            return Err(DomainError::new(
                ErrorCode::GraphMismatch,
                "DerivedVariantVersion.persistence_verification_id must equal the authorizing PersistenceVerification",
            ));
        }
        None => derived.bind_persistence_verification(verification.id())?,
    }
    derived.freeze_published()
}
