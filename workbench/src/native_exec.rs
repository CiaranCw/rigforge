//! Native Host Transfer execution.
//!
//! Widgets do not call `bpy`. WorkbenchHost uses the blender-worker adapter
//! crate only to launch isolated processes for a user-requested Transfer.
//! ADR-0005 remains Accepted: Domain stays backend-neutral; Blender is not
//! Product authority.

use rigforge_app::{AppError, Application, TerminalOutcome, TransferOutcome};
use rigforge_blender_worker::BlenderWorker;

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
