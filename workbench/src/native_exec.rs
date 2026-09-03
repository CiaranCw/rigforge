//! Native Host Transfer execution.
//!
//! Widgets do not call `bpy`. WorkbenchHost uses the blender-worker adapter
//! crate only to launch isolated processes for a user-requested Transfer.
//! ADR-0005 remains Accepted: Domain stays backend-neutral; Blender is not
//! Product authority.
//!
//! Blender-length waits run on a background thread. Application/Catalog
//! mutation stays on the UI thread.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use egui::Context;
use rigforge_app::{
    inspect_durable_persistence_artifact, reopen_durable_persistence_artifact, AppError,
    Application, ArtifactInspectionEvidence, DispatchReceipt, PersistenceReopenRequest,
    QcAcquisitionRequest, TerminalOutcome, TransferOutcome, WorkerCompletionPort, WorkerPort,
};
use rigforge_app::rigforge_domain::VerificationOutcome;
use rigforge_blender_worker::BlenderWorker;

use crate::long_op::WaitMsg;

pub fn collect_execute_wait<C: WorkerCompletionPort + ?Sized>(
    worker: &mut C,
    receipt: &DispatchReceipt,
) -> Result<(TerminalOutcome, Option<PathBuf>), AppError> {
    let outcome = worker.collect(receipt)?;
    let staged = worker.last_staged_artifact().map(PathBuf::from);
    Ok((outcome, staged))
}

pub fn spawn_execute_collect<C>(
    mut worker: C,
    receipt: DispatchReceipt,
    run_id: String,
    ctx: Option<Context>,
) -> Receiver<WaitMsg>
where
    C: WorkerCompletionPort + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let outcome = collect_execute_wait(&mut worker, &receipt).map_err(|err| err.to_string());
        let _ = tx.send(WaitMsg::Execute { run_id, outcome });
        if let Some(ctx) = ctx {
            ctx.request_repaint();
        }
    });
    rx
}

pub fn spawn_qc_acquire(
    run_id: String,
    request: QcAcquisitionRequest,
    ctx: Option<Context>,
    acquire: impl FnOnce(&QcAcquisitionRequest) -> Result<ArtifactInspectionEvidence, String>
        + Send
        + 'static,
) -> Receiver<WaitMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let derived_version_id = request.derived_version_id.clone();
        let outcome = acquire(&request);
        let _ = tx.send(WaitMsg::Qc {
            run_id,
            derived_version_id,
            outcome,
        });
        if let Some(ctx) = ctx {
            ctx.request_repaint();
        }
    });
    rx
}

pub fn spawn_reopen_acquire(
    run_id: String,
    request: PersistenceReopenRequest,
    ctx: Option<Context>,
    acquire: impl FnOnce(
            &PersistenceReopenRequest,
        ) -> Result<(VerificationOutcome, VerificationOutcome), String>
        + Send
        + 'static,
) -> Receiver<WaitMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let derived_version_id = request.derived_version_id.clone();
        let outcome = acquire(&request);
        let _ = tx.send(WaitMsg::Reopen {
            run_id,
            derived_version_id,
            outcome,
        });
        if let Some(ctx) = ctx {
            ctx.request_repaint();
        }
    });
    rx
}

pub fn production_qc_acquire(
    request: &QcAcquisitionRequest,
) -> Result<ArtifactInspectionEvidence, String> {
    inspect_durable_persistence_artifact(
        &request.artifact_path,
        &request.expected_sha256,
        Some(&request.mapping),
    )
    .map_err(|err| err.to_string())
}

pub fn production_reopen_acquire(
    request: &PersistenceReopenRequest,
) -> Result<(VerificationOutcome, VerificationOutcome), String> {
    reopen_durable_persistence_artifact(
        &request.artifact_path,
        &request.expected_sha256,
        &request.mapping,
    )
    .map_err(|err| err.to_string())
}

pub fn launch_and_spawn_execute(
    app: &mut Application,
    run_id: &str,
    ctx: Option<Context>,
) -> Result<Receiver<WaitMsg>, AppError> {
    app.mark_dispatchable(run_id)?;
    let mut worker = match BlenderWorker::production() {
        Ok(worker) => worker,
        Err(err) => {
            let _ = app.complete_failure(run_id, err.to_string());
            return Err(err);
        }
    };
    let (_run, receipt) = app.dispatch(run_id, &mut worker)?;
    Ok(spawn_execute_collect(
        worker,
        receipt,
        run_id.to_string(),
        ctx,
    ))
}

pub fn launch_and_spawn_execute_with_worker<W>(
    app: &mut Application,
    run_id: &str,
    mut worker: W,
    ctx: Option<Context>,
) -> Result<Receiver<WaitMsg>, AppError>
where
    W: WorkerPort + WorkerCompletionPort + Send + 'static,
{
    app.mark_dispatchable(run_id)?;
    let (_run, receipt) = app.dispatch(run_id, &mut worker)?;
    Ok(spawn_execute_collect(
        worker,
        receipt,
        run_id.to_string(),
        ctx,
    ))
}

/// Blocking helper for non-UI callers. WorkbenchHost does not use this.
pub fn complete_native_transfer(
    app: &mut Application,
    run_id: &str,
) -> Result<TransferOutcome, AppError> {
    let mut worker = BlenderWorker::production()?;
    app.mark_dispatchable(run_id)?;
    app.dispatch(run_id, &mut worker)?;
    let (run, outcome) = app.collect(run_id, &mut worker)?;
    match outcome {
        TerminalOutcome::Success(_) => {
            let staged = worker.last_staged_blend().ok_or_else(|| {
                AppError::Worker(
                    "successful collect did not retain a staged artifact path".into(),
                )
            })?;
            app.finalize_transfer(&run.run_id, staged)
        }
        TerminalOutcome::Failed { reason, .. } => Err(AppError::Worker(reason)),
    }
}
