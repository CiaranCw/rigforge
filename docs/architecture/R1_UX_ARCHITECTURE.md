# R1 UX Architecture

First-use ingest UX and responsive long operations.

Status: `R1-0 PASS / DESIGN COMPLETE / BASELINED`

Current implementation lifecycle:

```text
R1-1 First-Use Ingest Experience COMPLETE / PASS / BASELINED
R1-2 Responsive Execution Experience READY / NOT STARTED
R1-V Integrated Validation + Human UAT NOT STARTED
R1-FIX CONDITIONAL
R1 Gate NOT STARTED
```

Historical R1-0 design decomposition: `R1-A / R1-B / R1-C`.
Current implementation batching: `R1-1 / R1-2 / R1-V / optional R1-FIX / R1 Gate`.
The R1-A / R1-B / R1-C headings below remain the accepted design decomposition.
They are not the current execution sequence.

This document is post-V1 design. It does not change V1 Product authority.
V1 remains `COMPLETE`. Gate D remains `PASS / CLOSED`.

Requirements: [R1_FIRST_USE_UX_REQUIREMENTS.md](../product/R1_FIRST_USE_UX_REQUIREMENTS.md).
Roadmap: [R1_ROADMAP.md](../development/R1_ROADMAP.md).
UAT: [R1_UAT_PLAN.md](../development/R1_UAT_PLAN.md).
File-dialog selection: [ADR-0007](decisions/ADR-0007-r1-native-file-dialog.md) (`Accepted`; `rfd` 0.17.2 is added in R1-1).

## Current V1 UX (source-confirmed)

Native Host is `WorkbenchHost` (`workbench/src/lib.rs`): one `Application`
plus one `WorkbenchApp`. `WorkbenchHost::open_default` opens exactly one
Catalog (`RIGFORGE_CATALOG` or `./rigforge-catalog.sqlite`) and therefore
runs V1-8 `reconcile_interrupted_runtime` once at startup.

### Character / Motion registration

Current collapsing panels (V1 baseline this design was written against) required typed fields:

- Character: display name, local path
- Motion: display name, local path, Source Skeleton name, clip identifier,
  start frame, end frame, FPS numerator, FPS denominator

`on_register_character_clicked` / `on_register_motion_clicked` historically
called `Application::register_local_*` with `BlenderSkeletonInspector::production()`.
R1-1 Workbench uses `register_*_from_inspection` after background
`inspect_source`. `register_local_*` remains for tests and fixtures.
That inspector launches an isolated Blender `inspect` process and waits on
`Command::output()` (`blender-worker/src/inspect.rs`). The wait currently
runs on the egui thread.

`register_local_motion` constructs `TimeDomainProvenance` from caller-supplied
clip identity and `TimePoint::frames`. Paths and SHA-256 are
`SourceArtifactEvidence` only. Product IDs remain generated Domain IDs.

V1 worker `inspect_skeleton` (`blender-worker/python/worker.py`) imported FBX with
`use_anim=False`. If several armatures existed, it silently selected
`max(..., key=len(bones))`.

That silent max-bones choice is forbidden on every R1 production inspect
path. R1 does not add a durable Product Armature-subobject schema. Files
with two or more usable Armatures **fail closed**.

### Transfer / Preview

`on_transfer_action` calls `request_transfer` (exact Compatibility graph vs
current Character/Motion) then `native_exec::complete_native_transfer`:

```text
mark_dispatchable
  → dispatch (spawn Blender, persist RUNNING, DispatchReceipt)
  → collect (wait Child + envelope)
  → finalize_transfer (ingest, QC, PersistenceVerification, publish)
```

`WorkerPort::dispatch_resolved` is launch-only. `WorkerCompletionPort::collect`
waits and **does not** mark JobRun state. `SqliteCatalog::collect` currently
wraps that wait and then persists `SUCCEEDED / FAILED`. R1-C reuses the
existing completion port on a background thread; it does not redesign
Catalog orchestration.

Preview remains Application-owned payload + local host. Changing Character
or Motion still invalidates pair-bound workflow state (Gate D MAJOR-002).
Published Transfer still binds the exact returned Derived version (Gate D
MINOR-003).

## File picker (R1-A)

Accepted dependency: `rfd` **0.17.2** (MIT). Do not add it in R1-0.
See ADR-0007. Linux XDG portal / Zenity is release-packaging, not an
R1-A functional blocker.

Workbench owns the dialog. Application never opens a file picker and never
treats a path as Product identity.

Normal flow:

```text
Browse / Change
  → rfd::FileDialog (FBX filter)
  → Some(path): store selected path, show filename label, tooltip = full path
  → None (cancel): keep previous selection
  → if display name is empty or still the previous auto-derived stem,
    replace it with the new filename stem
```

There is no path text field on the first-use surface. Cancel must not clear
a prior valid selection.

`FileDialog::pick_file` is synchronous and modal at the OS. That is
user-modal Browse, not a Blender-length wait. It may run on the UI/main
thread. Do not introduce Tokio for the picker.

## Inspection boundary (R1-B)

### Names

Do not treat Blender Action or Armature names as Product IDs.

Recommended Application-facing names (subject to R1-0 review):

| Name | Role |
| --- | --- |
| `SourceInspectionProvider` | backend-neutral inspect trait |
| `BlenderSourceInspector` | V1 adapter implementation |
| `CharacterSourceInspection` | transient Character evidence |
| `MotionSourceInspection` | transient Motion evidence |

`MotionSourceInspector` / `MotionMetadataProvider` are rejected as the
trait name: “metadata” sounds like Product authority, and Armature
ambiguity is fail-closed in R1 rather than a Motion-only picker.

`SkeletonEvidenceProvider` remains the V1 Mapping inspect port. It still
does **not** accept a source-local Armature selection. R1 therefore cannot
promise that a UI Armature picker would be honored by later Mapping /
`SkeletonSummary` / execute. Multi-Armature picking is POST-R1.

Registration may reuse discovered joints from `inspect_source` and must not
launch a second Blender inspect when the digest still matches.

### Unique-Armature invariant (R1-0-MAJOR-001)

R1 minimal rule:

```text
usable Armatures (type ARMATURE and joint_count > 0):

0
→ reject / “No skeleton found in this FBX.”

1
→ auto-select

2+
→ FAIL CLOSED
   “Multiple usable skeletons were detected in this FBX.
    R1 currently requires one unambiguous skeleton per source file.”
```

Never silently pick max bones, first, or lexical first.

Durable Product subobject-selection schema: **not added**.

### Which inspector (A + narrow B)

```text
A. inspect_source is the R1 registration/discovery route and requires
   exactly one usable Armature.

B-narrow. The historical inspect_skeleton max-bones line is also replaced
   with the same 0/1/2+ fail-closed rule when R1-B lands, so Mapping’s
   existing SkeletonEvidenceProvider path cannot silently choose.
```

Do not rewrite the historical V1 inspect operation in R1-0 (design only).
R1-B implements both so **no R1 production path** still runs
`max(armatures, key=len(bones))`.

Mapping after registration (`propose_and_store_mapping_for_selection`):

```text
Application.inspect_and_store_source_summary
Application.inspect_and_store_character_summary
  → SkeletonEvidenceProvider.inspect_*
  → BlenderSkeletonInspector.inspect_path
  → worker mode inspect (today: inspect_skeleton)
```

Invariant after R1 registration:

- a persisted Character/Motion came from a file with exactly one usable
  Armature at inspect time
- later inspect uses the same path + digest; digest mismatch fail-closes
- with B-narrow, even a 2+ Armature file cannot be silently reduced

`all bpy.data.actions` is **not** the Motion clip list.

### Transient structs (smallest Domain-aligned schema)

These are **not** Domain records and **not** Catalog rows.

```text
CharacterSourceInspection
  source_path
  source_digest
  size_bytes
  observed_media_type
  usable_armature_count
  skeleton_candidates[]          0 or 1 usable after R1 filter
  producer
  diagnostics[]

MotionSourceInspection
  source_path
  source_digest
  size_bytes
  observed_media_type
  usable_armature_count
  skeleton_candidates[]          0 or 1 usable after R1 filter
  animation_candidates[]         strongly associated clips only
  timing_context                 ObservedTimingContext { fps_num, fps_den }
  producer
  diagnostics[]

SkeletonCandidate
  source_local_key               armature object name; evidence, not Product ID
  display_name
  joint_count
  joints[]

AnimationCandidate
  clip_identity                  exact TimeDomainProvenance.clip_identity_evidence
  display_label
  source_skeleton_local_key      the unique usable Armature’s source_local_key
  association_kind               direct_action | nla_strip | pose_channels
  association_evidence           transient diagnostic/evidence only
  start_frame                    i64 integral, or candidate unusable
  end_frame
  fps_num                        u32 reduced
  fps_den                        u32 reduced
  duration_presentation          display only
  usable
  unusable_reason
```

`source_skeleton_local_key` and `association_evidence` are transient
inspection evidence, not Product identity. Do not introduce a durable
Product subobject ID.

`association_kind` is the strongest proven kind (order:
`direct_action` > `nla_strip` > `pose_channels`). Additional proven kinds
and slot identifiers belong in `association_evidence`.

`slot_suitable` is **not** an `association_kind` and does **not** qualify
a candidate.

Workbench must not list camera, material, mesh/object-only, or
other-imported Actions as Motion choices merely because they exist in
`bpy.data.actions`.

Conceptual `association_evidence` (transient, not Product identity):

```text
kinds_present[]                  subset of {direct_action, nla_strip, pose_channels}
assigned_slot_identifier         if direct_action and slotted
nla_track_name / nla_strip_name
nla_slot_identifier              if the strip carries a slot
pose_slot_identifier             slot whose channelbag supplied pose.bones
pose_data_path_samples[]         a few pose.bones[...] paths
resolved_pose_bones[]            exact bone keys present on the unique Armature
unresolved_pose_bones[]          exact extracted keys absent from that Armature
weak_notes[]                     target_id_type / id_root / suitable_slot ids
                                 diagnostic only; never eligibility
```

Workbench may hold a `MotionInspectionSelection`:

```text
bound_digest
bound_size
clip_identity                 required if 2+ usable clips; implied if exactly one
```

There is no Armature picker field in R1. Unique Armature is implied.

Workbench does not invent `TimePoint` numerics. Application copies exact
frames/FPS from the matching clip candidate after a live digest check.

### Deterministic clip rules (unique Armature only)

Usable clips are **strongly associated** with the unique usable Armature
(see association rules below). Weak slot-type suitability is not enough.

```text
strongly associated usable clips:

0  → reject / “No animation clips were found.”
1  → auto-select
2+ → explicit user selection
```

Never pick first, latest, or lexicographic clip.

The selected `clip_identity` flows **unchanged** into
`TimeDomainProvenance::clip_identity_evidence`.

### Animation candidate association (Blender 5.2.1)

Pin: official Blender 5.2.1 LTS (ADR-0005).

Official Action Slot facts (`OFFICIAL_DOC`, Blender 5.2 Manual
`animation/actions.html` and `bpy.types.ActionSlot` / `AnimData` /
`NlaStrip`):

- An animated data-block specifies **both** an Action **and** a slot.
- Slots organize animation inside an Action. They are **not** an
  intrinsic exact binding to one scene data-block. The Manual states
  slots “don’t have any intrinsic attachment to anything in the scene.”
- `AnimData.action_suitable_slots` / `NlaStrip.action_suitable_slots`
  list slots whose `target_id_type` is compatible with the owner. That is
  type-organization filtering, not proof that the Action belongs to the
  unique source Armature.
- `NlaStrip.action` and `NlaStrip.action_slot` exist. Direct assignment
  uses `AnimData.action` and `AnimData.action_slot`.

Current execute `assign_action` assigns `ad.action` and, if slots exist,
`ad.action_slot = slots[0]` (`SOURCE_CONFIRMED` in `worker.py`). That
first-slot assignment is execution convenience for an **already chosen**
clip. It is not a discovery rule and must not define the clip list.

Frozen:

```text
slot_suitable alone
→ NOT AN ELIGIBILITY PROOF
```

The same freeze applies to `ActionSlot.target_id_type` /
`action_suitable_slots` / legacy `Action.id_root` used by themselves.

R1 discovery (`RIGFORGE_INFERENCE` of the inspect walk; confirm on the
pinned 5.2.1 worker in R1-B) exposes an Action as a Motion candidate
**only** when at least one **strong** association with the unique usable
Armature Object is proven.

#### Strong kinds (eligibility)

**A. `direct_action`**

```text
armature.animation_data is present
and armature.animation_data.action == action
```

If the Action uses slots, retain the **actually assigned** slot in
`association_evidence`:

```text
animation_data.action_slot
  identifier / handle   (exact association, not slots[0] guess)
```

Do not invent a slot by taking `slots[0]` or the first suitable slot.

**B. `nla_strip`**

A track/strip on that same Armature Object’s `animation_data.nla_tracks`
explicitly references the Action (`strip.type == CLIP` and
`strip.action == action`). Nested `META` strips are walked.

If the NLA strip carries slot identity, retain it:

```text
strip.action_slot
  identifier / handle
```

`strip.action_suitable_slots` is not an NLA assignment.

**C. `pose_channels`**

The Action, or an exact Action slot / channelbag, contains animation
channels whose data paths demonstrably target `pose.bones[...]` in an
Armature animation context.

For slotted Actions, inspect the FCurves of the **relevant**
channelbag(s), not a blind legacy `Action.fcurves` dump:

```text
relevant slot:
  assigned action_slot if kind A
  strip.action_slot if kind B
  otherwise each slot that actually has a channelbag

channelbag:
  action.layers[0].strips[0].channelbag(slot)   (5.2 layered Action)

pose evidence:
  extract the exact quoted bone key from pose.bones["..."] / pose.bones['...']
  that key ∈ the unique usable Armature's exact bone-name set
  at least one pose-bone path must resolve
  every pose-bone target used as pose_channels proof must resolve
  mixed resolved + unresolved pose-bone paths: pose_channels NOT eligible
  substring "pose.bones[" alone is not eligibility
```

If the Action has no slots / empty layers, a legacy `action.fcurves`
walk is allowed **only** as the equivalent channel set for that Action,
and still requires extracted pose-bone keys that resolve on the unique
Armature. R1-B confirms which representation FBX import actually produces
on the pinned 5.2.1 worker.

Because R1 already fail-closes files with two or more usable Armatures,
`pose_channels` as a fallback is interpreted against the unique Armature
bone-name set. An Action whose pose-bone channels resolve only to bones
absent from that Armature is not eligible. Direct Action assignment and
exact Armature-owned NLA strips remain independently eligible even when
auxiliary pose-channel evidence does not qualify. It does not license
scanning every Action in `bpy.data.actions` that might animate a camera,
material, or mesh.

#### Weak evidence (not eligibility)

These MAY appear in `association_evidence` or diagnostics **after** a
strong kind is proven, or as compatibility/filter notes. They MUST NOT,
by themselves, make an Action visible as a Motion clip:

```text
ActionSlot.target_id_type
AnimData.action_suitable_slots
NlaStrip.action_suitable_slots
legacy Action.id_root
```

Do not add an Action merely because it exists in `bpy.data.actions`.
Do not expose camera, material, mesh/object-only, or unrelated imported
Actions.

Restore any temporary `animation_data` mutation before the inspect
process exits. Inspection remains non-authoritative.

### Timing (R1-0-MAJOR-002)

Preserve rational frame provenance:

```text
TimePoint::frames(start, fps_num, fps_den)
TimePoint::frames(end, fps_num, fps_den)
SamplingInterpretation::BakedEverySourceFrame
```

Do not convert Product truth to floating-point-only values.
Do not silently round fractional **frame points**.
Do not confuse fractional FPS with fractional frame endpoints.

Blender effective FPS (`OFFICIAL_DOC` / `SOURCE_CONFIRMED` in execute):

```text
effective_fps = scene.render.fps / scene.render.fps_base
```

`fps` is an integer. `fps_base` may be a non-integer such as `1.001`.
That is legitimate. Convert it. Do **not** reject solely because
`fps_base` is non-integral.

Worker-side conversion: deterministic exact decimal-string
rationalization. Frozen **forbidden** techniques:

```text
short decimal heuristic
decimal-length threshold
limit_denominator
floating-point rounding
tolerance-based approximation
Fraction(float) binary-mantissa reconstruction
```

Conceptual algorithm:

```python
from fractions import Fraction
import math

U32_MAX = 2**32 - 1

def rational_fps(fps, fps_base) -> tuple[int, int]:
    fps_i = int(fps)
    if fps_i <= 0:
        raise ValueError("scene.render.fps must be > 0")
    if isinstance(fps_base, bool) or not isinstance(fps_base, (int, float)):
        raise ValueError("scene.render.fps_base is not a real number")
    if not math.isfinite(fps_base) or fps_base <= 0:
        raise ValueError("scene.render.fps_base must be finite and > 0")
    try:
        base = Fraction(str(fps_base))
    except (ValueError, ZeroDivisionError) as exc:
        raise ValueError(
            f"str(fps_base) did not parse as Fraction: {fps_base!r}"
        ) from exc
    if base <= 0:
        raise ValueError("fps_base Fraction must be > 0")
    effective = Fraction(fps_i, 1) / base  # Fraction reduces automatically
    num, den = effective.numerator, effective.denominator
    if num <= 0 or den <= 0:
        raise ValueError("effective FPS must be a positive rational")
    if num > U32_MAX or den > U32_MAX:
        raise ValueError("reduced FPS exceeds Product u32/u32")
    return int(num), int(den)
```

A timing context is usable only if all of the following hold:

```text
scene.render.fps > 0
scene.render.fps_base is finite and > 0
str(fps_base) parses deterministically as Fraction
effective numerator > 0
effective denominator > 0
reduced numerator fits u32
reduced denominator fits u32
```

Otherwise the timing / clip is **unusable** with a clear diagnostic.
No approximation. No silent rounding.

`Fraction(str(...))` uses the decimal representation of the observed
value, not IEEE Product truth. Required test vectors:

```text
fps=30  fps_base=1      → 30 / 1
fps=30  fps_base=1.001  → 30000 / 1001
fps=24  fps_base=1.001  → 24000 / 1001
```

Do not confuse:

```text
fractional FPS:
  supported through rational provenance (example 30000/1001)

fractional Action frame endpoint:
  unsupported by the current Blender execution contract
  (TimePoint::frames / integral worker frames)
  → that candidate is unusable
```

Duration is presentation-only from the reduced rational FPS. It is not
Product time.

Current execute `find_action` has endswith fallbacks. Discovery still
records the **exact** `bpy.data.actions[].name`. R1 must not invent
`Armature|Armature|…` strings unless that exact name is present.

## Blender discovery (one pass)

Add a worker mode conceptually named `inspect_source`. Envelope kind:
`source_observation`. Not a `WorkerResult`. Not publication authority.

```text
read_factory_settings(use_empty=True)
import_scene.fbx(use_anim=True, automatic_bone_orientation=False)
count usable Armatures (fail closed if not exactly one)
observe Actions with a strong association to that unique Armature
  (direct_action | nla_strip | pose_channels; never slot_suitable alone)
observe scene.render.fps / fps_base → rational_fps
sha256 the source file (already checked against expected_digest)
```

One isolated Blender process per inspect. Do not launch Skeleton inspect
and Action inspect separately.

Character and Motion both use this observation. Character registration
ignores animation candidates. Motion registration requires a usable clip
on the unique Armature.

Do not keep `max(armatures, key=len(bones))` on `inspect_source` **or** on
the R1-B-hardened `inspect_skeleton`.

`expected_digest` remains mandatory, matching today’s inspect path.

## Registration integration

Decision: **C, smallest Product-safe split** — keep the V1 constructors,
add inspection-bound wrappers.

```text
A. register_discovered_motion(...)     rejected as a second Product constructor
B. replace register_local_motion       rejected; tests and fixtures still need exact fields
C. inspect_* + register_*_from_inspection wrapping the existing constructors
```

Application remains Product authority.

```text
Workbench
  → Browse (rfd)
  → Application.inspect_character_source / inspect_motion_source
       (background Blender wait; returns transient inspection)
  → user confirms name + clip choice if 2+ usable clips
  → Application.register_character_from_inspection
  → Application.register_motion_from_inspection
       1. sha256 live file
       2. fail if digest or size ≠ inspection
       3. fail if usable_armature_count ≠ 1
       4. resolve selected AnimationCandidate (Motion)
       5. call existing register_local_* internals
            SourceArtifactEvidence
            CharacterAsset / MotionAsset + Published version
            SourceSkeletonReference::new(unique Armature display_name)
            TimeDomainProvenance from exact clip candidate fields
            SkeletonSummary from the unique Armature joints (no second Blender if digest matches)
            Catalog atomic persist
```

Workbench must not construct Domain time records from typed numbers on the
first-use path.

Keep `register_local_character` / `register_local_motion` for tests that
supply exact fields without discovery.

Do not reuse an existing `SourceSkeletonReference` by display-name lookup.
V1 persist already creates a new reference per Motion registration. Silent
name matching would be a guess.

## TOCTOU / digest binding

```text
select file
  → inspect (bound_digest, bound_size, bound_path)
  → user waits / chooses clip
  → Add
  → registration re-hashes bound_path
```

| Check | Result |
| --- | --- |
| file missing | fail; inspect again |
| digest ≠ inspection | invalidate inspection; “This file changed after it was inspected. Inspect it again.” |
| size ≠ inspection | same as digest mismatch |
| path equal, digest equal | proceed |
| path changed, same bytes elsewhere | do not chase; user must Browse again |

Do not treat path equality as sufficient. Digest is the binding.

Invalidate Workbench inspection state on mismatch. Do not persist old
clip/FPS against new bytes.

## Responsive operations (R1-C)

### What must not block egui

| Operation | UI-thread vs background | Why |
| --- | --- | --- |
| Native file dialog | UI-thread, OS-modal | user is in the dialog; short and expected |
| Character Blender inspect | background wait | isolated Blender process |
| Motion Blender discover | background wait | isolated Blender process |
| Catalog persist of already-inspected registration | UI-thread | SQLite; Application authority; short |
| Transfer `start_transfer` + `mark_dispatchable` + `dispatch` | UI-thread | Catalog writes + spawn only; launch-only |
| Transfer execute wait (`WorkerCompletionPort::collect`) | **background** | existing wait port; no JobRun persist |
| Transfer QC inspect wait (`inspect_durable_persistence_artifact`) | **background** | free function; no Catalog |
| Persistence reopen wait (`reopen_durable_persistence_artifact`) | **background** | free function; no Catalog |
| `complete_success` / terminal failure / `bind_qc` / publish | UI-thread | Application + SQLite only |
| Mapping propose if it launches inspect | background wait | same Blender class as discovery |

SQLite `rusqlite::Connection` is not a second-thread Application. All
authoritative Catalog mutations stay on the UI-thread `Application`.

### Forbidden

```text
thread::spawn(move || complete_native_transfer(&mut app))
Application::open(...) during a live Transfer
a second WorkbenchHost / Catalog connection on the same file
Tokio as a general runtime
worker pool
automatic retry
Cancel as an R1-0/R1-C default
```

`complete_native_transfer` today holds `&mut Application` across `collect`.
Moving that function onto a thread would either move Application off the UI
or require a second open. Both are rejected.

### Non-blocking Transfer ownership (R1-0-MINOR-001)

Do **not** begin R1-C by redesigning Catalog orchestration.
`WorkerCompletionPort::collect(receipt)` already means: wait for the
launched worker; **do not** mark JobRun state. `Catalog::collect` is only
that wait plus persist.

Keep **one** `Application` on `WorkbenchHost` for the process lifetime.

`BlenderWorker` must be `Send`. R1-C implementation **must** compile-assert:

```rust
fn assert_send<T: Send>() {}
assert_send::<BlenderWorker>();
```

Do not assume `Send`. If the assertion fails: **STOP R1-C** and return to
architecture review. Do not hide Application/worker behind unsafe shared
state.

```text
UI thread (Product authority)
  WorkbenchHost.app: Application
  WorkbenchHost.shell: WorkbenchApp
  WorkbenchHost.long_op: Option<LongOp>

  1. request_transfer (exact Compatibility vs current selection)
  2. start_transfer → JobSpec + JobRun QUEUED
  3. mark_dispatchable
  4. construct BlenderWorker::production() on the host (not inside Application)
  5. app.dispatch(run_id, &mut worker)
       writes dispatch intent
       spawn Child
       persist RUNNING + worker_execution_ref
       clear intent
       obtain DispatchReceipt
  6. move BlenderWorker + DispatchReceipt onto std::thread
  7. UI remains in update(); request_repaint while long_op is Some

Background thread (execution wait only)
  WorkerCompletionPort::collect(&receipt) → TerminalOutcome
  capture exact staged artifact path if Success (BlenderWorker::last_staged_blend)
  do NOT open SQLite
  do NOT call Application
  do NOT call Application::open
  send WaitMsg::Execute { run_id, TerminalOutcome, staged_path }

UI thread
  TerminalOutcome::Success → Application.complete_success
      (existing JobSpec / attempt_id / worker_execution_ref / WorkerResult correlation)
  TerminalOutcome::Failed → Application.complete_failure / catalog terminal-failure binding
  if SUCCEEDED: candidate ingest (existing finalize ingest)
```

Do not call `Catalog::collect` after the background wait (that would wait
again). Reuse the persist half that `Catalog::collect` already uses:
`complete_success` / `complete_terminal_failure`.

### QC / reopen (existing free functions)

Today `finalize_transfer` is UI-synchronous:

```text
ingest candidate
  → inspect_durable_persistence_artifact  (Blender wait)
  → bind_qc_from_evidence
  → reopen_durable_persistence_artifact   (Blender wait)
  → bind_verification_from_outcomes
  → publish
```

`inspect_durable_persistence_artifact` and
`reopen_durable_persistence_artifact` (`app/src/pinned_qc.rs`) already take
path + digest + mapping projection and do **not** open Catalog. R1-C splits
only the wait vs bind:

```text
UI:
  candidate ingest
  prepare exact immutable QC request
    { artifact_path, expected_sha256, mapping projection copy }

BACKGROUND:
  inspect_durable_persistence_artifact(...)
  (pin + worker-package integrity remain mandatory)

UI:
  bind_qc_from_evidence

UI:
  prepare exact immutable reopen request
    { artifact_path, expected_sha256, mapping projection copy }

BACKGROUND:
  reopen_durable_persistence_artifact(...)

UI:
  bind_verification_from_outcomes
  → publish
  apply_transfer_outcome (exact Derived version → Preview selection)
```

Required:

```text
Application: UI thread only
SQLite: UI thread only
Blender-backed waits: background
second Application::open: FORBIDDEN
```

Do not weaken worker/package integrity verification.

### Discovery / registration responsiveness

```text
file picker: UI thread modal
Blender source inspection: background (no Catalog)
Product registration from already inspected evidence: UI / Application
TOCTOU digest/size check: mandatory before persist
```

### Proof vs V1-8 restart reconciliation

`SqliteCatalog::open` / `open_in_memory` call
`reconcile_interrupted_runtime`:

- `DISPATCHABLE` + leftover intent → durable `FAILED`
- leftover `RUNNING` → durable `FAILED` (no reattachment)

R1-C is safe if and only if:

```text
the live Transfer’s Application/Catalog connection remains the only open
Catalog for that file, and the background thread never calls Application::open
```

Then:

- leftover `RUNNING` is **not** reconciled while the Host is alive
- the in-process Child wait is the retained supervision
- if the user quits during wait, the next `open_default` correctly fail-closes
  RUNNING (V1-8 remains true)

A second `Application::open` on the same sqlite file during a live RUNNING
job would mark that job `FAILED` and must not be introduced.

### Concurrency policy

```text
one active long-running mutation operation per Workbench
```

While a long op is active:

| Interaction | Allowed? |
| --- | --- |
| Asset Browser list / select for browsing | yes |
| Preview of already-valid local payloads | yes |
| second Transfer | no |
| Add Character / Add Motion / inspect | no |
| Propose / Accept / Evaluate | no |
| selection change that starts another mutation | no |

JobSpec for the running Transfer is already exact. Browsing other list
rows must not dispatch a second worker. Disable Transfer and registration
buttons while `long_op` is `Some`.

### Background primitive

Prefer:

```text
std::thread
std::sync::mpsc
egui::Context::request_repaint()
```

No Tokio. No worker pool.

### Progress UX

No fake percentages.

Allowed: spinner, elapsed time, truthful coarse phase.

Transfer phases (only when a real event exists):

```text
Preparing      start_transfer / mark_dispatchable
Launching      dispatch returned RUNNING
Running        background execute wait
Validating     QC inspect started / bound
Publishing     PersistenceVerification / publish
Complete       Published Derived bound into Preview
Failed         no Derived published; durable failure visible
```

If an implementation slice can only prove execute wait, show **Running**
until QC/reopen are also evented. Do not label QC as Publishing without a
QC event.

Discovery/registration may use:

```text
Inspecting
Complete
Failed
```

### Cancel / retry / pool

```text
non-blocking UI: YES
visible Running / Success / Failure: YES
Cancel: NO DEFAULT REQUIREMENT
automatic retry: NO
worker pool: NO
```

R1-UAT may later promote Cancel if observed durations make it necessary.
That would be R1-D, not R1-0.

## Error UX

Map known cases to [R1_FIRST_USE_UX_REQUIREMENTS.md](../product/R1_FIRST_USE_UX_REQUIREMENTS.md)
primary strings. Keep `err.to_string()` as secondary technical detail, not
the headline.

Compatibility `Unsupported` uses “This Character cannot currently be
transferred with this Motion.” PublicationDenied uses “Transfer failed.
No Derived Variant was published.”

## V1 regression risks

| Risk | Guard |
| --- | --- |
| Path/digest become Product IDs | IDs still generated; path/digest remain evidence |
| Workbench authors time provenance | Application copies exact discovered candidate fields after digest check |
| Silent armature/clip pick | 0/1 fail-closed Armatures; 0/1/2+ **strongly associated** clips; no max-bones; `slot_suitable` is not eligibility |
| Rounded frames | fractional frame endpoints unusable; FPS uses exact `Fraction(str(fps_base))`, not `limit_denominator` or a short-decimal heuristic |
| Second Application during Transfer | forbidden; Host retains the live Catalog |
| Fake WorkerResult | collect persist still requires correlated Validated\<WorkerResult\> |
| Preview as Product | unchanged ADR-0006 |
| V1-7 export via file picker | picker is ingest only |
| Mapping auto-accept | Propose ≠ Accept unchanged |
| Compatibility not exact | request_transfer graph check unchanged |

## R1-A / R1-B / R1-C implementation boundaries

Historical R1-0 design decomposition. Current implementation batching maps:

```text
old R1-A + old R1-B  →  R1-1
old R1-C Transfer/QC/reopen responsiveness  →  R1-2
```

Do not implement these in R1-0. Expected modules after design acceptance:

### R1-A — Native Asset Selection

```text
workbench/Cargo.toml                 add rfd 0.17.2 (Workbench only)
workbench/src/lib.rs                 Browse/Change, filename label, tooltip
workbench/src/file_pick.rs           optional thin wrapper around rfd::FileDialog
workbench/tests/shell.rs             cancel / filter / name-derivation tests
```

Linux XDG portal / Zenity is a **release-packaging** consideration, not an
R1-A functional blocker on Windows.

### R1-B — Motion / Character discovery

```text
blender-worker/python/worker.py      inspect_source; inspect_skeleton 2+ fail-closed
blender-worker/src/inspect.rs        BlenderSourceInspector; no max-bones
app/src/registration.rs              inspect_* + register_*_from_inspection
app/src/inspection.rs                optional transient structs (not Domain records)
domain/                              no new Product subobject schema
workbench/src/lib.rs                 clip selector when 2+ usable clips
app/tests/registration.rs
blender-worker/tests/                unique Armature; strong association kinds;
                                     slot_suitable does not qualify;
                                     FPS 30/1, 30/1.001, 24/1.001
```

### R1-C — Responsive long operations

```text
workbench/src/native_exec.rs         dispatch on UI; collect on moved worker
workbench/src/long_op.rs             optional mpsc + LongOp state
workbench/src/lib.rs                 WorkbenchHost retains sole Application
blender-worker/tests or unit         assert_send::<BlenderWorker>()
app/src/pinned_qc.rs                 called from background; bind stays on Application
workbench/tests/shell.rs             Transfer button disabled while long_op
```

Do not add Tokio. Do not open a second Application.

## Implementation freeze for R1-0

Do not modify `domain/`, production `app/src`, production `workbench/src`,
production `blender-worker`, or Cargo dependencies in R1-0.
