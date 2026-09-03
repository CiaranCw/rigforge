# ADR-0001: V1 scope and Blender-backed execution

Status: **Accepted**

Date: 2026-08-31

## Context

Clarified product requirement:

```text
asset browser-like workbench
select rigged Character
select Motion
one-click transfer
Derived Variant
Preview / Version
```

Character input is already mesh + skeleton/armature + skin weights. Raw
unrigged mesh, Auto-Rig, skeleton generation, and automatic skinning are not
the V1 job. Motion requires source Skeleton context. The result must be a
managed, previewable, versioned Derived Variant.

Earlier W0 research reasonably optimized for a broad Character / Skeleton /
Motion infrastructure platform: heavy Canonical objects, native format
adapters, and a RigForge-owned evaluation/retarget runtime. That destination
does not match the clarified user transaction.

```text
rigged Character A + Motion B → Transfer → Derived Variant C
```

W0.1–W0.4, POC-CORE-01, and POC-FBX-01 remain accepted evidence. They do not
by themselves require shipping a complete Canonical runtime or a native
FBX/glTF stack in V1.

Rationale package (historical):
[W0_SCOPE_REVISION_PROPOSAL.md](../../research/decisions/W0_SCOPE_REVISION_PROPOSAL.md).

## Decision

Adopt:

```text
thin Workflow Domain

Blender as V1 execution backend
ACCEPT_BLENDER_BACKEND_WITH_GUARDS
behind replaceable worker boundary
```

and remove heavy native asset/runtime infrastructure from the V1 critical
path.

Current contracts:

- Product: [PRODUCT_VISION.md](../../product/PRODUCT_VISION.md),
  [V1_SCOPE.md](../../product/V1_SCOPE.md)
- Architecture: [V1_BLENDER_BACKED_ARCHITECTURE.md](../V1_BLENDER_BACKED_ARCHITECTURE.md),
  [V1_WORKFLOW_DOMAIN.md](../V1_WORKFLOW_DOMAIN.md)

RigForge owns durable product semantics: Asset, Mapping, Compatibility,
Retarget Policy, Job, Derived Variant, Validation, Preview orchestration, and
versioning. Blender may own execution mechanics (import, armature/animation
evaluation, constraints, IK when required, retarget mechanics, bake,
conversion, preview-artifact generation, practical output generation) inside
an isolated worker.

Blender data-block names/types and Python API objects must not leak into
durable product contracts. Future worker replacement must be possible without
rewriting those product objects.

Native format parsers are not mandatory V1 infrastructure. POC-FBX-01 remains
`COMPLETE / PASS` with `KEEP_UFBX_WITH_GUARDS`; ufbx placement (inspector,
optimization, specialized adapter, or unused in V1) is **not decided**.

At ADR-0001 adoption time: Core language had not yet been selected.
POC-CORE-01 remains `INCONCLUSIVE` historical evidence. C++ was not silently
selected. This architecture made that selection less urgent at the time.

Current disposition: superseded for this question by
[ADR-0002](ADR-0002-v1-core-language.md) **Accepted** (Rust) at V1-1 Gate A.

## Alternatives considered

- **Heavy Canonical / native infrastructure.** Keep Canonical Character /
  Skeleton / Motion plus native adapters as V1 foundation. Rejected for V1:
  it optimizes for an underspecified infrastructure platform, not the
  clarified browse/select/transfer/preview/version workflow.
- **Custom retarget runtime.** RigForge owns solver math and evaluation.
  Rejected as V1 critical path if the Blender worker proves controlled
  execution; policy remains product-owned either way.
- **Multi-DCC V1.** Ship Blender and Maya (and peers) as first backends.
  Rejected: one replaceable backend is enough; boundaries provide generality
  without multiple implementations.
- **Direct engine integrations.** Unreal / Unity / Godot adapters in V1.
  Rejected (`DROP_FROM_V1`): the accepted workflow has no engine dependency.
  Preview is engine-independent.

## Consequences

Benefits:

- Product matches the real user transaction.
- Mapping, Compatibility, QC, Preview, and versioning stay first-class.
- Implementation can proceed without owning a complete DCC clone.
- Engine independence and format non-authority are preserved at the product
  layer.
- Historical W0 evidence is retained as risk/semantic knowledge.

Risks:

- Blender becomes an unproven operational dependency (version/API churn,
  headless gaps, startup cost, crash isolation, importer loss, packaging,
  GPL/worker-script obligations).
- A too-thin Skeleton Summary can hide Blender coupling; a too-rich summary
  can recreate the old Canonical model.
- One-click UX can hide mapping ambiguity or result-quality limits.
- Worker success can be mistaken for QC success.
- A humanoid first pair can accidentally hard-code humanoid roles.
- Reopening “DCC is never mandatory for ingest” is an explicit V1 operational
  conflict with historical research; it is not a silent rewrite of that
  history.

## Evidence

- W0.1 Canonical Foundations — rest/bind/IBM distinctions, names ≠ identity,
  format limits (partially applicable; risk knowledge).
- W0.2 Mapping / Compatibility / Retarget — still applicable as product
  capability and policy.
- W0.3 Adapter / Infrastructure — adapter ≠ authority, derived Preview,
  process isolation (partially applicable; worker boundary survives).
- W0.4 Product / Tech Decisions — option/risk research; native
  format/Core/runtime rankings no longer determine V1 critical path.
- POC-CORE-01 — `COMPLETE / PASS`; impact `INCONCLUSIVE`.
- POC-FBX-01 — `COMPLETE / PASS`; impact `KEEP_UFBX_WITH_GUARDS`.
- POC-BLENDER-E2E-01 — `COMPLETE / PASS`; impact
  `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`. Demonstrated, for one real
  different-Skeleton pair: backend-neutral Job Spec, backend-neutral Policy,
  QC-gated publication, fresh-process persistence, and no large custom
  RigForge retarget runtime.
- POC-PREVIEW-01R — `COMPLETE / PASS`; impact
  `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`. Demonstrated, for one research
  Character / Motion / Derived Variant set: derived rebuildable
  non-authoritative Preview Artifacts, independent Product binding, payload
  integrity checks, and engine-independent click-to-preview without Blender
  or a game engine at view time.
- Accepted W0-SR proposal package under `*_PROPOSAL.md` and
  [W0P_REPLAN_PROPOSAL.md](../../research/poc/W0P_REPLAN_PROPOSAL.md).

This is a W0 research/architecture validation result. At ADR-0001 adoption
time, Product implementation remained unauthorized until W0-RS and IA-1
completed the remaining required gates.

Current disposition: W0-RS and IA-1 are complete. V1-1 through V1-8 are
implemented and baselined. Implementation is authorized for those completed
stages. Gate D is **PASS / CLOSED**. Final independent result:
`GATE_D_PASS_CANDIDATE`. Blender is not product authority and is not
permanently final.

At ADR-0001 adoption time: viewer library and Preview payload format were
not selected. Current disposition: superseded for this question by
[ADR-0006](ADR-0006-v1-engine-independent-preview.md) **Accepted**
(GLB + `@google/model-viewer` 4.3.1).

Database / catalog storage: at adoption time not selected; current
disposition superseded by [ADR-0003](ADR-0003-v1-local-catalog-storage.md)
**Accepted** (SQLite / rusqlite bundled). GUI: superseded by
[ADR-0004](ADR-0004-v1-workbench-gui.md) **Accepted** (egui/eframe).
Blender worker process packaging: superseded by
[ADR-0005](ADR-0005-v1-blender-worker-process.md) **Accepted**.

```text
POC-BLENDER-E2E-01:
COMPLETE / PASS

Decision Impact:
ACCEPT_BLENDER_BACKEND_WITH_GUARDS

POC-PREVIEW-01R:
COMPLETE / PASS

Decision Impact:
ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
```

## Open questions (historical at ADR-0001 adoption; current dispositions)

1. Preview Artifact / viewer path: is a generated Preview Artifact
   sufficient for click-to-preview, or is another payload required?
   **YES, WITH GUARDS**, for the tested Character / Motion / Derived Variant
   vertical slice (POC-PREVIEW-01R). At adoption time, viewer technology and
   production payload remained open. Current disposition: ADR-0006 **Accepted**
   (GLB + `@google/model-viewer` 4.3.1). Material fidelity and broader
   browser/platform qualification remain later hardening.
2. Is the thin Skeleton Summary sufficient beyond the tested humanoid pair
   for Mapping, Compatibility, QC, Preview, and backend replacement?
   Current: architecture does not assume humanoid-only roles; sufficiency
   for arbitrary real pairs remains an evidence question, not a Product
   prohibition of non-humanoid input.
3. Non-humanoid hardening: at adoption time, real E2E remained later
   mandatory validation. Current V1 technical release requirement is
   backend-neutral/general Product semantics, no mandatory humanoid role
   table, and honest fail-closed Compatibility for unsupported pairs.
   Successful real non-humanoid retarget quality is **not** a V1 release
   guarantee (`POST-V1 HARDENING / FUTURE QUALIFICATION`).
4. Auto-Mapping quality remains later validation.
5. Worker reliability hardening (timeout/cancel/pools) remains later.
   Spawn-to-RUNNING crash recovery and runtime worker-package integrity
   were implemented in V1-8.
6. Can Blender packaging and worker-script licensing be cleared?
   V1-8 inventoried notices and pins; distribution legal clearance remains
   open / POST-V1.
7. Does a concrete downstream consumer require Export Artifacts or versioned
   Export Profiles in V1? V1-7 remains `SKIPPED / OPTIONAL`.

At ADR-0001 adoption time, viewer technology, production Preview payload,
Core language, database, and exact schemas remained `OPEN` or `DEFER`.
Those questions are **not** current truth. Current dispositions:

```text
Core language:     ADR-0002 Accepted (Rust)
Catalog:           ADR-0003 Accepted (SQLite / rusqlite bundled)
GUI:               ADR-0004 Accepted (egui / eframe)
Blender worker:    ADR-0005 Accepted
Preview:           ADR-0006 Accepted (GLB + @google/model-viewer 4.3.1)
V1-1 through V1-8: implemented / baselined
Gate D:            PASS / CLOSED
Independent result: GATE_D_PASS_CANDIDATE
```
