mod common;

use std::path::PathBuf;

use common::{fake_blender, graph_with_sources, graph_with_sources_time, seed, temp_dir, time_domain_points, write_file};
use rigforge_app::{
    JobRunState, SqliteCatalog, TerminalOutcome, WorkerFailureClass, WorkerPort,
};
use rigforge_blender_worker::{
    assert_safety_flags, blender_argv, bundled_worker_script, diagnostic_tail,
    parse_blender_version_output, project_supported_policy, AttemptWorkspace, BlenderPin,
    BlenderWorker, ADAPTER_VERSION, BLENDER_BUILD, BLENDER_VERSION,
};
use rigforge_domain::{RecordType, TimePoint, Validated};

fn worker_with_behavior(behavior: &str) -> (BlenderWorker, PathBuf) {
    let root = temp_dir();
    let mut worker = BlenderWorker::for_fake_executable(fake_blender(), root.clone());
    worker.set_test_behavior(behavior).unwrap();
    (worker, root)
}

fn dispatchable_run(behavior: &str) -> (SqliteCatalog, String, BlenderWorker) {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = graph_with_sources(r"C:\research\knight.bin", r"C:\research\motion.bin", "clip:walk");
    let spec = seed(&mut catalog, &g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let (worker, _root) = worker_with_behavior(behavior);
    (catalog, run.run_id, worker)
}

#[test]
fn command_construction_includes_safety_flags() {
    let blender = PathBuf::from(r"C:\pin\blender.exe");
    let script = bundled_worker_script();
    let job = PathBuf::from(r"C:\ws\job.json");
    let argv = blender_argv(&blender, &script, "execute", &job);
    assert!(assert_safety_flags(&argv));
    let text: Vec<String> = argv.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    assert!(text.contains(&"--background".into()));
    assert!(text.contains(&"--factory-startup".into()));
    assert!(text.contains(&"--disable-autoexec".into()));
    assert!(text.windows(2).any(|w| w[0] == "--python-exit-code" && w[1] == "1"));
    assert!(text.contains(&"--python".into()));
    assert_eq!(text[text.len() - 2], "execute");
}

#[test]
fn pin_parser_accepts_official_version_text() {
    let stdout = "Blender 5.2.1 LTS\n\tbuild date: 2026-08-25\n\tbuild hash: 9e2066aef7ef\n";
    let (version, build) = parse_blender_version_output(stdout).unwrap();
    assert!(version.contains("5.2.1"));
    assert_eq!(build, BLENDER_BUILD);
    assert_eq!(BLENDER_VERSION, "5.2.1 LTS");
    assert_eq!(ADAPTER_VERSION, "rigforge-blender-worker/0.1.0");
}

#[test]
fn missing_blender_executable_is_launch_failure() {
    let pin = BlenderPin::accepted().with_executable(PathBuf::from(r"C:\missing\blender.exe"));
    let err = rigforge_blender_worker::enforce_pin(&pin.executable, &pin).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("launch failure") || msg.contains("missing"), "{msg}");
}

#[test]
fn attempt_workspace_isolates_temp_and_user_dirs() {
    let root = temp_dir().join("attempt");
    let ws = AttemptWorkspace::create(&root).unwrap();
    let env = ws.isolated_env();
    assert_eq!(env.get("TEMP").unwrap(), &ws.tmp.display().to_string());
    assert_eq!(env.get("TMP").unwrap(), &ws.tmp.display().to_string());
    assert_eq!(env.get("TMPDIR").unwrap(), &ws.tmp.display().to_string());
    assert_eq!(
        env.get("BLENDER_USER_CONFIG").unwrap(),
        &ws.user_config.display().to_string()
    );
    assert_eq!(
        env.get("BLENDER_USER_SCRIPTS").unwrap(),
        &ws.user_scripts.display().to_string()
    );
    assert_eq!(
        env.get("BLENDER_USER_DATAFILES").unwrap(),
        &ws.user_datafiles.display().to_string()
    );
    assert!(ws.tmp.starts_with(&ws.root));
}

#[test]
fn policy_projection_emits_proven_tokens() {
    let policy = rigforge_domain::RetargetPolicyVersion::proven_draft(
        rigforge_domain::RetargetPolicyId::generate(),
    )
    .unwrap();
    let projected = project_supported_policy(&policy).unwrap();
    assert_eq!(projected.channel_policy, "rotation_only_mapped_non_root");
    assert_eq!(projected.quaternion_normalization_policy, "normalize_before_key");
    assert_eq!(projected.quaternion_continuity_policy, "consecutive_hemisphere");
    assert_eq!(projected.ik_policy, "explicit_no_ik");
    assert_eq!(projected.scale_policy, "keep_target_rest_scale");
}

#[test]
fn dispatch_projects_exact_ids_and_preserves_attempt_identity() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = graph_with_sources(r"C:\research\knight.bin", r"C:\research\motion.bin", "clip:walk");
    let spec = seed(&mut catalog, &g);
    let spec_id = spec.as_record().id().canonical();
    let run = catalog.enqueue_job(spec).unwrap();
    let attempt = run.attempt_id.clone();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let (mut worker, _root) = worker_with_behavior("success");
    let (running, receipt) = catalog.dispatch(&run.run_id, &mut worker).unwrap();
    assert_eq!(running.state, JobRunState::Running);
    assert_eq!(receipt.attempt_id, attempt);
    assert_eq!(receipt.worker_execution_ref, format!("blender-worker:{attempt}"));
    assert_eq!(running.attempt_id, attempt);
    assert_eq!(
        running.worker_execution_ref.as_deref(),
        Some(receipt.worker_execution_ref.as_str())
    );
    let request = catalog.assemble_worker_dispatch_request(&running).unwrap();
    assert_eq!(request.job_spec().as_record().id().canonical(), spec_id);
    assert_eq!(
        request.mapping().as_record().id(),
        g.mapping_version.id()
    );
    assert_eq!(request.policy().as_record().id(), g.policy_version.id());
    assert_eq!(request.attempt_id(), attempt);
    assert_eq!(request.motion_clip_id(), "clip:walk");
}

#[test]
fn worker_execution_ref_is_stable_for_the_same_attempt() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    let (running, receipt) = catalog.dispatch(&run_id, &mut worker).unwrap();
    assert_eq!(
        receipt.worker_execution_ref,
        format!("blender-worker:{}", running.attempt_id)
    );
    assert_eq!(
        running.worker_execution_ref.as_ref(),
        Some(&receipt.worker_execution_ref)
    );
}

#[test]
fn successful_collect_binds_worker_result_without_publication() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    match outcome {
        TerminalOutcome::Success(result) => {
            assert!(result.as_record().worker_success());
            assert_eq!(
                result.as_record().job_spec_id().canonical(),
                done.job_spec_id
            );
            assert_eq!(
                result.as_record().execution().backend_kind(),
                "Blender"
            );
            assert_eq!(result.as_record().execution().build(), BLENDER_BUILD);
            assert!(!result.as_record().staged_artifact_digests().is_empty());
            assert!(!result.as_record().implies_qc_pass());
            assert!(!result.as_record().implies_publication());
        }
        other => panic!("expected success, got {other:?}"),
    }
    assert!(catalog
        .list_assets(RecordType::DerivedVariant)
        .unwrap()
        .is_empty());
}

#[test]
fn stdout_stderr_are_captured_on_success() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (_done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    match outcome {
        TerminalOutcome::Success(result) => {
            let joined = result.as_record().diagnostics().join("\n");
            assert!(
                joined.contains("execute_stdout_captured")
                    || joined.contains("execute_stderr_captured")
                    || joined.contains("workspace="),
                "{joined}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn missing_result_envelope_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("missing_envelope");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, .. } => {
            assert!(matches!(
                class,
                WorkerFailureClass::MissingResultEnvelope | WorkerFailureClass::WorkerScriptException
            ));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn invalid_result_envelope_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("invalid_envelope");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, .. } => {
            assert_eq!(class, WorkerFailureClass::InvalidResultEnvelope);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn job_spec_mismatch_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("job_spec_mismatch");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, worker_result, .. } => {
            assert_eq!(class, WorkerFailureClass::JobSpecMismatch);
            let result = worker_result.unwrap();
            assert!(!result.as_record().worker_success());
            assert_eq!(result.as_record().job_spec_id().canonical(), done.job_spec_id);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn backend_build_mismatch_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("backend_mismatch");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, .. } => {
            assert_eq!(class, WorkerFailureClass::BackendBuildMismatch);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn source_digest_mismatch_fails_before_or_at_worker() {
    let root = temp_dir();
    let character = root.join("knight.bin");
    let motion = root.join("motion.bin");
    write_file(&character, b"aaa");
    write_file(&motion, b"bbb");
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let mut g = graph_with_sources(
        character.to_str().unwrap(),
        motion.to_str().unwrap(),
        "clip:walk",
    );
    // Domain digest is digest(1)/digest(2), not the real file hash.
    let spec = seed(&mut catalog, &g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::for_fake_executable(fake_blender(), root.join("ws"));
    worker.enable_source_verification_for_test().unwrap();
    worker.set_test_behavior("success").unwrap();
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("digest mismatch") || msg.contains("source_digest"), "{msg}");
    assert_eq!(
        catalog.load_job_run(&run.run_id).unwrap().state,
        JobRunState::Failed
    );
    let _ = &mut g;
}

#[test]
fn unsupported_policy_failure_class_is_distinct() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("unsupported_policy");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, worker_result, .. } => {
            assert_eq!(class, WorkerFailureClass::UnsupportedPolicy);
            assert!(worker_result.is_some());
            assert!(!worker_result.unwrap().as_record().worker_success());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn launch_failure_never_marks_running() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = graph_with_sources(r"C:\research\knight.bin", r"C:\research\motion.bin", "clip:walk");
    let spec = seed(&mut catalog, &g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = BlenderWorker::for_fake_executable(
        PathBuf::from(r"C:\missing\rigforge-not-a-blender.exe"),
        temp_dir(),
    );
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    assert!(err.to_string().contains("launch failure"), "{err}");
    let loaded = catalog.load_job_run(&run.run_id).unwrap();
    assert_eq!(loaded.state, JobRunState::Failed);
    assert!(loaded.worker_result_id.is_none());
}

#[test]
fn nonzero_exit_persists_failed_worker_result() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("nonzero_exit");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, worker_result, .. } => {
            assert!(matches!(
                class,
                WorkerFailureClass::WorkerScriptException | WorkerFailureClass::StructuredWorkerFail
            ));
            let result = worker_result.expect("failed WorkerResult");
            assert!(!result.as_record().worker_success());
            assert_eq!(
                catalog
                    .load_worker_result(&result.as_record().id().canonical())
                    .unwrap()
                    .as_record()
                    .id(),
                result.as_record().id()
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn failed_structured_result_does_not_publish() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("fail_structured");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, _) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    assert!(catalog
        .list_assets(RecordType::DerivedVariant)
        .unwrap()
        .is_empty());
}

#[test]
fn job_spec_only_dispatch_is_refused() {
    let mut worker = BlenderWorker::for_fake_executable(fake_blender(), temp_dir());
    let g = graph_with_sources(r"C:\research\knight.bin", r"C:\research\motion.bin", "clip:walk");
    let spec = Validated::certify(g.job).unwrap();
    let err = WorkerPort::dispatch(&mut worker, &spec, "attempt-x").unwrap_err();
    assert!(err.to_string().contains("resolved WorkerDispatchRequest"), "{err}");
}

#[test]
fn success_without_staged_file_is_not_success() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success_without_blend");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, .. } => {
            assert_eq!(class, WorkerFailureClass::StructuredWorkerFail);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn production_worker_cannot_disable_pin_verification() {
    common::ensure_test_runtime();
    let mut worker = BlenderWorker::production().unwrap();
    assert!(worker.is_production());
    assert!(worker.pin_verification_is_mandatory());
    let err = worker
        .set_test_behavior("skip-pin")
        .expect_err("production worker has no test harness");
    assert!(err.to_string().contains("production"), "{err}");
}

#[test]
fn production_worker_cannot_disable_source_verification() {
    common::ensure_test_runtime();
    let mut worker = BlenderWorker::production().unwrap();
    assert!(worker.source_verification_is_mandatory());
    let err = worker
        .enable_source_verification_for_test()
        .expect_err("production source verification is sealed");
    assert!(err.to_string().contains("mandatory"), "{err}");
    let err = worker
        .skip_reopen_for_test()
        .expect_err("production reopen cannot be skipped");
    assert!(err.to_string().contains("mandatory"), "{err}");
    assert_eq!(
        worker.worker_script(),
        rigforge_blender_worker::production_worker_script().as_path()
    );
}

#[test]
fn production_success_requires_real_reopen() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    match outcome {
        TerminalOutcome::Success(result) => {
            let joined = result.as_record().diagnostics().join("\n");
            assert!(joined.contains("reopen_status=PASS"), "{joined}");
            assert!(joined.contains("reopen_exit_code=0"), "{joined}");
            assert!(joined.contains("phase_reopen=PASS"), "{joined}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn reopen_skipped_cannot_report_reopen_pass() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    worker.skip_reopen_for_test().unwrap();
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            worker_result,
            ..
        } => {
            assert_eq!(class, WorkerFailureClass::ReopenFailure);
            let joined = worker_result.unwrap().as_record().diagnostics().join("\n");
            assert!(!joined.contains("phase_reopen=PASS"), "{joined}");
        }
        other => panic!("skipped reopen must not succeed: {other:?}"),
    }
}

#[test]
fn backend_version_mismatch_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("backend_version_mismatch");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            worker_result,
            ..
        } => {
            assert_eq!(class, WorkerFailureClass::BackendVersionMismatch);
            let result = worker_result.unwrap();
            assert!(!result.as_record().worker_success());
            assert_eq!(result.as_record().execution().backend_version(), "2.9.0");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn adapter_version_mismatch_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("adapter_version_mismatch");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            worker_result,
            ..
        } => {
            assert_eq!(class, WorkerFailureClass::AdapterVersionMismatch);
            let result = worker_result.unwrap();
            assert!(!result.as_record().worker_success());
            assert_eq!(
                result.as_record().execution().adapter_version(),
                "other-adapter/9.9.9"
            );
            assert_ne!(
                result.as_record().execution().adapter_version(),
                ADAPTER_VERSION
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn reopen_launch_failure_marks_job_failed() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    worker
        .set_reopen_executable_for_test(PathBuf::from(
            r"C:\missing\rigforge-not-a-blender.exe",
        ))
        .unwrap();
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            reason,
            worker_result,
        } => {
            assert_eq!(class, WorkerFailureClass::ReopenFailure);
            assert!(reason.contains("reopen launch failure"), "{reason}");
            assert!(worker_result.is_some());
            assert!(!worker_result.unwrap().as_record().worker_success());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn invalid_reopen_envelope_marks_job_failed() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("reopen_invalid_envelope");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            reason,
            worker_result,
        } => {
            assert_eq!(class, WorkerFailureClass::ReopenFailure);
            assert!(reason.contains("invalid"), "{reason}");
            assert!(worker_result.is_some());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn staged_artifact_collection_failure_marks_job_failed() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("reopen_delete_staged");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            worker_result,
            ..
        } => {
            assert_eq!(
                class,
                WorkerFailureClass::Other("staged_artifact_collection_failure".into())
            );
            let result = worker_result.unwrap();
            assert!(!result.as_record().worker_success());
            assert!(result.as_record().staged_artifact_digests().is_empty());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn successful_collect_binds_attempt_correlation() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let run = catalog.load_job_run(&run_id).unwrap();
    let (_done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    match outcome {
        TerminalOutcome::Success(result) => {
            let rec = result.as_record();
            assert_eq!(rec.attempt_id(), run.attempt_id);
            assert_eq!(
                rec.worker_execution_ref(),
                run.worker_execution_ref.as_deref().unwrap()
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn terminal_collect_removes_attempt_bookkeeping() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    assert_eq!(worker.launched_count(), 1);
    catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(worker.launched_count(), 0);

    let (mut catalog, run_id, mut worker) = dispatchable_run("fail_structured");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    assert_eq!(worker.launched_count(), 1);
    catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(worker.launched_count(), 0);
}

#[test]
fn unicode_stderr_tail_does_not_panic() {
    let mut body = "é漢".repeat(800);
    body.push_str(&"x".repeat(50));
    let tail = diagnostic_tail(&body, 1200);
    assert!(!tail.is_empty());
    assert!(std::str::from_utf8(tail.as_bytes()).is_ok());
    assert!(tail.len() <= 1200 + "é漢".len());
}

#[test]
fn ascii_tail_behavior_preserved() {
    let text = "a".repeat(2000);
    let tail = diagnostic_tail(&text, 1200);
    assert_eq!(tail.len(), 1200);
    assert_eq!(tail, "a".repeat(1200));
}

#[test]
fn unicode_stderr_collect_does_not_panic() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("unicode_stderr");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    match outcome {
        TerminalOutcome::Success(result) => {
            let joined = result.as_record().diagnostics().join("\n");
            assert!(joined.contains("stderr_tail=") || joined.contains("execute_stderr"), "{joined}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn root_scale_policy_violation_cannot_return_success() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("scale_audit_fail");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed { class, .. } => {
            assert_eq!(class, WorkerFailureClass::StructuredWorkerFail);
        }
        other => panic!("scale audit FAIL must not succeed: {other:?}"),
    }
}

#[test]
fn fractional_frame_never_reaches_blender_worker() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = graph_with_sources_time(
        r"C:\research\knight.bin",
        r"C:\research\motion.bin",
        time_domain_points(
            "clip:walk",
            TimePoint::frames_rational(3, 2, 30, 1).unwrap(),
            TimePoint::frames(4, 30, 1).unwrap(),
        ),
    );
    let spec = seed(&mut catalog, &g);
    let run = catalog.enqueue_job(spec).unwrap();
    catalog.mark_dispatchable(&run.run_id).unwrap();
    let (mut worker, _root) = worker_with_behavior("success");
    let err = catalog.dispatch(&run.run_id, &mut worker).unwrap_err();
    assert!(
        err.to_string().contains("unsupported execution-time provenance"),
        "{err}"
    );
    assert_eq!(worker.launched_count(), 0);
    assert_eq!(
        catalog.load_job_run(&run.run_id).unwrap().state,
        JobRunState::Failed
    );
}

#[test]
fn missing_root_scale_audit_reopen_fails() {
    let (mut catalog, run_id, mut worker) =
        dispatchable_run("reopen_missing_root_scale_audit");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            reason,
            worker_result,
        } => {
            assert_eq!(class, WorkerFailureClass::ReopenFailure);
            assert!(
                reason.contains("root rest scale") || reason.contains("root_scale_audit"),
                "{reason}"
            );
            let joined = worker_result
                .as_ref()
                .map(|r| r.as_record().diagnostics().join("\n"))
                .unwrap_or_default();
            assert!(
                joined.contains("reopen_root_scale_audit=missing"),
                "{joined}"
            );
            assert!(!joined.contains("phase_reopen=PASS"), "{joined}");
        }
        other => panic!("missing root_scale_audit must fail collect: {other:?}"),
    }
}

#[test]
fn failed_root_scale_audit_reopen_fails() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("reopen_fail_root_scale_audit");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Failed);
    match outcome {
        TerminalOutcome::Failed {
            class,
            reason,
            worker_result,
        } => {
            assert_eq!(class, WorkerFailureClass::ReopenFailure);
            assert!(reason.contains("root rest scale"), "{reason}");
            let joined = worker_result
                .as_ref()
                .map(|r| r.as_record().diagnostics().join("\n"))
                .unwrap_or_default();
            assert!(
                joined.contains("reopen_root_scale_audit=FAIL"),
                "{joined}"
            );
            assert!(!joined.contains("phase_reopen=PASS"), "{joined}");
        }
        other => panic!("FAIL root_scale_audit must fail collect: {other:?}"),
    }
}

#[test]
fn pass_root_scale_audit_reopen_succeeds() {
    let (mut catalog, run_id, mut worker) = dispatchable_run("success");
    catalog.dispatch(&run_id, &mut worker).unwrap();
    let (done, outcome) = catalog.collect(&run_id, &mut worker).unwrap();
    assert_eq!(done.state, JobRunState::Succeeded);
    match outcome {
        TerminalOutcome::Success(result) => {
            let joined = result.as_record().diagnostics().join("\n");
            assert!(joined.contains("reopen_status=PASS"), "{joined}");
            assert!(joined.contains("reopen_root_scale_audit=PASS"), "{joined}");
            assert!(joined.contains("phase_reopen=PASS"), "{joined}");
            assert!(result.as_record().worker_success());
        }
        other => panic!("PASS root_scale_audit must succeed: {other:?}"),
    }
}
