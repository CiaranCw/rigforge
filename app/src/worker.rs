//! Backend-neutral worker dispatch and completion ports.
//!
//! `dispatch` / `dispatch_resolved` are launch-oriented. Terminal collection
//! is a separate `WorkerCompletionPort`. V1-2 ships `FakeWorker` only.

use std::path::Path;

use rigforge_domain::{JobSpec, Validated, WorkerResult};

use crate::dispatch::WorkerDispatchRequest;
use crate::error::AppError;

/// Opaque receipt from a worker port. Not a Product ID.
///
/// `attempt_id` must echo the orchestrator-owned attempt identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchReceipt {
    pub attempt_id: String,
    pub worker_execution_ref: String,
}

/// Distinguishes ordinary worker terminal failures. Not a universal taxonomy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerFailureClass {
    LaunchFailure,
    WorkerScriptException,
    StructuredWorkerFail,
    MissingResultEnvelope,
    InvalidResultEnvelope,
    JobSpecMismatch,
    BackendBuildMismatch,
    BackendVersionMismatch,
    AdapterVersionMismatch,
    SourceDigestMismatch,
    UnsupportedPolicy,
    ReopenFailure,
    Other(String),
}

impl WorkerFailureClass {
    pub fn as_str(&self) -> &str {
        match self {
            Self::LaunchFailure => "launch_failure",
            Self::WorkerScriptException => "worker_script_exception",
            Self::StructuredWorkerFail => "structured_worker_fail",
            Self::MissingResultEnvelope => "missing_result_envelope",
            Self::InvalidResultEnvelope => "invalid_result_envelope",
            Self::JobSpecMismatch => "job_spec_mismatch",
            Self::BackendBuildMismatch => "backend_build_mismatch",
            Self::BackendVersionMismatch => "backend_version_mismatch",
            Self::AdapterVersionMismatch => "adapter_version_mismatch",
            Self::SourceDigestMismatch => "source_digest_mismatch",
            Self::UnsupportedPolicy => "unsupported_policy",
            Self::ReopenFailure => "reopen_failure",
            Self::Other(value) => value,
        }
    }
}

/// Terminal collection outcome. Success still requires catalog `complete_success`.
#[derive(Clone, Debug)]
pub enum TerminalOutcome {
    Success(Validated<WorkerResult>),
    Failed {
        class: WorkerFailureClass,
        reason: String,
        worker_result: Option<Validated<WorkerResult>>,
    },
}

pub trait WorkerPort {
    fn dispatch(
        &mut self,
        spec: &Validated<JobSpec>,
        attempt_id: &str,
    ) -> Result<DispatchReceipt, AppError>;

    /// Launch from an already-resolved exact Catalog projection.
    ///
    /// Default: ignore source files and call [`WorkerPort::dispatch`] with the
    /// JobSpec and orchestrator `attempt_id`. Real workers override this.
    fn dispatch_resolved(
        &mut self,
        request: &WorkerDispatchRequest,
    ) -> Result<DispatchReceipt, AppError> {
        self.dispatch(request.job_spec(), request.attempt_id())
    }
}

/// Wait for a previously launched attempt. Do not mark JobRun state here.
///
/// Terminal failure conditions should return [`TerminalOutcome::Failed`].
/// If `collect` still returns `Err`, Catalog persists `JobRun FAILED` and
/// must not leave the attempt RUNNING.
pub trait WorkerCompletionPort {
    fn collect(&mut self, receipt: &DispatchReceipt) -> Result<TerminalOutcome, AppError>;

    /// Exact staged artifact path retained by a successful collect, if any.
    ///
    /// Default none. Production Blender collect records this after wait.
    /// Catalog persistence does not read this path.
    fn last_staged_artifact(&self) -> Option<&Path> {
        let _ = self;
        None
    }
}

/// In-memory fake worker. Does not spawn Blender, import bpy, or touch FBX.
#[derive(Clone, Debug, Default)]
pub struct FakeWorker {
    pub last_job_spec_id: Option<String>,
    pub last_character_version_id: Option<String>,
    pub last_motion_version_id: Option<String>,
    pub last_mapping_version_id: Option<String>,
    pub last_policy_version_id: Option<String>,
    pub last_attempt_id: Option<String>,
    pub last_character_location: Option<String>,
    pub fail: bool,
}

impl WorkerPort for FakeWorker {
    fn dispatch(
        &mut self,
        spec: &Validated<JobSpec>,
        attempt_id: &str,
    ) -> Result<DispatchReceipt, AppError> {
        if self.fail {
            return Err(AppError::Worker(
                "fake worker refused dispatch (test failure injection)".into(),
            ));
        }
        let record = spec.as_record();
        self.last_job_spec_id = Some(record.id().canonical());
        self.last_character_version_id = Some(record.character_version_id().canonical());
        self.last_motion_version_id = Some(record.motion_version_id().canonical());
        self.last_attempt_id = Some(attempt_id.to_string());
        Ok(DispatchReceipt {
            attempt_id: attempt_id.to_string(),
            worker_execution_ref: format!("fake-worker:{attempt_id}"),
        })
    }

    fn dispatch_resolved(
        &mut self,
        request: &WorkerDispatchRequest,
    ) -> Result<DispatchReceipt, AppError> {
        self.last_mapping_version_id = Some(request.mapping().as_record().id().canonical());
        self.last_policy_version_id = Some(request.policy().as_record().id().canonical());
        self.last_character_location = Some(request.character().location.display().to_string());
        self.dispatch(request.job_spec(), request.attempt_id())
    }
}
