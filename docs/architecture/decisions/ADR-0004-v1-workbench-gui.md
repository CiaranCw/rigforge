# ADR-0004: V1 Workbench GUI

Status: **Accepted**

Date: 2026-09-01

Accepted at: **V1-2 focused review** (`PASS`).

Owner stage: V1-2 (framework selection and shell). Later UI depth is owned by
the stages that need it (Asset Browser polish, Transfer Tray, V1-6 Preview).

This decision is a Workbench integration choice. It is **not** a Preview
viewer selection, **not** a Preview payload selection, and **not** a Core
language decision. Rust Core does not imply a Rust GUI.

## Context

V1 is a desktop Character Animation Asset Workbench. The accepted product
requires an Asset Browser, selection, a Transfer Tray, job status, and later
click-to-preview. GUI framework was OPEN after V1-1. V1-2 is the owner of
the direction.

Required of V1-2 (this ADR + shell):

- select a GUI framework with rationale
- define the application boundary
- provide a minimal Workbench shell, or prove the selected integration
  compiles and can initialize without network
- describe Asset Browser architecture
- leave a future Preview *embedding* boundary without choosing a viewer

Not required of V1-2: polished UI, pixel-perfect tests, Blender execution,
or a Preview renderer.

## Requirements

| Requirement | Notes |
| --- | --- |
| Desktop Asset Browser | Lists/grids of Character, Motion, Derived Variant |
| Local-first | No cloud account, no mandatory network for shell init |
| Selection | Exact version resolution, not `latest` at click time |
| Transfer Tray later | One Character + one Motion; V1-2 only needs the boundary |
| Job status | Read orchestration state; do not mutate JobSpec |
| Preview embedding later | A slot/host, not a chosen viewer or GLB pipeline |
| Cross-platform packaging | Windows is a V1 target |
| Accessibility / input | Keyboard + pointer for lists/selection; native a11y is a tradeoff |
| Maintainability | Small team, few GUI crates |
| Rust Core integration | Call `app` / Domain APIs in-process; never `bpy` |

Forbidden path:

```text
GUI → bpy / Blender
```

Required path:

```text
GUI
 ↓
Application / Catalog / Orchestrator
 ↓
Worker Port
 ↓
future V1-3 Blender worker
```

## Candidate set

Smallest set that can realistically host a local desktop workbench. No GUI
framework survey.

| Candidate | Why it is in the set |
| --- | --- |
| egui + eframe | Immediate-mode Rust UI used for desktop tools; no web runtime |
| Tauri 2 | Desktop webview shell; strong OS packaging story |
| Slint | Compiled native UI toolkit with a Rust runtime |
| iced | Elm-style Rust GUI; wgpu desktop rendering |

GTK was not added: Windows packaging and event-loop cost are not justified
for a first V1 shell. Qt/QML was not added: C++/QML binding would reopen a
language split ADR-0002 already closed for Core, and it is not required to
evaluate Qt to choose a first Workbench shell.

## Decision

**Selected: egui (immediate-mode) hosted by eframe.**

Crate: `workbench/` (`rigforge_workbench`).

Application boundary crate: `app/` (`rigforge_app`). The GUI may not contain
Product invariants that exist only in UI code.

Asset Browser architecture:

```text
Workbench lists
        ↓
Application queries (Character / Motion / Derived separately)
        ↓
Catalog exact IDs
        ↓
Domain Validated<T>
```

The Browser presents **logical assets** and can resolve a **selected exact
version**. It does not query Blender, FBX parsers, or Preview viewers.

Preview embedding boundary (intentionally empty of viewer types):

```text
PreviewEmbeddingSlot
  occupied: bool
  host: not selected
  payload format: not selected
```

V1-6 owns viewer library and payload format. W0 POC-PREVIEW-01R used a
GLB / browser path as *candidate evidence only*. That is not adopted here.

## Why egui/eframe

- **Local-first / no network:** native process, no webview, no JS runtime,
  no Chromium download. Shell init in tests constructs egui context without
  opening sockets.
- **Asset Browser:** lists, grids, selection, and a status panel are ordinary
  egui widgets. Enough for V1-2; not a polished DCC.
- **Core integration:** the Workbench crate depends on `rigforge_app` and
  calls it in-process. No FFI-to-GUI-framework tax, and no implication that
  “Core is Rust therefore GUI must be Rust” was the selection rule — Tauri
  could also call a Rust Core. egui was chosen because it avoids a web
  substrate that would make the PoC browser Preview path the default.
- **Worker independence:** no path from UI to `bpy`.
- **Packaging:** one native executable with winit/egui. Heavier than a CLI,
  much lighter than shipping a browser engine.
- **Preview later:** an egui panel or a future native child surface can host
  a V1-6 viewer without this ADR naming that viewer.

This is **not** selected because Core is Rust. Core language and GUI
framework are independent decisions (ADR-0002 vs this ADR).

## Alternatives considered

### Tauri 2

Would give OS webviews, HTML/CSS layout, and an easy HTML Preview embed.

Rejected for V1:

- makes a webview the Workbench substrate
- strongly biases later Preview toward the W0 GLB / `model-viewer` / browser
  candidate, which this stage must not preselect
- extra webview/OS WebView2 dependency is not needed for lists and job status
- still local-offline capable, but not the smallest local-first shell

Revisit if V1-6 produces a web-only Preview host that cannot be embedded any
other way *and* product evidence says the Workbench must host it in-process.

### Slint

Native compiled UI, designed for desktop.

Rejected for V1:

- dual licensing (GPLv3 / commercial) is a poor fit for an UNLICENSED
  product tree that has not chosen a distribution license
- extra `.slint` toolchain for a shell that only needs lists and a slot

Revisit if a chosen distribution license is compatible and native widgets
become a product requirement.

### iced

Elm architecture, wgpu, MIT/Apache.

Rejected for V1:

- more moving parts for a first Asset Browser shell
- wgpu coupling is closer to a renderer than V1-2 needs
- weaker “tool UI now” ecosystem than egui for this slice

Revisit if immediate-mode egui becomes a maintenance problem and iced’s
retained model is clearly better for Transfer Tray / Preview host.

## Core integration

```text
rigforge_workbench
        ↓
rigforge_app  (catalog, queries, orchestration, WorkerPort)
        ↓
rigforge_domain  (Validated<T>, IDs, JobSpec)
```

The GUI never holds a Blender session. Job buttons enqueue validated
`JobSpec` values that already contain exact version IDs.

## Preview integration implications

| This ADR decides | This ADR does not decide |
| --- | --- |
| There is a Workbench surface that can later host Preview | Viewer library |
| Preview is not required to initialize the shell | Payload format (GLB or otherwise) |
| Preview is derived / non-authoritative (already V1-1) | Engine runtime |

Do not add `model-viewer`, a GLB crate, or a browser Preview pane in V1-2
“to save time for V1-6”.

## Packaging implications

- V1 Workbench ships as a native desktop binary (`workbench`).
- No mandatory browser, Node, or cloud service.
- egui/eframe pull windowing (winit) and a renderer. That is accepted for a
  GUI shell and is not a Preview-viewer choice.
- Cross-platform: Windows first in this repository; winit is the portability
  layer. Platform look-and-feel is not a V1-2 acceptance criterion.

## Revisit triggers

Reopen this ADR if:

- V1-6 cannot embed its chosen Preview host in an egui slot and product
  requires in-window Preview (not an external process)
- accessibility requirements exceed what egui can honestly provide
- packaging size / renderer dependencies become a release blocker
- the team must use OS-native widgets as a product constraint

Do not reopen because a different Rust GUI crate is trending.

## Evidence

Implementation: `workbench/` (`rigforge_workbench`).

Contracts:

- [V1_2_IMPLEMENTATION_PLAN.md](../../development/V1_2_IMPLEMENTATION_PLAN.md)
- [V1_2_CATALOG_CONTRACT.md](../../development/V1_2_CATALOG_CONTRACT.md)
- [V1_2_ORCHESTRATION_CONTRACT.md](../../development/V1_2_ORCHESTRATION_CONTRACT.md)

Related: [ADR-0003](ADR-0003-v1-local-catalog-storage.md) (SQLite catalog).
