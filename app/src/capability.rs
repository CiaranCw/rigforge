//! Backend-neutral worker capability for Compatibility preflight.
//! Product records must not mention bpy operators or PoseBone features.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCapabilityProfile {
    pub integral_frames_only: bool,
    pub ik_supported: bool,
    pub proven_rest_relative_policy: bool,
}

impl WorkerCapabilityProfile {
    /// Accepted V1-3 production boundary. Not a Blender API inventory.
    pub fn v1_3_isolated_worker() -> Self {
        Self {
            integral_frames_only: true,
            ik_supported: false,
            proven_rest_relative_policy: true,
        }
    }

    pub fn capability_tokens(&self) -> Vec<&'static str> {
        let mut tokens = Vec::new();
        if self.proven_rest_relative_policy {
            tokens.push("proven_rest_relative_rotation_only");
            tokens.push("keep_target_rest_scale");
        }
        if !self.ik_supported {
            tokens.push("explicit_no_ik");
        }
        if self.integral_frames_only {
            tokens.push("integral_frame_execution");
        }
        tokens
    }
}
