//! Structured worker envelope. Not a Domain WorkerResult and not Product identity.

use serde::{Deserialize, Serialize};

pub const ENVELOPE_SCHEMA: &str = "rigforge.blender_worker.envelope.v1";
pub const JOB_SCHEMA: &str = "rigforge.blender_worker.job.v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeBackend {
    pub kind: String,
    pub version: String,
    pub build: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeMeasurement {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerEnvelope {
    pub schema: String,
    pub status: String,
    #[serde(default)]
    pub failure_class: Option<String>,
    pub job_spec_id: String,
    pub attempt_id: String,
    pub backend: EnvelopeBackend,
    #[serde(default)]
    pub adapter_version: Option<String>,
    #[serde(default)]
    pub staged_blend: Option<String>,
    #[serde(default)]
    pub measurements: Vec<EnvelopeMeasurement>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReopenEnvelope {
    pub status: String,
    #[serde(default)]
    pub target_present: bool,
    #[serde(default)]
    pub baked_action: Option<String>,
    #[serde(default)]
    pub frame_start: Option<i64>,
    #[serde(default)]
    pub frame_end: Option<i64>,
    #[serde(default)]
    pub bone_count: Option<i64>,
    pub root_scale_audit: String,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

impl WorkerEnvelope {
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("SUCCESS")
    }
}
