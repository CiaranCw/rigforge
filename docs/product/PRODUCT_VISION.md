# Product Vision

RigForge is a **Character Animation Asset Workbench**.

Users organize and preview rigged Character and Motion assets, select one of
each, run Transfer, and receive a previewable, versioned Derived Variant.

Decision record: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
V1 boundary: [V1_SCOPE.md](V1_SCOPE.md).

## Problem

Animation work is still a file-copy problem. A rigged Character and a Motion
from a different Skeleton do not become a managed, previewable, versioned
result without leaving the workbench for a DCC or an engine.

Formats, DCCs, and engines do not share one product contract for:

- Character / Motion identity and versions
- source Skeleton context
- Mapping and Compatibility
- Transfer policy and QC
- Derived Variant provenance
- engine-independent Preview

## Product

```text
Asset Browser
    select Character  (already rigged)
    select Motion     (with source Skeleton context)
        ↓
automatic Mapping / Compatibility preflight
        ↓
Ready  →  Transfer
        ↓
Derived Variant
        ↓
Preview / Version
```

Mapping and Compatibility use **progressive disclosure**. Ordinary happy-path
users do not enter a Mapping editor. Export is an optional derived artifact,
not the defining V1 transaction.

## Objects users work with

- **Character Asset** — mesh + skeleton/armature + skin weights. Already
  rigged. Analogous at the product level to a skeletal mesh asset; not an
  Unreal type and not an Auto-Rig request.
- **Motion Asset** — animation data + source Skeleton reference/context +
  time-domain provenance. Several Motions may share one Source Skeleton.
- **Derived Variant** — first-class result with provenance (Character,
  Motion, Source Skeleton, Mapping, Retarget Policy, Validation, Preview,
  generation metadata). Not “another copy of the whole character file.”
- **Preview** — rebuildable, derived, non-authoritative. Engine-independent
  derived Preview path is `VALIDATED_WITH_GUARDS` by POC-PREVIEW-01R.
- **Version** — Character, Motion, Mapping, policy, and Derived Variant are
  versioned.

## Explicit boundaries

```text
Character inputs are already rigged / skinned.
RigForge is not an Auto-Rig product.
Blender is not product authority.
Engine independence remains.
Formats are transport / boundary concerns.
```

Unreal, Unity, Blender, Maya, and peers are not Canonical authority. Blender
is a hidden V1 execution backend (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`,
validated by POC-BLENDER-E2E-01). It is not product authority and is not
permanently final. Engine-independent derived Preview is
`VALIDATED_WITH_GUARDS` by POC-PREVIEW-01R
(`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`). Preview Artifact remains
derived / rebuildable / non-authoritative. Viewer library and Preview payload
format are not selected. Preview must not require Unreal, Unity, or Blender
as the user-facing viewer. Product implementation remains unauthorized until
W0-RS and IA-1 complete the remaining required gates.

## What survives from earlier research

Mapping, Compatibility, Retarget Policy, QC, authoritative-vs-derived
separation, names-as-evidence, and engine-independent Preview remain product
capabilities. The earlier heavy Canonical Character / Skeleton / Motion
runtime is not the current V1 product definition. Historical findings:
[../research/R1_RESEARCH_BASELINE.md](../research/R1_RESEARCH_BASELINE.md).
