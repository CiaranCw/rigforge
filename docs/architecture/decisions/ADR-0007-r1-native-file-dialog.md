# ADR-0007: R1 native file dialog

Status:
Accepted

This ADR is post-V1. It does not reopen ADR-0004 (egui/eframe) or Gate D.
Do not add the crate in R1-0. R1-A adds it after R1-0 closure.

## Context

V1 Workbench Character/Motion registration currently uses typed filesystem
path fields (`workbench/src/lib.rs`). First-use requires users to paste
FBX paths. R1 requires a native Browse path without making path Product
identity.

Constraints:

```text
eframe 0.31.1
egui 0.31.1
Rust 1.98
Windows primary development environment
cross-platform architecture preferred
Workbench crate only; Application never opens dialogs
```

## Decision

Use **`rfd` 0.17.2** (`FileDialog::pick_file`, synchronous) in the
Workbench crate for Character and Motion source selection.

Classification of upstream facts: `OFFICIAL_DOC` (crates.io / docs.rs /
GitHub README for rfd 0.17.2, retrieved 2026-09-03).

| Topic | Evidence |
| --- | --- |
| Current stable | 0.17.2 (published 2026-01-12) |
| License | MIT |
| MSRV | 1.88 (changelog: 0.17.2 lowered MSRV to 1.88). Compatible with RigForge 1.98 |
| Windows | native file dialog; filters supported |
| macOS | native; docs recommend spawning on the main thread |
| Linux | default `xdg-portal`; GTK3 alternative via features; runtime portal/Zenity. **Release-packaging / later distribution consideration, not an R1-A functional blocker** (Windows is the primary development environment) |
| Filters | `add_filter("FBX", &["fbx"])` |
| Sync vs async | both exist; R1 uses sync `pick_file` so Tokio is unnecessary |
| Cancel | `Option::None` |
| WASM | unsupported for this Workbench; irrelevant |

Workbench shows filename as the label and full path as tooltip. Application
receives a `Path` only as `SourceArtifactEvidence` location.

## Alternatives considered

| Option | Why not (R1) |
| --- | --- |
| Keep typed paths | fails the frozen first-use requirement |
| `tinyfiledialogs` | C/bundled dialogs; weaker native integration; extra native toolchain |
| `native-dialog` | less common with egui; no reason to prefer over rfd |
| egui-only path widget | still typing; not native Browse |
| Async `rfd` + Tokio | rejected; R1 forbids Tokio merely for elegance |
| No dependency / Win32 `IFileOpenDialog` only | Windows-only; fights “cross-platform architecture preferred” |

## Consequences

- Workbench gains one MIT dialog crate at R1-A, not R1-0.
- Linux XDG Desktop Portal / Zenity runtime is a **release-packaging**
  implication. It is not an R1-A functional blocker and not public
  redistribution work.
- Sync modal Browse blocks the egui loop while the OS dialog is open. That
  is accepted as user-modal, not as a Blender-length wait.
- File picking is ingest UX, not V1-7 export.

## Evidence

- https://crates.io/crates/rfd
- https://docs.rs/rfd/0.17.2/rfd/
- https://github.com/PolyMeilex/rfd
- RigForge `workbench/Cargo.toml`: `eframe` 0.31.1, `rust-version` 1.98

## Open questions

None for the R1-0 design selection. Pin exact 0.17.2 at R1-A unless a
newer patch is reviewed then.
