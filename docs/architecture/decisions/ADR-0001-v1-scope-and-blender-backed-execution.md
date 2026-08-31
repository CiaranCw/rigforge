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

Blender as PROVISIONAL V1 execution backend
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

Core language remains **not selected**. POC-CORE-01 remains `INCONCLUSIVE`.
This architecture makes that selection less urgent. C++ is not silently
selected.

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
- Accepted W0-SR proposal package under `*_PROPOSAL.md` and
  [W0P_REPLAN_PROPOSAL.md](../../research/poc/W0P_REPLAN_PROPOSAL.md).

This ADR does **not** claim the Blender architecture is finally proven.

```text
PROVISIONAL_POC_GATED
pending POC-BLENDER-E2E-01
```

## Open questions

1. Can Blender execute the complete workflow reliably on a real
   different-Skeleton Character/Motion pair behind a backend-neutral Job Spec?
2. Is the thin Skeleton Summary sufficient for Mapping, Compatibility, QC,
   Preview, and backend replacement?
3. Is a generated Preview Artifact sufficient for click-to-preview, or is
   another payload required?
4. Does Asset Browser latency later justify a native ufbx inspector?
5. Can Blender packaging and worker-script licensing be cleared?
6. Does a concrete downstream consumer require Export Artifacts or versioned
   Export Profiles in V1?

Viewer technology, Core language, database, and exact schemas remain `OPEN`
or `DEFER`.
