# POC-BLENDER-E2E-01 — Blender-backed Character + Motion → Derived Variant

**Status:** `COMPLETE / PASS / BASELINED`

**Date:** 2026-08-31

**Classification:** RESEARCH ONLY / W0-P / NON-PRODUCTION

**Decision impact:** `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`

This is a W0 research/architecture validation result. Product implementation
remains unauthorized until POC-PREVIEW-01R, W0-RS, and IA-1 complete the
remaining required gates.

---

## Accepted closeout

External focused review accepted this PoC.

```text
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_BLENDER_BACKEND_WITH_GUARDS
```

All findings are CLOSED:

```text
BLENDER-R1-MAJOR-001 inspect reproducibility
BLENDER-R1-MAJOR-002 policy execution / loop closure
BLENDER-R1-CLOSEOUT-003 reopen-before-publication
BLENDER-R2-MAJOR-001 quaternion Policy / execution alignment
BLENDER-R2-CLOSEOUT-002 publication-order provenance
```

Rev1 and Rev2 sections below remain historical correction records.

---

## Rev2 closeout

External review of Rev1 remained **REVIEW_PENDING**. Two remaining findings
were closed on the same frozen pair and the same Blender 5.2.1 LTS pin
(`9e2066aef7ef`). Rev1 inspect / per-frame rest reset / loop-closure /
reopen-before-publish gates were **not** reopened; they still PASS.

| Finding | Closure |
| --- | --- |
| BLENDER-R2-MAJOR-001 quaternion Policy / execution mismatch | **FIXED** |
| BLENDER-R2-CLOSEOUT-002 publication-order provenance wording | **FIXED** |

Frozen Retarget Policy quaternion semantics are now backend-neutral and
match worker execution:

```text
normalization: NORMALIZE_BEFORE_KEY
continuity:    CONSECUTIVE_HEMISPHERE
interpolation: backend interpolation after normalized
               hemisphere-consistent key values
```

The Blender Adapter **reads and validates** those modes. Unsupported modes
are `FAIL / REPORT`. The Adapter implements the policy **before** key
insertion. Interpolation remains backend execution detail. Policy text does
not name PoseBone, FCurve, or bpy.

`quaternion_policy_audit.json` (run 1):

```text
keys/samples checked: 1403
sign flips applied:   317
max |norm-1| after normalize: 1.19209e-07
minimum consecutive dot:      0.97618
status: PASS
```

QC `quaternion_policy_execution`: PASS.

Publication metadata now matches the implemented harness:

```text
worker -> reopen PASS -> QC PASS -> publish
```

`published: true` is written only after those three gates.

Hashes (expected to change vs Rev1 because Policy changed):

```text
Mapping SHA:            683603627bbe431a8438c8b6bacca4d6779046e4ef09ede7fa3218c41e8a602b
Retarget Policy SHA:    43eee4b181ffad617b270eeb8b076e650e39c4e2d7ed8425918d825c9cab1b8b
Job Spec semantic hash: bc953822b6498bf3f87c8ca40ddf03365e920adff359a84a291bb27cffc42d80
run semantic hash 3/3:  f2d650ec8a79f47a30bf485af58321e17258daac2756a30b4f12c83dd3aeaf82
```

Frozen Mapping unchanged and still valid against regenerated inspect.

Candidate decision is unchanged: `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`.

---

## Rev1 closeout

External review of the first package was **NOT PASS YET**. Three findings were
closed on the same frozen pair (Knight_Male + UAL2 `Walk_Carry_Loop`) and the
same Blender 5.2.1 LTS pin (`9e2066aef7ef`).

| Finding | Closure |
| --- | --- |
| BLENDER-R1-MAJOR-001 inspect reproducibility | **FIXED** |
| BLENDER-R1-MAJOR-002 policy execution / loop closure | **FIXED** |
| BLENDER-R1-CLOSEOUT-003 publication ordering | **FIXED** |

Inspect path (repository-controlled):

```text
python experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py
        ↓
blender_worker.py mode=inspect
        ↓
knight_blender_inspect.json
ual2_blender_inspect.json
        ↓
Skeleton Summaries / difference / Motion Summary
```

No hidden external inspect script. Inspect JSON is regenerated each harness
invocation (no cache). Frozen Mapping remained valid and unchanged.

Policy execution (tolerance 0.01 rad / 0.001 loc):

```text
desired-vs-baked: PASS (max error 0.0 rad)
loop closure:     PASS (source, desired, baked, reopen endpoints)
ROTATION_ONLY:    PASS (non-root location FCurves absent; pose location 0)
root policy:      PASS (source and target endpoint delta both 0.0)
```

Per-frame solve resets target pose to rest before applying the current source
frame. Non-root bones set `rotation_quaternion` only. Keys are written after
independent per-frame solves.

Publication: `published: true` is written only after reopen verification PASS.
`derived_variant.json` records `reopen_verification_sha256` and
`publication_order`. Reopen samples feed loop-closure QC, so the implemented
order is worker → reopen → QC (including reopen) → publish.

Repeatability (3 clean execute processes after one regenerated inspect):

```text
1c9d191f6a11f129323c3443e9a734d667f36e2b489e1e74b6fa9bc934136c91
3/3 consistent
```

Candidate decision is unchanged: `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`.

---

This is not product implementation, not a production Blender worker, not a
generic retarget library, not Auto-Mapping, not Preview product code, and
not a release-qualification campaign.

```text
Python outside Blender + Blender Python inside the worker
!=
RigForge Core language = Python
```

Core language remains **NOT SELECTED**. POC-CORE-01 remains `INCONCLUSIVE`.

---

## 1. Question

> Can Blender serve as a hidden execution backend for the real RigForge
> Character + Motion → Derived Variant workflow while durable RigForge
> product semantics remain backend-neutral?

Success requires **both**:

```text
A. Blender can perform the necessary real transfer/bake work.
B. RigForge does not need Blender concepts to define durable product truth.
```

A successful retarget alone is insufficient.

---

## 2. Architecture Under Test

```text
RigForge Workbench
        ↓
Thin Workflow Domain
        ↓
backend-neutral Job Spec
        ↓
Worker Boundary
        ↓
Blender Worker
        ↓
backend-neutral Result Envelope
        ↓
RigForge validation
        ↓
Derived Variant
```

Blender V1 execution backend: `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`,
validated by this PoC. Not product authority. Not permanently final.

Allowed outcomes: `ACCEPT_BLENDER_BACKEND` |
`ACCEPT_BLENDER_BACKEND_WITH_GUARDS` | `REJECT_BLENDER_BACKEND` |
`INCONCLUSIVE`.

---

## 3. Environment / Blender Pin

| Field | Value |
| --- | --- |
| Host | Windows 11 10.0.26100 |
| Harness Python | 3.13.12 (Anaconda) — **not** a Core language selection |
| Blender | **5.2.1 LTS** |
| Build hash | `9e2066aef7ef` |
| Build date | 2026-08-25 |
| Branch | `blender-v5.2-release` |
| Archive | `blender-5.2.1-windows-x64.zip` (404851964 bytes) |
| Official URL | `https://download.blender.org/release/Blender5.2/blender-5.2.1-windows-x64.zip` |
| Archive SHA-256 (official = actual) | `0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c` |
| Retrieval | 2026-08-31; official distribution; extracted outside git |
| Layout | portable Windows x64 zip extract |

Complete `blender --version` is in external `environment.txt`.

Command shape:

```text
blender.exe
  --background
  --factory-startup
  --disable-autoexec
  --python-exit-code 1
  --python blender_worker.py
  --
  <mode> <job-spec.json>
```

Per-run isolation: `TEMP` / `TMP` / `TMPDIR` and `BLENDER_USER_*` redirected
into the run workspace. No user startup, no add-ons, no marketplace retarget
plugins.

Approximate cold start + execute (run 1): 10.7 s. Not a performance target.

---

## 4. Asset Selection

### Character A

| Field | Value |
| --- | --- |
| Package | Ultimate Animated Character Pack - Nov 2019 |
| File | `Knight_Male.fbx` |
| Source | Quaternius; reused POC-FBX-01 external extract |
| License | CC0 1.0 Universal (packaged `License.txt`) |
| Size | 2270172 |
| SHA-256 | `fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f` |
| Why | already hashed, license-researched, structurally inspected |

Exact expected bytes were present. Not copied into git.

### Motion B

| Field | Value |
| --- | --- |
| Package | Universal Animation Library 2 [Standard] |
| File | `Unity\UAL2_Standard.fbx` |
| Source | Quaternius; already-local zip inventoried 2026-08-31 |
| License | CC0 1.0 Universal (packaged `License.txt`) |
| Zip size / SHA-256 | 18735003 / `4008ea208a604773a2b2177d965f0f5d3195498b5bf838c3f5785d68e95f2a68` |
| FBX size / SHA-256 | 24778332 / `d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d` |
| Clip | `Armature|Armature|Walk_Carry_Loop` (frames 1–61, 30 fps) |
| Source Skeleton context | YES (same FBX; 65-bone armature) |

**LEGAL CLEARANCE:** CC0 packaged licenses for both packs.

**SEMANTIC SUITABILITY:** independent of license. Standard pack inspect
listed no plain `Walk` take (only `Walk_Carry_Loop` and
`Zombie_Walk_Fwd_Loop`). Frozen clip is `Walk_Carry_Loop`. This is an
asset-pack limitation, not a silent product substitution.

Knight's own Walk was **not** used as Motion B (same Skeleton identity).

---

## 5. Skeleton Difference

Independent summaries (backend-neutral joint id/parent/deform + rest
observation). Fingerprint = SHA-256 of `{skeleton_id, joints[{id,parent,deforming}]}`.

| | Target A (Knight) | Source B (UAL2 Standard) |
| --- | --- | --- |
| Joint count | 32 | 65 |
| Root count | 1 (`Bone`) | 1 (`root`) |
| Vocabulary | Blender `.L/.R`, `Hips`, `Torso`, `Bone` | Unreal `pelvis`, `thigh_l`, `spine_01`, fingers |
| Name overlap | `Head` only | |
| Fingerprint | `4ede9fa38982f6a5e2659e0c514412eca4e8445695b9b9c7a58b9332dae6b3a8` | `d939383270bc872fbc3d94158313446641d8b0492745ba6c3551b6db2eff2184` |
| Manufactured by renaming | NO | NO |

Material differences: bone-name vocabulary, bone count, hierarchy (Knight
`Foot.*` parents to `Bone`, not `LowerLeg`; UAL2 has a full finger set).

The pair is **not** the same Skeleton identity with another clip.

---

## 6. Thin Skeleton / Motion Observations

Summaries are research observations, not Canonical Authority.

Rest/base pose: imported armature rest observation (Knight after pose-clear;
UAL2 rest from `A_TPose` at the first source frame for transfer).

```text
bind_pose: UNKNOWN
inverse_bind: NOT_OBSERVED
geometry_bind: NOT_REQUIRED_FOR_THIS_POC
```

These were **not** collapsed into one fabricated rest matrix.

Motion observation for B:

| Field | Value |
| --- | --- |
| clip | `Armature|Armature|Walk_Carry_Loop` |
| Source Skeleton | `UAL2_Standard_Armature` |
| fps / range | 30 fps; frames 1–61 |
| duration | 2.0 s |
| root motion | Standard library file is non-RM; observed root span 0 |
| missing channels | unmapped target joints remain at rest; required must resolve |

Motion data stayed in the source asset / Blender execution scene. No full
Canonical Motion representation.

---

## 7. Frozen Mapping

File: `experiments/w0p/poc_blender_e2e_01/config/mapping_frozen.json`

| Field | Value |
| --- | --- |
| Mapping count | 23 entries |
| Required | 21 |
| Optional | 2 (`ball_*` → `Foot.*_end`) |
| Helpers ignored | UAL2 fingers + `ball_leaf`; Knight `PoleTarget.*` and several `*_end` |
| Root / pelvis | `root`→`Bone`; `pelvis`→`Body` |
| Ambiguities | Foot IK-style parent; 3-spine onto Hips/Abdomen/Torso by chain order; no plain Walk take |
| SHA-256 | `683603627bbe431a8438c8b6bacca4d6779046e4ef09ede7fa3218c41e8a602b` |

Reviewed and frozen before the decision run. Worker does not invent missing
required correspondences (`FAIL / REPORT`).

Compatibility preflight: `READY` (hash match, skeletons present, required
entries resolve, time domain valid, policy representable, worker capability
sufficient for this slice).

---

## 8. Frozen Retarget Policy

File: `experiments/w0p/poc_blender_e2e_01/config/retarget_policy.json`

Pair-specific. Not universal.

| Topic | Frozen statement |
| --- | --- |
| Rest alignment | `REST_RELATIVE_WORLD_DELTA`; source rest = UAL2 `A_TPose`; target rest = imported pose-cleared rest |
| Root / trajectory | copy rest-relative world translation on `Bone`; Standard clip may be ~0 |
| Translation | non-root `ROTATION_ONLY` |
| Rotation | rest-relative world rotation delta |
| Scale | keep target rest scale |
| Twist / helper | ignore unmapped |
| IK | `use: false` |
| Bake | every source frame 1–61 at 30 fps; quaternion keys |
| Time domain | source clip authority |
| Missing channels | unmapped stay at rest; missing required = fail |
| Quaternion | `NORMALIZE_BEFORE_KEY` + `CONSECUTIVE_HEMISPHERE`; interpolation is backend detail after those keys |

Policy SHA-256: `43eee4b181ffad617b270eeb8b076e650e39c4e2d7ed8425918d825c9cab1b8b`

RigForge frozen Policy requires normalized, hemisphere-consistent consecutive
quaternion keys. The Blender Adapter implements that policy before key
insertion. Actual interpolation remains backend execution detail for this
PoC. This is product-owned policy, not Blender API semantics.

---

## 9. Job Spec

Durable semantic Job Spec (hashed) carries identities/hashes, expected
capabilities, and a declared determinism context (Blender 5.2.1 /
`9e2066aef7ef` as **execution pin**, not product bpy types).

Job Spec semantic hash:
`bc953822b6498bf3f87c8ca40ddf03365e920adff359a84a291bb27cffc42d80`

Worker invocation copies add local filesystem paths. Those are harness
bindings, not durable product types.

Forbidden durable fields (`bpy`, PoseBone, constraint class names) are not
present as product semantics.

```text
Blender product semantics leaked: NO
Job Spec backend-neutral: YES
```

Capability tokens such as `fbx_import` / `glb_export` name **worker
capabilities**, not Blender operator identifiers as product truth.

---

## 10. Worker Execution

Harness: `experiments/w0p/poc_blender_e2e_01/scripts/`

Adapter-internal method (not Job Spec semantics):

1. Factory empty scene; import Character (anim off) and Motion (anim on).
2. Classify armatures by bone-name vocabulary (adapter).
3. Mute pose constraints if present (0 on this Knight import).
4. Capture target rest after pose-clear; source rest from `A_TPose`.
5. Resolve frozen mapping; fail if a required joint is missing.
6. Per frame: rest-reset, rest-relative world delta onto pose bones (parent
   flush). After independent per-frame solves, apply frozen quaternion
   Policy (`NORMALIZE_BEFORE_KEY` + `CONSECUTIVE_HEMISPHERE`) then key
   quaternion (and root location).
7. Assign Blender 5.2 action slot; sample; strip source; save `.blend`;
   export candidate GLB; render research stills; emit envelope.

No Auto-Rig Pro / Rokoko / Mixamo. No substantial standalone RigForge
retarget runtime.

---

## 11. Result Envelope

Decision-run (run 1) envelope status: `SUCCESS`.

| Field | Value |
| --- | --- |
| Worker | `poc-blender-e2e-01.blender_worker` v1 |
| Backend identity | `blender` (envelope field; not product Character identity) |
| Clip observed | `Armature|Armature|Walk_Carry_Loop` |
| Baked action | `CharacterArmatureAction` |
| Time | frames 1–61, 30 fps, 2.0 s |
| NaN/Inf desired matrices | 0 |
| Errors | none |
| Worker result SHA-256 (run_1 file) | `0c055c16e4fa3a8ec5d4a1a123fc489eea68997448515eaf82f0401d729917d7` |

Transient Blender object names appear only under `adapter_diagnostics`.

Worker success did **not** publish a Derived Variant by itself.

---

## 12. Structural QC

QC is measurement/verdict only. `mutates_subject: false`.

| Check | Result |
| --- | --- |
| no NaN / Inf in sampled target | PASS |
| all required mapped bones resolve | PASS |
| baked target animation exists | PASS |
| duration/time domain sane | PASS (2.0 s) |
| target animation non-degenerate | PASS (max sample location span ≈ 1.174) |
| no gross invalid scale | PASS |
| root trajectory follows policy | PASS (root span 0; non-RM + declared copy) |
| representative mapped bones move when source moves | PASS |

QC status: **PASS**. Only then was a Derived Variant manifest written.

---

## 13. Motion Evidence

Six normalized times. Representative joints: source
`root` / `pelvis` / `spine_03` / `upperarm_l` / `thigh_l` / `Head`;
target `Bone` / `Body` / `Torso` / `UpperArm.L` / `UpperLeg.L` / `Head`.

Source motion exists (e.g. source max sample location span ≈ 0.131).
Target motion exists and is not rest-frozen (span ≈ 1.174). Scales ~1.
No equality oracle across Skeletons.

Example target `Head` world location:

| u | frame | location |
| --- | --- | --- |
| 0.0 | 1 | (-0.085, -0.601, 1.168) |
| 0.4 | 25 | (see `motion_samples_target.json`) |
| 1.0 | 61 | (-0.020, 0.011, 2.168) |

Source `Head` at u=0 matches u=1 (loop). Target `Head` does not. Class:
pair-specific rotation-only rest-relative copy on a different hierarchy —
**RETARGET POLICY / pair**, not an architecture leak.

---

## 14. Visual Evidence

Research stills (Workbench, 640×360), copied to external evidence:

```text
frame_u00_f001.png
frame_u02_f013.png
frame_u04_f025.png
frame_u06_f037.png
frame_u08_f049.png
frame_u10_f061.png
```

Observation: Knight mesh is skinned; carry/walk pose is visible; no mesh
explosion in these frames. Camera is conservative (character small in
frame). Visual appearance is **not** a PASS criterion.

This is **not** POC-PREVIEW-01R.

---

## 15. Persistence / Re-open

External persistence (run 1):

| Artifact | Size | SHA-256 |
| --- | --- | --- |
| `derived_result.blend` | 16359051 | `882e2c0a9e3fec121abdabb3807eec4bc12bf9283776532f70f12fb4bc8762eb` |

Fresh-process reopen (new Blender process, `--factory-startup`):

```text
status: SUCCESS
target_present: true
baked_action: CharacterArmatureAction
frame_start/end: 1 / 61
fps: 30
bone_count: 32
representative sampled bones: evaluable; checks consistent with worker samples
```

Result: **PASS**. The baked animation survives the worker process boundary.

---

## 16. Candidate Preview Artifact

| Field | Value |
| --- | --- |
| Type | GLB (candidate only) |
| Generation | SUCCESS |
| Size | 853396 |
| SHA-256 | `4cec630a74ae0bdbc41f6782f8973121f3d45a6ef556aee1e91fe1f2f75bbddb` |
| Limitations | may drop FBX pivot/layer semantics; not product authority; Preview architecture **not selected** |

Not POC-PREVIEW-01R. No browser rendering. No glTF parser campaign.

---

## 17. Repeatability

Three clean-process runs. Semantic run summaries compared (not `.blend` /
GLB bytes).

```text
run 1 semantic hash: 76d68b4598fc58a1c686b0c983f1bbdc36cfcf72f5c81967871442b3b2e521e0
run 2 semantic hash: 76d68b4598fc58a1c686b0c983f1bbdc36cfcf72f5c81967871442b3b2e521e0
run 3 semantic hash: 76d68b4598fc58a1c686b0c983f1bbdc36cfcf72f5c81967871442b3b2e521e0
consistent: YES (3/3)
```

---

## 18. Failure Observability

Controlled case: nonexistent Character path
(`MISSING_Knight_Male.fbx`).

```text
worker_returncode: 1
worker_status: FAILURE
diagnostic_captured: true
accepted_derived_variant_published: false
partial_output_treated_as_success: false
```

No crash-recovery / timeout / pool campaign. Success is distinguishable from
failure; partial output cannot authorize publication.

---

## 19. Timing

Approximate; no performance claim; not a reject criterion.

| Stage | Run 1 (s) |
| --- | --- |
| Cold start + execute | 10.667 |
| Import | 3.241 |
| Retarget / bake | 0.092 |
| Save `.blend` | 0.095 |
| GLB export | 4.028 |
| Research render | 1.430 |
| Worker total (inside Blender) | 8.932 |
| Reopen wall | 0.989 |

Runs 2–3 similar (worker wall ≈ 8.8–9.4 s). Startup is not optimized.

Portable layout: official zip extract; factory startup; built-in FBX import,
pose eval, keyframe, `wm.save_as_mainfile`, `export_scene.gltf`, Workbench.

Blender licensing/distribution remains a future productization question.

---

## 20. Boundary Audit

| Question | Answer |
| --- | --- |
| Does Job Spec require Blender semantics? | **NO** (pin in determinism context only) |
| Does product identity require Blender? | **NO** |
| Does Mapping truth require Blender? | **NO** (source/target joint ids + roles) |
| Does Retarget Policy require Blender? | **NO** (backend-neutral statements) |
| Does QC meaning require Blender? | **NO** (envelope samples + hashes) |
| Does Derived Variant identity require Blender? | **NO** (provenance hashes; `.blend` is an experiment artifact) |
| Substantial custom RigForge retarget runtime? | **NO** |
| Could another future worker consume the same durable contract? | **YES, in principle** (not demonstrated) |

Acceptable adapter details used: FBX import options, PoseBone resolution,
temporary scene objects, bake/keyframes, `.blend` persistence, candidate GLB.

---

## 21. Requirements / Generality Audit

```text
Character already rigged: YES
Motion has Source Skeleton context: YES
Character + Motion selection representable: YES
Mapping product-owned: YES
Compatibility product-owned: YES
Retarget Policy product-owned: YES
Blender execution hidden: YES (durable contracts); worker is explicitly Blender
Job Spec backend-neutral: YES
Result Envelope backend-neutral: YES
Derived Variant publication controlled by RigForge: YES
basic QC outside worker-success semantics: YES
fresh-process re-open: PASS
engine-specific dependency: NONE (product layer)
custom RigForge retarget runtime required: NO
native FBX parser required: NO
Blender product authority: NO
Core language selected: NO
```

Generality (this slice is one humanoid pair; do **not** claim):

```text
product domain is not Unreal-specific: confirmed at contract layer
product domain is not Unity-specific: confirmed (UAL2 Unity folder is a pack layout, not product truth)
product domain is not FBX-specific: contracts are asset-hash + joint ids; FBX is the vehicle
Job Spec is not Blender-specific: YES
Mapping model is not Mixamo-only: YES
Skeleton Summary is not humanoid-only by structure: YES (id/parent/deform)
Derived Variant is not FBX-only: YES (manifest + optional candidate GLB)
```

Non-humanoid real E2E remains later mandatory hardening. One pair does not
prove all humanoids, all FBX, all Blender versions, or all policies.

---

## 22. Gaps

- One pair only; `Walk_Carry_Loop` not a plain Walk take.
- Knight `Foot.*` IK-style hierarchy; no foot-plant QC.
- Rotation-only policy does not preserve source loop identity on target world Head.
- Research camera is conservative; not a viewer.
- GLB is a candidate, not a frozen Preview format.
- Blender packaging / GPL / redistribution not qualified.
- Worker crash/timeout/cancel/pools not tested.
- Auto-Mapping not tested.
- Native ufbx inspector still deferred (POC-FBX-01 evidence retained, not mandated).
- Inspect JSON for skeletons was produced by an external Blender inspect script, not committed as a second worker.

---

## 23. Decision Impact

```text
ACCEPT_BLENDER_BACKEND_WITH_GUARDS
```

**A** is demonstrated for this frozen pair: isolated stock Blender 5.2.1 LTS
imported, transferred, baked, persisted, and re-opened.

**B** is demonstrated for this slice: Mapping, Policy, Job Spec, Envelope, QC,
and Derived Variant provenance stay backend-neutral; worker success ≠
publication.

Bare `ACCEPT_BLENDER_BACKEND` is too strong: guards below are material.

`REJECT` is not justified: no requirement forced bpy into durable product
truth, and no substantial custom retarget runtime was required.

`INCONCLUSIVE` is not required: the vertical slice ran, QC-gated, repeated
3/3, and failed closed on a missing input.

### Guards (minimum)

1. Pin isolated Blender; factory-startup; disable-autoexec; record build hash.
2. Frozen reviewed Mapping; worker FAIL on missing required joints.
3. Pair-specific Retarget Policy; do not treat it as a general retargeter.
4. QC-gated publication; QC must not mutate.
5. Persistence + fresh-process reopen before trusting a bake.
6. Do not upgrade `ACCEPT_BLENDER_BACKEND_WITH_GUARDS` to product authority.
7. Do not select Core language from this Python/bpy harness.
8. Do not freeze GLB / viewer tech from the candidate artifact.
9. Later hardening still required: non-humanoid E2E, mapping breadth,
   contact QC, packaging/legal productization, worker reliability campaign.

---

## Files / lifecycle (this report)

Harness: [../../experiments/w0p/poc_blender_e2e_01/](../../experiments/w0p/poc_blender_e2e_01/).

External evidence: `F:\NewResearch\rigforge_w0p_evidence\poc_blender_e2e_01\`.

```text
W0: ACTIVE
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: READY / NOT STARTED
W0-RS: NOT STARTED
IA-1: NOT STARTED
product implementation: NOT STARTED
```
