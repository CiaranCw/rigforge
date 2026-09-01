//! RigForge V1-3 pinned isolated Blender worker adapter.
//!
//! Not Product Domain, not GUI, and not the research harness.

pub mod adapter;
pub mod command;
pub mod envelope;
pub mod inspect;
pub mod isolation;
pub mod pin;
pub mod policy;
pub mod projection;

pub use adapter::{
    bundled_worker_script, diagnostic_tail, production_worker_script, BlenderWorker,
};
pub use command::{
    assert_safety_flags, blender_argv, blender_command, BACKGROUND, DISABLE_AUTOEXEC,
    FACTORY_STARTUP, PYTHON, PYTHON_EXIT_CODE,
};
pub use inspect::BlenderSkeletonInspector;
pub use isolation::{attempt_workspace_root, AttemptWorkspace};
pub use pin::{
    default_archive_path, default_blender_executable, enforce_pin, parse_blender_version_output,
    sha256_bytes, sha256_file, verify_archive_sha256, BlenderPin, ADAPTER_VERSION, BACKEND_KIND,
    BLENDER_ARCHIVE_SHA256, BLENDER_BUILD, BLENDER_VERSION,
};
pub use policy::{project_supported_policy, PolicyProjection};
pub use projection::{job_document, project_mapping, SOURCE_REST_ACTION_CANDIDATES};
