//! RigForge V1-2 Workbench shell.
//!
//! GUI → Application / Catalog / Orchestrator → WorkerPort.
//! This crate does not call Blender and does not select a Preview viewer.

use eframe::egui;
use rigforge_app::{Application, AssetListItem, JobRunState};

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
        Ok(shell)
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
