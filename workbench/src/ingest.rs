//! Background Blender source inspection. Does not open Catalog or Application.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use egui::Context;
use rigforge_app::{
    AppError, CharacterSourceInspection, MotionSourceInspection, SourceInspectionProvider,
    INGEST_INSPECT_FAILED,
};
use rigforge_blender_worker::BlenderSourceInspector;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IngestKind {
    Character,
    Motion,
}

pub enum IngestResult {
    Character {
        request_id: u64,
        outcome: Result<CharacterSourceInspection, String>,
    },
    Motion {
        request_id: u64,
        outcome: Result<MotionSourceInspection, String>,
    },
}

pub fn spawn_source_inspect(
    request_id: u64,
    kind: IngestKind,
    path: PathBuf,
    ctx: Context,
) -> Result<Receiver<IngestResult>, AppError> {
    let inspector = BlenderSourceInspector::production()?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = match kind {
            IngestKind::Character => IngestResult::Character {
                request_id,
                outcome: inspector
                    .inspect_character_source(&path)
                    .map_err(user_facing_inspect_error),
            },
            IngestKind::Motion => IngestResult::Motion {
                request_id,
                outcome: inspector
                    .inspect_motion_source(&path)
                    .map_err(user_facing_inspect_error),
            },
        };
        let _ = tx.send(result);
        ctx.request_repaint();
    });
    Ok(rx)
}

fn user_facing_inspect_error(err: AppError) -> String {
    match err {
        AppError::Catalog(msg) if !msg.trim().is_empty() => msg,
        other => {
            let detail = other.to_string();
            if detail.trim().is_empty() {
                INGEST_INSPECT_FAILED.to_string()
            } else {
                format!("{INGEST_INSPECT_FAILED} {detail}")
            }
        }
    }
}
