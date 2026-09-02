# ADR-0006: V1 engine-independent Preview

Status: **Accepted**

Date: 2026-09-02

Accepted at: **V1-6 focused review / focused closure** (`PASS / CLOSED`).

Owner stage: V1-6

This decision selects a V1 Preview **viewer library**, **payload media
type**, and **view-time surface**. It does not make Preview Product
authority. It does not reopen ADR-0001, ADR-0004, or ADR-0005. It is not a
V1-7 export contract.

## Context

POC-PREVIEW-01R validated `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS` on a
research harness. V1-6 must ship production click-to-preview for exact
Character, Motion, and Derived Variant versions without Unreal, Unity,
Blender UI, or Maya at view time.

ADR-0004 selected egui/eframe for the Workbench shell. That selection does
not choose a Preview viewer or payload.

## Decision

- Preview payload: GLB / `model/gltf-binary`. This is a derived internal
  viewing representation, not a Product format and not an export.
- Viewer library: `@google/model-viewer` **4.3.1**, vendored offline.
  Official npm integrity:
  `sha512-GP+inXhAtY31E8rILVmByA6z8CZZjdlNajddppyI1/j1eIaSQiZcMRaUqTFe7+jv4mzRzwKIOiKBud0apiv+WQ==`.
  Local min.js SHA-256:
  `283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b`.
- Surface: Workbench sidecar local HTTP + OS browser window. Not Product
  authority. `wry` is not added. `WorkbenchApp::requires_blender()` remains
  `false`.
- Generator: pinned Blender 5.2.1 LTS build `9e2066aef7ef` at generation
  time only, sealed like QC.
- Binding: `PreviewArtifact → exact Product Version`. Product truth
  validates Preview; Preview never validates Product truth.

License (to record, not to clear for release): package Apache-2.0; min.js
also contains bundled Lit BSD-3-Clause headers.

## Alternatives considered

- Embed a native `wry` WebView in the egui process: not selected. Sidecar
  reuses the proven PoC browser path without adding a WebView crate now.
  WebView bootstrap remains V1-8 if needed.
- glTF JSON / USD Preview / engine viewport: rejected for V1-6. Extra
  Product-adjacent formats and engine runtimes are out of scope.
- Treat GLB as a V1-7 export: rejected. Preview payload is viewing-only.
- Infer viewer from ADR-0004: rejected. GUI crate ≠ viewer.

## Consequences

Workbench can request Preview through Application. View time stays
engine-independent. Transfer native checkbox/button wiring
(`GATE-C-OBS-001`) is unchanged unless separately event-wired.

V1-8 leftovers: installer bundling, dependency licensing clearance,
WebView runtime bootstrap, offline installer, worker/viewer upgrade
rollback, broad browser/GPU matrix, full non-humanoid asset campaign,
process recovery.

## Evidence

POC-PREVIEW-01R (historical; not rewritten) plus V1-6 production Domain /
Catalog / Application / generator / viewer tests, including a frozen-pair
Character / Motion / Derived checkpoint. Focused review closed
`V1-6-MAJOR-001` (Catalog producer existence) and `V1-6-MAJOR-002`
(Motion / Derived fail closed when the loaded GLB has no animations).

## Open questions

Viewer/payload/surface selection is Accepted. Remaining work is V1-8
hardening, not a reopen of this decision: installer, licensing clearance,
WebView bootstrap, browser/GPU matrix, broader real-asset coverage,
process recovery, and upgrade rollback. `GATE-C-OBS-001` remains historical
non-blocking (native Transfer checkbox/button wiring).
