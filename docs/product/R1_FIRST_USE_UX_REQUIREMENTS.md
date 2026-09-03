# R1 First-Use UX Requirements

Post-V1 product requirements for first-use hardening.

This is not a V1 stage, not V1-9, and not a Gate D reopening.

Status: `R1-0 PASS / DESIGN COMPLETE / BASELINED`

Current implementation lifecycle:

```text
R1-1 First-Use Ingest Experience COMPLETE / PASS / BASELINED
R1-2 Responsive Execution Experience COMPLETE / PASS / BASELINED
R1-V Integrated Validation + Human UAT FINDINGS / awaiting re-UAT
R1-FIX implementation complete candidate
R1 Gate NOT STARTED
```

Historical R1-0 design decomposition: `R1-A / R1-B / R1-C`.
Current implementation batching: `R1-1 / R1-2 / R1-V / optional R1-FIX / R1 Gate`.

V1 technical baseline remains `COMPLETE`. Gate D remains `PASS / CLOSED`.
Independent Gate D result remains `GATE_D_PASS_CANDIDATE`.

## Frozen Product requirement

```text
A first-time user with one supported already-rigged Character FBX
and one supported Motion FBX must be able to produce and preview a
Derived Variant without manually typing filesystem paths, skeleton
names, clip identifiers, frame ranges, or FPS values whenever
RigForge can determine them unambiguously.
```

R1 does not expand the core Retarget feature set. It removes first-use
friction that currently exposes implementation details as required typing.

## Convenience vs Product truth

```text
machine can determine reliably
→ automatic

multiple choices already supported by an exact durable Product binding
→ ask user

ambiguity cannot currently be durably bound through Product provenance
→ fail closed with a clear diagnostic

NEVER silently guess
```

Ease of use must not invent Product authority the V1 model cannot preserve.
Do not add a durable Product subobject-selection schema in R1.

## What “unambiguous” means

| Observation | Required UX |
| --- | --- |
| Exactly one usable Skeleton (Armature) | auto-select |
| Two or more usable Skeletons | **FAIL CLOSED** (no UI picker in R1) |
| Zero usable Skeletons | reject with diagnostic |
| Exactly one usable animation candidate **strongly associated with that unique Armature** | auto-select |
| Two or more usable animation candidates **strongly associated with that unique Armature** | explicit user selection |
| Zero usable strongly associated animation candidates on that Armature | reject with diagnostic |
| File digest changed after inspection | invalidate; inspect again |
| Non-integral Action frame endpoints | do not silently round; fail that clip candidate closed |
| Non-integral `fps_base` | convert with exact `Fraction(str(fps_base))` (not rejection) |
| Timing that cannot be a reduced positive `u32`/`u32` rational | fail that timing/clip closed; no approximation |

A Motion clip is usable only when RigForge proves a **strong** association
with the unique Armature: direct Action assignment, NLA assignment on that
Armature, or pose-channel evidence (`pose.bones[...]`). Slot type
suitability (`action_suitable_slots`, `ActionSlot.target_id_type`,
`Action.id_root`) is **not** eligibility.

Multi-Armature explicit selection is **POST-R1 / separately authorized**, not
a first-use blocker. Multi-Clip selection on one unique Armature **is** R1
mandatory.

Automatic Mapping remains a proposal. Explicit Mapping acceptance remains
required. Automatic file/metadata convenience is not Mapping acceptance.

## First-use happy path (conceptual)

```text
Add Character
  → Browse
  → select .fbx
  → display name derived from filename (editable)
  → inspect (no typed path)
  → unique usable Skeleton required (2+ fail closed)
  → Add Character
        ↓
Add Motion
  → Browse
  → select .fbx
  → display name derived from filename (editable)
  → inspect once
  → unique usable Skeleton required (2+ fail closed)
  → unique clip auto-select, or choose among **strongly associated** clips on that Armature
  → Add Motion
        ↓
select Character + Motion
  → Propose Mapping
  → Accept Mapping
  → Evaluate Compatibility
  → acknowledge warnings if required
  → Transfer (UI remains responsive)
  → Preview exact Published Derived Variant
```

The user is not told skeleton names, clip IDs, frame ranges, or FPS in
advance. Those values are discovered, selected, or rejected.

## Required first-use diagnostics

Primary UX must use human-readable Product-facing text when a known case
matches. Preserve detailed technical errors for logs / secondary evidence.

| Situation | Primary message |
| --- | --- |
| No armature / zero joints | No skeleton found in this FBX. |
| No usable animation candidates | No animation clips were found. |
| Two or more usable skeletons | Multiple usable skeletons were detected in this FBX. R1 currently requires one unambiguous skeleton per source file. |
| Two or more clips on the unique skeleton | Multiple animation clips were found. Choose one clip. |
| Observed timing cannot be represented as reduced u32/u32 FPS | This Motion source has timing that RigForge cannot represent exactly. |
| Digest/size changed after inspect | This file changed after it was inspected. Inspect it again. |
| Unsupported pair | This Character cannot currently be transferred with this Motion. |
| Transfer did not publish | Transfer failed. No Derived Variant was published. |
| Unsupported format | This file is not a supported FBX source. |
| Missing file | The selected file is missing. |

Do not show a raw Rust `AppError` as the only first-use message when a
row above applies.

## File selection

Normal path is native Browse, not a typed filesystem string.

| Element | Requirement |
| --- | --- |
| Browse / Change | native file dialog |
| File label | filename only |
| Full path | tooltip / secondary, not the typing surface |
| Filter | FBX |
| Cancel | leave the previous selection unchanged |
| Display name | derived from filename stem; remains editable |
| Drag/drop | not R1-1 acceptance; optional later polish |

Manual path typing is not the normal user path.

## Long operations

Operations expected to take materially longer than a normal UI frame must
not block the egui event loop.

This reclassifies the V1 accepted limitation “native Transfer waits on the
UI thread” as:

```text
R1 USER-FACING RELEASE UX ISSUE
```

It is not a Product-authority defect.

Visible Running / Success / Failure is required. Cancel, automatic retry,
and worker pools are not R1 default requirements.

## V1 invariants R1 must preserve

```text
Product identity != path/digest
automatic Mapping != accepted Mapping
Compatibility exact graph
RetargetPolicy exact selection
Worker SUCCESS != publication
QC Product-owned / non-mutating
PersistenceVerification required
Derived Variant exact versions
Preview derived / rebuildable / non-authoritative
Blender not Product authority
engine independence
V1-7 remains SKIPPED / OPTIONAL
```

## Exclusions

R1 must not start:

- Auto-Rig or automatic skinning
- a new Retarget solver
- advanced artistic QC
- multi-DCC backends
- Unity / Unreal integration
- V1-7 export
- worker pool / automatic retry
- a general animation editor
- full Mapping visualization redesign
- batch multi-clip import unless later UAT proves it necessary
- installer / public redistribution work

Native file picking and Motion discovery are ingest UX, not export.

## Related

- [R1_ROADMAP.md](../development/R1_ROADMAP.md)
- [R1_UX_ARCHITECTURE.md](../architecture/R1_UX_ARCHITECTURE.md)
- [R1_UAT_PLAN.md](../development/R1_UAT_PLAN.md)
