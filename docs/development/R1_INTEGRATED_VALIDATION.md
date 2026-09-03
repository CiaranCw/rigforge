# R1 Integrated Validation

R1-V record. Not a Gate D reopening and not an independent R1 Gate audit.

Status: `FINDINGS / awaiting re-UAT`

R1-FIX is authorized and implemented as a candidate. Do not mark R1-V PASS.
Do not start R1 Gate until Human re-UAT of this same R1-V stage.

## Current lifecycle

```text
R1-0: PASS / BASELINED
R1-1: PASS / BASELINED
R1-2: PASS / BASELINED
R1-V: FINDINGS / awaiting re-UAT
R1-FIX: implementation complete candidate
R1 Gate: NOT STARTED
```

## Baseline

```text
branch: main
HEAD at R1-V start: 88933b702bcfcda2c71332192e2abb1e2c72a04d
origin/main at R1-V start: 88933b702bcfcda2c71332192e2abb1e2c72a04d
message: feat: make workbench transfer responsive
```

Automated source review, full regression, and real backend path were completed
on that SHA before Human UAT.

## Human UAT finding — R1-V-MAJOR-001

```text
R1-V-MAJOR-001:
CONFIRMED BY HUMAN UAT

classification: MAJOR

reproduction:
Mapping proposal showed 21 mapped entries and 55 unmapped entries.
The right-side workflow placed Mapping lists before Accept Mapping,
Evaluate Compatibility, and Transfer. CentralPanel had no vertical
ScrollArea. Lower workflow controls were clipped. The user could not
scroll to them. The ordinary first-use path could not continue.

root cause:
Workbench CentralPanel rendered the full Mapping joint lists before
the next workflow actions and did not wrap the central workflow in a
vertical ScrollArea.

R1-FIX:
ACTIVE / implementation complete candidate
```

Related Human UAT confusion (corrected in the same R1-FIX, presentation
only):

```text
Selection tray showed raw Character/Derived UUIDs.
Selected Motion was not prominent.
Clip selection in Add Motion was easy to confuse with a registered
Motion Product version.
```

## New user-authorized requirement

```text
Chinese-first Workbench UI
```

This is now authoritative for the first user-facing R1 experience.
Chinese is the default user-facing language. Internal Product identity,
protocol, logs, and exact clip_identity remain unchanged.

## R1-FIX presentation corrections (candidate)

```text
central workflow: vertical ScrollArea
left asset/ingest area: vertical ScrollArea
Mapping: summary-first; joint rows in collapsed 映射详情
workflow controls remain above detailed joint lists
selection tray: friendly Character / Motion / clip / Derived names
raw Product IDs: collapsed 技术详情
Add Motion: clip picker distinct from 添加到资产库
CJK system-font fallback: Microsoft YaHei / YaHei UI via WINDIR\Fonts
  (no vendored font file; no hard-coded architectural font path)
```

Product semantics are unchanged: Mapping acceptance, Compatibility
authority, Transfer authorization, WorkerResult, QC,
PersistenceVerification, publication, and Preview/ADR-0006.

Do not mark R1-V PASS from this correction. Human re-UAT is required.

## Source-review result

### R1-1 first-use ingest (production GUI)

Ordinary Add Character / Add Motion path:

```text
Browse (rfd FBX picker) → filename label + tooltip path
→ filename-derived editable Display name
→ background Blender inspect (workbench/src/ingest.rs)
→ unique usable Armature fail-closed
→ Add Character / Add Motion
```

Motion extras, all presentation of inspection evidence:

```text
Source Skeleton label from unique candidate
usable-clip ComboBox when 2+
observed Frames a–b · n/d FPS as labels
```

Confirmed **not** required as typed fields on the ordinary UI:

```text
filesystem path
Source Skeleton name
exact clip ID
start frame / end frame
FPS numerator / denominator
```

Display name is the only ordinary text field, and Browse auto-fills it from
the filename stem.

Stale inspect protection: `character_request_id` / `motion_request_id` must
match before applying a background result.

### R1-1 authority

```text
path/digest: SourceArtifactEvidence only
Product IDs: CharacterAsset / MotionAsset / SourceSkeletonReference generated
source inspection: transient CharacterSourceInspection / MotionSourceInspection
Workbench: does not construct TimeDomainProvenance
Application.register_*_from_inspection: inspection-bound registration
TOCTOU: require_live_source_matches + digest/size recheck before persist
```

Armature:

```text
0 usable → reject
1 usable → auto-select
2+ usable → fail closed
no max-bones / first / latest selection on inspect_source or inspect_skeleton
```

`inspect_skeleton` uses `require_unique_usable_armature`. Diagnostics state
that silent max-bones selection is forbidden.

### Motion association

Production `worker.py`:

```text
direct_action: strong
nla_strip: strong
pose_channels: strong only if pose_channels_qualifies(resolved, unresolved)
               == bool(resolved) and not unresolved
slot_suitable / slot target type: weak_notes only; not eligibility
```

`pose.bones["alien_root"]` against Armature bones `{root, spine}` does not
qualify (`unresolved_pose_bones=["alien_root", ...]`, `qualifies=False`).
Substring `pose.bones[` is not enough. Mixed resolved+unresolved fails closed.

### Timing

```text
effective FPS = scene.render.fps / scene.render.fps_base
rational_fps uses Fraction(str(fps_base))
30/1 → 30/1
30/1.001 → 30000/1001
24/1.001 → 24000/1001
fractional frame endpoints → unusable_reason=fractional_frames, fail closed
silent rounding: NO (integral_frame requires number.is_integer())
```

Frozen UAL2 inspect on this SHA: `fps=30/1`, integral frames.

### R1-2 Transfer path

Ordinary Workbench UI (`Transfer` button / `on_transfer_action`):

```text
start_responsive_transfer
→ start_transfer (JobRun)
→ mark_dispatchable + BlenderWorker::production + dispatch (RUNNING)
→ background WorkerCompletionPort::collect
→ UI Application::apply_terminal_outcome
→ ingest_worker_success_candidate
→ background inspect_durable_persistence_artifact
→ UI bind_qc_from_evidence
→ background reopen_durable_persistence_artifact
→ UI bind_verification_from_outcomes
→ UI complete_publication_decision → complete_finalize
```

`complete_native_transfer` remains a blocking helper. `workbench/src/lib.rs`
does not call it. UI `update` polls `poll_long_op`.

### Thread / authority

```text
Application / SQLite: WorkbenchHost.app, UI thread only
background: Blender wait, inspect, QC, reopen; no Application/Catalog
second Application::open during live Transfer: not used
BlenderWorker: Send asserted in blender-worker/tests/r1_2_send.rs
unsafe Product sharing: none in production crates
```

### Terminal / QC / publication

`apply_terminal_outcome` persists Catalog correlation:

```text
Success → complete_success (JobSpec, attempt_id, worker_execution_ref)
mismatch → fail closed to FAILED, no fabricated evidence
Failed + valid WorkerResult → preserved
terminal persist exactly once (second apply InvalidTransition)
```

Workbench does not decide WorkerResult validity.

Publication still requires QC Pass + PersistenceVerification Pass inside
`complete_finalize`. Worker SUCCESS is not publication.

Exact Derived version: `apply_transfer_outcome` binds
`outcome.derived_variant_version_id`. Preview Derived uses
`selected_derived_variant_version`. Pair switching is locked while Transfer
is active.

### Responsive UX (source)

```text
one active long mutation
Transfer / Browse / Add / Propose / Accept / Evaluate / pair select: disabled
spinner + TransferPhase.user_label + elapsed MM:SS
no percentage
```

Phases: Launching Blender… / Running Transfer… / Validating result… /
Checking persisted result… / Publishing Derived Variant… /
Transfer complete / Transfer failed.

### Adjacent still-synchronous Blender (not R1-2 Transfer)

These remain UI-thread waits. They are **not** classified MAJOR/MINOR until
Human UAT shows they block first-use:

```text
Propose Mapping → BlenderSkeletonInspector inspect
Preview Character / Motion / Derived → generate_preview
```

## Test result

R1-V automated baseline on 88933b7:

```text
command: cargo test --offline --color=never -- --test-threads=1
baseline: 550
current total: 550
PASS: 550
FAIL: 0
IGNORED: 0
```

R1-FIX regression:

```text
command: cargo test --offline --color=never -- --test-threads=1
previous: 550
current total: 560
PASS: 560
FAIL: 0
IGNORED: 0
```

Added focused Workbench presentation tests (friendly selection labels,
Chinese workflow/error/progress labels, CJK font discovery). Product
semantic tests remain.

## Real backend result

Frozen local pair, pinned Blender 5.2.1, no download.

Discovery (`r1_real_inspect`, `--nocapture`):

```text
Knight_Male.fbx: usable_armature_count=1, skeleton=CharacterArmature, 32 joints
UAL2_Standard.fbx: usable_armature_count=1, skeleton=Armature, 65 joints
FPS: 30/1
strongly-associated usable clips: 43
Walk_Carry_Loop present: Armature|Armature|Walk_Carry_Loop
  label=Walk Carry Loop  frames=1–61  kind=pose_channels
Zombie_Walk_Fwd_Loop also present (UAT-2 can use the same file)
no first/latest auto-pick in GUI when usable_count > 1
```

Full technical path (`gate_d_registration_e2e`, 25.68s on this run):

```text
registration: PASS (fresh Catalog, not pre-seeded)
Mapping: accepted
Compatibility: Ready / ReadyWithWarnings as evaluated then acknowledged if required
worker: SUCCESS
QC: Pass
PersistenceVerification: Pass (fresh_reopen Pass)
publication: Published
exact DerivedVariantVersion: 01a065f3-f1cc-7be1-9433-f31906e57f37
Preview: valid for that exact version (selected_derived_preview_version_id match)
```

This technical path used `on_transfer_action` + `drive_transfer_to_terminal`
(async Transfer). It does **not** substitute for Human UAT.

## Human UAT environment

First Human UAT used:

```text
fresh Catalog: F:\NewResearch\rigforge_w0p_work\r1_v_uat\catalog.sqlite
runtime root: F:\NewResearch\rigforge_w0p_work\r1_v_uat\runtime-root
pinned Blender: junction to blender-5.2.1-windows-x64
launch: F:\NewResearch\rigforge_w0p_work\r1_v_uat\launch_workbench.cmd
build: main 88933b7 via cargo run -p rigforge_workbench
assets (do not redownload):
  Knight_Male.fbx
  UAL2_Standard.fbx
```

That environment reproduced R1-V-MAJOR-001. Do not reuse its Catalog for
re-UAT.

Re-UAT environment (fresh Catalog, same frozen assets):

```text
fresh Catalog: F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\catalog.sqlite
  (absent until first Workbench open; not a production user Catalog)
runtime root: F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\runtime-root
pinned Blender: junction to blender-5.2.1-windows-x64
launch: F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\launch_workbench.cmd
assets (do not redownload):
  F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\Knight_Male.fbx
  F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\UAL2_Standard.fbx
```

## Human UAT observations

First Human UAT reproduced R1-V-MAJOR-001 (Mapping list overflow / no
central scroll; Accept Mapping / Compatibility / Transfer unreachable).
R1-FIX is the correction candidate. Re-UAT has not yet accepted it.

```text
fresh Catalog v1: human-opened; MAJOR-001 reproduced
Character Browse: OBSERVED (first-use continued until Mapping overflow)
manual Character path typing: NOT REQUIRED
Motion Browse: OBSERVED
manual Skeleton typing: NOT REQUIRED
manual clip-ID typing: NOT REQUIRED
manual frame/FPS typing: NOT REQUIRED
multi-Clip selector: OBSERVED
Mapping review/accept: BLOCKED by R1-V-MAJOR-001
Compatibility: BLOCKED by R1-V-MAJOR-001
Transfer UI responsive: NOT REACHED
Published Derived: NOT REACHED
Preview: NOT REACHED
```

## Human UAT-1 checklist

Completed against 88933b7 and reproduced R1-V-MAJOR-001.

## Human re-UAT checklist

Same R1-V stage after R1-FIX. Use the v2 launcher only.

1. Run `F:\NewResearch\rigforge_w0p_work\r1_v_uat_v2\launch_workbench.cmd`.
2. Confirm Chinese UI text renders without missing-glyph boxes.
3. Add Character → 浏览 → select `Knight_Male.fbx` only. Do not type a path.
4. Add Motion → 浏览 → select `UAL2_Standard.fbx` only.
   Choose a clip from the friendly list, then 添加到资产库.
5. Confirm 当前选择 shows friendly Character / Motion / clip names, not UUIDs.
6. At 1100×720 and at a shorter window height, confirm the user can reach
   生成映射, 接受映射, 检查兼容性, 警告确认, 执行迁移, and 预览 by scrolling.
7. 生成映射 → read the compact summary → 接受映射 → 检查兼容性.
   If warnings, acknowledge before 执行迁移.
8. Press 执行迁移. While it runs: move/resize the window, scroll, read
   spinner / phase / 已用时. Confirm a second Transfer cannot start.
9. When finished, confirm 迁移完成 / 已发布 without SQLite.
10. 预览派生资产. State whether animation is visibly playing.

Record confusion, misclicks, unexpected typed fields, freeze, errors.

## Finding inventory

```text
OPEN MAJOR: 0
OPEN MINOR: 0
CONFIRMED MAJOR (R1-FIX candidate): 1
OBSERVATION: 4
```

| ID | Note |
| --- | --- |
| R1-V-MAJOR-001 | Mapping list overflow blocked Accept/Compatibility/Transfer. CONFIRMED BY HUMAN UAT. R1-FIX candidate: central/left scroll + summary-first Mapping + Chinese-first UI. |
| R1-V-OBS-001 | Propose Mapping still waits on Blender on the UI thread. |
| R1-V-OBS-002 | Preview generation still waits on Blender on the UI thread. |
| R1-V-OBS-003 | Historical: selection tray showed raw Product version IDs. Addressed in R1-FIX presentation. |
| R1-V-OBS-004 | UAL2 presents 43 strongly associated clips in one ComboBox. |

## R1-FIX decision

```text
R1-FIX: AUTHORIZED by R1-V-MAJOR-001
status: implementation complete candidate
Chinese-first Workbench UI: user-authorized, included in this R1-FIX
```

Do not mark R1-V PASS. Do not mark R1 Gate READY.

## R1-V disposition

```text
R1_V_FINDINGS_AWAITING_REUAT
```

Resume the same R1-V stage with one Human re-UAT after this R1-FIX.
