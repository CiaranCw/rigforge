//! Presentation-owned Transfer long-operation state. Not Product identity.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use rigforge_app::{
    ArtifactInspectionEvidence, PersistenceReopenRequest, QcAcquisitionRequest, TerminalOutcome,
};
use rigforge_app::rigforge_domain::VerificationOutcome;

pub type QcAcquireFn =
    Box<dyn FnOnce(&QcAcquisitionRequest) -> Result<ArtifactInspectionEvidence, String> + Send>;
pub type ReopenAcquireFn = Box<
    dyn FnOnce(
            &PersistenceReopenRequest,
        ) -> Result<(VerificationOutcome, VerificationOutcome), String>
        + Send,
>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferPhase {
    Launching,
    RunningExecute,
    BindingWorkerResult,
    ValidatingQc,
    ValidatingPersistence,
    Publishing,
    Complete,
    Failed,
}

impl TransferPhase {
    pub fn is_active(self) -> bool {
        !matches!(self, Self::Complete | Self::Failed)
    }

    pub fn user_label(self) -> &'static str {
        match self {
            Self::Launching => crate::i18n::PHASE_LAUNCHING,
            Self::RunningExecute => crate::i18n::PHASE_RUNNING,
            Self::BindingWorkerResult | Self::ValidatingQc => crate::i18n::PHASE_VALIDATING,
            Self::ValidatingPersistence => crate::i18n::PHASE_PERSISTENCE,
            Self::Publishing => crate::i18n::PHASE_PUBLISHING,
            Self::Complete => crate::i18n::PHASE_COMPLETE,
            Self::Failed => crate::i18n::PHASE_FAILED,
        }
    }
}

pub fn format_elapsed(duration: Duration) -> String {
    let secs = duration.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

pub enum WaitMsg {
    Execute {
        run_id: String,
        outcome: Result<(TerminalOutcome, Option<PathBuf>), String>,
    },
    Qc {
        run_id: String,
        derived_version_id: String,
        outcome: Result<ArtifactInspectionEvidence, String>,
    },
    Reopen {
        run_id: String,
        derived_version_id: String,
        outcome: Result<(VerificationOutcome, VerificationOutcome), String>,
    },
}

pub struct LongOpSession {
    pub phase: TransferPhase,
    pub run_id: String,
    pub locked_character: Option<String>,
    pub locked_motion: Option<String>,
    pub derived_version_id: Option<String>,
    pub derived_variant_id: Option<String>,
    pub started: Instant,
    pub wait: Option<Receiver<WaitMsg>>,
}

impl LongOpSession {
    pub fn new(run_id: String, locked_character: Option<String>, locked_motion: Option<String>) -> Self {
        Self {
            phase: TransferPhase::Launching,
            run_id,
            locked_character,
            locked_motion,
            derived_version_id: None,
            derived_variant_id: None,
            started: Instant::now(),
            wait: None,
        }
    }

    pub fn elapsed_label(&self) -> String {
        format_elapsed(self.started.elapsed())
    }
}
