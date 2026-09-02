//! RigForge V1-2 Workbench shell.
//!
//! GUI → Application / Catalog / Orchestrator → WorkerPort.
//! This crate does not call Blender and does not select a Preview viewer.

use eframe::egui;
use rigforge_app::rigforge_domain::{
    BoneMappingVersion, CompatibilityResult, Lifecycle,
};
use rigforge_app::{
    Application, AssetListItem, JobRunState, TransferAuthorization, TransferOutcome,
    TransferOutcomeKind,
};

/// Future Preview embedding boundary. Viewer library and payload remain V1-6.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PreviewEmbeddingSlot {
    pub occupied: bool,
}

impl PreviewEmbeddingSlot {
    pub fn viewer_library() -> Option<&'static str> {
        None
    }

    pub fn payload_format() -> Option<&'static str> {
        None
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
    jobs: Vec<JobStatusView>,
    preview: PreviewEmbeddingSlot,
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
    derived_variant_id: Option<String>,
    derived_variant_version_id: Option<String>,
    publication_state: Option<String>,
    qc_verdict: Option<String>,
    persistence_verification: Option<String>,
}

impl WorkbenchApp {
    pub fn empty() -> Self {
        Self {
            characters: Vec::new(),
            motions: Vec::new(),
            derived: Vec::new(),
            selected_character_version: None,
            selected_motion_version: None,
            jobs: Vec::new(),
            preview: PreviewEmbeddingSlot::default(),
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
            derived_variant_id: None,
            derived_variant_version_id: None,
            publication_state: None,
            qc_verdict: None,
            persistence_verification: None,
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
        shell.bind_published_mapping_for_current_selection(app)?;
        Ok(shell)
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

    pub fn select_character_version(&mut self, version_id: impl Into<String>) {
        self.selected_character_version = Some(version_id.into());
    }

    pub fn select_motion_version(&mut self, version_id: impl Into<String>) {
        self.selected_motion_version = Some(version_id.into());
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
        self.transfer_auth
            .as_ref()
            .map(|auth| auth.eligible)
            .unwrap_or(false)
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

impl eframe::App for WorkbenchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("RigForge Workbench");
                ui.separator();
                ui.label("local-first");
                ui.separator();
                ui.label("no Blender in this process");
            });
        });
        egui::SidePanel::left("browser")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Asset Browser");
                ui.separator();
                ui.collapsing("Characters", |ui| {
                    for item in &self.characters {
                        ui.label(&item.display_name);
                    }
                    if self.characters.is_empty() {
                        ui.weak("No Character assets");
                    }
                });
                ui.collapsing("Motions", |ui| {
                    for item in &self.motions {
                        ui.label(&item.display_name);
                    }
                    if self.motions.is_empty() {
                        ui.weak("No Motion assets");
                    }
                });
                ui.collapsing("Derived Variants", |ui| {
                    for item in &self.derived {
                        ui.label(&item.display_name);
                    }
                    if self.derived.is_empty() {
                        ui.weak("No Derived Variants");
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
                "Motion version: {}",
                self.selected_motion_version.as_deref().unwrap_or("(none)")
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
                ui.checkbox(
                    &mut self.warnings_acknowledged,
                    "I acknowledge these Compatibility warnings",
                );
            }
            ui.add_enabled(self.transfer_available(), egui::Button::new("Transfer"));
            if !self.transfer_available() {
                ui.weak(self.transfer_eligibility_label());
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
                ui.strong("Preview slot");
                ui.label("Embedding boundary only. Viewer library: not selected.");
                ui.label("Payload format: not selected (V1-6).");
                ui.label(Self::preview_unavailable_reason());
                ui.label(format!("occupied: {}", self.preview.occupied));
            });
            let _ = JobRunState::Queued;
        });
    }
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
