//! Application session behind the machine protocol.
//!
//! Long Blender waits happen in this process, not in a GUI thread. Mapping
//! proposals are never auto-accepted. Preview bytes are not Product authority.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rigforge_app::{
    sha256_file, AnimationCandidate, AppError, Application, CharacterSourceInspection,
    FixtureSkeletonInspector, MappingAssistProfile, MotionSourceInspection, PreviewGenerationRequest,
    PreviewSubject, SkeletonCandidate, SourceInspectionProvider, TransferOutcomeKind,
    WorkerCapabilityProfile,
};
use rigforge_app::rigforge_domain::Lifecycle;
use rigforge_blender_worker::{BlenderSkeletonInspector, BlenderWorker};
use serde_json::{json, Value};

use crate::protocol::{param_bool, param_str, param_str_opt, Envelope, MachineMessage};
use crate::review::mapping_review_from_version;

pub struct MachineSession {
    app: Option<Application>,
    backend: Backend,
    character_inspections: HashMap<String, CharacterSourceInspection>,
    motion_inspections: HashMap<String, MotionSourceInspection>,
}

enum Backend {
    Production,
    Test {
        inspector: TestInspector,
        skeleton: FixtureSkeletonInspector,
    },
}

#[derive(Clone)]
struct TestInspector {
    clips: Vec<String>,
}

impl TestInspector {
    fn one_skeleton(path: &Path) -> Result<(String, u64, Vec<SkeletonCandidate>), AppError> {
        let digest = sha256_file(path)?;
        let size = std::fs::metadata(path)?.len();
        Ok((
            digest,
            size,
            vec![SkeletonCandidate {
                source_local_key: "Armature".into(),
                display_name: "Armature".into(),
                joint_count: 1,
                joints: vec![rigforge_app::InspectedJoint {
                    joint_key: "root".into(),
                    display_name: "root".into(),
                    parent_key: None,
                    is_root: true,
                    deform_observation: Some("deforming".into()),
                    rest_evidence: Some("rest".into()),
                }],
            }],
        ))
    }
}

impl SourceInspectionProvider for TestInspector {
    fn inspect_character_source(
        &self,
        path: &Path,
    ) -> Result<CharacterSourceInspection, AppError> {
        let (digest, size, skeleton_candidates) = Self::one_skeleton(path)?;
        Ok(CharacterSourceInspection {
            source_path: path.to_path_buf(),
            source_digest: digest,
            size_bytes: size,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates,
            diagnostics: Vec::new(),
        })
    }

    fn inspect_motion_source(&self, path: &Path) -> Result<MotionSourceInspection, AppError> {
        let (digest, size, skeleton_candidates) = Self::one_skeleton(path)?;
        let clips = if self.clips.is_empty() {
            vec!["Walk".to_string(), "Idle".to_string()]
        } else {
            self.clips.clone()
        };
        let animation_candidates = clips
            .into_iter()
            .enumerate()
            .map(|(index, clip_identity)| AnimationCandidate {
                display_label: clip_identity.clone(),
                source_skeleton_local_key: "Armature".into(),
                association_kind: "direct_action".into(),
                association_evidence: Default::default(),
                start_frame: Some(1),
                end_frame: Some(30 + index as i64),
                fps_num: Some(30),
                fps_den: Some(1),
                usable: true,
                unusable_reason: None,
                clip_identity,
            })
            .collect();
        Ok(MotionSourceInspection {
            source_path: path.to_path_buf(),
            source_digest: digest,
            size_bytes: size,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates,
            animation_candidates,
            timing_context: Some(rigforge_app::ObservedTimingContext {
                fps_num: 30,
                fps_den: 1,
            }),
            diagnostics: Vec::new(),
        })
    }
}

impl MachineSession {
    pub fn production() -> Self {
        Self {
            app: None,
            backend: Backend::Production,
            character_inspections: HashMap::new(),
            motion_inspections: HashMap::new(),
        }
    }

    pub fn for_tests() -> Self {
        Self {
            app: None,
            backend: Backend::Test {
                inspector: TestInspector { clips: Vec::new() },
                skeleton: FixtureSkeletonInspector::usable(),
            },
            character_inspections: HashMap::new(),
            motion_inspections: HashMap::new(),
        }
    }

    pub fn handle(
        &mut self,
        envelope: &Envelope,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        match envelope.op.as_str() {
            "ping" => Ok(json!({ "ok": true })),
            "runtime_status" => self.runtime_status(),
            "open_catalog" => self.open_catalog(&envelope.params),
            "inspect_character_source" => self.inspect_character(&envelope.params, emit),
            "inspect_motion_source" => self.inspect_motion(&envelope.params, emit),
            "register_character" => self.register_character(&envelope.params),
            "register_motion_clip" => self.register_motion_clip(&envelope.params),
            "propose_mapping" => self.propose_mapping(&envelope.params, emit),
            "mapping_review" => self.mapping_review(&envelope.params),
            "accept_mapping" => self.accept_mapping(&envelope.params),
            "run_compatibility" => self.run_compatibility(&envelope.params),
            "authorize_transfer" => self.authorize_transfer(&envelope.params),
            "run_transfer" => self.run_transfer(&envelope.params, emit),
            "generate_preview" => self.generate_preview(&envelope.params, emit),
            "resolve_preview" => self.resolve_preview(&envelope.params),
            "resolve_persistence" => self.resolve_persistence(&envelope.params),
            "shutdown" => Ok(json!({ "shutdown": true })),
            other => Err(format!("unknown machine op '{other}'")),
        }
    }

    fn app(&mut self) -> Result<&mut Application, String> {
        self.app
            .as_mut()
            .ok_or_else(|| "catalog is not open".to_string())
    }

    fn runtime_status(&self) -> Result<Value, String> {
        let mut status = json!({
            "machine_version": env!("CARGO_PKG_VERSION"),
            "adapter_version": rigforge_blender_worker::ADAPTER_VERSION,
            "backend_kind": rigforge_blender_worker::BACKEND_KIND,
            "backend_version": rigforge_blender_worker::BLENDER_VERSION,
            "execution_policy_version": rigforge_blender_worker::EXECUTION_POLICY_VERSION,
        });
        match rigforge_app::RuntimeLayout::resolve() {
            Ok(layout) => {
                status["runtime_root"] = json!(layout.root().display().to_string());
                status["blender"] = json!(layout.blender_executable().display().to_string());
                status["blender_present"] = json!(layout.blender_executable().is_file());
                status["worker_present"] = json!(layout.worker_script().is_file());
            }
            Err(err) => {
                status["runtime_root"] = Value::Null;
                status["error"] = json!(err.to_string());
                status["blender_present"] = json!(false);
            }
        }
        Ok(status)
    }

    fn open_catalog(&mut self, params: &Value) -> Result<Value, String> {
        let path = param_str(params, "path")?;
        let parent = Path::new(&path).parent().ok_or("catalog path has no parent")?;
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        self.app = Some(Application::open(&path).map_err(err_str)?);
        Ok(json!({ "path": path }))
    }

    fn inspect_character(
        &mut self,
        params: &Value,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        emit(MachineMessage::progress("inspect", "inspecting"));
        let path = PathBuf::from(param_str(params, "source_path")?);
        let inspection = self.inspect_character_path(&path)?;
        let key = path_key(&path);
        let body = serde_json::to_value(&inspection).map_err(|err| err.to_string())?;
        self.character_inspections.insert(key, inspection);
        Ok(body)
    }

    fn inspect_motion(
        &mut self,
        params: &Value,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        emit(MachineMessage::progress("inspect", "inspecting"));
        let path = PathBuf::from(param_str(params, "source_path")?);
        let inspection = self.inspect_motion_path(&path)?;
        let usable: Vec<Value> = inspection
            .usable_clips()
            .into_iter()
            .map(|clip| {
                json!({
                    "clip_identity": clip.clip_identity,
                    "display_label": clip.display_label,
                    "start_frame": clip.start_frame,
                    "end_frame": clip.end_frame,
                    "fps_num": clip.fps_num,
                    "fps_den": clip.fps_den,
                    "source_skeleton_local_key": clip.source_skeleton_local_key,
                    "presentation": clip.presentation_line(),
                })
            })
            .collect();
        let key = path_key(&path);
        let inspection_json = serde_json::to_value(&inspection).map_err(|err| err.to_string())?;
        self.motion_inspections.insert(key, inspection);
        Ok(json!({
            "inspection": inspection_json,
            "usable_clips": usable,
        }))
    }

    fn register_character(&mut self, params: &Value) -> Result<Value, String> {
        let display_name = param_str(params, "display_name")?;
        let path = PathBuf::from(param_str(params, "source_path")?);
        let inspection = self.cached_or_inspect_character(&path)?;
        let registered = self
            .app()?
            .register_character_from_inspection(&display_name, &inspection)
            .map_err(err_str)?;
        Ok(json!({
            "asset_id": registered.asset_id,
            "version_id": registered.version_id,
            "source_digest": inspection.source_digest,
            "source_path": inspection.source_path.display().to_string(),
        }))
    }

    fn register_motion_clip(&mut self, params: &Value) -> Result<Value, String> {
        let display_name = param_str(params, "display_name")?;
        let path = PathBuf::from(param_str(params, "source_path")?);
        let clip_identity = param_str(params, "clip_identity")?;
        let inspection = self.cached_or_inspect_motion(&path)?;
        if inspection.usable_clip(&clip_identity).is_none() {
            return Err(format!(
                "exact clip_identity '{clip_identity}' is not a usable clip on this Motion source"
            ));
        }
        let registered = self
            .app()?
            .register_motion_from_inspection(&display_name, &inspection, &clip_identity)
            .map_err(err_str)?;
        let clip = inspection
            .usable_clip(&clip_identity)
            .expect("clip checked above");
        Ok(json!({
            "asset_id": registered.asset_id,
            "version_id": registered.version_id,
            "source_skeleton_id": registered.source_skeleton_id,
            "clip_identity": clip.clip_identity,
            "start_frame": clip.start_frame,
            "end_frame": clip.end_frame,
            "fps_num": clip.fps_num,
            "fps_den": clip.fps_den,
            "source_path": inspection.source_path.display().to_string(),
            "source_digest": inspection.source_digest,
        }))
    }

    fn propose_mapping(
        &mut self,
        params: &Value,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        emit(MachineMessage::progress("mapping", "analyzing_skeleton"));
        let character_version_id = param_str(params, "character_version_id")?;
        let motion_version_id = param_str(params, "motion_version_id")?;
        let display_name = param_str_opt(params, "display_name")
            .unwrap_or_else(|| "automatic mapping proposal".into());
        let profile = parse_profile(param_str_opt(params, "profile").as_deref())?;
        let (mapping, version) = self.propose_store(
            &character_version_id,
            &motion_version_id,
            &display_name,
            profile,
        )?;
        let review = mapping_review_from_version(version.as_record());
        Ok(json!({
            "mapping_id": mapping.as_record().id().canonical(),
            "mapping_version_id": version.as_record().id().canonical(),
            "lifecycle": format!("{:?}", version.as_record().lifecycle()),
            "automatic_proposal": true,
            "accepted": false,
            "review": review,
        }))
    }

    fn mapping_review(&mut self, params: &Value) -> Result<Value, String> {
        let mapping_version_id = param_str(params, "mapping_version_id")?;
        let version = self
            .app()?
            .lookup_mapping_version(&mapping_version_id)
            .map_err(err_str)?;
        Ok(serde_json::to_value(mapping_review_from_version(version.as_record()))
            .map_err(|err| err.to_string())?)
    }

    fn accept_mapping(&mut self, params: &Value) -> Result<Value, String> {
        let mapping_id = param_str(params, "mapping_id")?;
        let mapping_version_id = param_str(params, "mapping_version_id")?;
        let version = self
            .app()?
            .accept_mapping_version(&mapping_id, &mapping_version_id)
            .map_err(err_str)?;
        let review = mapping_review_from_version(version.as_record());
        if !review.accepted {
            return Err("accept_mapping_version did not persist explicit review".into());
        }
        Ok(json!({
            "mapping_id": mapping_id,
            "mapping_version_id": version.as_record().id().canonical(),
            "lifecycle": format!("{:?}", version.as_record().lifecycle()),
            "accepted": true,
            "review": review,
        }))
    }

    fn run_compatibility(&mut self, params: &Value) -> Result<Value, String> {
        let character_version_id = param_str(params, "character_version_id")?;
        let motion_version_id = param_str(params, "motion_version_id")?;
        let mapping_version_id = param_str(params, "mapping_version_id")?;
        let policy_version_id = match param_str_opt(params, "policy_version_id") {
            Some(id) => id,
            None => self
                .app()?
                .ensure_published_proven_policy()
                .map_err(err_str)?,
        };
        let capability = WorkerCapabilityProfile::v1_3_isolated_worker();
        let result = self
            .app()?
            .run_compatibility_preflight(
                &character_version_id,
                &motion_version_id,
                &mapping_version_id,
                &policy_version_id,
                &capability,
            )
            .map_err(err_str)?;
        let record = result.as_record();
        let auth = self
            .app()?
            .authorize_transfer(&record.id().canonical(), false)
            .map_err(err_str)?;
        Ok(json!({
            "compatibility_id": record.id().canonical(),
            "policy_version_id": policy_version_id,
            "summary": format!("{:?}", record.summary()),
            "notes": record.notes(),
            "requires_acknowledgement": auth.requires_acknowledgement,
            "eligible_without_ack": auth.eligible,
            "denial_reason": auth.denial_reason,
        }))
    }

    fn authorize_transfer(&mut self, params: &Value) -> Result<Value, String> {
        let compatibility_id = param_str(params, "compatibility_id")?;
        let acknowledged = param_bool(params, "warnings_acknowledged", false);
        let auth = self
            .app()?
            .authorize_transfer(&compatibility_id, acknowledged)
            .map_err(err_str)?;
        Ok(json!({
            "compatibility_id": auth.compatibility_id,
            "summary": format!("{:?}", auth.summary),
            "eligible": auth.eligible,
            "requires_acknowledgement": auth.requires_acknowledgement,
            "denial_reason": auth.denial_reason,
            "warnings_acknowledged": acknowledged,
        }))
    }

    fn run_transfer(
        &mut self,
        params: &Value,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        if matches!(self.backend, Backend::Test { .. }) {
            return Err("run_transfer requires the production Blender worker".into());
        }
        let compatibility_id = param_str(params, "compatibility_id")?;
        let acknowledged = param_bool(params, "warnings_acknowledged", false);
        let display_name =
            param_str_opt(params, "display_name").unwrap_or_else(|| "derived variant".into());
        emit(MachineMessage::progress(&params_id(params), "generating_derived"));
        let (_spec, run) = self
            .app()?
            .start_transfer(&compatibility_id, acknowledged, None, display_name)
            .map_err(err_str)?;
        let run_id = run.run_id.clone();
        emit(MachineMessage::progress(&run_id, "generating_derived"));
        let mut worker = BlenderWorker::production().map_err(err_str)?;
        self.app()?
            .mark_dispatchable(&run_id)
            .map_err(err_str)?;
        self.app()?
            .dispatch(&run_id, &mut worker)
            .map_err(err_str)?;
        let (collected, outcome) = self
            .app()?
            .collect(&run_id, &mut worker)
            .map_err(err_str)?;
        match outcome {
            rigforge_app::TerminalOutcome::Success(_) => {
                let staged = worker
                    .last_staged_blend()
                    .map(PathBuf::from)
                    .ok_or_else(|| {
                        "successful collect did not retain a staged artifact path".to_string()
                    })?;
                emit(MachineMessage::progress(&run_id, "quality_check"));
                emit(MachineMessage::progress(&run_id, "verifying"));
                let transfer = self
                    .app()?
                    .finalize_transfer(&collected.run_id, &staged)
                    .map_err(err_str)?;
                if transfer.kind != TransferOutcomeKind::Published {
                    return Err(transfer
                        .reason
                        .unwrap_or_else(|| "QC or persistence verification denied publication".into()));
                }
                Ok(json!({
                    "job_run_id": transfer.job_run_id,
                    "job_spec_id": transfer.job_spec_id,
                    "derived_variant_id": transfer.derived_variant_id,
                    "derived_variant_version_id": transfer.derived_variant_version_id,
                    "qc_verdict": transfer.qc_verdict,
                    "qc_report_id": transfer.qc_report_id,
                    "persistence_verification_id": transfer.persistence_verification_id,
                    "persistence_artifact_id": transfer.persistence_artifact_id,
                    "lifecycle": "Published",
                    "kind": "Published",
                }))
            }
            rigforge_app::TerminalOutcome::Failed { reason, .. } => Err(reason),
        }
    }

    fn generate_preview(
        &mut self,
        params: &Value,
        emit: &mut dyn FnMut(MachineMessage),
    ) -> Result<Value, String> {
        let version_id = param_str(params, "derived_variant_version_id")?;
        self.require_published_derived(&version_id)?;
        emit(MachineMessage::progress(&version_id, "preparing_preview"));
        self.app()?
            .generate_preview(PreviewGenerationRequest::derived_variant(&version_id))
            .map_err(err_str)?;
        self.resolve_preview_inner(&version_id)
    }

    fn resolve_preview(&mut self, params: &Value) -> Result<Value, String> {
        let version_id = param_str(params, "derived_variant_version_id")?;
        self.resolve_preview_inner(&version_id)
    }

    fn resolve_preview_inner(&mut self, version_id: &str) -> Result<Value, String> {
        let view = self
            .app()?
            .resolve_preview_for_display(PreviewSubject::DerivedVariant {
                version_id: version_id.to_string(),
            });
        if let Some(failure) = view.failure {
            return Err(format!("{}: {}", failure.kind.as_str(), failure.detail));
        }
        let artifact = view
            .artifact
            .as_ref()
            .ok_or_else(|| "preview artifact missing after resolve".to_string())?;
        let path = artifact
            .as_record()
            .location()
            .map(|location| location.value().to_string())
            .ok_or_else(|| "preview artifact has no location evidence".to_string())?;
        Ok(json!({
            "derived_variant_version_id": version_id,
            "preview_artifact_id": artifact.as_record().id().canonical(),
            "payload_path": path,
            "digest": artifact.as_record().digest().sha256(),
            "size_bytes": artifact.as_record().size_bytes(),
            "media_type": artifact.as_record().media_type(),
            "descriptor": view.descriptor,
            "authoritative": false,
        }))
    }

    fn resolve_persistence(&mut self, params: &Value) -> Result<Value, String> {
        let version_id = param_str(params, "derived_variant_version_id")?;
        let version = self.require_published_derived(&version_id)?;
        let artifact_id = version
            .persistence_artifact_id()
            .ok_or_else(|| "published derived version has no PersistenceArtifact".to_string())?;
        let artifact = self
            .app()?
            .load_artifact_metadata(&artifact_id.canonical())
            .map_err(err_str)?;
        let path = self
            .app()?
            .catalog()
            .durable_artifact_path(artifact.as_record());
        Ok(json!({
            "derived_variant_version_id": version_id,
            "lifecycle": format!("{:?}", version.lifecycle()),
            "persistence_artifact_id": artifact_id.canonical(),
            "artifact_path": path.display().to_string(),
            "digest": artifact.as_record().digest().sha256(),
            "size_bytes": artifact.as_record().size_bytes(),
        }))
    }

    fn require_published_derived(
        &mut self,
        version_id: &str,
    ) -> Result<rigforge_app::rigforge_domain::DerivedVariantVersion, String> {
        let version = self
            .app()?
            .catalog()
            .load_derived_variant_version(version_id)
            .map_err(err_str)?
            .into_record();
        if version.lifecycle() != Lifecycle::Published {
            return Err(format!(
                "DerivedVariantVersion {version_id} is {:?}; Host review requires Published",
                version.lifecycle()
            ));
        }
        Ok(version)
    }

    fn propose_store(
        &mut self,
        character_version_id: &str,
        motion_version_id: &str,
        display_name: &str,
        profile: MappingAssistProfile,
    ) -> Result<
        (
            rigforge_app::rigforge_domain::Validated<rigforge_app::rigforge_domain::BoneMapping>,
            rigforge_app::rigforge_domain::Validated<
                rigforge_app::rigforge_domain::BoneMappingVersion,
            >,
        ),
        String,
    > {
        match &self.backend {
            Backend::Production => {
                let inspector = BlenderSkeletonInspector::production().map_err(err_str)?;
                self.app()?
                    .propose_and_store_mapping_for_selection(
                        character_version_id,
                        motion_version_id,
                        &inspector,
                        profile,
                        display_name,
                    )
                    .map_err(err_str)
            }
            Backend::Test { skeleton, .. } => {
                let skeleton = skeleton.clone();
                self.app()?
                    .propose_and_store_mapping_for_selection(
                        character_version_id,
                        motion_version_id,
                        &skeleton,
                        profile,
                        display_name,
                    )
                    .map_err(err_str)
            }
        }
    }

    fn inspect_character_path(
        &mut self,
        path: &Path,
    ) -> Result<CharacterSourceInspection, String> {
        match &self.backend {
            Backend::Production => BlenderSkeletonInspector::production()
                .map_err(err_str)?
                .inspect_character_source(path)
                .map_err(err_str),
            Backend::Test { inspector, .. } => inspector
                .inspect_character_source(path)
                .map_err(err_str),
        }
    }

    fn inspect_motion_path(&mut self, path: &Path) -> Result<MotionSourceInspection, String> {
        match &self.backend {
            Backend::Production => BlenderSkeletonInspector::production()
                .map_err(err_str)?
                .inspect_motion_source(path)
                .map_err(err_str),
            Backend::Test { inspector, .. } => inspector.inspect_motion_source(path).map_err(err_str),
        }
    }

    fn cached_or_inspect_character(
        &mut self,
        path: &Path,
    ) -> Result<CharacterSourceInspection, String> {
        if let Some(existing) = self.character_inspections.get(&path_key(path)) {
            live_source_matches(path, &existing.source_digest, existing.size_bytes)?;
            return Ok(existing.clone());
        }
        let inspection = self.inspect_character_path(path)?;
        self.character_inspections
            .insert(path_key(path), inspection.clone());
        Ok(inspection)
    }

    fn cached_or_inspect_motion(&mut self, path: &Path) -> Result<MotionSourceInspection, String> {
        if let Some(existing) = self.motion_inspections.get(&path_key(path)) {
            live_source_matches(path, &existing.source_digest, existing.size_bytes)?;
            return Ok(existing.clone());
        }
        let inspection = self.inspect_motion_path(path)?;
        self.motion_inspections
            .insert(path_key(path), inspection.clone());
        Ok(inspection)
    }
}

pub fn handle_line(
    session: &mut MachineSession,
    line: &str,
    emit: &mut dyn FnMut(MachineMessage),
) -> Result<bool, String> {
    let envelope = crate::protocol::parse_line(line)?;
    if envelope.op == "shutdown" {
        emit(MachineMessage::ok(&envelope.id, json!({ "shutdown": true })));
        return Ok(true);
    }
    match session.handle(&envelope, emit) {
        Ok(result) => emit(MachineMessage::ok(&envelope.id, result)),
        Err(err) => emit(MachineMessage::err(&envelope.id, err)),
    }
    Ok(false)
}

fn parse_profile(value: Option<&str>) -> Result<MappingAssistProfile, String> {
    match value.unwrap_or("optional_humanoid") {
        "none" => Ok(MappingAssistProfile::None),
        "optional_humanoid" | "humanoid" => Ok(MappingAssistProfile::OptionalHumanoid),
        other => Err(format!("unknown mapping profile '{other}'")),
    }
}

fn path_key(path: &Path) -> String {
    path.display().to_string()
}

fn params_id(params: &Value) -> String {
    param_str_opt(params, "compatibility_id").unwrap_or_else(|| "transfer".into())
}

fn live_source_matches(path: &Path, expected_digest: &str, expected_size: u64) -> Result<(), String> {
    if !path.is_file() {
        return Err("This file changed after it was inspected. Inspect it again.".into());
    }
    let size = std::fs::metadata(path).map_err(|err| err.to_string())?.len();
    let digest = sha256_file(path).map_err(|err| err.to_string())?;
    if digest != expected_digest || size != expected_size {
        return Err("This file changed after it was inspected. Inspect it again.".into());
    }
    Ok(())
}

fn err_str(err: AppError) -> String {
    err.to_string()
}
