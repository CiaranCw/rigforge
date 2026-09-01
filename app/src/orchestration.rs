//! JobRun orchestration state. Separate from immutable Domain JobSpec.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Orchestration state for one attempt/run. Not a Domain record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobRunState {
    Queued,
    Dispatchable,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobRunState {
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        match raw {
            "QUEUED" => Ok(Self::Queued),
            "DISPATCHABLE" => Ok(Self::Dispatchable),
            "RUNNING" => Ok(Self::Running),
            "SUCCEEDED" => Ok(Self::Succeeded),
            "FAILED" => Ok(Self::Failed),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(AppError::Orchestration(format!("unknown job state {other}"))),
        }
    }

    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Queued => "QUEUED",
            Self::Dispatchable => "DISPATCHABLE",
            Self::Running => "RUNNING",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    pub fn can_transition(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Queued, Self::Dispatchable)
                | (Self::Queued, Self::Cancelled)
                |             (Self::Dispatchable, Self::Running)
                | (Self::Dispatchable, Self::Cancelled)
                | (Self::Dispatchable, Self::Failed)
                | (Self::Running, Self::Succeeded)
                | (Self::Running, Self::Failed)
        )
    }
}

/// Durable run row. `run_id` / `attempt_id` are orchestration IDs, not Product IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobRun {
    pub run_id: String,
    pub job_spec_id: String,
    pub state: JobRunState,
    pub attempt_id: String,
    pub worker_execution_ref: Option<String>,
    pub failure_reason: Option<String>,
    pub worker_result_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
