//! Product Mapping workflow: propose, explicit accept, override, persist.

use rigforge_domain::{
    BoneMapping, BoneMappingEntry, BoneMappingVersion, CompatibilityResult, JointKey,
    JointParticipation, JointRef, MappingReviewKind, MappingReviewProvenance, SkeletonSubjectKind,
    SkeletonSummary, UnmappedDisposition, UnmappedJoint, Validated,
};

use crate::candidates::{generate_mapping_proposal, MappingAssistProfile, MappingProposal};
use crate::capability::WorkerCapabilityProfile;
use crate::error::AppError;
use crate::preflight::evaluate_compatibility;
use crate::skeleton::SkeletonEvidenceProvider;
use crate::Application;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingWorkflowSnapshot {
    pub character_version_id: String,
    pub motion_version_id: String,
    pub source_skeleton_id: String,
    pub source_summary_id: Option<String>,
    pub target_summary_id: Option<String>,
    pub mapping_id: Option<String>,
    pub mapping_version_id: Option<String>,
    pub proposal_confirmation_required: bool,
    pub compatibility_id: Option<String>,
    pub compatibility_summary: Option<String>,
    pub transfer_available: bool,
}

impl MappingWorkflowSnapshot {
    pub fn transfer_unavailable_reason() -> &'static str {
        "Transfer is not eligible for this CompatibilityResult"
    }

    pub fn preview_unavailable_reason() -> &'static str {
        "Preview requires an exact Product version and a validated PreviewArtifact"
    }
}

impl Application {
    pub fn store_skeleton_summary(
        &mut self,
        summary: SkeletonSummary,
    ) -> Result<Validated<SkeletonSummary>, AppError> {
        let validated = Validated::certify(summary)?;
        self.catalog.put_validated(&validated)?;
        Ok(validated)
    }

    pub fn load_skeleton_summary(
        &self,
        id: &str,
    ) -> Result<Validated<SkeletonSummary>, AppError> {
        self.catalog.load_skeleton_summary(id)
    }

    pub fn inspect_and_store_character_summary<I: SkeletonEvidenceProvider>(
        &mut self,
        character_version_id: &str,
        inspector: &I,
    ) -> Result<Validated<SkeletonSummary>, AppError> {
        let character = self
            .catalog
            .load_character_version(character_version_id)?
            .into_record();
        let summary = inspector.inspect_character(&character)?;
        if summary.subject_character_version_id() != Some(character.id()) {
            return Err(AppError::Catalog(
                "inspected character summary is not bound to the exact CharacterAssetVersion".into(),
            ));
        }
        self.store_skeleton_summary(summary)
    }

    pub fn inspect_and_store_source_summary<I: SkeletonEvidenceProvider>(
        &mut self,
        source_skeleton_id: &str,
        inspector: &I,
        inspect_from_motion_version_id: &str,
    ) -> Result<Validated<SkeletonSummary>, AppError> {
        let source = self
            .catalog
            .load_source_skeleton(source_skeleton_id)?
            .into_record();
        let motion = self
            .catalog
            .load_motion_version(inspect_from_motion_version_id)?
            .into_record();
        if motion.source_skeleton_ref_id() != source.id() {
            return Err(AppError::Catalog(
                "inspect Motion does not reference this SourceSkeletonReference".into(),
            ));
        }
        let summary = inspector.inspect_source_skeleton(&source, motion.source())?;
        if summary.subject_source_skeleton_ref_id() != Some(source.id()) {
            return Err(AppError::Catalog(
                "inspected source summary is not bound to the exact SourceSkeletonReference".into(),
            ));
        }
        self.store_skeleton_summary(summary)
    }

    pub fn propose_mapping(
        &self,
        source_summary_id: &str,
        target_summary_id: &str,
        profile: MappingAssistProfile,
    ) -> Result<MappingProposal, AppError> {
        let source = self.catalog.load_skeleton_summary(source_summary_id)?;
        let target = self.catalog.load_skeleton_summary(target_summary_id)?;
        if source.as_record().subject_kind() != SkeletonSubjectKind::SourceSkeletonReference
            || target.as_record().subject_kind() != SkeletonSubjectKind::CharacterAssetVersion
        {
            return Err(AppError::Catalog(
                "propose_mapping requires source SkeletonSummary for SourceSkeletonReference and target SkeletonSummary for CharacterAssetVersion".into(),
            ));
        }
        generate_mapping_proposal(source.as_record(), target.as_record(), profile)
    }

    pub fn store_mapping_draft(
        &mut self,
        display_name: impl Into<String>,
        character_version_id: &str,
        source_skeleton_id: &str,
        proposal: &MappingProposal,
        source_summary_id: &str,
        target_summary_id: &str,
    ) -> Result<(Validated<BoneMapping>, Validated<BoneMappingVersion>), AppError> {
        let character = self.catalog.load_character_version(character_version_id)?;
        let source = self.catalog.load_source_skeleton(source_skeleton_id)?;
        let source_summary = self.catalog.load_skeleton_summary(source_summary_id)?;
        let target_summary = self.catalog.load_skeleton_summary(target_summary_id)?;
        let source_rec = source_summary.as_record();
        let target_rec = target_summary.as_record();
        if source_rec.subject_kind() != SkeletonSubjectKind::SourceSkeletonReference
            || target_rec.subject_kind() != SkeletonSubjectKind::CharacterAssetVersion
        {
            return Err(AppError::Catalog(
                "mapping draft rejected: SkeletonSummary sides are reversed or the wrong subject kind".into(),
            ));
        }
        if source_rec.subject_source_skeleton_ref_id() != Some(source.as_record().id()) {
            return Err(AppError::Catalog(
                "mapping draft rejected: source SkeletonSummary is not bound to the selected SourceSkeletonReference".into(),
            ));
        }
        if target_rec.subject_character_version_id() != Some(character.as_record().id()) {
            return Err(AppError::Catalog(
                "mapping draft rejected: target SkeletonSummary is not bound to the selected CharacterAssetVersion".into(),
            ));
        }
        let mut mapping = BoneMapping::new(display_name)?;
        let review = MappingReviewProvenance::with_kind(
            false,
            Some("deterministic candidate generation".into()),
            proposal
                .ambiguities
                .iter()
                .map(|a| format!("{}: {}", a.source_key, a.reason))
                .collect(),
            MappingReviewKind::AutomaticCandidate,
        )?;
        let mut version = BoneMappingVersion::draft(
            mapping.id(),
            character.as_record().id(),
            source.as_record().id(),
            proposal.to_domain_entries()?,
            review,
        )?;
        version.mark_generated_from_candidates()?;
        version.set_unmapped(proposal.unmapped_source.clone(), proposal.unmapped_target.clone())?;
        version.bind_skeleton_summaries(
            source_rec.id(),
            target_rec.id(),
        )?;
        mapping.bind_draft(version.id());
        let mapping = Validated::certify(mapping)?;
        let version = Validated::certify(version)?;
        self.catalog.put_validated_pair(&mapping, &version)?;
        Ok((mapping, version))
    }

    pub fn accept_mapping_version(
        &mut self,
        mapping_id: &str,
        mapping_version_id: &str,
    ) -> Result<Validated<BoneMappingVersion>, AppError> {
        let mut logical = self.catalog.load_bone_mapping(mapping_id)?.into_record();
        let mut version = self
            .catalog
            .load_mapping_version(mapping_version_id)?
            .into_record();
        if version.mapping_id() != logical.id() {
            return Err(AppError::Catalog(
                "mapping version does not belong to the logical BoneMapping".into(),
            ));
        }
        if let Some(sid) = version.source_skeleton_summary_id() {
            let summary = self.catalog.load_skeleton_summary(&sid.canonical())?;
            if summary.as_record().subject_source_skeleton_ref_id()
                != Some(version.source_skeleton_ref_id())
            {
                return Err(AppError::Catalog(
                    "cannot accept Mapping bound to a source SkeletonSummary for a different SourceSkeletonReference".into(),
                ));
            }
        }
        if let Some(tid) = version.target_skeleton_summary_id() {
            let summary = self.catalog.load_skeleton_summary(&tid.canonical())?;
            if summary.as_record().subject_character_version_id()
                != Some(version.target_character_version_id())
            {
                return Err(AppError::Catalog(
                    "cannot accept Mapping bound to a target SkeletonSummary for a different CharacterAssetVersion".into(),
                ));
            }
        }
        let kind = version.derived_accepted_kind();
        version.replace_entries_and_review(
            version.entries().to_vec(),
            MappingReviewProvenance::with_kind(
                true,
                Some("explicit Product acceptance".into()),
                version.review().ambiguities().to_vec(),
                kind,
            )?,
        )?;
        version.publish()?;
        logical.bind_published(version.id());
        let logical = Validated::certify(logical)?;
        let version = Validated::certify(version)?;
        self.catalog.put_validated_pair(&logical, &version)?;
        Ok(version)
    }

    pub fn override_mapping_entry(
        &mut self,
        mapping_version_id: &str,
        source_key: &str,
        target_key: Option<&str>,
        reason: &str,
    ) -> Result<Validated<BoneMappingVersion>, AppError> {
        let mut version = self
            .catalog
            .load_mapping_version(mapping_version_id)?
            .into_record();
        let src = JointKey::new(source_key)?;
        let existing = version
            .entries()
            .iter()
            .find(|e| e.source().joint_key() == &src);
        let participation = existing
            .map(|e| e.participation())
            .unwrap_or(JointParticipation::Required);
        let role_profile = existing.and_then(|e| e.role_profile().map(str::to_string));
        let disposition = disposition_for_unmap(&version, &src);
        let mut entries: Vec<BoneMappingEntry> = version
            .entries()
            .iter()
            .filter(|e| e.source().joint_key() != &src)
            .cloned()
            .collect();
        let mut unmapped_source = version.unmapped_source().to_vec();
        unmapped_source.retain(|u| u.joint_key() != &src);
        if let Some(target_key) = target_key {
            let dst = JointKey::new(target_key)?;
            entries.retain(|e| e.target().joint_key() != &dst);
            entries.push(BoneMappingEntry::new(
                JointRef::source(src),
                JointRef::target(dst),
                participation,
                role_profile,
                format!("user override: {reason}"),
            )?);
        } else {
            unmapped_source.push(UnmappedJoint::new(
                rigforge_domain::SkeletonSide::Source,
                src,
                format!("user marked unmapped: {reason}"),
                disposition,
            )?);
        }
        version.replace_entries_and_review(
            entries,
            MappingReviewProvenance::with_kind(
                false,
                Some("user override of mapping candidates".into()),
                version.review().ambiguities().to_vec(),
                MappingReviewKind::AutomaticCandidate,
            )?,
        )?;
        version.mark_user_modified()?;
        version.set_unmapped(unmapped_source, version.unmapped_target().to_vec())?;
        let version = Validated::certify(version)?;
        self.catalog.put_validated(&version)?;
        Ok(version)
    }

    pub fn run_compatibility_preflight(
        &mut self,
        character_version_id: &str,
        motion_version_id: &str,
        mapping_version_id: &str,
        policy_version_id: &str,
        capability: &WorkerCapabilityProfile,
    ) -> Result<Validated<CompatibilityResult>, AppError> {
        let character = self.catalog.load_character_version(character_version_id)?;
        let motion = self.catalog.load_motion_version(motion_version_id)?;
        let mapping = self.catalog.load_mapping_version(mapping_version_id)?;
        let policy = self.catalog.load_policy_version(policy_version_id)?;
        let source = self.catalog.load_source_skeleton(
            &mapping.as_record().source_skeleton_ref_id().canonical(),
        )?;
        let source_summary = mapping
            .as_record()
            .source_skeleton_summary_id()
            .map(|id| self.catalog.load_skeleton_summary(&id.canonical()))
            .transpose()?;
        let target_summary = mapping
            .as_record()
            .target_skeleton_summary_id()
            .map(|id| self.catalog.load_skeleton_summary(&id.canonical()))
            .transpose()?;
        let result = evaluate_compatibility(
            character.as_record(),
            motion.as_record(),
            source.as_record(),
            mapping.as_record(),
            policy.as_record(),
            source_summary.as_ref().map(|s| s.as_record()),
            target_summary.as_ref().map(|s| s.as_record()),
            capability,
        )?;
        let validated = Validated::certify(result)?;
        self.catalog.put_validated(&validated)?;
        Ok(validated)
    }

    pub fn load_compatibility_result(
        &self,
        id: &str,
    ) -> Result<Validated<CompatibilityResult>, AppError> {
        self.catalog.load_compatibility_result(id)
    }

    pub fn latest_compatibility_for_exact_set(
        &self,
        character_version_id: &str,
        motion_version_id: &str,
        mapping_version_id: &str,
        policy_version_id: &str,
    ) -> Result<Option<Validated<CompatibilityResult>>, AppError> {
        self.catalog.latest_compatibility_for_exact_set(
            character_version_id,
            motion_version_id,
            mapping_version_id,
            policy_version_id,
        )
    }

    pub fn mapping_workflow_snapshot(
        &self,
        character_version_id: Option<&str>,
        motion_version_id: Option<&str>,
    ) -> Result<MappingWorkflowSnapshot, AppError> {
        let mut snap = MappingWorkflowSnapshot {
            character_version_id: character_version_id.unwrap_or("").to_string(),
            motion_version_id: motion_version_id.unwrap_or("").to_string(),
            source_skeleton_id: String::new(),
            source_summary_id: None,
            target_summary_id: None,
            mapping_id: None,
            mapping_version_id: None,
            proposal_confirmation_required: false,
            compatibility_id: None,
            compatibility_summary: None,
            transfer_available: false,
        };
        if let Some(mid) = motion_version_id {
            let motion = self.catalog.load_motion_version(mid)?;
            snap.source_skeleton_id = motion.as_record().source_skeleton_ref_id().canonical();
        }
        Ok(snap)
    }

    pub fn load_mapping_version(
        &self,
        mapping_version_id: &str,
    ) -> Result<Validated<BoneMappingVersion>, AppError> {
        self.catalog.load_mapping_version(mapping_version_id)
    }

    /// Published Mapping bound to the exact selected Character version and the
    /// selected Motion's Source Skeleton. Unrelated first-list records are ignored.
    ///
    /// If several published versions match, the greatest `BoneMappingVersionId`
    /// (UUIDv7, later-created) wins. Ties break on logical Mapping id.
    pub fn published_mapping_for_selection(
        &self,
        character_version_id: &str,
        motion_version_id: &str,
    ) -> Result<Option<(String, Validated<BoneMappingVersion>)>, AppError> {
        let character = self.catalog.load_character_version(character_version_id)?;
        let motion = self.catalog.load_motion_version(motion_version_id)?;
        let wanted_character = character.as_record().id();
        let wanted_source = motion.as_record().source_skeleton_ref_id();
        let mut matches = Vec::new();
        for item in self.list_mappings()? {
            let Some(version_id) = item.published_version_id.as_deref() else {
                continue;
            };
            let version = self.catalog.load_mapping_version(version_id)?;
            let rec = version.as_record();
            if rec.lifecycle() != rigforge_domain::Lifecycle::Published {
                continue;
            }
            if rec.target_character_version_id() != wanted_character {
                continue;
            }
            if rec.source_skeleton_ref_id() != wanted_source {
                continue;
            }
            matches.push((item.logical_id, version));
        }
        matches.sort_by(|a, b| {
            b.1.as_record()
                .id()
                .canonical()
                .cmp(&a.1.as_record().id().canonical())
                .then_with(|| a.0.cmp(&b.0))
        });
        Ok(matches.into_iter().next())
    }
}

fn disposition_for_unmap(version: &BoneMappingVersion, src: &JointKey) -> UnmappedDisposition {
    if let Some(entry) = version
        .entries()
        .iter()
        .find(|e| e.source().joint_key() == src)
    {
        return match entry.participation() {
            JointParticipation::Required => UnmappedDisposition::Blocking,
            JointParticipation::Optional => UnmappedDisposition::Optional,
        };
    }
    if let Some(existing) = version
        .unmapped_source()
        .iter()
        .find(|u| u.joint_key() == src)
    {
        return existing.disposition();
    }
    UnmappedDisposition::Blocking
}
