//! Backend-neutral worker dispatch port. V1-2 does not implement Blender.

use rigforge_domain::{JobSpec, Validated};

use crate::error::AppError;

/// Opaque receipt from a worker port. Not a Product ID.
///
/// `attempt_id` must echo the orchestrator-owned attempt identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchReceipt {
    pub attempt_id: String,
    pub worker_execution_ref: String,
}

pub trait WorkerPort {
    fn dispatch(
        &mut self,
        spec: &Validated<JobSpec>,
        attempt_id: &str,
    ) -> Result<DispatchReceipt, AppError>;
}

/// In-memory fake worker. Does not spawn Blender, import bpy, or touch FBX.
#[derive(Clone, Debug, Default)]
pub struct FakeWorker {
    pub last_job_spec_id: Option<String>,
    pub last_character_version_id: Option<String>,
    pub last_motion_version_id: Option<String>,
    pub last_attempt_id: Option<String>,
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
            worker_execution_ref: "fake-worker:v1-2".into(),
        })
    }
}
