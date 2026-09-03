//! RigForge V1-2 Workbench shell.
//!
//! GUI → Application / Catalog / Orchestrator → WorkerPort.
//! Preview generation uses a hidden backend; this process does not require
//! the user to open Blender. Viewer library/payload are V1-6 implementation
//! details, not Product authority.

use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;
use rigforge_app::rigforge_domain::{
    BoneMappingVersion, CompatibilityResult, Lifecycle, VerificationOutcome,
};
use rigforge_app::{
    Application, AssetListItem, CharacterSourceInspection, JobRunState, MappingAssistProfile,
    MotionSourceInspection, PreviewGenerationRequest, PreviewSubject, SkeletonEvidenceProvider,
    TerminalOutcome, TransferAuthorization, TransferOutcome, TransferOutcomeKind,
    WorkerCapabilityProfile, INGEST_NO_CLIPS,
};
use rigforge_blender_worker::BlenderSkeletonInspector;

pub mod file_pick;
pub mod ingest;
pub mod long_op;
pub mod native_exec;
pub mod preview_host;

pub use long_op::{format_elapsed, QcAcquireFn, ReopenAcquireFn, TransferPhase};

use crate::file_pick::FilePickState;
use crate::ingest::{IngestKind, IngestResult};
use crate::long_op::{LongOpSession, WaitMsg};

/// Preview embedding boundary. Viewer library and payload are V1-6 implementation
/// details, not Product identity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PreviewEmbeddingSlot {
    pub occupied: bool,
}

impl PreviewEmbeddingSlot {
    pub fn viewer_library() -> Option<&'static str> {
        Some(preview_host::VIEWER_LIBRARY)
    }

    pub fn payload_format() -> Option<&'static str> {
        Some(preview_host::PAYLOAD_FORMAT)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobStatusView {
    pub run_id: String,
    pub job_spec_id: String,
    pub state: String,
}

pub struct WorkbenchApp {
    characters: Vec<AssetListItem>,
    motions: Vec<AssetListItem>,
    derived: Vec<AssetListItem>,
    selected_character_version: Option<String>,
    selected_motion_version: Option<String>,
    selected_derived_variant_version: Option<String>,
    jobs: Vec<JobStatusView>,
    preview: PreviewEmbeddingSlot,
    preview_status: String,
    preview_valid: bool,
    mapping_entries: Vec<String>,
    unmapped: Vec<String>,
    ambiguities: Vec<String>,
    compatibility_dimensions: Vec<(String, String)>,
    compatibility_summary: Option<String>,
    mapping_id: Option<String>,
    mapping_version_id: Option<String>,
    accepted_mapping_version_id: Option<String>,
    compatibility_id: Option<String>,
    compatibility_notes: Vec<String>,
    warnings_acknowledged: bool,
    transfer_auth: Option<TransferAuthorization>,
    transfer_phase: Option<String>,
    long_op: Option<LongOpSession>,
    qc_acquire: Option<QcAcquireFn>,
    reopen_acquire: Option<ReopenAcquireFn>,
    derived_variant_id: Option<String>,
    derived_variant_version_id: Option<String>,
    publication_state: Option<String>,
    qc_verdict: Option<String>,
    persistence_verification: Option<String>,
    workflow_status: Option<String>,
    selected_policy_version_id: Option<String>,
    preview_host: Option<preview_host::PreviewHost>,
    character_pick: FilePickState,
    motion_pick: FilePickState,
    character_inspection: Option<CharacterSourceInspection>,
    motion_inspection: Option<MotionSourceInspection>,
    motion_selected_clip: Option<String>,
    character_request_id: u64,
    motion_request_id: u64,
    character_wait: Option<Receiver<IngestResult>>,
    motion_wait: Option<Receiver<IngestResult>>,
    character_inspecting: bool,
    motion_inspecting: bool,
    character_ingest_status: Option<String>,
    motion_ingest_status: Option<String>,
}

impl WorkbenchApp {
    pub fn empty() -> Self {
        Self {
            characters: Vec::new(),
            motions: Vec::new(),
            derived: Vec::new(),
            selected_character_version: None,
            selected_motion_version: None,
            selected_derived_variant_version: None,
            jobs: Vec::new(),
            preview: PreviewEmbeddingSlot::default(),
            preview_status: "no Preview generated".into(),
            preview_valid: false,
            mapping_entries: Vec::new(),
            unmapped: Vec::new(),
            ambiguities: Vec::new(),
            compatibility_dimensions: Vec::new(),
            compatibility_summary: None,
            mapping_id: None,
            mapping_version_id: None,
            accepted_mapping_version_id: None,
            compatibility_id: None,
            compatibility_notes: Vec::new(),
            warnings_acknowledged: false,
            transfer_auth: None,
            transfer_phase: None,
            long_op: None,
            qc_acquire: None,
            reopen_acquire: None,
            derived_variant_id: None,
            derived_variant_version_id: None,
            publication_state: None,
            qc_verdict: None,
            persistence_verification: None,
            workflow_status: None,
            selected_policy_version_id: None,
            preview_host: None,
            character_pick: FilePickState::default(),
            motion_pick: FilePickState::default(),
            character_inspection: None,
            motion_inspection: None,
            motion_selected_clip: None,
            character_request_id: 0,
            motion_request_id: 0,
            character_wait: None,
            motion_wait: None,
            character_inspecting: false,
            motion_inspecting: false,
            character_ingest_status: None,
            motion_ingest_status: None,
        }
    }

    pub fn from_application(app: &Application) -> Result<Self, rigforge_app::AppError> {
        let mut shell = Self::empty();
        shell.characters = app.list_characters()?;
        shell.motions = app.list_motions()?;
        shell.derived = app.list_derived_variants()?;
        shell.jobs = app
            .catalog()
            .list_job_runs()?
            .into_iter()
            .map(|run| JobStatusView {
                run_id: run.run_id,
                job_spec_id: run.job_spec_id,
                state: run.state.as_db_str().to_string(),
            })
            .collect();
        if let Some(item) = shell.characters.first() {
            shell.selected_character_version = item.published_version_id.clone();
        }
        if let Some(item) = shell.motions.first() {
            shell.selected_motion_version = item.published_version_id.clone();
        }
        if let Some(item) = shell.derived.first() {
            shell.selected_derived_variant_version = item.published_version_id.clone();
        }
        shell.bind_published_mapping_for_current_selection(app)?;
        Ok(shell)
    }

    pub fn character_pick(&self) -> &FilePickState {
        &self.character_pick
    }

    pub fn motion_pick(&self) -> &FilePickState {
        &self.motion_pick
    }

    pub fn character_inspection(&self) -> Option<&CharacterSourceInspection> {
        self.character_inspection.as_ref()
    }

    pub fn motion_inspection(&self) -> Option<&MotionSourceInspection> {
        self.motion_inspection.as_ref()
    }

    pub fn motion_selected_clip(&self) -> Option<&str> {
        self.motion_selected_clip.as_deref()
    }

    pub fn character_ingest_status(&self) -> Option<&str> {
        self.character_ingest_status.as_deref()
    }

    pub fn motion_ingest_status(&self) -> Option<&str> {
        self.motion_ingest_status.as_deref()
    }

    pub fn character_inspecting(&self) -> bool {
        self.character_inspecting
    }

    pub fn motion_inspecting(&self) -> bool {
        self.motion_inspecting
    }

    pub fn apply_character_selection(&mut self, picked: Option<PathBuf>) -> bool {
        if crate::file_pick::apply_picked_path(&mut self.character_pick, picked) {
            self.character_inspection = None;
            self.character_ingest_status = None;
            self.character_wait = None;
            self.character_inspecting = false;
            self.character_request_id = self.character_request_id.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn apply_motion_selection(&mut self, picked: Option<PathBuf>) -> bool {
        if crate::file_pick::apply_picked_path(&mut self.motion_pick, picked) {
            self.motion_inspection = None;
            self.motion_selected_clip = None;
            self.motion_ingest_status = None;
            self.motion_wait = None;
            self.motion_inspecting = false;
            self.motion_request_id = self.motion_request_id.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn begin_character_inspect(&mut self, ctx: &egui::Context) {
        if self.mutation_locked() {
            return;
        }
        let Some(path) = self.character_pick.path.clone() else {
            return;
        };
        self.character_inspecting = true;
        self.character_ingest_status = Some("Inspecting Character…".into());
        match crate::ingest::spawn_source_inspect(
            self.character_request_id,
            IngestKind::Character,
            path,
            ctx.clone(),
        ) {
            Ok(rx) => self.character_wait = Some(rx),
            Err(err) => {
                self.character_inspecting = false;
                self.character_ingest_status = Some(err.to_string());
            }
        }
    }

    pub fn begin_motion_inspect(&mut self, ctx: &egui::Context) {
        if self.mutation_locked() {
            return;
        }
        let Some(path) = self.motion_pick.path.clone() else {
            return;
        };
        self.motion_inspecting = true;
        self.motion_ingest_status = Some("Inspecting Motion…".into());
        match crate::ingest::spawn_source_inspect(
            self.motion_request_id,
            IngestKind::Motion,
            path,
            ctx.clone(),
        ) {
            Ok(rx) => self.motion_wait = Some(rx),
            Err(err) => {
                self.motion_inspecting = false;
                self.motion_ingest_status = Some(err.to_string());
            }
        }
    }

    pub fn apply_character_inspect_result(
        &mut self,
        request_id: u64,
        outcome: Result<CharacterSourceInspection, String>,
    ) {
        if request_id != self.character_request_id {
            return;
        }
        self.character_inspecting = false;
        match outcome {
            Ok(inspection) => {
                self.character_inspection = Some(inspection);
                self.character_ingest_status = Some("Character source is ready.".into());
            }
            Err(err) => {
                self.character_inspection = None;
                self.character_ingest_status = Some(err);
            }
        }
    }

    pub fn apply_motion_inspect_result(
        &mut self,
        request_id: u64,
        outcome: Result<MotionSourceInspection, String>,
    ) {
        if request_id != self.motion_request_id {
            return;
        }
        self.motion_inspecting = false;
        match outcome {
            Ok(inspection) => {
                let usable: Vec<String> = inspection
                    .usable_clips()
                    .into_iter()
                    .map(|clip| clip.clip_identity.clone())
                    .collect();
                let usable_count = usable.len();
                self.motion_selected_clip = match usable_count {
                    1 => usable.into_iter().next(),
                    _ => None,
                };
                self.motion_ingest_status = Some(if usable_count == 0 {
                    rigforge_app::motion_ingest_failure_message(&inspection)
                        .unwrap_or(INGEST_NO_CLIPS)
                        .to_string()
                } else if usable_count > 1 {
                    "Multiple animation clips were found. Choose one clip.".into()
                } else {
                    "Motion source is ready.".into()
                });
                self.motion_inspection = Some(inspection);
            }
            Err(err) => {
                self.motion_inspection = None;
                self.motion_selected_clip = None;
                self.motion_ingest_status = Some(err);
            }
        }
    }

    pub fn poll_ingest(&mut self, ctx: &egui::Context) {
        if self.character_inspecting || self.motion_inspecting {
            ctx.request_repaint();
        }
        if let Some(rx) = self.character_wait.take() {
            match rx.try_recv() {
                Ok(IngestResult::Character {
                    request_id,
                    outcome,
                }) => self.apply_character_inspect_result(request_id, outcome),
                Ok(_) => {}
                Err(TryRecvError::Empty) => self.character_wait = Some(rx),
                Err(TryRecvError::Disconnected) => {
                    self.character_inspecting = false;
                    if self.character_inspection.is_none() {
                        self.character_ingest_status =
                            Some(rigforge_app::INGEST_INSPECT_FAILED.into());
                    }
                }
            }
        }
        if let Some(rx) = self.motion_wait.take() {
            match rx.try_recv() {
                Ok(IngestResult::Motion {
                    request_id,
                    outcome,
                }) => self.apply_motion_inspect_result(request_id, outcome),
                Ok(_) => {}
                Err(TryRecvError::Empty) => self.motion_wait = Some(rx),
                Err(TryRecvError::Disconnected) => {
                    self.motion_inspecting = false;
                    if self.motion_inspection.is_none() {
                        self.motion_ingest_status =
                            Some(rigforge_app::INGEST_INSPECT_FAILED.into());
                    }
                }
            }
        }
    }

    pub fn character_can_add(&self) -> bool {
        !self.mutation_locked()
            && !self.character_inspecting
            && self.character_inspection.is_some()
            && !self.character_pick.display_name.trim().is_empty()
    }

    pub fn motion_can_add(&self) -> bool {
        !self.mutation_locked()
            && !self.motion_inspecting
            && self.motion_inspection.is_some()
            && self.motion_selected_clip.is_some()
            && !self.motion_pick.display_name.trim().is_empty()
    }

    /// Bind a Published Mapping only when it matches the current Character and
    /// the selected Motion's Source Skeleton. No silent unrelated fallback.
    pub fn bind_published_mapping_for_current_selection(
        &mut self,
        app: &Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.mapping_id = None;
        self.mapping_version_id = None;
        self.accepted_mapping_version_id = None;
        self.mapping_entries.clear();
        self.unmapped.clear();
        self.ambiguities.clear();
        let (Some(character_version_id), Some(motion_version_id)) = (
            self.selected_character_version.as_deref(),
            self.selected_motion_version.as_deref(),
        ) else {
            return Ok(());
        };
        if let Some((logical_id, version)) =
            app.published_mapping_for_selection(character_version_id, motion_version_id)?
        {
            self.mapping_id = Some(logical_id);
            self.apply_product_mapping(version.as_record());
        }
        Ok(())
    }

    pub fn requires_network() -> bool {
        false
    }

    pub fn requires_blender() -> bool {
        false
    }

    pub fn preview_slot(&self) -> &PreviewEmbeddingSlot {
        &self.preview
    }

    pub fn selected_character_version(&self) -> Option<&str> {
        self.selected_character_version.as_deref()
    }

    pub fn selected_motion_version(&self) -> Option<&str> {
        self.selected_motion_version.as_deref()
    }

    pub fn selected_derived_variant_version(&self) -> Option<&str> {
        self.selected_derived_variant_version.as_deref()
    }

    pub fn preview_status(&self) -> &str {
        &self.preview_status
    }

    pub fn preview_valid(&self) -> bool {
        self.preview_valid
    }

    pub fn select_character_version(&mut self, version_id: impl Into<String>) {
        if self.mutation_locked() {
            return;
        }
        self.selected_character_version = Some(version_id.into());
        self.invalidate_selection_bound_workflow_state();
    }

    pub fn select_motion_version(&mut self, version_id: impl Into<String>) {
        if self.mutation_locked() {
            return;
        }
        self.selected_motion_version = Some(version_id.into());
        self.invalidate_selection_bound_workflow_state();
    }

    pub fn select_derived_variant_version(&mut self, version_id: impl Into<String>) {
        self.selected_derived_variant_version = Some(version_id.into());
        self.clear_preview_presentation();
    }

    /// Workbench-local derived state for the current Character + Motion pair.
    /// Does not mutate durable Catalog history.
    fn invalidate_selection_bound_workflow_state(&mut self) {
        self.mapping_id = None;
        self.mapping_version_id = None;
        self.accepted_mapping_version_id = None;
        self.mapping_entries.clear();
        self.unmapped.clear();
        self.ambiguities.clear();
        self.compatibility_id = None;
        self.compatibility_summary = None;
        self.compatibility_dimensions.clear();
        self.compatibility_notes.clear();
        self.warnings_acknowledged = false;
        self.transfer_auth = None;
        self.transfer_phase = None;
        self.derived_variant_id = None;
        self.derived_variant_version_id = None;
        self.publication_state = None;
        self.qc_verdict = None;
        self.persistence_verification = None;
        self.clear_preview_presentation();
    }

    /// Test helper: construct a mismatched selection without clearing retained Compatibility.
    #[doc(hidden)]
    pub fn force_selection_without_invalidation_for_test(
        &mut self,
        character_version_id: impl Into<String>,
        motion_version_id: impl Into<String>,
    ) {
        self.selected_character_version = Some(character_version_id.into());
        self.selected_motion_version = Some(motion_version_id.into());
    }

    fn clear_preview_presentation(&mut self) {
        self.preview_host = None;
        self.preview.occupied = false;
        self.preview_valid = false;
        self.preview_status = "selection changed; previous Preview is not valid for this Product version".into();
    }

    pub fn select_policy_version(&mut self, version_id: impl Into<String>) {
        if self.mutation_locked() {
            return;
        }
        self.selected_policy_version_id = Some(version_id.into());
    }

    pub fn selected_policy_version_id(&self) -> Option<&str> {
        self.selected_policy_version_id.as_deref()
    }

    pub fn replace_preview_host(
        &mut self,
        session: &rigforge_app::PreviewSession,
    ) -> Result<(), String> {
        self.preview_host = None;
        self.preview_host = Some(preview_host::PreviewHost::serve(session)?);
        Ok(())
    }

    pub fn owned_preview_host_count(&self) -> usize {
        match &self.preview_host {
            Some(host) if host.is_serving() => 1,
            _ => 0,
        }
    }

    /// Non-authoritative presentation helper. Does not accept a Product Mapping.
    pub fn show_mapping_proposal(
        &mut self,
        entries: Vec<String>,
        unmapped: Vec<String>,
        ambiguities: Vec<String>,
    ) {
        self.mapping_entries = entries;
        self.unmapped = unmapped;
        self.ambiguities = ambiguities;
        self.accepted_mapping_version_id = None;
    }

    pub fn bind_mapping_draft(&mut self, mapping_id: impl Into<String>, version: &BoneMappingVersion) {
        self.mapping_id = Some(mapping_id.into());
        self.apply_product_mapping(version);
    }

    pub fn accept_current_mapping(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        let mapping_id = self.mapping_id.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog("Workbench has no Mapping to accept".into())
        })?;
        let version_id = self.mapping_version_id.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog("Workbench has no MappingVersion to accept".into())
        })?;
        let published = app.accept_mapping_version(&mapping_id, &version_id)?;
        self.apply_product_mapping(published.as_record());
        Ok(())
    }

    pub fn override_current_mapping(
        &mut self,
        app: &mut Application,
        source_key: &str,
        target_key: Option<&str>,
        reason: &str,
    ) -> Result<(), rigforge_app::AppError> {
        let version_id = self.mapping_version_id.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog("Workbench has no MappingVersion to override".into())
        })?;
        let draft = app.override_mapping_entry(&version_id, source_key, target_key, reason)?;
        self.apply_product_mapping(draft.as_record());
        Ok(())
    }

    fn apply_product_mapping(&mut self, version: &BoneMappingVersion) {
        self.mapping_version_id = Some(version.id().canonical());
        self.mapping_entries = version
            .entries()
            .iter()
            .map(|e| {
                format!(
                    "{} → {}",
                    e.source().joint_key().as_str(),
                    e.target().joint_key().as_str()
                )
            })
            .collect();
        self.unmapped = version
            .unmapped_source()
            .iter()
            .chain(version.unmapped_target())
            .map(|u| format!("{} {:?}", u.joint_key().as_str(), u.disposition()))
            .collect();
        self.ambiguities = version.review().ambiguities().to_vec();
        if version.lifecycle() == Lifecycle::Published {
            self.accepted_mapping_version_id = Some(version.id().canonical());
        } else {
            self.accepted_mapping_version_id = None;
        }
    }

    /// Non-authoritative presentation helper. Prefer `apply_compatibility_result`.
    pub fn show_compatibility(
        &mut self,
        dimensions: Vec<(String, String)>,
        summary: impl Into<String>,
    ) {
        self.compatibility_dimensions = dimensions;
        self.compatibility_summary = Some(summary.into());
        self.compatibility_id = None;
    }

    pub fn apply_compatibility_result(&mut self, result: &CompatibilityResult) {
        self.compatibility_dimensions = vec![
            (
                "mapping_completeness".into(),
                format!("{:?}", result.mapping_completeness()),
            ),
            (
                "structural_compatibility".into(),
                format!("{:?}", result.structural_compatibility()),
            ),
            (
                "method_eligibility".into(),
                format!("{:?}", result.method_eligibility()),
            ),
            (
                "motion_suitability".into(),
                format!("{:?}", result.motion_suitability()),
            ),
            (
                "result_acceptability".into(),
                format!("{:?}", result.result_acceptability()),
            ),
        ];
        self.compatibility_summary = Some(format!("{:?}", result.summary()));
        self.compatibility_id = Some(result.id().canonical());
        self.compatibility_notes = result.notes().to_vec();
        self.warnings_acknowledged = false;
        self.transfer_auth = None;
    }

    pub fn refresh_transfer_authorization(
        &mut self,
        app: &Application,
    ) -> Result<(), rigforge_app::AppError> {
        let Some(id) = self.compatibility_id.clone() else {
            self.transfer_auth = None;
            return Ok(());
        };
        self.transfer_auth = Some(app.authorize_transfer(&id, self.warnings_acknowledged)?);
        Ok(())
    }

    pub fn acknowledge_compatibility_warnings(
        &mut self,
        app: &Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.warnings_acknowledged = true;
        self.refresh_transfer_authorization(app)
    }

    pub fn warnings_acknowledged(&self) -> bool {
        self.warnings_acknowledged
    }

    pub fn compatibility_notes(&self) -> &[String] {
        &self.compatibility_notes
    }

    pub fn transfer_authorization(&self) -> Option<&TransferAuthorization> {
        self.transfer_auth.as_ref()
    }

    pub fn transfer_available(&self) -> bool {
        !self.mutation_locked()
            && self
                .transfer_auth
                .as_ref()
                .map(|auth| auth.eligible)
                .unwrap_or(false)
    }

    pub fn mutation_locked(&self) -> bool {
        self.long_op
            .as_ref()
            .map(|op| op.phase.is_active())
            .unwrap_or(false)
    }

    pub fn long_op_phase(&self) -> Option<TransferPhase> {
        self.long_op.as_ref().map(|op| op.phase)
    }

    pub fn transfer_elapsed_label(&self) -> Option<String> {
        self.long_op.as_ref().map(|op| op.elapsed_label())
    }

    pub fn long_op_derived_version_id(&self) -> Option<&str> {
        self.long_op
            .as_ref()
            .and_then(|op| op.derived_version_id.as_deref())
    }

    fn require_no_active_mutation(&self) -> Result<(), rigforge_app::AppError> {
        if self.mutation_locked() {
            Err(rigforge_app::AppError::Catalog(
                "A Transfer is already running".into(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn transfer_requires_acknowledgement(&self) -> bool {
        self.transfer_auth
            .as_ref()
            .map(|auth| auth.requires_acknowledgement && !self.warnings_acknowledged)
            .unwrap_or(false)
    }

    pub fn transfer_eligibility_label(&self) -> String {
        match &self.transfer_auth {
            Some(auth) if auth.eligible => "eligible".into(),
            Some(auth) => auth
                .denial_reason
                .clone()
                .unwrap_or_else(|| rigforge_app::MappingWorkflowSnapshot::transfer_unavailable_reason().into()),
            None => "Transfer requires an exact CompatibilityResult from Application".into(),
        }
    }

    pub fn request_transfer(
        &mut self,
        app: &mut Application,
        existing_derived_variant_id: Option<&str>,
    ) -> Result<String, rigforge_app::AppError> {
        let id = self.compatibility_id.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Workbench Transfer requires an exact CompatibilityResult".into(),
            )
        })?;
        let selected_character = self.selected_character_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Transfer FAIL CLOSED: no Character version is selected".into(),
            )
        })?;
        let selected_motion = self.selected_motion_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Transfer FAIL CLOSED: no Motion version is selected".into(),
            )
        })?;
        let retained = app.catalog().load_compatibility_result(&id)?;
        let graph_character = retained.as_record().character_version_id().canonical();
        let graph_motion = retained.as_record().motion_version_id().canonical();
        if graph_character != selected_character || graph_motion != selected_motion {
            return Err(rigforge_app::AppError::Catalog(
                "Transfer FAIL CLOSED: CompatibilityResult does not match the current Character/Motion selection".into(),
            ));
        }
        self.refresh_transfer_authorization(app)?;
        if !self.transfer_available() {
            return Err(rigforge_app::AppError::Catalog(self.transfer_eligibility_label()));
        }
        let (_spec, run) = app.start_transfer(
            &id,
            self.warnings_acknowledged,
            existing_derived_variant_id,
            "Workbench Derived",
        )?;
        self.transfer_phase = Some("queued".into());
        self.jobs.insert(
            0,
            JobStatusView {
                run_id: run.run_id.clone(),
                job_spec_id: run.job_spec_id.clone(),
                state: run.state.as_db_str().to_string(),
            },
        );
        Ok(run.run_id)
    }

    pub fn propose_mapping_with<I: SkeletonEvidenceProvider>(
        &mut self,
        app: &mut Application,
        inspector: &I,
    ) -> Result<(), rigforge_app::AppError> {
        let character = self.selected_character_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Propose Mapping requires a selected Character version".into(),
            )
        })?;
        let motion = self.selected_motion_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Propose Mapping requires a selected Motion version".into(),
            )
        })?;
        let (logical, draft) = app.propose_and_store_mapping_for_selection(
            &character,
            &motion,
            inspector,
            MappingAssistProfile::OptionalHumanoid,
            "Workbench Mapping",
        )?;
        self.bind_mapping_draft(logical.as_record().id().canonical(), draft.as_record());
        self.workflow_status = Some("Mapping draft stored; explicit accept is required".into());
        Ok(())
    }

    pub fn on_propose_mapping_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        let inspector = BlenderSkeletonInspector::production()?;
        self.propose_mapping_with(app, &inspector)
    }

    pub fn on_accept_mapping_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        self.accept_current_mapping(app)?;
        self.workflow_status = Some("Mapping accepted".into());
        Ok(())
    }

    pub fn on_evaluate_compatibility_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        let character = self.selected_character_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Compatibility requires a selected Character version".into(),
            )
        })?;
        let motion = self.selected_motion_version.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Compatibility requires a selected Motion version".into(),
            )
        })?;
        let mapping = self.accepted_mapping_version_id.clone().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Compatibility requires an accepted MappingVersion".into(),
            )
        })?;
        let policy = if let Some(selected) = self.selected_policy_version_id.clone() {
            app.require_published_policy_version(&selected)?
        } else {
            app.ensure_published_proven_policy()?
        };
        let result = app.run_compatibility_preflight(
            &character,
            &motion,
            &mapping,
            &policy,
            &WorkerCapabilityProfile::v1_3_isolated_worker(),
        )?;
        self.apply_compatibility_result(result.as_record());
        self.refresh_transfer_authorization(app)?;
        self.workflow_status = Some(format!(
            "Compatibility {}",
            self.compatibility_summary.as_deref().unwrap_or("recorded")
        ));
        Ok(())
    }

    pub fn on_warnings_checkbox_changed(
        &mut self,
        app: &mut Application,
        acknowledged: bool,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        self.warnings_acknowledged = acknowledged;
        self.refresh_transfer_authorization(app)
    }

    pub fn on_transfer_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<String, rigforge_app::AppError> {
        self.request_transfer(app, None)
    }

    pub fn on_transfer_action(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.start_responsive_transfer(app, None)
    }

    pub fn start_responsive_transfer(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        let locked_character = self.selected_character_version.clone();
        let locked_motion = self.selected_motion_version.clone();
        let run_id = self.on_transfer_clicked(app)?;
        self.long_op = Some(LongOpSession::new(
            run_id.clone(),
            locked_character,
            locked_motion,
        ));
        self.sync_transfer_phase_label();
        match crate::native_exec::launch_and_spawn_execute(app, &run_id, ctx.cloned()) {
            Ok(rx) => {
                self.arm_execute_wait(rx);
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
                Ok(())
            }
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                Err(err)
            }
        }
    }

    pub fn start_scripted_transfer_for_test<W>(
        &mut self,
        app: &mut Application,
        worker: W,
    ) -> Result<(), rigforge_app::AppError>
    where
        W: rigforge_app::WorkerPort + rigforge_app::WorkerCompletionPort + Send + 'static,
    {
        self.require_no_active_mutation()?;
        let locked_character = self.selected_character_version.clone();
        let locked_motion = self.selected_motion_version.clone();
        let run_id = self.on_transfer_clicked(app)?;
        self.long_op = Some(LongOpSession::new(
            run_id.clone(),
            locked_character,
            locked_motion,
        ));
        self.sync_transfer_phase_label();
        match crate::native_exec::launch_and_spawn_execute_with_worker(app, &run_id, worker, None) {
            Ok(rx) => {
                self.arm_execute_wait(rx);
                Ok(())
            }
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                Err(err)
            }
        }
    }

    pub fn set_scripted_qc_acquire_for_test(&mut self, acquire: QcAcquireFn) {
        self.qc_acquire = Some(acquire);
    }

    pub fn set_scripted_reopen_acquire_for_test(&mut self, acquire: ReopenAcquireFn) {
        self.reopen_acquire = Some(acquire);
    }

    pub fn install_disconnected_execute_wait_for_test(&mut self, run_id: String) {
        let (tx, rx) = std::sync::mpsc::channel();
        drop(tx);
        let locked_character = self.selected_character_version.clone();
        let locked_motion = self.selected_motion_version.clone();
        let mut op = LongOpSession::new(run_id, locked_character, locked_motion);
        op.phase = TransferPhase::RunningExecute;
        op.wait = Some(rx);
        self.long_op = Some(op);
        self.sync_transfer_phase_label();
    }

    fn arm_execute_wait(&mut self, rx: Receiver<WaitMsg>) {
        if let Some(op) = self.long_op.as_mut() {
            op.phase = TransferPhase::RunningExecute;
            op.wait = Some(rx);
        }
        self.sync_transfer_phase_label();
    }

    fn sync_transfer_phase_label(&mut self) {
        if let Some(op) = &self.long_op {
            self.transfer_phase = Some(op.phase.user_label().into());
        }
    }

    pub fn poll_long_op(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
    ) -> Result<(), rigforge_app::AppError> {
        if self.mutation_locked() {
            if let Some(ctx) = ctx {
                ctx.request_repaint();
                ctx.request_repaint_after(Duration::from_millis(250));
            }
        }
        let Some(rx) = self.long_op.as_mut().and_then(|op| op.wait.take()) else {
            return Ok(());
        };
        match rx.try_recv() {
            Ok(msg) => self.apply_wait_msg(app, ctx, msg),
            Err(TryRecvError::Empty) => {
                if let Some(op) = self.long_op.as_mut() {
                    op.wait = Some(rx);
                }
                Ok(())
            }
            Err(TryRecvError::Disconnected) => {
                self.fail_long_operation(
                    app,
                    "background Transfer channel disconnected unexpectedly".into(),
                );
                Ok(())
            }
        }
    }

    pub fn drive_transfer_to_terminal(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        let deadline = std::time::Instant::now() + Duration::from_secs(900);
        while self.mutation_locked() {
            if std::time::Instant::now() > deadline {
                return Err(rigforge_app::AppError::Catalog(
                    "Transfer did not complete before the test deadline".into(),
                ));
            }
            self.poll_long_op(app, None)?;
            if self.mutation_locked() {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        Ok(())
    }

    fn apply_wait_msg(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
        msg: WaitMsg,
    ) -> Result<(), rigforge_app::AppError> {
        match msg {
            WaitMsg::Execute { run_id, outcome } => {
                if !self.matches_active_run(&run_id) {
                    return Ok(());
                }
                match outcome {
                    Ok((terminal, staged)) => {
                        self.apply_execute_outcome(app, ctx, run_id, terminal, staged)
                    }
                    Err(err) => {
                        self.fail_long_operation(app, err);
                        Ok(())
                    }
                }
            }
            WaitMsg::Qc {
                run_id,
                derived_version_id,
                outcome,
            } => {
                if !self.matches_active_run(&run_id)
                    || self.long_op.as_ref().and_then(|op| op.derived_version_id.as_deref())
                        != Some(derived_version_id.as_str())
                {
                    return Ok(());
                }
                match outcome {
                    Ok(evidence) => self.apply_qc_evidence(app, ctx, derived_version_id, evidence),
                    Err(err) => {
                        self.fail_long_operation(app, err);
                        Ok(())
                    }
                }
            }
            WaitMsg::Reopen {
                run_id,
                derived_version_id,
                outcome,
            } => {
                if !self.matches_active_run(&run_id)
                    || self.long_op.as_ref().and_then(|op| op.derived_version_id.as_deref())
                        != Some(derived_version_id.as_str())
                {
                    return Ok(());
                }
                match outcome {
                    Ok((fresh, structural)) => {
                        self.apply_reopen_outcomes(app, derived_version_id, fresh, structural)
                    }
                    Err(err) => {
                        self.fail_long_operation(app, err);
                        Ok(())
                    }
                }
            }
        }
    }

    fn matches_active_run(&self, run_id: &str) -> bool {
        self.long_op
            .as_ref()
            .map(|op| op.run_id == run_id && op.phase.is_active())
            .unwrap_or(false)
    }

    fn apply_execute_outcome(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
        run_id: String,
        terminal: TerminalOutcome,
        staged: Option<std::path::PathBuf>,
    ) -> Result<(), rigforge_app::AppError> {
        if let Some(op) = self.long_op.as_mut() {
            op.phase = TransferPhase::BindingWorkerResult;
        }
        self.sync_transfer_phase_label();
        let failed_reason = match &terminal {
            TerminalOutcome::Failed { reason, .. } => Some(reason.clone()),
            TerminalOutcome::Success(_) => None,
        };
        match app.apply_terminal_outcome(&run_id, terminal) {
            Ok(_) => {}
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                return Ok(());
            }
        }
        let _ = self.reload_from_application(app);
        if let Some(reason) = failed_reason {
            self.fail_long_operation(app, reason);
            return Ok(());
        }
        let Some(staged) = staged else {
            self.fail_long_operation(
                app,
                "successful worker with missing staged path".into(),
            );
            return Ok(());
        };
        match app.ingest_worker_success_candidate(&run_id, &staged) {
            Ok((logical, version)) => {
                let derived_version_id = version.as_record().id().canonical();
                let derived_variant_id = logical.as_record().id().canonical();
                if let Some(op) = self.long_op.as_mut() {
                    op.derived_version_id = Some(derived_version_id.clone());
                    op.derived_variant_id = Some(derived_variant_id);
                    op.phase = TransferPhase::ValidatingQc;
                }
                self.sync_transfer_phase_label();
                self.spawn_qc(app, ctx, run_id, derived_version_id)
            }
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                Ok(())
            }
        }
    }

    fn spawn_qc(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
        run_id: String,
        derived_version_id: String,
    ) -> Result<(), rigforge_app::AppError> {
        let request = match app.prepare_qc_acquisition(&derived_version_id) {
            Ok(request) => request,
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                return Ok(());
            }
        };
        let acquire = self.qc_acquire.take();
        let rx = crate::native_exec::spawn_qc_acquire(
            run_id,
            request,
            ctx.cloned(),
            move |request| match acquire {
                Some(scripted) => scripted(request),
                None => crate::native_exec::production_qc_acquire(request),
            },
        );
        if let Some(op) = self.long_op.as_mut() {
            op.wait = Some(rx);
        }
        Ok(())
    }

    fn apply_qc_evidence(
        &mut self,
        app: &mut Application,
        ctx: Option<&egui::Context>,
        derived_version_id: String,
        evidence: rigforge_app::ArtifactInspectionEvidence,
    ) -> Result<(), rigforge_app::AppError> {
        if let Err(err) = app.bind_qc_from_evidence(&derived_version_id, evidence) {
            self.fail_long_operation(app, err.to_string());
            return Ok(());
        }
        if let Some(op) = self.long_op.as_mut() {
            op.phase = TransferPhase::ValidatingPersistence;
        }
        self.sync_transfer_phase_label();
        let request = match app.prepare_reopen_acquisition(&derived_version_id) {
            Ok(request) => request,
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                return Ok(());
            }
        };
        let run_id = self
            .long_op
            .as_ref()
            .map(|op| op.run_id.clone())
            .expect("active Transfer");
        let acquire = self.reopen_acquire.take();
        let rx = crate::native_exec::spawn_reopen_acquire(
            run_id,
            request,
            ctx.cloned(),
            move |request| match acquire {
                Some(scripted) => scripted(request),
                None => crate::native_exec::production_reopen_acquire(request),
            },
        );
        if let Some(op) = self.long_op.as_mut() {
            op.wait = Some(rx);
        }
        Ok(())
    }

    fn apply_reopen_outcomes(
        &mut self,
        app: &mut Application,
        derived_version_id: String,
        fresh: VerificationOutcome,
        structural: VerificationOutcome,
    ) -> Result<(), rigforge_app::AppError> {
        if let Err(err) =
            app.bind_verification_from_outcomes(&derived_version_id, fresh, structural)
        {
            self.fail_long_operation(app, err.to_string());
            return Ok(());
        }
        if let Some(op) = self.long_op.as_mut() {
            op.phase = TransferPhase::Publishing;
        }
        self.sync_transfer_phase_label();
        let run_id = self
            .long_op
            .as_ref()
            .map(|op| op.run_id.clone())
            .expect("active Transfer");
        match app.complete_publication_decision(&run_id, &derived_version_id) {
            Ok(outcome) => {
                self.apply_transfer_outcome(&outcome);
                if let Some(op) = self.long_op.as_mut() {
                    op.phase = TransferPhase::Complete;
                    op.wait = None;
                }
                self.sync_transfer_phase_label();
                let _ = self.reload_from_application(app);
                Ok(())
            }
            Err(err) => {
                self.fail_long_operation(app, err.to_string());
                Ok(())
            }
        }
    }

    fn fail_long_operation(&mut self, app: &mut Application, message: String) {
        if let Some(op) = self.long_op.as_mut() {
            op.phase = TransferPhase::Failed;
            op.wait = None;
        }
        self.transfer_phase = Some(format!("failed: {message}"));
        self.workflow_status = Some(message.clone());
        if let Some(run_id) = self.long_op.as_ref().map(|op| op.run_id.clone()) {
            if let Ok(run) = app.job_status(&run_id) {
                if !run.state.is_terminal() {
                    if run.state == JobRunState::Queued {
                        let _ = app.mark_dispatchable(&run_id);
                    }
                    let _ = app.complete_failure(&run_id, &message);
                }
            }
        }
        let _ = self.reload_from_application(app);
    }

    pub fn on_register_character_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        let display_name = parse_required_text(&self.character_pick.display_name, "display name")?;
        let inspection = self.character_inspection.as_ref().ok_or_else(|| {
            rigforge_app::AppError::Catalog(
                "Inspect a Character FBX before adding it.".into(),
            )
        })?;
        let registered = app.register_character_from_inspection(&display_name, inspection)?;
        self.reload_from_application(app)?;
        self.select_character_version(&registered.version_id);
        self.bind_published_mapping_for_current_selection(app)?;
        self.workflow_status = Some(format!(
            "Registered Character version {}",
            registered.version_id
        ));
        Ok(())
    }

    pub fn on_register_motion_clicked(
        &mut self,
        app: &mut Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.require_no_active_mutation()?;
        let display_name = parse_required_text(&self.motion_pick.display_name, "display name")?;
        let inspection = self.motion_inspection.as_ref().ok_or_else(|| {
            rigforge_app::AppError::Catalog("Inspect a Motion FBX before adding it.".into())
        })?;
        let clip = self
            .motion_selected_clip
            .as_deref()
            .ok_or_else(|| rigforge_app::AppError::Catalog(INGEST_NO_CLIPS.into()))?;
        let registered =
            app.register_motion_from_inspection(&display_name, inspection, clip)?;
        self.reload_from_application(app)?;
        self.select_motion_version(&registered.version_id);
        self.bind_published_mapping_for_current_selection(app)?;
        self.workflow_status = Some(format!(
            "Registered Motion version {}",
            registered.version_id
        ));
        Ok(())
    }

    pub fn reload_from_application(
        &mut self,
        app: &Application,
    ) -> Result<(), rigforge_app::AppError> {
        self.characters = app.list_characters()?;
        self.motions = app.list_motions()?;
        self.derived = app.list_derived_variants()?;
        self.jobs = app
            .catalog()
            .list_job_runs()?
            .into_iter()
            .map(|run| JobStatusView {
                run_id: run.run_id,
                job_spec_id: run.job_spec_id,
                state: run.state.as_db_str().to_string(),
            })
            .collect();
        Ok(())
    }

    pub fn workflow_status(&self) -> Option<&str> {
        self.workflow_status.as_deref()
    }

    pub fn apply_transfer_outcome(&mut self, outcome: &TransferOutcome) {
        self.derived_variant_id = Some(outcome.derived_variant_id.clone()).filter(|s| !s.is_empty());
        self.derived_variant_version_id =
            Some(outcome.derived_variant_version_id.clone()).filter(|s| !s.is_empty());
        self.qc_verdict = outcome.qc_verdict.clone();
        self.persistence_verification = outcome.persistence_verification_id.clone();
        match outcome.kind {
            TransferOutcomeKind::Published => {
                self.publication_state = Some("Published".into());
                self.transfer_phase = Some("published".into());
                if let Some(version_id) = Some(outcome.derived_variant_version_id.clone())
                    .filter(|s| !s.is_empty())
                {
                    self.selected_derived_variant_version = Some(version_id);
                    self.clear_preview_presentation();
                }
            }
            TransferOutcomeKind::PublicationDenied => {
                self.publication_state = Some("Publication denied".into());
                self.transfer_phase = Some("publication_denied".into());
            }
        }
        if let Some(reason) = &outcome.reason {
            self.transfer_phase = Some(format!(
                "{} ({reason})",
                self.transfer_phase.clone().unwrap_or_default()
            ));
        }
    }

    pub fn derived_variant_id(&self) -> Option<&str> {
        self.derived_variant_id.as_deref()
    }
    pub fn derived_variant_version_id(&self) -> Option<&str> {
        self.derived_variant_version_id.as_deref()
    }
    pub fn publication_state(&self) -> Option<&str> {
        self.publication_state.as_deref()
    }
    pub fn qc_verdict(&self) -> Option<&str> {
        self.qc_verdict.as_deref()
    }
    pub fn persistence_verification_id(&self) -> Option<&str> {
        self.persistence_verification.as_deref()
    }
    pub fn transfer_phase(&self) -> Option<&str> {
        self.transfer_phase.as_deref()
    }
    pub fn preview_unavailable_reason() -> &'static str {
        rigforge_app::MappingWorkflowSnapshot::preview_unavailable_reason()
    }

    pub fn mapping_entries(&self) -> &[String] {
        &self.mapping_entries
    }
    pub fn unmapped_joints(&self) -> &[String] {
        &self.unmapped
    }
    pub fn ambiguities(&self) -> &[String] {
        &self.ambiguities
    }
    pub fn compatibility_dimensions(&self) -> &[(String, String)] {
        &self.compatibility_dimensions
    }
    pub fn compatibility_summary(&self) -> Option<&str> {
        self.compatibility_summary.as_deref()
    }
    pub fn mapping_accepted(&self) -> bool {
        self.accepted_mapping_version_id.is_some()
    }
    pub fn accepted_mapping_version_id(&self) -> Option<&str> {
        self.accepted_mapping_version_id.as_deref()
    }
    pub fn mapping_version_id(&self) -> Option<&str> {
        self.mapping_version_id.as_deref()
    }
    pub fn compatibility_id(&self) -> Option<&str> {
        self.compatibility_id.as_deref()
    }
    pub fn transfer_placeholder() -> &'static str {
        rigforge_app::MappingWorkflowSnapshot::transfer_unavailable_reason()
    }

    pub fn resolve_selected_character_version(
        &self,
        app: &Application,
    ) -> Result<Option<String>, rigforge_app::AppError> {
        match &self.selected_character_version {
            Some(id) => {
                let loaded = app.resolve_exact_character_version(id)?;
                Ok(Some(loaded.as_record().id().canonical()))
            }
            None => Ok(None),
        }
    }

    pub fn request_preview(
        &mut self,
        app: &mut Application,
        subject: PreviewSubject,
        generate_if_missing: bool,
        open_sidecar: bool,
    ) -> Result<rigforge_app::PreviewSession, rigforge_app::AppError> {
        if generate_if_missing {
            let request = match &subject {
                PreviewSubject::Character { version_id } => {
                    PreviewGenerationRequest::character(version_id)
                }
                PreviewSubject::Motion { version_id } => {
                    PreviewGenerationRequest::motion(version_id)
                }
                PreviewSubject::DerivedVariant { version_id } => {
                    PreviewGenerationRequest::derived_variant(version_id)
                }
            };
            let resolved = app.resolve_preview_for_display(subject.clone());
            if !resolved.is_valid() {
                match app.generate_preview(request) {
                    Ok(_) => {}
                    Err(err) => {
                        self.preview.occupied = false;
                        self.preview_valid = false;
                        self.preview_status = format!("Preview generation failed: {err}");
                        let session = app.materialize_preview_session(subject)?;
                        return Ok(session);
                    }
                }
            }
        }
        let session = app.materialize_preview_session(subject)?;
        self.preview.occupied = session.document.valid;
        self.preview_valid = session.document.valid;
        self.preview_status = if session.document.valid {
            format!(
                "valid Preview for {} {}",
                session.document.selected_product_kind,
                session.document.selected_product_version_id
            )
        } else {
            format!(
                "{}: {}",
                session
                    .document
                    .failure_kind
                    .as_deref()
                    .unwrap_or("Preview unavailable"),
                session
                    .document
                    .failure_detail
                    .as_deref()
                    .unwrap_or("see diagnostic")
            )
        };
        if open_sidecar {
            self.preview_host = None;
            match preview_host::PreviewHost::serve(&session) {
                Ok(host) => {
                    let opened = host.open_sidecar();
                    self.preview_host = Some(host);
                    if let Err(err) = opened {
                        self.preview_status =
                            format!("{} (viewer host: {err})", self.preview_status);
                    }
                }
                Err(err) => {
                    self.preview_status = format!("{} (viewer host: {err})", self.preview_status);
                }
            }
        }
        Ok(session)
    }

    pub fn regenerate_preview(
        &mut self,
        app: &mut Application,
        subject: PreviewSubject,
        open_sidecar: bool,
    ) -> Result<rigforge_app::PreviewSession, rigforge_app::AppError> {
        let existing = app.resolve_preview_for_display(subject.clone());
        if let Some(artifact) = existing.artifact {
            let _ = app.delete_preview(&artifact.as_record().id().canonical());
        }
        let request = match &subject {
            PreviewSubject::Character { version_id } => {
                PreviewGenerationRequest::character(version_id)
            }
            PreviewSubject::Motion { version_id } => PreviewGenerationRequest::motion(version_id),
            PreviewSubject::DerivedVariant { version_id } => {
                PreviewGenerationRequest::derived_variant(version_id)
            }
        };
        match app.generate_preview(request) {
            Ok(_) => {}
            Err(err) => {
                self.preview.occupied = false;
                self.preview_valid = false;
                self.preview_status = format!("Preview generation failed: {err}");
                return app.materialize_preview_session(subject);
            }
        }
        self.request_preview(app, subject, false, open_sidecar)
    }

    pub fn job_states(&self) -> &[JobStatusView] {
        &self.jobs
    }

    pub fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100.0, 720.0])
                .with_title("RigForge Workbench"),
            ..Default::default()
        }
    }
}

impl WorkbenchApp {
    pub fn draw(&mut self, ctx: &egui::Context, mut app: Option<&mut Application>) {
        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("RigForge Workbench");
                ui.separator();
                ui.label("local-first");
                ui.separator();
                ui.label("no Blender in this process");
                if self.mutation_locked() {
                    ui.separator();
                    ui.spinner();
                    if let Some(op) = &self.long_op {
                        ui.label(op.phase.user_label());
                        ui.label(format!("Elapsed {}", op.elapsed_label()));
                    }
                }
            });
        });
        egui::SidePanel::left("browser")
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Asset Browser");
                ui.separator();
                ui.collapsing("Characters", |ui| {
                    let mut picked = None;
                    let can_switch = !self.mutation_locked();
                    for item in &self.characters {
                        let id = item
                            .published_version_id
                            .clone()
                            .unwrap_or_else(|| item.logical_id.clone());
                        let selected = self.selected_character_version.as_deref() == Some(id.as_str());
                        if ui
                            .add_enabled(
                                can_switch,
                                egui::SelectableLabel::new(selected, &item.display_name),
                            )
                            .clicked()
                        {
                            picked = Some(id);
                        }
                    }
                    if let Some(id) = picked {
                        self.select_character_version(id);
                        if let Some(app) = app.as_mut() {
                            if let Err(err) = self.bind_published_mapping_for_current_selection(app)
                            {
                                self.workflow_status = Some(err.to_string());
                            }
                        }
                    }
                    if self.characters.is_empty() {
                        ui.weak("No Character assets");
                    }
                });
                ui.collapsing("Motions", |ui| {
                    let mut picked = None;
                    let can_switch = !self.mutation_locked();
                    for item in &self.motions {
                        let id = item
                            .published_version_id
                            .clone()
                            .unwrap_or_else(|| item.logical_id.clone());
                        let selected = self.selected_motion_version.as_deref() == Some(id.as_str());
                        if ui
                            .add_enabled(
                                can_switch,
                                egui::SelectableLabel::new(selected, &item.display_name),
                            )
                            .clicked()
                        {
                            picked = Some(id);
                        }
                    }
                    if let Some(id) = picked {
                        self.select_motion_version(id);
                        if let Some(app) = app.as_mut() {
                            if let Err(err) = self.bind_published_mapping_for_current_selection(app)
                            {
                                self.workflow_status = Some(err.to_string());
                            }
                        }
                    }
                    if self.motions.is_empty() {
                        ui.weak("No Motion assets");
                    }
                });
                ui.collapsing("Derived Variants", |ui| {
                    let mut picked = None;
                    for item in &self.derived {
                        let id = item
                            .published_version_id
                            .clone()
                            .unwrap_or_else(|| item.logical_id.clone());
                        let selected =
                            self.selected_derived_variant_version.as_deref() == Some(id.as_str());
                        if ui.selectable_label(selected, &item.display_name).clicked() {
                            picked = Some(id);
                        }
                    }
                    if let Some(id) = picked {
                        self.select_derived_variant_version(id);
                    }
                    if self.derived.is_empty() {
                        ui.weak("No Derived Variants");
                    }
                });
                ui.separator();
                ui.collapsing("Add Character", |ui| {
                    ui.label("Display name");
                    ui.text_edit_singleline(&mut self.character_pick.display_name);
                    ui.horizontal(|ui| {
                        let browse = if self.character_pick.path.is_some() {
                            "Change"
                        } else {
                            "Browse"
                        };
                        if ui
                            .add_enabled(!self.mutation_locked(), egui::Button::new(browse))
                            .clicked()
                        {
                            let picked = crate::file_pick::pick_fbx_file();
                            if self.apply_character_selection(picked) {
                                self.begin_character_inspect(ctx);
                            }
                        }
                        if self.character_inspecting {
                            ui.spinner();
                            ui.label("Inspecting Character…");
                        }
                    });
                    if self.character_pick.filename.is_empty() {
                        ui.weak("No FBX selected");
                    } else {
                        ui.label(format!("File: {}", self.character_pick.filename))
                            .on_hover_text(
                                self.character_pick
                                    .path
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_default(),
                            );
                    }
                    if let Some(status) = &self.character_ingest_status {
                        ui.label(status.clone());
                    }
                    let add_enabled = self.character_can_add() && app.is_some();
                    if ui
                        .add_enabled(add_enabled, egui::Button::new("Add Character"))
                        .clicked()
                    {
                        if let Some(app) = app.as_mut() {
                            if let Err(err) = self.on_register_character_clicked(app) {
                                self.workflow_status = Some(err.to_string());
                            }
                        } else {
                            self.workflow_status = Some(
                                "Native Application path is required for Character registration"
                                    .into(),
                            );
                        }
                    }
                });
                ui.collapsing("Add Motion", |ui| {
                    ui.label("Display name");
                    ui.text_edit_singleline(&mut self.motion_pick.display_name);
                    ui.horizontal(|ui| {
                        let browse = if self.motion_pick.path.is_some() {
                            "Change"
                        } else {
                            "Browse"
                        };
                        if ui
                            .add_enabled(!self.mutation_locked(), egui::Button::new(browse))
                            .clicked()
                        {
                            let picked = crate::file_pick::pick_fbx_file();
                            if self.apply_motion_selection(picked) {
                                self.begin_motion_inspect(ctx);
                            }
                        }
                        if self.motion_inspecting {
                            ui.spinner();
                            ui.label("Inspecting Motion…");
                        }
                    });
                    if self.motion_pick.filename.is_empty() {
                        ui.weak("No FBX selected");
                    } else {
                        ui.label(format!("File: {}", self.motion_pick.filename))
                            .on_hover_text(
                                self.motion_pick
                                    .path
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_default(),
                            );
                    }
                    let motion_skeleton = self.motion_inspection.as_ref().and_then(|inspection| {
                        inspection
                            .skeleton_candidates
                            .first()
                            .map(|skeleton| skeleton.display_name.clone())
                    });
                    let clips: Vec<(String, String, String)> = self
                        .motion_inspection
                        .as_ref()
                        .map(|inspection| {
                            inspection
                                .usable_clips()
                                .into_iter()
                                .map(|clip| {
                                    (
                                        clip.clip_identity.clone(),
                                        clip.display_label.clone(),
                                        clip.presentation_line(),
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(skeleton) = motion_skeleton {
                        ui.label(format!("Source Skeleton: {skeleton}"));
                    }
                    if clips.len() > 1 {
                        let mut selected = self.motion_selected_clip.clone().unwrap_or_default();
                        let current_label = clips
                            .iter()
                            .find(|(id, _, _)| *id == selected)
                            .map(|(_, label, line)| format!("{label}  {line}"))
                            .unwrap_or_else(|| "Choose a clip".into());
                        egui::ComboBox::from_label("Animation clip")
                            .selected_text(current_label)
                            .show_ui(ui, |ui| {
                                for (id, label, line) in &clips {
                                    ui.selectable_value(
                                        &mut selected,
                                        id.clone(),
                                        format!("{label}  {line}"),
                                    );
                                }
                            });
                        self.motion_selected_clip = if selected.is_empty() {
                            None
                        } else {
                            Some(selected)
                        };
                    } else if let Some((_, label, line)) = clips.first() {
                        ui.label(format!("Clip: {label}"));
                        ui.label(line);
                    }
                    if let Some(status) = &self.motion_ingest_status {
                        ui.label(status.clone());
                    }
                    let add_enabled = self.motion_can_add() && app.is_some();
                    if ui
                        .add_enabled(add_enabled, egui::Button::new("Add Motion"))
                        .clicked()
                    {
                        if let Some(app) = app.as_mut() {
                            if let Err(err) = self.on_register_motion_clicked(app) {
                                self.workflow_status = Some(err.to_string());
                            }
                        } else {
                            self.workflow_status = Some(
                                "Native Application path is required for Motion registration"
                                    .into(),
                            );
                        }
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Selection / Transfer Tray");
            ui.label(format!(
                "Character version: {}",
                self.selected_character_version
                    .as_deref()
                    .unwrap_or("(none)")
            ));
            ui.label(format!(
                "Derived version: {}",
                self.selected_derived_variant_version
                    .as_deref()
                    .unwrap_or("(none)")
            ));
            ui.separator();
            ui.heading("Mapping / Compatibility");
            ui.label(format!("proposed entries: {}", self.mapping_entries.len()));
            for entry in &self.mapping_entries {
                ui.label(entry);
            }
            ui.label(format!("unmapped: {}", self.unmapped.len()));
            for item in &self.unmapped {
                ui.label(item);
            }
            ui.label(format!("ambiguities: {}", self.ambiguities.len()));
            for item in &self.ambiguities {
                ui.label(item);
            }
            ui.label(format!(
                "mapping accepted: {}",
                self.mapping_accepted()
            ));
            if let Some(id) = &self.accepted_mapping_version_id {
                ui.label(format!("accepted MappingVersion: {id}"));
            }
            ui.horizontal(|ui| {
                let mapping_enabled = !self.mutation_locked();
                if ui
                    .add_enabled(mapping_enabled, egui::Button::new("Propose Mapping"))
                    .clicked()
                {
                    if let Some(app) = app.as_mut() {
                        if let Err(err) = self.on_propose_mapping_clicked(app) {
                            self.workflow_status = Some(err.to_string());
                        }
                    } else {
                        self.workflow_status =
                            Some("Native Application path is required for Mapping".into());
                    }
                }
                if ui
                    .add_enabled(mapping_enabled, egui::Button::new("Accept Mapping"))
                    .clicked()
                {
                    if let Some(app) = app.as_mut() {
                        if let Err(err) = self.on_accept_mapping_clicked(app) {
                            self.workflow_status = Some(err.to_string());
                        }
                    } else {
                        self.workflow_status =
                            Some("Native Application path is required for Mapping".into());
                    }
                }
                if ui
                    .add_enabled(mapping_enabled, egui::Button::new("Evaluate Compatibility"))
                    .clicked()
                {
                    if let Some(app) = app.as_mut() {
                        if let Err(err) = self.on_evaluate_compatibility_clicked(app) {
                            self.workflow_status = Some(err.to_string());
                        }
                    } else {
                        self.workflow_status =
                            Some("Native Application path is required for Compatibility".into());
                    }
                }
            });
            ui.heading("Compatibility dimensions");
            for (name, value) in &self.compatibility_dimensions {
                ui.label(format!("{name}: {value}"));
            }
            ui.label(format!(
                "overall: {}",
                self.compatibility_summary.as_deref().unwrap_or("(none)")
            ));
            for note in &self.compatibility_notes {
                ui.label(format!("warning: {note}"));
            }
            ui.label(format!(
                "Transfer eligibility: {}",
                self.transfer_eligibility_label()
            ));
            if self.transfer_auth.as_ref().map(|a| a.requires_acknowledgement).unwrap_or(false) {
                let mut ack = self.warnings_acknowledged;
                let ack_enabled = !self.mutation_locked();
                let changed = ui
                    .add_enabled(
                        ack_enabled,
                        egui::Checkbox::new(
                            &mut ack,
                            "I acknowledge these Compatibility warnings",
                        ),
                    )
                    .changed();
                if changed
                {
                    if let Some(app) = app.as_mut() {
                        if let Err(err) = self.on_warnings_checkbox_changed(app, ack) {
                            self.workflow_status = Some(err.to_string());
                        }
                    } else {
                        self.warnings_acknowledged = ack;
                    }
                }
            }
            if ui
                .add_enabled(self.transfer_available(), egui::Button::new("Transfer"))
                .clicked()
            {
                if let Some(app) = app.as_mut() {
                    if let Err(err) = self.start_responsive_transfer(app, Some(ctx)) {
                        self.workflow_status = Some(err.to_string());
                    }
                } else {
                    self.workflow_status =
                        Some("Native Application path is required for Transfer".into());
                }
            }
            if !self.transfer_available() {
                ui.weak(self.transfer_eligibility_label());
            }
            if let Some(op) = &self.long_op {
                ui.horizontal(|ui| {
                    if op.phase.is_active() {
                        ui.spinner();
                    }
                    ui.label(op.phase.user_label());
                    ui.label(format!("Elapsed {}", op.elapsed_label()));
                });
            }
            if let Some(phase) = &self.transfer_phase {
                ui.label(format!("Transfer phase: {phase}"));
            }
            if let Some(id) = &self.derived_variant_id {
                ui.label(format!("DerivedVariant: {id}"));
            }
            if let Some(id) = &self.derived_variant_version_id {
                ui.label(format!("DerivedVariantVersion: {id}"));
            }
            if let Some(state) = &self.publication_state {
                ui.label(format!("publication: {state}"));
            }
            if let Some(verdict) = &self.qc_verdict {
                ui.label(format!("QC verdict: {verdict}"));
            }
            if let Some(id) = &self.persistence_verification {
                ui.label(format!("PersistenceVerification: {id}"));
            }
            if let Some(status) = &self.workflow_status {
                ui.label(format!("workflow: {status}"));
            }
            ui.separator();
            ui.heading("Job status");
            if self.jobs.is_empty() {
                ui.weak("No jobs");
            }
            for job in &self.jobs {
                ui.label(format!("{}  {}", job.state, job.run_id));
            }
            ui.separator();
            ui.group(|ui| {
                ui.strong("Preview");
                ui.label(format!(
                    "Viewer: {} {}",
                    PreviewEmbeddingSlot::viewer_library().unwrap_or("unset"),
                    preview_host::VIEWER_VERSION
                ));
                ui.label(format!(
                    "Payload: {} (Preview payload, not Product format)",
                    PreviewEmbeddingSlot::payload_format().unwrap_or("unset")
                ));
                ui.label(format!("status: {}", self.preview_status));
                ui.label(format!("valid: {}", self.preview_valid));
                ui.label(format!("occupied: {}", self.preview.occupied));
                ui.weak("Preview is derived, rebuildable, and non-authoritative.");
                if let Some(app) = app.as_mut() {
                    ui.horizontal(|ui| {
                        if ui.button("Preview Character").clicked() {
                            if let Some(id) = self.selected_character_version.clone() {
                                let _ = self.request_preview(
                                    app,
                                    PreviewSubject::Character { version_id: id },
                                    true,
                                    true,
                                );
                            } else {
                                self.preview_valid = false;
                                self.preview_status = "no Preview generated".into();
                            }
                        }
                        if ui.button("Preview Motion").clicked() {
                            if let Some(id) = self.selected_motion_version.clone() {
                                let _ = self.request_preview(
                                    app,
                                    PreviewSubject::Motion { version_id: id },
                                    true,
                                    true,
                                );
                            } else {
                                self.preview_valid = false;
                                self.preview_status = "no Preview generated".into();
                            }
                        }
                        if ui.button("Preview Derived Variant").clicked() {
                            if let Some(id) = self.selected_derived_variant_version.clone() {
                                let _ = self.request_preview(
                                    app,
                                    PreviewSubject::DerivedVariant { version_id: id },
                                    true,
                                    true,
                                );
                            } else {
                                self.preview_valid = false;
                                self.preview_status = "no Preview generated".into();
                            }
                        }
                        if ui.button("Regenerate Preview").clicked() {
                            if let Some(id) = self.selected_derived_variant_version.clone() {
                                let _ = self.regenerate_preview(
                                    app,
                                    PreviewSubject::DerivedVariant { version_id: id },
                                    true,
                                );
                            } else if let Some(id) = self.selected_character_version.clone() {
                                let _ = self.regenerate_preview(
                                    app,
                                    PreviewSubject::Character { version_id: id },
                                    true,
                                );
                            } else if let Some(id) = self.selected_motion_version.clone() {
                                let _ = self.regenerate_preview(
                                    app,
                                    PreviewSubject::Motion { version_id: id },
                                    true,
                                );
                            }
                        }
                    });
                } else {
                    ui.weak("Native Application path is required for click-to-preview.");
                }
            });
            let _ = JobRunState::Queued;
        });
    }
}

impl eframe::App for WorkbenchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_ingest(ctx);
        self.draw(ctx, None);
    }
}

/// Native Workbench path that retains Application for Mapping, Compatibility,
/// Transfer, and Preview. Historical GATE-C-OBS-001 remains historical;
/// V1-8 wires these actions through Application.
pub struct WorkbenchHost {
    pub app: Application,
    pub shell: WorkbenchApp,
}

impl WorkbenchHost {
    pub fn open_default() -> Result<Self, rigforge_app::AppError> {
        let path = std::env::var_os("RIGFORGE_CATALOG")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let mut p = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                p.push("rigforge-catalog.sqlite");
                p
            });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let app = Application::open(&path)?;
        let shell = WorkbenchApp::from_application(&app)?;
        Ok(Self { app, shell })
    }
}

impl eframe::App for WorkbenchHost {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shell.poll_ingest(ctx);
        let _ = self.shell.poll_long_op(&mut self.app, Some(ctx));
        self.shell.draw(ctx, Some(&mut self.app));
    }
}

fn parse_required_text(raw: &str, field: &str) -> Result<String, rigforge_app::AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(rigforge_app::AppError::Catalog(format!(
            "registration: {field} is required"
        )));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn egui_context_initializes_without_network() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        assert!(!super::WorkbenchApp::requires_network());
        assert!(!super::WorkbenchApp::requires_blender());
    }
}
