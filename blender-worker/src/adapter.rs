//! Launch-only Blender adapter plus separate terminal collection.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};

use rigforge_app::{
    AppError, DispatchReceipt, TerminalOutcome, WorkerCompletionPort, WorkerDispatchRequest,
    WorkerFailureClass, WorkerPort,
};
use rigforge_domain::{
    ingest_validated, BackendExecutionContext, ContentDigest, ExecutionCorrelation, JobSpec,
    NamedMeasurement, Validated, WorkerResult,
};
use serde_json::Value;

use crate::command::{assert_safety_flags, blender_argv, blender_command};
use crate::envelope::{ReopenEnvelope, WorkerEnvelope, ENVELOPE_SCHEMA};
use crate::isolation::{attempt_workspace_root, AttemptWorkspace};
use crate::pin::{
    enforce_pin, sha256_file, ADAPTER_VERSION, BACKEND_KIND, BLENDER_BUILD, BLENDER_VERSION,
    BlenderPin, EXECUTION_POLICY_VERSION,
};
use crate::policy::project_supported_policy;
use crate::projection::job_document;

struct LaunchedAttempt {
    attempt_id: String,
    job_spec: Validated<JobSpec>,
    workspace: AttemptWorkspace,
    child: Option<Child>,
    job_json: PathBuf,
    result_path: PathBuf,
    staged_blend: PathBuf,
    reopen_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

/// Explicit non-production test boundary. Ordinary production callers never
/// construct this and cannot disable production guards.
#[derive(Clone, Debug)]
struct TestHarness {
    behavior: Option<String>,
    verify_sources: bool,
    skip_reopen: bool,
    reopen_executable: Option<PathBuf>,
}

enum ExecutePhase {
    Waited {
        code: i32,
        envelope: Option<WorkerEnvelope>,
        class: WorkerFailureClass,
    },
    CollectFailed {
        class: WorkerFailureClass,
        reason: String,
    },
}

/// Production Blender adapter. Safety guards are sealed: pin, source digest,
/// fresh reopen, and the crate-bundled worker script cannot be disabled.
pub struct BlenderWorker {
    pin: BlenderPin,
    blender: PathBuf,
    script: PathBuf,
    workspace_root: PathBuf,
    launched: HashMap<String, LaunchedAttempt>,
    test: Option<TestHarness>,
    last_staged_blend: Option<PathBuf>,
}

impl BlenderWorker {
    pub fn production() -> Result<Self, AppError> {
        let pin = BlenderPin::accepted();
        Ok(Self {
            blender: pin.executable.clone(),
            script: production_worker_script(),
            workspace_root: std::env::temp_dir().join("rigforge-v1-3-attempts"),
            pin,
            launched: HashMap::new(),
            test: None,
            last_staged_blend: None,
        })
    }

    pub fn with_workspace_root(mut self, workspace_root: PathBuf) -> Self {
        self.workspace_root = workspace_root;
        self
    }

    /// Test-only fake executable. Not a production constructor.
    ///
    /// Pin verification is skipped because the fake is not Blender 5.2.1.
    /// Source verification defaults off so absent fixture paths can exercise
    /// command/envelope paths. Reopen remains required for success.
    pub fn for_fake_executable(executable: PathBuf, workspace_root: PathBuf) -> Self {
        let pin = BlenderPin::accepted().with_executable(executable.clone());
        Self {
            blender: executable,
            script: production_worker_script(),
            workspace_root,
            pin,
            launched: HashMap::new(),
            test: Some(TestHarness {
                behavior: Some("success".into()),
                verify_sources: false,
                skip_reopen: false,
                reopen_executable: None,
            }),
            last_staged_blend: None,
        }
    }

    pub fn last_staged_blend(&self) -> Option<&Path> {
        self.last_staged_blend.as_deref()
    }

    pub fn is_production(&self) -> bool {
        self.test.is_none()
    }

    pub fn pin_verification_is_mandatory(&self) -> bool {
        self.is_production()
    }

    pub fn source_verification_is_mandatory(&self) -> bool {
        self.is_production()
    }

    pub fn reopen_is_mandatory_for_success(&self) -> bool {
        true
    }

    pub fn worker_script(&self) -> &Path {
        &self.script
    }

    pub fn launched_count(&self) -> usize {
        self.launched.len()
    }

    pub fn set_test_behavior(&mut self, behavior: &str) -> Result<(), AppError> {
        match self.test.as_mut() {
            Some(test) => {
                test.behavior = Some(behavior.to_string());
                Ok(())
            }
            None => Err(AppError::Worker(
                "test behavior is not available on the production Blender worker".into(),
            )),
        }
    }

    pub fn enable_source_verification_for_test(&mut self) -> Result<(), AppError> {
        match self.test.as_mut() {
            Some(test) => {
                test.verify_sources = true;
                Ok(())
            }
            None => Err(AppError::Worker(
                "production source verification is mandatory and cannot be reconfigured".into(),
            )),
        }
    }

    pub fn skip_reopen_for_test(&mut self) -> Result<(), AppError> {
        match self.test.as_mut() {
            Some(test) => {
                test.skip_reopen = true;
                Ok(())
            }
            None => Err(AppError::Worker(
                "production fresh-process reopen is mandatory and cannot be disabled".into(),
            )),
        }
    }

    pub fn set_reopen_executable_for_test(&mut self, executable: PathBuf) -> Result<(), AppError> {
        match self.test.as_mut() {
            Some(test) => {
                test.reopen_executable = Some(executable);
                Ok(())
            }
            None => Err(AppError::Worker(
                "production reopen executable cannot be redirected off the pinned Blender".into(),
            )),
        }
    }

    pub fn last_command_argv(
        &self,
        request: &WorkerDispatchRequest,
        mode: &str,
    ) -> Result<Vec<std::ffi::OsString>, AppError> {
        let ws = attempt_workspace_root(&self.workspace_root, request.attempt_id());
        let job = ws.join("job.json");
        Ok(blender_argv(&self.blender, &self.script, mode, &job))
    }

    fn execution_ref(attempt_id: &str) -> String {
        format!("blender-worker:{attempt_id}")
    }

    fn verify_sources(&self) -> bool {
        self.test
            .as_ref()
            .map(|test| test.verify_sources)
            .unwrap_or(true)
    }

    fn verify_request_sources(&self, request: &WorkerDispatchRequest) -> Result<(), AppError> {
        if !self.verify_sources() {
            return Ok(());
        }
        for (label, src) in [
            ("character", request.character()),
            ("motion", request.motion()),
        ] {
            if !src.location.is_file() {
                return Err(AppError::Worker(format!(
                    "source digest mismatch: missing {label} file {}",
                    src.location.display()
                )));
            }
            let found = sha256_file(&src.location)?;
            if found != src.digest_sha256.to_ascii_lowercase() {
                return Err(AppError::Worker(format!(
                    "source digest mismatch for {label}: expected {} found {found}",
                    src.digest_sha256
                )));
            }
        }
        Ok(())
    }

    fn spawn_blender(
        &self,
        blender: &Path,
        attempt: &LaunchedAttempt,
        mode: &str,
        stdout_path: &Path,
        stderr_path: &Path,
    ) -> Result<Child, AppError> {
        let argv = blender_argv(blender, &self.script, mode, &attempt.job_json);
        if !assert_safety_flags(&argv) {
            return Err(AppError::Worker(
                "internal error: blender command is missing required safety flags".into(),
            ));
        }
        let mut cmd = blender_command(blender, &self.script, mode, &attempt.job_json);
        cmd.current_dir(&attempt.workspace.root);
        cmd.envs(attempt.workspace.isolated_env());
        let stdout = File::create(stdout_path)?;
        let stderr = File::create(stderr_path)?;
        cmd.stdout(Stdio::from(stdout));
        cmd.stderr(Stdio::from(stderr));
        cmd.spawn().map_err(|err| {
            AppError::Worker(format!("launch failure: spawn blender: {err}"))
        })
    }

    fn wait_child(child: &mut Child) -> Result<i32, AppError> {
        let status = child.wait().map_err(|err| {
            AppError::Worker(format!("worker collect failed while waiting: {err}"))
        })?;
        Ok(status.code().unwrap_or(-1))
    }

    fn read_text(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    fn parse_envelope(&self, path: &Path) -> Result<WorkerEnvelope, WorkerFailureClass> {
        if !path.is_file() {
            return Err(WorkerFailureClass::MissingResultEnvelope);
        }
        let text = fs::read_to_string(path)
            .map_err(|_| WorkerFailureClass::InvalidResultEnvelope)?;
        let parsed: WorkerEnvelope = serde_json::from_str(&text)
            .map_err(|_| WorkerFailureClass::InvalidResultEnvelope)?;
        if parsed.schema != ENVELOPE_SCHEMA {
            return Err(WorkerFailureClass::InvalidResultEnvelope);
        }
        if parsed.job_spec_id.trim().is_empty() || parsed.attempt_id.trim().is_empty() {
            return Err(WorkerFailureClass::InvalidResultEnvelope);
        }
        Ok(parsed)
    }

    fn backend_context(
        &self,
        envelope: Option<&WorkerEnvelope>,
    ) -> Result<BackendExecutionContext, AppError> {
        let kind = envelope
            .map(|e| e.backend.kind.as_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(BACKEND_KIND);
        let version = envelope
            .map(|e| e.backend.version.as_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(BLENDER_VERSION);
        let build = envelope
            .map(|e| e.backend.build.as_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(BLENDER_BUILD);
        let adapter = envelope
            .and_then(|e| e.adapter_version.as_deref())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(ADAPTER_VERSION);
        BackendExecutionContext::new(
            kind,
            version,
            build,
            adapter,
            EXECUTION_POLICY_VERSION,
        )
        .map_err(AppError::from)
    }

    fn worker_result(
        &self,
        attempt: &LaunchedAttempt,
        success: bool,
        terminal_status: &str,
        envelope: Option<&WorkerEnvelope>,
        extra_diagnostics: Vec<String>,
        staged: Vec<ContentDigest>,
    ) -> Result<Validated<WorkerResult>, AppError> {
        let execution = self.backend_context(envelope)?;
        let mut measurements = Vec::new();
        if let Some(env) = envelope {
            for item in &env.measurements {
                if let Ok(m) = NamedMeasurement::new(&item.name, &item.value, None) {
                    measurements.push(m);
                }
            }
        }
        let mut diagnostics = extra_diagnostics;
        if let Some(env) = envelope {
            diagnostics.extend(env.diagnostics.clone());
            diagnostics.extend(env.errors.clone());
        }
        let stdout = Self::read_text(&attempt.stdout_path);
        let stderr = Self::read_text(&attempt.stderr_path);
        if !stdout.trim().is_empty() {
            diagnostics.push(format!("stdout_bytes={}", stdout.len()));
        }
        if !stderr.trim().is_empty() {
            diagnostics.push(format!("stderr_tail={}", tail(&stderr, 1200)));
        }
        diagnostics.push(format!(
            "workspace={}",
            attempt.workspace.root.display()
        ));
        diagnostics.push("derived_variant_published=false".into());
        let mut result = WorkerResult::new(
            attempt.job_spec.as_record().id(),
            execution,
            success,
            terminal_status,
            ExecutionCorrelation::new(
                &attempt.attempt_id,
                Self::execution_ref(&attempt.attempt_id),
            )?,
        )?;
        result = result.with_measurements(measurements)?;
        result = result.with_diagnostics(diagnostics)?;
        result = result.with_applied_mapping_projection_note(
            "exact BoneMappingVersion projected to worker joint correspondence; no auto-map",
        )?;
        if !staged.is_empty() {
            result = result.with_staged_artifact_digests(staged)?;
        }
        ingest_validated(&rigforge_domain::to_json(&result)?).map_err(AppError::from)
    }

    fn failed_outcome(
        &self,
        attempt: LaunchedAttempt,
        class: WorkerFailureClass,
        reason: String,
        envelope: Option<&WorkerEnvelope>,
        diagnostics: Vec<String>,
    ) -> Result<TerminalOutcome, AppError> {
        let result = self.worker_result(
            &attempt,
            false,
            class.as_str(),
            envelope,
            diagnostics,
            Vec::new(),
        )?;
        Ok(TerminalOutcome::Failed {
            class,
            reason,
            worker_result: Some(result),
        })
    }

    fn collect_execute(
        &mut self,
        receipt: &DispatchReceipt,
    ) -> Result<(LaunchedAttempt, ExecutePhase), AppError> {
        let key = receipt.worker_execution_ref.clone();
        let mut attempt = self.launched.remove(&key).ok_or_else(|| {
            AppError::Worker(format!(
                "unknown worker_execution_ref {}; workspace should be {}",
                key,
                attempt_workspace_root(&self.workspace_root, &receipt.attempt_id).display()
            ))
        })?;
        if attempt.attempt_id != receipt.attempt_id {
            return Ok((
                attempt,
                ExecutePhase::CollectFailed {
                    class: WorkerFailureClass::Other("collection_failure".into()),
                    reason: "worker_execution_ref does not match orchestrator attempt_id".into(),
                },
            ));
        }
        let code = if let Some(child) = attempt.child.as_mut() {
            match Self::wait_child(child) {
                Ok(code) => code,
                Err(err) => {
                    attempt.child = None;
                    return Ok((
                        attempt,
                        ExecutePhase::CollectFailed {
                            class: WorkerFailureClass::Other("collection_failure".into()),
                            reason: err.to_string(),
                        },
                    ));
                }
            }
        } else {
            return Ok((
                attempt,
                ExecutePhase::CollectFailed {
                    class: WorkerFailureClass::Other("collection_failure".into()),
                    reason: "launched process handle is missing".into(),
                },
            ));
        };
        attempt.child = None;
        let result_path = attempt.result_path.clone();
        let job_spec_id = attempt.job_spec.as_record().id().canonical();
        let attempt_id = attempt.attempt_id.clone();
        let parsed = self.parse_envelope(&result_path);
        match parsed {
            Ok(envelope) => {
                let mut class = if envelope.is_success() {
                    WorkerFailureClass::Other("success".into())
                } else {
                    class_from_envelope(&envelope)
                };
                if envelope.job_spec_id != job_spec_id {
                    class = WorkerFailureClass::JobSpecMismatch;
                }
                if envelope.attempt_id != attempt_id {
                    class = WorkerFailureClass::Other("attempt_id mismatch".into());
                }
                if envelope.is_success()
                    && matches!(class, WorkerFailureClass::Other(ref s) if s == "success")
                {
                    if let Some(id_class) = success_envelope_identity_failure(&envelope) {
                        class = id_class;
                    }
                }
                Ok((
                    attempt,
                    ExecutePhase::Waited {
                        code,
                        envelope: Some(envelope),
                        class,
                    },
                ))
            }
            Err(class) => {
                if code != 0 {
                    Ok((
                        attempt,
                        ExecutePhase::Waited {
                            code,
                            envelope: None,
                            class: WorkerFailureClass::WorkerScriptException,
                        },
                    ))
                } else {
                    Ok((
                        attempt,
                        ExecutePhase::Waited {
                            code,
                            envelope: None,
                            class,
                        },
                    ))
                }
            }
        }
    }
}

impl WorkerPort for BlenderWorker {
    fn dispatch(
        &mut self,
        _spec: &Validated<JobSpec>,
        _attempt_id: &str,
    ) -> Result<DispatchReceipt, AppError> {
        Err(AppError::Worker(
            "blender worker requires resolved WorkerDispatchRequest; refusing JobSpec-only dispatch so latest/current cannot be resolved inside the worker".into(),
        ))
    }

    fn dispatch_resolved(
        &mut self,
        request: &WorkerDispatchRequest,
    ) -> Result<DispatchReceipt, AppError> {
        project_supported_policy(request.policy().as_record())?;
        if self.pin_verification_is_mandatory() {
            enforce_pin(&self.blender, &self.pin)?;
        }
        self.verify_request_sources(request).map_err(|err| {
            if err.to_string().contains("digest mismatch") {
                AppError::Worker(format!("{} ({})", WorkerFailureClass::SourceDigestMismatch.as_str(), err))
            } else {
                err
            }
        })?;
        if !request.character().location.is_file() && self.verify_sources() {
            return Err(AppError::Worker(
                "launch failure: character source file is missing".into(),
            ));
        }
        let root = attempt_workspace_root(&self.workspace_root, request.attempt_id());
        let workspace = AttemptWorkspace::create(&root)?;
        let job_json = workspace.root.join("job.json");
        let result_path = workspace.root.join("result.json");
        let staged_blend = workspace.staged.join("derived_result.blend");
        let reopen_path = workspace.root.join("reopen_verification.json");
        let stdout_path = workspace.root.join("execute_stdout.txt");
        let stderr_path = workspace.root.join("execute_stderr.txt");
        let test_behavior = self.test.as_ref().and_then(|t| t.behavior.as_deref());
        let job = job_document(
            request,
            &workspace.root,
            &staged_blend,
            &result_path,
            &reopen_path,
            test_behavior,
        )?;
        fs::write(&job_json, serde_json::to_vec_pretty(&job).map_err(|e| {
            AppError::Worker(format!("failed to serialize worker job: {e}"))
        })?)?;
        let mut launch = serde_json::json!({
            "attempt_id": request.attempt_id(),
            "job_spec_id": request.job_spec().as_record().id().canonical(),
            "worker_execution_ref": Self::execution_ref(request.attempt_id()),
            "workspace": workspace.root,
            "command": blender_argv(&self.blender, &self.script, "execute", &job_json)
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        });
        fs::write(
            workspace.root.join("launch.json"),
            serde_json::to_vec_pretty(&launch).unwrap_or_default(),
        )?;
        let mut attempt = LaunchedAttempt {
            attempt_id: request.attempt_id().to_string(),
            job_spec: request.job_spec().clone(),
            workspace,
            child: None,
            job_json,
            result_path,
            staged_blend,
            reopen_path,
            stdout_path,
            stderr_path,
        };
        let child = self.spawn_blender(
            &self.blender,
            &attempt,
            "execute",
            &attempt.stdout_path,
            &attempt.stderr_path,
        )?;
        launch
            .as_object_mut()
            .map(|o| o.insert("pid".into(), Value::from(child.id())));
        let _ = fs::write(
            attempt.workspace.root.join("launch.json"),
            serde_json::to_vec_pretty(&launch).unwrap_or_default(),
        );
        let receipt = DispatchReceipt {
            attempt_id: request.attempt_id().to_string(),
            worker_execution_ref: Self::execution_ref(request.attempt_id()),
        };
        attempt.child = Some(child);
        self.launched
            .insert(receipt.worker_execution_ref.clone(), attempt);
        Ok(receipt)
    }
}

impl WorkerCompletionPort for BlenderWorker {
    fn collect(&mut self, receipt: &DispatchReceipt) -> Result<TerminalOutcome, AppError> {
        let (attempt, phase) = self.collect_execute(receipt)?;
        let stdout = Self::read_text(&attempt.stdout_path);
        let stderr = Self::read_text(&attempt.stderr_path);
        let mut diagnostics = Vec::new();
        if !stdout.is_empty() {
            diagnostics.push("execute_stdout_captured=true".into());
        }
        if !stderr.is_empty() {
            diagnostics.push("execute_stderr_captured=true".into());
        }

        let (code, envelope, mut class) = match phase {
            ExecutePhase::CollectFailed { class, reason } => {
                diagnostics.push(format!("execute_collect_error={reason}"));
                return self.failed_outcome(attempt, class, reason, None, diagnostics);
            }
            ExecutePhase::Waited {
                code,
                envelope,
                class,
            } => (code, envelope, class),
        };
        diagnostics.insert(0, format!("execute_exit_code={code}"));

        let envelope_ok = envelope.as_ref().map(|e| e.is_success()).unwrap_or(false);
        if envelope_ok && code != 0 {
            class = WorkerFailureClass::WorkerScriptException;
            diagnostics.push("process status and structured SUCCESS envelope disagree".into());
        }

        let success_candidate = envelope_ok
            && code == 0
            && matches!(class, WorkerFailureClass::Other(ref s) if s == "success");

        if !success_candidate {
            let reason = format!(
                "{}: {}",
                class.as_str(),
                envelope
                    .as_ref()
                    .and_then(|e| e.errors.first().cloned())
                    .unwrap_or_else(|| "worker attempt did not succeed".into())
            );
            return self.failed_outcome(attempt, class, reason, envelope.as_ref(), diagnostics);
        }

        let env = envelope.as_ref().unwrap();
        if !scale_success_measurements_pass(env) {
            diagnostics.push("keep_target_rest_scale_or_root_scale_audit!=PASS".into());
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::StructuredWorkerFail,
                "keep_target_rest_scale / root_scale_audit did not PASS".into(),
                Some(env),
                diagnostics,
            );
        }
        if !attempt.staged_blend.is_file() {
            let mut d = diagnostics.clone();
            d.push("success envelope without staged blend; file existence is not success".into());
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::StructuredWorkerFail,
                "success envelope without staged blend".into(),
                Some(env),
                d,
            );
        }

        if self.test.as_ref().map(|t| t.skip_reopen).unwrap_or(false) {
            diagnostics.push("reopen_status=SKIPPED".into());
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::ReopenFailure,
                "fresh-process reopen is required for successful V1-3 execution".into(),
                Some(env),
                diagnostics,
            );
        }

        let reopen_stdout = attempt.workspace.root.join("reopen_stdout.txt");
        let reopen_stderr = attempt.workspace.root.join("reopen_stderr.txt");
        let reopen_blender = self
            .test
            .as_ref()
            .and_then(|t| t.reopen_executable.clone())
            .unwrap_or_else(|| self.blender.clone());
        let mut cmd = blender_command(
            &reopen_blender,
            &self.script,
            "reopen",
            &attempt.job_json,
        );
        cmd.current_dir(&attempt.workspace.root);
        cmd.envs(attempt.workspace.isolated_env());
        let stdout_file = match File::create(&reopen_stdout) {
            Ok(file) => file,
            Err(err) => {
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    format!("reopen launch failure: {err}"),
                    Some(env),
                    diagnostics,
                );
            }
        };
        let stderr_file = match File::create(&reopen_stderr) {
            Ok(file) => file,
            Err(err) => {
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    format!("reopen launch failure: {err}"),
                    Some(env),
                    diagnostics,
                );
            }
        };
        cmd.stdout(Stdio::from(stdout_file));
        cmd.stderr(Stdio::from(stderr_file));
        let status = match cmd.status() {
            Ok(status) => status,
            Err(err) => {
                diagnostics.push(format!("reopen_launch_error={err}"));
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    format!("reopen launch failure: {err}"),
                    Some(env),
                    diagnostics,
                );
            }
        };
        let reopen_code = status.code().unwrap_or(-1);
        diagnostics.push(format!("reopen_exit_code={reopen_code}"));
        if !attempt.reopen_path.is_file() || reopen_code != 0 {
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::ReopenFailure,
                "fresh-process reopen did not produce a successful verification".into(),
                Some(env),
                diagnostics,
            );
        }
        let reopen_text = match fs::read_to_string(&attempt.reopen_path) {
            Ok(text) => text,
            Err(err) => {
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    format!("invalid reopen envelope: {err}"),
                    Some(env),
                    diagnostics,
                );
            }
        };
        let reopen_value: Value = match serde_json::from_str(&reopen_text) {
            Ok(value) => value,
            Err(_) => {
                diagnostics.push("reopen_envelope=invalid".into());
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    "reopen verification envelope is invalid".into(),
                    Some(env),
                    diagnostics,
                );
            }
        };
        let scale_audit = reopen_value
            .get("root_scale_audit")
            .and_then(Value::as_str)
            .unwrap_or("");
        diagnostics.push(format!(
            "reopen_root_scale_audit={}",
            if scale_audit.is_empty() {
                "missing"
            } else {
                scale_audit
            }
        ));
        if !reopen_root_scale_is_pass(scale_audit) {
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::ReopenFailure,
                "fresh-process reopen did not prove target root rest scale PASS".into(),
                Some(env),
                diagnostics,
            );
        }
        let reopen: ReopenEnvelope = match serde_json::from_value(reopen_value) {
            Ok(value) => value,
            Err(_) => {
                diagnostics.push("reopen_envelope=invalid".into());
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::ReopenFailure,
                    "reopen verification envelope is invalid".into(),
                    Some(env),
                    diagnostics,
                );
            }
        };
        if !reopen.status.eq_ignore_ascii_case("SUCCESS")
            || !reopen.target_present
            || reopen.baked_action.as_deref().unwrap_or("").is_empty()
        {
            diagnostics.push("reopen_status=FAIL".into());
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::ReopenFailure,
                "fresh-process reopen failed".into(),
                Some(env),
                diagnostics,
            );
        }
        diagnostics.push("reopen_status=PASS".into());
        diagnostics.push(format!(
            "reopen_baked_action={}",
            reopen.baked_action.clone().unwrap_or_default()
        ));

        if !attempt.staged_blend.is_file() {
            diagnostics.push("staged_blend_missing_after_reopen=true".into());
            return self.failed_outcome(
                attempt,
                WorkerFailureClass::Other("staged_artifact_collection_failure".into()),
                "staged output read/hash failure after reopen".into(),
                Some(env),
                diagnostics,
            );
        }
        let digest = match sha256_file(&attempt.staged_blend) {
            Ok(hex) => match ContentDigest::parse(&hex) {
                Ok(digest) => digest,
                Err(err) => {
                    return self.failed_outcome(
                        attempt,
                        WorkerFailureClass::Other("staged_artifact_collection_failure".into()),
                        format!("staged output digest failure: {err}"),
                        Some(env),
                        diagnostics,
                    );
                }
            },
            Err(err) => {
                return self.failed_outcome(
                    attempt,
                    WorkerFailureClass::Other("staged_artifact_collection_failure".into()),
                    format!("staged output read/hash failure: {err}"),
                    Some(env),
                    diagnostics,
                );
            }
        };
        let mut extra = diagnostics;
        extra.push("phase_reopen=PASS".into());
        extra.push("product_publication=false".into());
        let staged = attempt.staged_blend.clone();
        let result = self.worker_result(
            &attempt,
            true,
            "completed",
            Some(env),
            extra,
            vec![digest],
        )?;
        self.last_staged_blend = Some(staged);
        Ok(TerminalOutcome::Success(result))
    }
}

pub fn production_worker_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("python")
        .join("worker.py")
}

pub fn bundled_worker_script() -> PathBuf {
    production_worker_script()
}

fn blender_version_matches_pin(found: &str) -> bool {
    let found = found.trim();
    found == BLENDER_VERSION || found == "5.2.1" || found.starts_with("5.2.1 ")
}

fn measurement_is_pass(envelope: &WorkerEnvelope, name: &str) -> bool {
    envelope
        .measurements
        .iter()
        .any(|m| m.name == name && m.value.eq_ignore_ascii_case("PASS"))
}

fn scale_success_measurements_pass(envelope: &WorkerEnvelope) -> bool {
    measurement_is_pass(envelope, "keep_target_rest_scale")
        && measurement_is_pass(envelope, "root_scale_audit")
}

fn reopen_root_scale_is_pass(audit: &str) -> bool {
    !audit.trim().is_empty() && audit.trim().eq_ignore_ascii_case("PASS")
}

fn success_envelope_identity_failure(envelope: &WorkerEnvelope) -> Option<WorkerFailureClass> {
    if envelope.backend.kind != BACKEND_KIND
        || !blender_version_matches_pin(&envelope.backend.version)
    {
        return Some(WorkerFailureClass::BackendVersionMismatch);
    }
    if envelope.backend.build != BLENDER_BUILD {
        return Some(WorkerFailureClass::BackendBuildMismatch);
    }
    match envelope.adapter_version.as_deref() {
        Some(version) if version == ADAPTER_VERSION => None,
        _ => Some(WorkerFailureClass::AdapterVersionMismatch),
    }
}

fn class_from_envelope(envelope: &WorkerEnvelope) -> WorkerFailureClass {
    match envelope.failure_class.as_deref() {
        Some("launch_failure") => WorkerFailureClass::LaunchFailure,
        Some("worker_script_exception") => WorkerFailureClass::WorkerScriptException,
        Some("structured_worker_fail") => WorkerFailureClass::StructuredWorkerFail,
        Some("missing_result_envelope") => WorkerFailureClass::MissingResultEnvelope,
        Some("invalid_result_envelope") => WorkerFailureClass::InvalidResultEnvelope,
        Some("job_spec_mismatch") => WorkerFailureClass::JobSpecMismatch,
        Some("backend_build_mismatch") => WorkerFailureClass::BackendBuildMismatch,
        Some("backend_version_mismatch") => WorkerFailureClass::BackendVersionMismatch,
        Some("adapter_version_mismatch") => WorkerFailureClass::AdapterVersionMismatch,
        Some("source_digest_mismatch") => WorkerFailureClass::SourceDigestMismatch,
        Some("unsupported_policy") => WorkerFailureClass::UnsupportedPolicy,
        Some("reopen_failure") => WorkerFailureClass::ReopenFailure,
        Some(other) => WorkerFailureClass::Other(other.to_string()),
        None => WorkerFailureClass::StructuredWorkerFail,
    }
}

pub fn diagnostic_tail(text: &str, max_bytes: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_bytes {
        return trimmed.to_string();
    }
    let mut idx = trimmed.len() - max_bytes;
    while idx < trimmed.len() && !trimmed.is_char_boundary(idx) {
        idx += 1;
    }
    trimmed[idx..].to_string()
}

fn tail(text: &str, max: usize) -> String {
    diagnostic_tail(text, max)
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}
