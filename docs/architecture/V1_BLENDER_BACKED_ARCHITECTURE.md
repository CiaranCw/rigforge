# V1 Blender-Backed Architecture

Current V1 execution architecture.

Blender as backend is **`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`**, validated by
POC-BLENDER-E2E-01. It is not product authority, not permanently final, and
not coverage of all Characters/Motions. Product implementation remains
unauthorized until W0-RS and IA-1 complete the remaining required gates.

Decision: [ADR-0001](decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Domain: [V1_WORKFLOW_DOMAIN.md](V1_WORKFLOW_DOMAIN.md).
Rationale (historical): [V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md](V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md).

## Shape

```text
RigForge Workbench
        ↓
Application / Workflow Domain
        ↓
backend-neutral Job Spec
        ↓
Worker Boundary
        ↓
Blender Worker
```

Users do not operate Blender. Blender does not define Asset identity, Mapping,
Compatibility meaning, Retarget Policy, QC policy, Derived Variant identity,
or versioning.

```text
Resolve Character + Motion + Source Skeleton
Acquire Skeleton Summaries
Automatic Mapping / Compatibility preflight
Ready → Transfer  (or explanation / Mapping confirmation)
Freeze Retarget Policy + Job Spec
Isolated Blender worker: import → evaluate → retarget → bake
Worker result envelope
Structural validation / QC interpretation
Register Preview Artifact
Create Derived Variant version
Optional Export Artifact only when required
```

Blender-internal steps (add constraint, operator names, datablock types) are
not product states.

## Ownership

| Capability | RigForge | Blender worker |
| --- | --- | --- |
| Asset identity, versions, relationships | owns | may extract observations |
| Skeleton Summary definition / provenance | owns portable requirements | extracts candidate facts |
| Accepted Bone Mapping | owns | translates a projection; does not author truth |
| Compatibility meaning | owns | reports capability / measurements |
| Retarget Policy intent | owns | executes supported mechanics |
| Job orchestration, retries, cancellation | owns | one process executes one Job Spec |
| QC meaning, severity, interpretation | owns | may compute execution-local measurements |
| Preview orchestration | owns | may generate candidate artifact bytes |
| Viewer | owns (engine-independent) | none |
| Mesh/skin/curve/evaluator copies | not required in V1 | import / evaluate / bake in ephemeral scene |

Source bytes remain external representations. Blender scene state is
disposable.

## Worker contract

The Job Spec is versioned and backend-neutral. It carries immutable Asset
Version references and hashes, Source Skeleton Reference, Skeleton Summary
versions, accepted Mapping version, Retarget Policy version, expected
capability profile, artifact settings, isolation limits, and a declared
determinism context.

The worker returns backend/build version, verified hashes, phase outcomes,
the Mapping projection actually applied, bake facts, QC measurements with
units/spaces, structured diagnostics and declared losses, staged artifact
hashes, and a terminal status.

**Worker success alone cannot create a Derived Variant.** The product accepts
outputs only after validating the envelope.

Replaceability tests:

1. Job Spec contains no `bpy` type, Blender datablock name as stable identity,
   Blender operator as product policy, or `.blend` path as the only source
   identity.
2. Mapping is translated into a worker projection.
3. Policy describes intent and required capability, not implementation steps.
4. Result/QC envelopes use backend-neutral meanings.
5. Preview and export outputs are artifacts, not scene authority.
6. A worker may reject unsupported policy before mutation.

A future worker must implement this contract without pretending to be Blender.

## Process controls (from POC-BLENDER-E2E-01)

Enough to distinguish success from failure and to prevent automatic
publication of partial output:

```text
one isolated process per job
pinned official Blender build
background mode
factory startup / isolated user configuration
automatic embedded-file script execution disabled
explicit Python exception exit code
structured stdout/result + captured log file
job-specific TEMP / workspace
staged output not published after non-zero/error result
no reuse of mutable interactive scenes
```

Timeout/recovery campaigns, pooling, resource scheduling, upgrade rehearsal,
and complete packaging qualification are later V1 hardening.

## Preview

```text
Authoritative Product Object
        ↓
Preview Generator   (Blender is an acceptable first candidate, replaceable)
        ↓
Preview Artifact
        ↓
binding / integrity validation
        ↓
engine-independent Viewer
```

POC-PREVIEW-01R validated this derived Preview path with guards
(`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`). Preview Artifact is **DERIVED /
REBUILDABLE / NON-AUTHORITATIVE**. Viewer runtime is not Blender. Blender is
not a permanent Preview Generator. Derived GLB is a candidate, not a frozen
choice. Viewer library remains OPEN. Preview must remain usable without
Blender at view time. Unreal and Unity are not required.

## Reconciliation with historical DCC-independence

W0.1–W0.4 said a DCC should not be a mandatory ingest dependency. V1 reopens
that **operational** rule:

> Product semantics must not depend on Blender's data model.
> V1 may depend operationally on a pinned hidden Blender worker for the
> first execution path, provided the worker boundary is replaceable and
> limitations are explicit.

Long-term DCC independence survives as a boundary property, not as a
requirement to ship multiple backends in V1.

## Risks (not closed by this document)

Version pinning, headless gaps, Python API churn, startup cost, crash and
temp-file isolation, reproducibility, untrusted `.blend` scripts, importer
loss, packaging size, GPL/worker-script obligations, process-level
concurrency, and IK/bake determinism remain open. Process isolation is an
engineering boundary, not a legal conclusion. A material blocker can still
reject Blender.

## What this does not decide

Viewer technology, GLB, database, Core language, native ufbx inspector, native
glTF stack, multiple Export Profiles, and multiple execution backends are not
selected here.
