#[path = "../tests/common/mod.rs"]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use common::{
    backend, certify, persist_core, unpublished_graph, Graph,
};
use rigforge_app::{
    evaluate_structural_qc, sha256_file, AppError, Application, FakeWorker, JobRunState,
    TransferOutcomeKind,
};
use crate::qc::{
    ArtifactInspector, MemoryArtifactInspector, MemoryPersistenceReopener,
};
use rigforge_domain::*;
use sha2::{Digest, Sha256};

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_staged(bytes: &[u8]) -> (PathBuf, String) {
    let dir = temp_dir("rf-v15-staged");
    let path = dir.join("derived_result.blend");
    fs::write(&path, bytes).unwrap();
    (path, sha256_bytes(bytes))
}

fn worker_success(
    job: &JobSpec,
    run: &rigforge_app::JobRun,
    sha: &str,
) -> Validated<WorkerResult> {
    certify(
        WorkerResult::new(
            job.id(),
            backend(),
            true,
            "completed",
            ExecutionCorrelation::new(
                &run.attempt_id,
                run.worker_execution_ref
                    .as_deref()
                    .expect("RUNNING JobRun must have worker_execution_ref"),
            )
            .unwrap(),
        )
        .unwrap()
        .with_staged_artifact_digests(vec![ContentDigest::parse(sha).unwrap()])
        .unwrap(),
    )
}

fn complete_fake_success(
    app: &mut Application,
    run_id: &str,
    bytes: &[u8],
) -> (PathBuf, Validated<JobSpec>) {
    app.mark_dispatchable(run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(run_id, &mut worker).unwrap();
    let running = app.job_status(run_id).unwrap();
    let spec = app.load_job_spec(&running.job_spec_id).unwrap();
    let (path, sha) = write_staged(bytes);
    app.complete_success(run_id, &worker_success(spec.as_record(), &running, &sha))
        .unwrap();
    (path, spec)
}

fn persist_graph(app: &mut Application) -> Graph {
    let g = unpublished_graph();
    persist_core(app.catalog_mut(), &g);
    g
}

#[test]
fn ready_authorizes_transfer_without_acknowledgement() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let auth = app
        .authorize_transfer(&g.compatibility.id().canonical(), false)
        .unwrap();
    assert_eq!(auth.summary, CompatibilitySummary::Ready);
    assert!(auth.eligible);
    assert!(!auth.requires_acknowledgement);
    let (spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    assert_eq!(run.state, JobRunState::Queued);
    assert_eq!(
        spec.as_record().compatibility_result_id(),
        Some(g.compatibility.id())
    );
    assert!(!spec.as_record().compatibility_warnings_acknowledged());
    assert!(spec.as_record().target_derived_variant_id().is_some());
}

#[test]
fn ready_with_warnings_requires_durable_acknowledgement() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let warnings = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::PassWithWarnings,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["optional helper remains unmapped".into()],
    )
    .unwrap();
    app.catalog_mut()
        .put_validated(&certify(warnings.clone()))
        .unwrap();
    let denied = app
        .authorize_transfer(&warnings.id().canonical(), false)
        .unwrap();
    assert_eq!(denied.summary, CompatibilitySummary::ReadyWithWarnings);
    assert!(!denied.eligible);
    assert!(denied.requires_acknowledgement);
    let err = app
        .start_transfer(&warnings.id().canonical(), false, None, "Knight Walk Carry")
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("acknowledgement"), "{msg}"),
        other => panic!("expected Catalog denial, got {other}"),
    }
    assert!(app.catalog().list_job_runs().unwrap().is_empty());
    let accepted = app
        .authorize_transfer(&warnings.id().canonical(), true)
        .unwrap();
    assert!(accepted.eligible);
    let (spec, run) = app
        .start_transfer(&warnings.id().canonical(), true, None, "Knight Walk Carry")
        .unwrap();
    assert_eq!(run.state, JobRunState::Queued);
    assert!(spec.as_record().compatibility_warnings_acknowledged());
    assert_eq!(
        spec.as_record().compatibility_result_id(),
        Some(warnings.id())
    );
}

#[test]
fn mapping_confirmation_required_cannot_transfer() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let mut mapping = BoneMapping::new("unconfirmed").unwrap();
    let version = BoneMappingVersion::draft(
        mapping.id(),
        g.character_version.id(),
        g.source_skeleton.id(),
        vec![common::mapping_entry("root", "Bone")],
        MappingReviewProvenance::with_kind(
            false,
            Some("candidate only".into()),
            vec![],
            MappingReviewKind::AutomaticCandidate,
        )
        .unwrap(),
    )
    .unwrap();
    mapping.bind_draft(version.id());
    app.catalog_mut()
        .put_validated_pair(&certify(mapping), &certify(version.clone()))
        .unwrap();
    let result = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        version.id(),
        g.policy_version.id(),
        Judgment::Unknown,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["confirmation required".into()],
    )
    .unwrap();
    app.catalog_mut()
        .put_validated(&certify(result.clone()))
        .unwrap();
    let auth = app
        .authorize_transfer(&result.id().canonical(), true)
        .unwrap();
    assert_eq!(auth.summary, CompatibilitySummary::MappingConfirmationRequired);
    assert!(!auth.eligible);
    app.start_transfer(&result.id().canonical(), true, None, "Knight Walk Carry")
        .unwrap_err();
    assert!(app.catalog().list_job_runs().unwrap().is_empty());
}

#[test]
fn unsupported_cannot_transfer() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let result = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Fail,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["unsupported fixture".into()],
    )
    .unwrap();
    app.catalog_mut()
        .put_validated(&certify(result.clone()))
        .unwrap();
    let auth = app
        .authorize_transfer(&result.id().canonical(), true)
        .unwrap();
    assert_eq!(auth.summary, CompatibilitySummary::Unsupported);
    assert!(!auth.eligible);
    app.start_transfer(&result.id().canonical(), true, None, "Knight Walk Carry")
        .unwrap_err();
    assert!(app.catalog().list_job_runs().unwrap().is_empty());
}

#[test]
fn worker_success_creates_candidate_not_publication() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-candidate");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
    assert!(version.as_record().qc_report_id().is_none());
    assert!(logical.as_record().published_version_id().is_none());
    let listed = app.list_derived_variants().unwrap();
    assert_eq!(listed.len(), 1);
}

#[test]
fn worker_failure_does_not_publish() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(&run.run_id, &mut worker).unwrap();
    app.complete_failure(&run.run_id, "injected worker failure")
        .unwrap();
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            Path::new("missing.blend"),
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::PublicationDenied);
    let listed = app.list_derived_variants().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].published_version_id.is_none());
}

#[test]
fn qc_fail_leaves_non_published_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-qc-fail");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::failing_finite_transforms(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::PublicationDenied);
    assert_eq!(outcome.qc_verdict.as_deref(), Some("Fail"));
    let version = app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
    let run = app.job_status(&run.run_id).unwrap();
    assert_eq!(run.state, JobRunState::Succeeded);
}

#[test]
fn staged_digest_mismatch_does_not_publish() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    app.mark_dispatchable(&run.run_id).unwrap();
    let mut worker = FakeWorker::default();
    app.dispatch(&run.run_id, &mut worker).unwrap();
    let running = app.job_status(&run.run_id).unwrap();
    let spec = app.load_job_spec(&running.job_spec_id).unwrap();
    let (path, _) = write_staged(b"bytes-b");
    app.complete_success(
        &run.run_id,
        &worker_success(spec.as_record(), &running, &sha256_bytes(b"bytes-a")),
    )
    .unwrap();
    let err = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("does not equal"), "{msg}"),
        other => panic!("expected digest mismatch, got {other}"),
    }
    let listed = app.list_derived_variants().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].published_version_id.is_none());
}

#[test]
fn qc_and_verification_failures_deny_publication() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-verify-fail");
    let mut reopener = MemoryPersistenceReopener::default();
    reopener.fresh_reopen = VerificationOutcome::Fail;
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &reopener,
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::PublicationDenied);
    let version = app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
}

#[test]
fn missing_and_fail_reopen_outcomes_deny_publication() {
    for (fresh, structural) in [
        (VerificationOutcome::Missing, VerificationOutcome::Pass),
        (VerificationOutcome::Fail, VerificationOutcome::Pass),
        (VerificationOutcome::Pass, VerificationOutcome::Missing),
        (VerificationOutcome::Pass, VerificationOutcome::Fail),
    ] {
        let mut app = Application::open_in_memory().unwrap();
        let g = persist_graph(&mut app);
        let (_spec, run) = app
            .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
            .unwrap();
        let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-reopen-outcomes");
        let reopener = MemoryPersistenceReopener {
            fresh_reopen: fresh,
            structural,
        };
        let outcome = app
            .finalize_transfer_for_test(
                &run.run_id,
                &path,
                &MemoryArtifactInspector::passing(None),
                &reopener,
            )
            .unwrap();
        assert_eq!(outcome.kind, TransferOutcomeKind::PublicationDenied);
        let version = app
            .catalog()
            .load_derived_variant_version(&outcome.derived_variant_version_id)
            .unwrap();
        assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
    }
}

#[test]
fn successful_finalize_publishes_and_rerun_is_new_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-run-1");
    let first = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(first.kind, TransferOutcomeKind::Published);
    let first_version = app
        .catalog()
        .load_derived_variant_version(&first.derived_variant_version_id)
        .unwrap();
    assert_eq!(first_version.as_record().lifecycle(), Lifecycle::Published);
    let (_spec2, run2) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            Some(&first.derived_variant_id),
            "Knight Walk Carry",
        )
        .unwrap();
    let (path2, _) = complete_fake_success(&mut app, &run2.run_id, b"v15-run-2-different");
    let second = app
        .finalize_transfer_for_test(
            &run2.run_id,
            &path2,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(second.kind, TransferOutcomeKind::Published);
    assert_eq!(second.derived_variant_id, first.derived_variant_id);
    assert_ne!(
        second.derived_variant_version_id,
        first.derived_variant_version_id
    );
    let still = app
        .catalog()
        .load_derived_variant_version(&first.derived_variant_version_id)
        .unwrap();
    assert_eq!(still.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        still.as_record().job_spec_id(),
        first_version.as_record().job_spec_id()
    );
}

#[test]
fn qc_inspector_does_not_mutate_artifact_bytes() {
    let (path, sha) = write_staged(b"deterministic-qc-fixture");
    let inspector = MemoryArtifactInspector::passing(None);
    let before = sha256_file(&path).unwrap();
    assert_eq!(before, sha);
    inspector.inspect(&path, &sha).unwrap();
    let after = sha256_file(&path).unwrap();
    assert_eq!(before, after);
}

#[test]
fn forged_qc_pass_with_missing_checks_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let mut payload = serde_json::to_value(&g.qc).unwrap();
    payload["verdict"] = serde_json::json!("pass");
    payload["checks"] = serde_json::json!([]);
    let err = ingest_validated::<QcReport>(&payload.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::QcChecksIncomplete);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.qc.clone()))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("QcReport"), "{msg}"),
        other => panic!("QC generic put must be rejected, got {other}"),
    }
}

#[test]
fn generic_put_cannot_publish_without_jobrun_and_graph() {
    let mut app = Application::open_in_memory().unwrap();
    let mut g = persist_graph(&mut app);
    app.catalog_mut()
        .put_validated(&certify(g.job.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.worker.clone()))
        .unwrap();
    app.catalog_mut()
        .put_validated(&certify(g.derived.clone()))
        .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.derived_version.clone()))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains("candidate transaction") || msg.contains("DerivedVariantVersion"),
            "{msg}"
        ),
        other => panic!("expected candidate creation denial, got {other}"),
    }
    g.try_publish();
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.derived_version.clone()))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains("candidate transaction") || msg.contains("DerivedVariantVersion"),
            "{msg}"
        ),
        other => panic!("expected generic DVV denial, got {other}"),
    }
}

#[test]
fn persistence_verification_wrong_instance_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    app.catalog_mut()
        .put_validated(&certify(g.derived.clone()))
        .unwrap();
    let other = PersistenceArtifact::new(
        g.derived_version.id(),
        common::digest(9),
        128,
        "application/octet-stream",
        g.backend.id(),
    )
    .unwrap();
    let forged = PersistenceVerification::new(
        g.persistence.id(),
        other.instance_id(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(forged))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("generic persistence"), "{msg}"),
        other => panic!("expected generic persistence rejection, got {other}"),
    }
}

#[test]
fn qc_report_bound_to_another_artifact_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let a = persist_graph(&mut app);
    let b = persist_graph(&mut app);
    let forged = QcReport::for_derived_variant(
        a.derived_version.id(),
        a.policy_version.id(),
        a.worker.id(),
        b.persistence.id(),
        b.persistence.instance_id(),
        b.persistence.digest().clone(),
        passing_structural_qc_checks(),
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(forged))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("generic persistence") || msg.contains("QcReport"), "{msg}"),
        other => panic!("expected QC generic persistence rejection, got {other}"),
    }
}

#[test]
fn publication_transaction_is_atomic_with_logical_pointer() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-atomic");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    let logical = app
        .load_logical_derived(&outcome.derived_variant_id)
        .unwrap();
    assert_eq!(
        logical.as_record().published_version_id().unwrap().canonical(),
        outcome.derived_variant_version_id
    );
    let version = app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
}

#[test]
fn db_reopen_preserves_published_lineage() {
    let dir = temp_dir("rf-v15-reopen-pub");
    let path = dir.join("catalog.sqlite");
    let ids;
    {
        let mut app = Application::open(&path).unwrap();
        let g = persist_graph(&mut app);
        let (_spec, run) = app
            .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
            .unwrap();
        let (staged, _) = complete_fake_success(&mut app, &run.run_id, b"v15-reopen-bytes");
        let outcome = app
            .finalize_transfer_for_test(
                &run.run_id,
                &staged,
                &MemoryArtifactInspector::passing(None),
                &MemoryPersistenceReopener::default(),
            )
            .unwrap();
        assert_eq!(outcome.kind, TransferOutcomeKind::Published);
        ids = (
            outcome.derived_variant_id,
            outcome.derived_variant_version_id,
        );
    }
    let app = Application::open(&path).unwrap();
    let logical = app.load_logical_derived(&ids.0).unwrap();
    let version = app
        .catalog()
        .load_derived_variant_version(&ids.1)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        logical.as_record().published_version_id().unwrap().canonical(),
        ids.1
    );
}

#[test]
fn structural_qc_checks_are_explicit() {
    let g = unpublished_graph();
    let inspector = MemoryArtifactInspector::passing(None);
    let (path, sha) = write_staged(b"qc-eval");
    let evidence = inspector.inspect(&path, &sha).unwrap();
    let checks =
        evaluate_structural_qc(&evidence, &g.mapping_version, &g.motion_version).unwrap();
    assert_eq!(checks.len(), 7);
    assert_eq!(derive_qc_verdict(&checks).unwrap(), QcVerdict::Pass);
}

fn assert_catalog_authority(err: AppError, needle: &str) {
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains(needle) || msg.contains("generic persistence"),
            "expected `{needle}` or generic persistence, got {msg}"
        ),
        other => panic!("expected Catalog denial, got {other}"),
    }
}

#[test]
fn generic_put_qc_report_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.qc.clone()))
        .unwrap_err();
    assert_catalog_authority(err, "QcReport");
}

#[test]
fn generic_put_pass_verification_is_rejected() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.verification.clone()))
        .unwrap_err();
    assert_catalog_authority(err, "PersistenceVerification");
}

#[test]
fn manual_all_pass_qc_cannot_authorize_publication() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-forged-qc");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let artifact = app
        .catalog()
        .load_artifact_metadata(
            &version
                .as_record()
                .persistence_artifact_id()
                .unwrap()
                .canonical(),
        )
        .unwrap();
    let fabricated = QcReport::for_derived_variant(
        version.as_record().id(),
        version.as_record().policy_version_id(),
        version.as_record().worker_result_id(),
        artifact.as_record().id(),
        artifact.as_record().instance_id(),
        artifact.as_record().digest().clone(),
        passing_structural_qc_checks(),
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(fabricated))
        .unwrap_err();
    assert_catalog_authority(err, "QcReport");
    let err = app
        .publish_derived_variant_version(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("QC") || msg.contains("QcReport") || msg.contains("publication"),
        "{msg}"
    );
    let still = app
        .catalog()
        .load_derived_variant_version(&version.as_record().id().canonical())
        .unwrap();
    assert_eq!(still.as_record().lifecycle(), Lifecycle::Draft);
}

#[test]
fn manual_pass_pass_verification_cannot_authorize_publication() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-forged-verify");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let artifact = app
        .catalog()
        .load_artifact_metadata(
            &version
                .as_record()
                .persistence_artifact_id()
                .unwrap()
                .canonical(),
        )
        .unwrap();
    let fabricated = PersistenceVerification::new(
        artifact.as_record().id(),
        artifact.as_record().instance_id(),
        artifact.as_record().digest().clone(),
        version.as_record().id(),
        artifact.as_record().producer_id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(fabricated))
        .unwrap_err();
    assert_catalog_authority(err, "PersistenceVerification");
    let err = app
        .publish_derived_variant_version(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("verification")
            || msg.contains("QC")
            || msg.contains("QcReport")
            || msg.contains("publication"),
        "{msg}"
    );
}

#[test]
fn trusted_qc_workflow_persists_qc_report() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-trusted-qc");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let qc = app
        .evaluate_and_bind_qc_for_test(
            &version.as_record().id().canonical(),
            &MemoryArtifactInspector::passing(None),
        )
        .unwrap();
    let loaded = app
        .catalog()
        .load_qc_report(&qc.as_record().id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().verdict(), QcVerdict::Pass);
    assert_eq!(
        loaded.as_record().subject_derived_variant_version_id(),
        Some(version.as_record().id())
    );
}

#[test]
fn trusted_reopen_workflow_persists_verification() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-trusted-reopen");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let verification = app
        .verify_and_bind_persistence_for_test(
            &version.as_record().id().canonical(),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    let loaded = app
        .catalog()
        .load_persistence_verification(&verification.as_record().id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().fresh_reopen(), VerificationOutcome::Pass);
    assert_eq!(
        loaded.as_record().structural_verification(),
        VerificationOutcome::Pass
    );
}

#[test]
fn test_memory_inspector_not_available_on_production_surface() {
    let lib = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    let qc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/qc.rs"));
    assert!(
        !lib.contains("MemoryArtifactInspector"),
        "production crate root must not export MemoryArtifactInspector"
    );
    assert!(
        !lib.contains("MemoryPersistenceReopener"),
        "production crate root must not export MemoryPersistenceReopener"
    );
    assert!(
        !qc.contains("pub struct MemoryArtifactInspector"),
        "production qc.rs must not define MemoryArtifactInspector"
    );
    assert!(
        !qc.contains("pub struct MemoryPersistenceReopener"),
        "production qc.rs must not define MemoryPersistenceReopener"
    );
}

#[test]
fn generic_put_cannot_set_nonexistent_published_version() {
    let mut app = Application::open_in_memory().unwrap();
    let mut logical = DerivedVariant::new("pointer-insert").unwrap();
    logical.bind_published(DerivedVariantVersionId::generate());
    let err = app
        .catalog_mut()
        .put_validated(&certify(logical))
        .unwrap_err();
    assert_catalog_authority(err, "published_version_id");
}

#[test]
fn generic_put_cannot_point_to_draft_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-draft-pointer");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let mut rec = logical.into_record();
    rec.bind_published(version.as_record().id());
    let err = app.catalog_mut().put_validated(&certify(rec)).unwrap_err();
    assert_catalog_authority(err, "published_version_id");
}

#[test]
fn generic_put_cannot_point_to_other_variant_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-other-pointer");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let mut other = DerivedVariant::new("other-variant").unwrap();
    other.bind_published(version.as_record().id());
    let err = app
        .catalog_mut()
        .put_validated(&certify(other))
        .unwrap_err();
    assert_catalog_authority(err, "published_version_id");
}

#[test]
fn generic_put_cannot_repoint_published_pointer() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-repoint");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    let mut logical = app
        .load_logical_derived(&outcome.derived_variant_id)
        .unwrap()
        .into_record();
    logical.bind_published(DerivedVariantVersionId::generate());
    let err = app
        .catalog_mut()
        .put_validated(&certify(logical))
        .unwrap_err();
    assert_catalog_authority(err, "published_version_id");
}

#[test]
fn publication_transaction_can_set_pointer() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-pointer-txn");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    let logical = app
        .load_logical_derived(&outcome.derived_variant_id)
        .unwrap();
    assert_eq!(
        logical.as_record().published_version_id().unwrap().canonical(),
        outcome.derived_variant_version_id
    );
}

#[test]
fn version_publication_and_logical_pointer_remain_atomic() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-atomic-named");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    let logical = app
        .load_logical_derived(&outcome.derived_variant_id)
        .unwrap();
    let version = app
        .catalog()
        .load_derived_variant_version(&outcome.derived_variant_version_id)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        logical.as_record().published_version_id().unwrap().canonical(),
        version.as_record().id().canonical()
    );
    assert_eq!(version.as_record().variant_id().canonical(), logical.as_record().id().canonical());
}

#[test]
fn start_transfer_binds_exact_target_derived_variant() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (spec, _run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let listed = app.list_derived_variants().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        spec.as_record()
            .target_derived_variant_id()
            .unwrap()
            .canonical(),
        listed[0].logical_id
    );
    assert_eq!(listed[0].display_name, "Knight Walk Carry");
}

#[test]
fn candidate_ingestion_uses_durable_transfer_target() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-durable-target");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    assert_eq!(
        version.as_record().variant_id(),
        spec.as_record().target_derived_variant_id().unwrap()
    );
    assert_eq!(logical.as_record().id(), version.as_record().variant_id());
}

#[test]
fn finalize_cannot_switch_target_variant() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    app.catalog_mut()
        .put_validated(&certify(DerivedVariant::new("unrelated-target").unwrap()))
        .unwrap();
    let extra = app.list_derived_variants().unwrap();
    assert_eq!(extra.len(), 1);
    let extra_id = extra[0].logical_id.clone();
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-no-switch");
    let outcome = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(outcome.kind, TransferOutcomeKind::Published);
    assert_ne!(outcome.derived_variant_id, extra_id);
    let extra_logical = app.load_logical_derived(&extra_id).unwrap();
    assert!(extra_logical.as_record().published_version_id().is_none());
}

#[test]
fn same_worker_result_cannot_create_two_candidates() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-one-candidate");
    app.ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let err = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("already created"), "{msg}"),
        other => panic!("duplicate candidate must be rejected, got {other}"),
    }
}

#[test]
fn same_jobrun_cannot_publish_two_variant_versions() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-one-jobrun");
    let first = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(first.kind, TransferOutcomeKind::Published);
    let err = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("already created"), "{msg}"),
        other => panic!("second version from one JobRun must be rejected, got {other}"),
    }
}

#[test]
fn rerun_with_new_worker_result_creates_new_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-rerun-a");
    let first = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    let (_spec2, run2) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            Some(&first.derived_variant_id),
            "Knight Walk Carry",
        )
        .unwrap();
    let (path2, _) = complete_fake_success(&mut app, &run2.run_id, b"v15-rerun-b");
    let second = app
        .finalize_transfer_for_test(
            &run2.run_id,
            &path2,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(second.kind, TransferOutcomeKind::Published);
    assert_ne!(second.job_run_id, first.job_run_id);
    assert_ne!(
        second.derived_variant_version_id,
        first.derived_variant_version_id
    );
}

#[test]
fn rerun_existing_variant_preserves_old_published_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-preserve-a");
    let first = app
        .finalize_transfer_for_test(
            &run.run_id,
            &path,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    let (_spec2, run2) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            Some(&first.derived_variant_id),
            "Knight Walk Carry",
        )
        .unwrap();
    let (path2, _) = complete_fake_success(&mut app, &run2.run_id, b"v15-preserve-b");
    let second = app
        .finalize_transfer_for_test(
            &run2.run_id,
            &path2,
            &MemoryArtifactInspector::passing(None),
            &MemoryPersistenceReopener::default(),
        )
        .unwrap();
    assert_eq!(second.derived_variant_id, first.derived_variant_id);
    let still = app
        .catalog()
        .load_derived_variant_version(&first.derived_variant_version_id)
        .unwrap();
    assert_eq!(still.as_record().lifecycle(), Lifecycle::Published);
    let logical = app
        .load_logical_derived(&first.derived_variant_id)
        .unwrap();
    assert_eq!(
        logical.as_record().published_version_id().unwrap().canonical(),
        second.derived_variant_version_id
    );
}

fn candidate_parts(
    app: &Application,
    run_id: &str,
    staged_path: &Path,
    variant_id: DerivedVariantId,
) -> (
    Validated<DerivedVariantVersion>,
    Validated<PersistenceArtifact>,
    crate::catalog::CatalogLocationEvidence,
) {
    let run = app.catalog().load_job_run(run_id).unwrap();
    let spec = app.load_job_spec(&run.job_spec_id).unwrap();
    let worker_id = run.worker_result_id.clone().unwrap();
    let worker = app.catalog().load_worker_result(&worker_id).unwrap();
    let staged_digest = worker.as_record().staged_artifact_digests().first().unwrap();
    let mut version = DerivedVariantVersion::draft(
        variant_id,
        spec.as_record().character_version_id(),
        spec.as_record().motion_version_id(),
        spec.as_record().source_skeleton_ref_id(),
        spec.as_record().mapping_version_id(),
        spec.as_record().policy_version_id(),
        spec.as_record().id(),
        worker.as_record().execution().id(),
        worker.as_record().id(),
    )
    .unwrap();
    let (artifact, dest) = crate::artifact::promote_staged_artifact(
        staged_path,
        staged_digest.sha256(),
        app.catalog().artifact_root(),
        version.id(),
        worker.as_record().execution().id(),
    )
    .unwrap();
    version
        .bind_persistence_artifact(artifact.as_record().id())
        .unwrap();
    let version = Validated::certify(version).unwrap();
    let location = crate::catalog::CatalogLocationEvidence {
        product_id: artifact.as_record().id().canonical(),
        instance_id: artifact.as_record().instance_id().canonical(),
        location_kind: "filesystem_path_evidence".into(),
        location_value: dest.display().to_string(),
        observed_at: crate::migrate::now_ms(),
        note: Some("durable Persistence Artifact; not Product identity".into()),
    };
    (version, artifact, location)
}

#[test]
fn candidate_transaction_rejects_version_for_other_logical() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let other = certify(DerivedVariant::new("other-logical").unwrap());
    app.catalog_mut().put_validated(&other).unwrap();
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-wrong-logical");
    let (version, artifact, location) =
        candidate_parts(&app, &run.run_id, &path, other.as_record().id());
    let err = app
        .catalog_mut()
        .persist_candidate_graph(&run.run_id, &version, &artifact, location)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains("variant_id") || msg.contains("target"),
            "{msg}"
        ),
        other => panic!("expected Catalog denial, got {other}"),
    }
    assert!(app
        .catalog()
        .load_derived_variant_version(&version.as_record().id().canonical())
        .is_err());
}

#[test]
fn candidate_transaction_rejects_wrong_jobspec_target() {
    candidate_transaction_rejects_version_for_other_logical();
}

#[test]
fn candidate_transaction_rejects_artifact_for_other_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, spec) = complete_fake_success(&mut app, &run.run_id, b"v15-wrong-artifact");
    let target = spec.as_record().target_derived_variant_id().unwrap();
    let (version, artifact, location) = candidate_parts(&app, &run.run_id, &path, target);
    let foreign = PersistenceArtifact::new(
        DerivedVariantVersionId::generate(),
        artifact.as_record().digest().clone(),
        1,
        "application/x-blender",
        artifact.as_record().producer_id(),
    )
    .unwrap();
    let foreign = Validated::certify(foreign).unwrap();
    let err = app
        .catalog_mut()
        .persist_candidate_graph(&run.run_id, &version, &foreign, location)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains("bound_derived_variant_version_id")
                || msg.contains("persistence_artifact_id")
                || msg.contains("candidate"),
            "{msg}"
        ),
        other => panic!("expected Catalog denial, got {other}"),
    }
}

#[test]
fn candidate_transaction_rejects_second_candidate_for_worker_result() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-second-candidate");
    app.ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let err = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("already created"), "{msg}"),
        other => panic!("duplicate candidate must be rejected, got {other}"),
    }
}

#[test]
fn forced_candidate_transaction_failure_rolls_back() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let target = spec
        .as_record()
        .target_derived_variant_id()
        .unwrap()
        .canonical();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-forced-rollback");
    let before = app.catalog().load_derived_variant(&target).unwrap();
    assert!(before.as_record().draft_version_id().is_none());
    let (version, artifact, location) = candidate_parts(
        &app,
        &run.run_id,
        &path,
        spec.as_record().target_derived_variant_id().unwrap(),
    );
    let version_id = version.as_record().id().canonical();
    let artifact_id = artifact.as_record().id().canonical();
    app.catalog_mut().fail_next_candidate_transaction();
    let err = app
        .catalog_mut()
        .persist_candidate_graph(&run.run_id, &version, &artifact, location)
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("forced candidate"), "{msg}"),
        other => panic!("expected forced failure, got {other}"),
    }
    let after = app.catalog().load_derived_variant(&target).unwrap();
    assert!(after.as_record().draft_version_id().is_none());
    assert!(
        app.catalog()
            .load_derived_variant_version(&version_id)
            .is_err(),
        "DerivedVariantVersion must be absent after rollback"
    );
    assert!(
        app.catalog().load_artifact_metadata(&artifact_id).is_err(),
        "PersistenceArtifact must be absent after rollback"
    );
    assert!(
        app.catalog()
            .list_payload_locations(&artifact_id)
            .unwrap()
            .is_empty(),
        "payload location must be absent after rollback"
    );
}

#[test]
fn generic_put_cannot_create_derived_variant_version_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    app.catalog_mut()
        .put_validated(&certify(g.derived.clone()))
        .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(g.derived_version.clone()))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("candidate transaction"), "{msg}"),
        other => panic!("expected candidate creation denial, got {other}"),
    }
}

#[test]
fn generic_put_pair_cannot_create_derived_variant_version_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let err = app
        .catalog_mut()
        .put_validated_pair(
            &certify(g.derived.clone()),
            &certify(g.derived_version.clone()),
        )
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("candidate transaction"), "{msg}"),
        other => panic!("expected pair candidate denial, got {other}"),
    }
    assert!(
        app.catalog()
            .load_derived_variant(&g.derived.id().canonical())
            .is_err(),
        "pair rollback must not persist the logical either"
    );
}

#[test]
fn generic_put_cannot_create_second_version_for_same_worker_result() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-second-wr");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let rec = version.as_record();
    let forged = DerivedVariantVersion::draft(
        rec.variant_id(),
        rec.character_version_id(),
        rec.motion_version_id(),
        rec.source_skeleton_ref_id(),
        rec.mapping_version_id(),
        rec.policy_version_id(),
        rec.job_spec_id(),
        rec.backend_id(),
        rec.worker_result_id(),
    )
    .unwrap();
    let err = app
        .catalog_mut()
        .put_validated(&certify(forged))
        .unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(msg.contains("candidate transaction"), "{msg}"),
        other => panic!("expected second-version generic denial, got {other}"),
    }
}

#[test]
fn generic_candidate_cannot_switch_jobspec_target_derived_variant() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let other = certify(DerivedVariant::new("switch-target").unwrap());
    app.catalog_mut().put_validated(&other).unwrap();
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-switch-target");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    let mutated = common::mutate_json_field(
        version.as_record(),
        "variant_id",
        serde_json::json!(other.as_record().id().canonical()),
    );
    let err = app.catalog_mut().put_validated(&mutated).unwrap_err();
    match err {
        AppError::Catalog(msg) => assert!(
            msg.contains("cannot change") || msg.contains("candidate"),
            "{msg}"
        ),
        other => panic!("expected target-switch denial, got {other}"),
    }
}

#[test]
fn publication_requires_logical_draft_pointer_to_exact_version() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-draft-pub");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    app.evaluate_and_bind_qc_for_test(
        &version.as_record().id().canonical(),
        &MemoryArtifactInspector::passing(None),
    )
    .unwrap();
    app.verify_and_bind_persistence_for_test(
        &version.as_record().id().canonical(),
        &MemoryPersistenceReopener::default(),
    )
    .unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(&to_json(logical.as_record()).unwrap()).unwrap();
    payload["draft_version_id"] = serde_json::Value::Null;
    app.catalog_mut()
        .replace_record_payload_for_tests(
            &logical.as_record().id().canonical(),
            &payload.to_string(),
        )
        .unwrap();
    let err = app
        .publish_derived_variant_version(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("draft_version_id") || msg.contains("draft pointer"),
        "{msg}"
    );
}

#[test]
fn publication_requires_jobspec_target_matches_derived_variant() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let other = certify(DerivedVariant::new("pub-target-other").unwrap());
    app.catalog_mut().put_validated(&other).unwrap();
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, spec) = complete_fake_success(&mut app, &run.run_id, b"v15-pub-target");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    app.evaluate_and_bind_qc_for_test(
        &version.as_record().id().canonical(),
        &MemoryArtifactInspector::passing(None),
    )
    .unwrap();
    app.verify_and_bind_persistence_for_test(
        &version.as_record().id().canonical(),
        &MemoryPersistenceReopener::default(),
    )
    .unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(&to_json(spec.as_record()).unwrap()).unwrap();
    payload["target_derived_variant_id"] =
        serde_json::json!(other.as_record().id().canonical());
    app.catalog_mut()
        .replace_record_payload_for_tests(&spec.as_record().id().canonical(), &payload.to_string())
        .unwrap();
    let err = app
        .publish_derived_variant_version(&version.as_record().id().canonical())
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("target_derived_variant_id") || msg.contains("target"),
        "{msg}"
    );
}

#[test]
fn candidate_transaction_still_creates_candidate() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-still-creates");
    let (_logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    assert_eq!(version.as_record().lifecycle(), Lifecycle::Draft);
    assert!(version.as_record().persistence_artifact_id().is_some());
}

#[test]
fn candidate_transaction_still_sets_draft_pointer_atomically() {
    let mut app = Application::open_in_memory().unwrap();
    let g = persist_graph(&mut app);
    let (_spec, run) = app
        .start_transfer(
            &g.compatibility.id().canonical(),
            false,
            None,
            "Knight Walk Carry",
        )
        .unwrap();
    let (path, _) = complete_fake_success(&mut app, &run.run_id, b"v15-still-draft");
    let (logical, version) = app
        .ingest_worker_success_candidate(&run.run_id, &path)
        .unwrap();
    assert_eq!(
        logical.as_record().draft_version_id(),
        Some(version.as_record().id())
    );
    let loaded = app
        .catalog()
        .load_derived_variant(&logical.as_record().id().canonical())
        .unwrap();
    assert_eq!(
        loaded.as_record().draft_version_id(),
        Some(version.as_record().id())
    );
}

#[test]
fn new_worker_result_can_create_new_version() {
    rerun_with_new_worker_result_creates_new_version();
}

#[test]
fn production_qc_binding_still_works() {
    trusted_qc_workflow_persists_qc_report();
}

#[test]
fn production_verification_binding_still_works() {
    trusted_reopen_workflow_persists_verification();
}
