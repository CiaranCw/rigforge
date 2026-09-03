use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rigforge_app::rigforge_domain::*;
use rigforge_app::{
    generate_mapping_proposal, sha256_file, AppError, Application, ArtifactInspectionEvidence,
    DispatchReceipt, FakeWorker, JobRunState, MappingAssistProfile, MemorySkeletonInspector,
    TerminalOutcome, WorkerCompletionPort, WorkerFailureClass, WorkerPort,
};
use rigforge_workbench::native_exec::collect_execute_wait;
use rigforge_workbench::{format_elapsed, TransferPhase, WorkbenchApp};

fn digest(n: u8) -> ContentDigest {
    ContentDigest::parse(&format!("{n:02x}").repeat(32)).unwrap()
}

fn evidence(name: &str, n: u8) -> SourceArtifactEvidence {
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(format!("C:/research/{name}.bin")).unwrap(),
        digest(n),
        1024,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".to_string()),
        None,
    )
    .unwrap()
}

fn j(key: &str, parent: Option<&str>, root: bool, deform: &str) -> JointObservation {
    JointObservation::new(
        JointKey::new(key).unwrap(),
        key,
        parent.map(|p| JointKey::new(p).unwrap()),
        root,
        Some(deform.into()),
        Some("rest".into()),
    )
    .unwrap()
}

struct Seeded {
    app: Application,
    character_version: String,
    motion_version: String,
}

fn seed() -> Seeded {
    let mut app = Application::open_in_memory().unwrap();
    let producer = BackendExecutionContext::new(
        "isolated-worker",
        "1.0.0",
        "build-test",
        "adapter-1",
        "exec-policy-1",
    )
    .unwrap();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", evidence("knight", 1)).unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            "clip:walk-carry",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence("motion", 2),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    let mut policy = RetargetPolicy::new("rest-relative").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();
    let source = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::SourceSkeletonReference,
                None,
                Some(source_skeleton.id()),
                producer.clone(),
                vec![
                    j("root", None, true, "deforming"),
                    j("head", Some("root"), false, "deforming"),
                ],
                vec!["source".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let target = app
        .store_skeleton_summary(
            SkeletonSummary::new(
                SkeletonSubjectKind::CharacterAssetVersion,
                Some(character_version.id()),
                None,
                producer,
                vec![
                    j("Bone", None, true, "deforming"),
                    j("Head", Some("Bone"), false, "deforming"),
                ],
                vec!["target".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let proposal = generate_mapping_proposal(
        source.as_record(),
        target.as_record(),
        MappingAssistProfile::None,
    )
    .unwrap();
    app.store_mapping_draft(
        "wb-map",
        &character_version.id().canonical(),
        &source_skeleton.id().canonical(),
        &proposal,
        &source.as_record().id().canonical(),
        &target.as_record().id().canonical(),
    )
    .unwrap();
    Seeded {
        app,
        character_version: character_version.id().canonical(),
        motion_version: motion_version.id().canonical(),
    }
}

fn inspector_from_seed(seeded: &Seeded) -> MemorySkeletonInspector {
    let motion = seeded
        .app
        .catalog()
        .load_motion_version(&seeded.motion_version)
        .unwrap();
    let source_id = motion.as_record().source_skeleton_ref_id().canonical();
    let target_ids = seeded
        .app
        .catalog()
        .list_skeleton_summaries_for_character(&seeded.character_version)
        .unwrap();
    let source_ids = seeded
        .app
        .catalog()
        .list_skeleton_summaries_for_source_skeleton(&source_id)
        .unwrap();
    let loaded_target = seeded
        .app
        .load_skeleton_summary(&target_ids[0])
        .unwrap()
        .into_record();
    let loaded_source = seeded
        .app
        .load_skeleton_summary(&source_ids[0])
        .unwrap()
        .into_record();
    MemorySkeletonInspector {
        character: SkeletonSummary::new(
            loaded_target.subject_kind(),
            loaded_target.subject_character_version_id(),
            loaded_target.subject_source_skeleton_ref_id(),
            loaded_target.producer().clone(),
            loaded_target.joints().to_vec(),
            loaded_target.diagnostics().to_vec(),
        )
        .unwrap(),
        source: SkeletonSummary::new(
            loaded_source.subject_kind(),
            loaded_source.subject_character_version_id(),
            loaded_source.subject_source_skeleton_ref_id(),
            loaded_source.producer().clone(),
            loaded_source.joints().to_vec(),
            loaded_source.diagnostics().to_vec(),
        )
        .unwrap(),
    }
}

fn ready_transfer() -> (Application, WorkbenchApp) {
    let mut seeded = seed();
    let inspector = inspector_from_seed(&seeded);
    let mut shell = WorkbenchApp::empty();
    shell.select_character_version(&seeded.character_version);
    shell.select_motion_version(&seeded.motion_version);
    shell
        .propose_mapping_with(&mut seeded.app, &inspector)
        .unwrap();
    shell.on_accept_mapping_clicked(&mut seeded.app).unwrap();
    shell
        .on_evaluate_compatibility_clicked(&mut seeded.app)
        .unwrap();
    if shell.transfer_requires_acknowledgement() {
        shell
            .on_warnings_checkbox_changed(&mut seeded.app, true)
            .unwrap();
    }
    assert!(shell.transfer_available());
    (seeded.app, shell)
}

fn write_staged(bytes: &[u8]) -> (PathBuf, String) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rf-r12-wb-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("derived_result.blend");
    fs::write(&path, bytes).unwrap();
    let sha = sha256_file(&path).unwrap();
    (path, sha)
}

fn passing_qc(digest: impl Into<String>) -> ArtifactInspectionEvidence {
    let digest = digest.into();
    ArtifactInspectionEvidence {
        digest_before: digest.clone(),
        digest_after: digest,
        structurally_readable: true,
        finite_transforms: true,
        present_joint_keys: vec!["Bone".into(), "Head".into()],
        baked_animation_present: true,
        duration_s: Some(2.0),
        expected_duration_s: Some(2.0),
        gross_scale_sane: true,
        root_trajectory_sane: true,
    }
}

fn failing_qc(digest: impl Into<String>) -> ArtifactInspectionEvidence {
    let mut evidence = passing_qc(digest);
    evidence.finite_transforms = false;
    evidence
}

struct ScriptedExecuteWorker {
    fake: FakeWorker,
    staged: Option<PathBuf>,
    sha: Option<String>,
    gate: Option<Receiver<()>>,
    fail_reason: Option<String>,
    omit_staged_path: bool,
}

impl ScriptedExecuteWorker {
    fn success(staged: PathBuf, sha: String, gate: Option<Receiver<()>>) -> Self {
        Self {
            fake: FakeWorker::default(),
            staged: Some(staged),
            sha: Some(sha),
            gate,
            fail_reason: None,
            omit_staged_path: false,
        }
    }

    fn success_missing_path(sha: String) -> Self {
        Self {
            fake: FakeWorker::default(),
            staged: None,
            sha: Some(sha),
            gate: None,
            fail_reason: None,
            omit_staged_path: true,
        }
    }

    fn failed(reason: impl Into<String>) -> Self {
        Self {
            fake: FakeWorker::default(),
            staged: None,
            sha: None,
            gate: None,
            fail_reason: Some(reason.into()),
            omit_staged_path: false,
        }
    }
}

impl WorkerPort for ScriptedExecuteWorker {
    fn dispatch(
        &mut self,
        spec: &Validated<JobSpec>,
        attempt_id: &str,
    ) -> Result<DispatchReceipt, AppError> {
        self.fake.dispatch(spec, attempt_id)
    }

    fn dispatch_resolved(
        &mut self,
        request: &rigforge_app::WorkerDispatchRequest,
    ) -> Result<DispatchReceipt, AppError> {
        self.fake.dispatch_resolved(request)
    }
}

impl WorkerCompletionPort for ScriptedExecuteWorker {
    fn collect(&mut self, receipt: &DispatchReceipt) -> Result<TerminalOutcome, AppError> {
        if let Some(gate) = self.gate.take() {
            let _ = gate.recv();
        }
        if let Some(reason) = &self.fail_reason {
            let spec_id = JobSpecId::parse(self.fake.last_job_spec_id.as_deref().unwrap()).unwrap();
            let failed = WorkerResult::new(
                spec_id,
                BackendExecutionContext::new(
                    "isolated-worker",
                    "1.0.0",
                    "build-test",
                    "adapter-1",
                    "exec-policy-1",
                )
                .unwrap(),
                false,
                "failed",
                ExecutionCorrelation::new(&receipt.attempt_id, &receipt.worker_execution_ref)
                    .unwrap(),
            )
            .unwrap();
            return Ok(TerminalOutcome::Failed {
                class: WorkerFailureClass::StructuredWorkerFail,
                reason: reason.clone(),
                worker_result: Some(Validated::certify(failed).unwrap()),
            });
        }
        let spec_id = JobSpecId::parse(self.fake.last_job_spec_id.as_deref().unwrap()).unwrap();
        let sha = self.sha.clone().expect("success worker needs digest");
        let result = WorkerResult::new(
            spec_id,
            BackendExecutionContext::new(
                "isolated-worker",
                "1.0.0",
                "build-test",
                "adapter-1",
                "exec-policy-1",
            )
            .unwrap(),
            true,
            "completed",
            ExecutionCorrelation::new(&receipt.attempt_id, &receipt.worker_execution_ref).unwrap(),
        )
        .unwrap()
        .with_staged_artifact_digests(vec![ContentDigest::parse(&sha).unwrap()])
        .unwrap();
        let _ = self.omit_staged_path;
        Ok(TerminalOutcome::Success(Validated::certify(result).unwrap()))
    }

    fn last_staged_artifact(&self) -> Option<&Path> {
        if self.omit_staged_path {
            None
        } else {
            self.staged.as_deref()
        }
    }
}

fn pump_until(shell: &mut WorkbenchApp, app: &mut Application, want: TransferPhase) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while shell.long_op_phase() != Some(want) {
        if std::time::Instant::now() > deadline {
            panic!(
                "timed out waiting for {want:?}, now {:?}",
                shell.long_op_phase()
            );
        }
        shell.poll_long_op(app, None).unwrap();
        if shell.long_op_phase() != Some(want) {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn install_pass_hooks(shell: &mut WorkbenchApp) {
    shell.set_scripted_qc_acquire_for_test(Box::new(|req| Ok(passing_qc(&req.expected_sha256))));
    shell.set_scripted_reopen_acquire_for_test(Box::new(|_| {
        Ok((VerificationOutcome::Pass, VerificationOutcome::Pass))
    }));
}

#[test]
fn elapsed_progress_state_is_presentation_only() {
    assert_eq!(format_elapsed(Duration::from_secs(8)), "00:08");
    assert_eq!(format_elapsed(Duration::from_secs(75)), "01:15");
    let orchestration = include_str!("../../app/src/orchestration.rs");
    assert!(
        !orchestration.contains("elapsed"),
        "JobRun must not store presentation elapsed time"
    );
}

#[test]
fn background_execute_collect_does_not_require_application() {
    struct Delayed {
        gate: Receiver<()>,
        fired: Arc<AtomicBool>,
    }
    impl WorkerCompletionPort for Delayed {
        fn collect(
            &mut self,
            _receipt: &DispatchReceipt,
        ) -> Result<TerminalOutcome, AppError> {
            let _ = self.gate.recv();
            self.fired.store(true, Ordering::SeqCst);
            Ok(TerminalOutcome::Failed {
                class: WorkerFailureClass::Other("delayed".into()),
                reason: "delayed".into(),
                worker_result: None,
            })
        }
    }
    let (tx, rx) = mpsc::channel();
    let fired = Arc::new(AtomicBool::new(false));
    let fired_thread = fired.clone();
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut worker = Delayed {
            gate: rx,
            fired: fired_thread,
        };
        let receipt = DispatchReceipt {
            attempt_id: "attempt".into(),
            worker_execution_ref: "ref".into(),
        };
        let result = collect_execute_wait(&mut worker, &receipt);
        done_tx.send(result.is_ok()).unwrap();
    });
    assert!(done_rx.try_recv().is_err());
    assert!(!fired.load(Ordering::SeqCst));
    tx.send(()).unwrap();
    assert!(done_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    assert!(fired.load(Ordering::SeqCst));
}

#[test]
fn start_returns_before_delayed_execute_completion() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    let (path, sha) = write_staged(b"r12-delayed-execute");
    let (gate_tx, gate_rx) = mpsc::channel();
    let worker = ScriptedExecuteWorker::success(path, sha, Some(gate_rx));
    shell
        .start_scripted_transfer_for_test(&mut app, worker)
        .unwrap();
    assert!(shell.mutation_locked());
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::RunningExecute));
    assert_ne!(shell.publication_state(), Some("Published"));
    shell.poll_long_op(&mut app, None).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::RunningExecute));
    gate_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Complete));
    assert_eq!(shell.publication_state(), Some("Published"));
}

#[test]
fn only_one_active_long_operation_allowed() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    let (path, sha) = write_staged(b"r12-one-op");
    let (gate_tx, gate_rx) = mpsc::channel();
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, Some(gate_rx)),
        )
        .unwrap();
    let err = shell
        .start_scripted_transfer_for_test(&mut app, ScriptedExecuteWorker::failed("second"))
        .unwrap_err();
    assert!(err.to_string().contains("already running"), "{err}");
    gate_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
}

#[test]
fn active_transfer_prevents_conflicting_pair_mutation() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    let original_character = shell.selected_character_version().unwrap().to_string();
    let original_motion = shell.selected_motion_version().unwrap().to_string();
    let (path, sha) = write_staged(b"r12-lock-pair");
    let (gate_tx, gate_rx) = mpsc::channel();
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, Some(gate_rx)),
        )
        .unwrap();
    assert!(!shell.transfer_available());
    shell.select_character_version("other-character");
    shell.select_motion_version("other-motion");
    assert_eq!(
        shell.selected_character_version(),
        Some(original_character.as_str())
    );
    assert_eq!(
        shell.selected_motion_version(),
        Some(original_motion.as_str())
    );
    let err = shell.on_accept_mapping_clicked(&mut app).unwrap_err();
    assert!(err.to_string().contains("already running"), "{err}");
    gate_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
}

#[test]
fn execute_completion_advances_to_qc_phase() {
    let (mut app, mut shell) = ready_transfer();
    let (qc_tx, qc_rx) = mpsc::channel();
    shell.set_scripted_qc_acquire_for_test(Box::new(move |req| {
        let _ = qc_rx.recv();
        Ok(passing_qc(&req.expected_sha256))
    }));
    shell.set_scripted_reopen_acquire_for_test(Box::new(|_| {
        Ok((VerificationOutcome::Pass, VerificationOutcome::Pass))
    }));
    let (path, sha) = write_staged(b"r12-to-qc");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    pump_until(&mut shell, &mut app, TransferPhase::ValidatingQc);
    let version_id = shell
        .long_op_derived_version_id()
        .expect("candidate version")
        .to_string();
    let version = app
        .catalog()
        .load_derived_variant_version(&version_id)
        .unwrap();
    assert!(version.as_record().qc_report_id().is_none());
    qc_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.publication_state(), Some("Published"));
}

#[test]
fn qc_evidence_is_acquired_before_qc_binding() {
    let (mut app, mut shell) = ready_transfer();
    let acquired = Arc::new(AtomicBool::new(false));
    let acquired_flag = acquired.clone();
    let (qc_tx, qc_rx) = mpsc::channel();
    shell.set_scripted_qc_acquire_for_test(Box::new(move |req| {
        let _ = qc_rx.recv();
        acquired_flag.store(true, Ordering::SeqCst);
        Ok(passing_qc(&req.expected_sha256))
    }));
    shell.set_scripted_reopen_acquire_for_test(Box::new(|_| {
        Ok((VerificationOutcome::Pass, VerificationOutcome::Pass))
    }));
    let (path, sha) = write_staged(b"r12-qc-order");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    pump_until(&mut shell, &mut app, TransferPhase::ValidatingQc);
    let version_id = shell.long_op_derived_version_id().unwrap().to_string();
    assert!(!acquired.load(Ordering::SeqCst));
    assert!(app
        .catalog()
        .load_derived_variant_version(&version_id)
        .unwrap()
        .as_record()
        .qc_report_id()
        .is_none());
    qc_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert!(acquired.load(Ordering::SeqCst));
    assert!(app
        .catalog()
        .load_derived_variant_version(&version_id)
        .unwrap()
        .as_record()
        .qc_report_id()
        .is_some());
}

#[test]
fn qc_failure_prevents_publication() {
    let (mut app, mut shell) = ready_transfer();
    shell.set_scripted_qc_acquire_for_test(Box::new(|req| Ok(failing_qc(&req.expected_sha256))));
    shell.set_scripted_reopen_acquire_for_test(Box::new(|_| {
        Ok((VerificationOutcome::Pass, VerificationOutcome::Pass))
    }));
    let (path, sha) = write_staged(b"r12-qc-fail");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Complete));
    assert_eq!(shell.publication_state(), Some("Publication denied"));
    let version_id = shell.derived_variant_version_id().unwrap();
    let version = app
        .catalog()
        .load_derived_variant_version(version_id)
        .unwrap();
    assert_ne!(version.as_record().lifecycle(), Lifecycle::Published);
}

#[test]
fn reopen_evidence_is_acquired_before_persistence_binding() {
    let (mut app, mut shell) = ready_transfer();
    let acquired = Arc::new(AtomicBool::new(false));
    let acquired_flag = acquired.clone();
    let (reopen_tx, reopen_rx) = mpsc::channel();
    shell.set_scripted_qc_acquire_for_test(Box::new(|req| Ok(passing_qc(&req.expected_sha256))));
    shell.set_scripted_reopen_acquire_for_test(Box::new(move |_| {
        let _ = reopen_rx.recv();
        acquired_flag.store(true, Ordering::SeqCst);
        Ok((VerificationOutcome::Pass, VerificationOutcome::Pass))
    }));
    let (path, sha) = write_staged(b"r12-reopen-order");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    pump_until(&mut shell, &mut app, TransferPhase::ValidatingPersistence);
    let version_id = shell.long_op_derived_version_id().unwrap().to_string();
    assert!(!acquired.load(Ordering::SeqCst));
    assert!(app
        .catalog()
        .load_derived_variant_version(&version_id)
        .unwrap()
        .as_record()
        .persistence_verification_id()
        .is_none());
    reopen_tx.send(()).unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert!(acquired.load(Ordering::SeqCst));
    assert!(app
        .catalog()
        .load_derived_variant_version(&version_id)
        .unwrap()
        .as_record()
        .persistence_verification_id()
        .is_some());
}

#[test]
fn reopen_failure_prevents_publication() {
    let (mut app, mut shell) = ready_transfer();
    shell.set_scripted_qc_acquire_for_test(Box::new(|req| Ok(passing_qc(&req.expected_sha256))));
    shell.set_scripted_reopen_acquire_for_test(Box::new(|_| {
        Ok((VerificationOutcome::Fail, VerificationOutcome::Fail))
    }));
    let (path, sha) = write_staged(b"r12-reopen-fail");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.publication_state(), Some("Publication denied"));
    let version_id = shell.derived_variant_version_id().unwrap();
    let version = app
        .catalog()
        .load_derived_variant_version(version_id)
        .unwrap();
    assert_ne!(version.as_record().lifecycle(), Lifecycle::Published);
}

#[test]
fn successful_full_state_machine_publishes_exact_returned_derived_version() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    let (path, sha) = write_staged(b"r12-full-success");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success(path, sha, None),
        )
        .unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Complete));
    assert_eq!(shell.publication_state(), Some("Published"));
    let selected = shell
        .selected_derived_variant_version()
        .expect("exact Derived selection")
        .to_string();
    assert_eq!(shell.derived_variant_version_id(), Some(selected.as_str()));
    let version = app
        .catalog()
        .load_derived_variant_version(&selected)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        version.as_record().id().canonical(),
        selected,
        "must bind the exact returned Derived version, not latest/first"
    );
}

#[test]
fn successful_worker_with_missing_staged_path_fails() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    let (_path, sha) = write_staged(b"r12-missing-staged");
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::success_missing_path(sha),
        )
        .unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Failed));
    assert_ne!(shell.publication_state(), Some("Published"));
    let jobs = app.catalog().list_job_runs().unwrap();
    assert_eq!(jobs[0].state, JobRunState::Succeeded);
}

#[test]
fn execute_failure_preserves_failed_job_and_publishes_nothing() {
    let (mut app, mut shell) = ready_transfer();
    install_pass_hooks(&mut shell);
    shell
        .start_scripted_transfer_for_test(
            &mut app,
            ScriptedExecuteWorker::failed("injected execute failure"),
        )
        .unwrap();
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Failed));
    assert_ne!(shell.publication_state(), Some("Published"));
    let jobs = app.catalog().list_job_runs().unwrap();
    assert_eq!(jobs[0].state, JobRunState::Failed);
    assert!(jobs[0].worker_result_id.is_some());
}

#[test]
fn background_channel_disconnect_fails_visibly() {
    let (mut app, mut shell) = ready_transfer();
    let run_id = shell.on_transfer_clicked(&mut app).unwrap();
    app.mark_dispatchable(&run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(&run_id, &mut worker).unwrap();
    assert_eq!(app.job_status(&run_id).unwrap().state, JobRunState::Running);
    shell.install_disconnected_execute_wait_for_test(run_id.clone());
    shell.poll_long_op(&mut app, None).unwrap();
    assert_eq!(shell.long_op_phase(), Some(TransferPhase::Failed));
    assert!(shell.workflow_status().unwrap().contains("disconnected"));
    assert_eq!(app.job_status(&run_id).unwrap().state, JobRunState::Failed);
}

#[test]
fn ui_thread_does_not_call_worker_collect_or_finalize_transfer() {
    let lib = include_str!("../src/lib.rs");
    let native = include_str!("../src/native_exec.rs");
    assert!(native.contains("thread::spawn"));
    assert!(native.contains("collect_execute_wait"));
    assert!(native.contains("inspect_durable_persistence_artifact"));
    assert!(native.contains("reopen_durable_persistence_artifact"));
    assert!(lib.contains("poll_long_op"));
    assert!(lib.contains("start_responsive_transfer"));
    assert!(!lib.contains("complete_native_transfer"));
    assert!(!lib.contains("app.collect("));
    assert!(!lib.contains("inspect_durable_persistence_artifact"));
    assert!(!lib.contains("reopen_durable_persistence_artifact"));
}

#[test]
fn apply_transfer_outcome_uses_exact_version_not_first() {
    let src = include_str!("../src/lib.rs");
    let apply = src
        .split("pub fn apply_transfer_outcome")
        .nth(1)
        .expect("apply_transfer_outcome");
    let body = apply.split("pub fn derived_variant_id").next().unwrap();
    assert!(body.contains("outcome.derived_variant_version_id"));
    assert!(!body.contains("self.derived.first()"));
    assert!(!body.contains("list_derived"));
}
