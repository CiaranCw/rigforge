# POC-FBX-01 — Real-asset FBX semantic validation (ufbx)

**Status:** `COMPLETE / PASS`

**Date:** 2026-08-31

**Classification:** RESEARCH ONLY / W0-P / NON-PRODUCTION

**Decision impact:** `KEEP_UFBX_WITH_GUARDS`

**Core language selected:** NO (`PROVISIONAL_W0P_GATED` unchanged)

```text
C is the test harness language only.
This is not evidence selecting the RigForge Core language.
```

This is not a production FBX Adapter, not Canonical, not a writer evaluation,
and not POC-GLTF-01.

W0.4 documents are **not** rewritten. FBX remains replaceable.

**Rev1 (2026-08-31):** repository-runner reproducibility, actual per-vertex
skin-weight inspection, independent binary cross-check, Maya L2 source
binding, populated L3 summary, explicit layered-animation guard.
External focused review accepted this closeout as **COMPLETE / PASS**.

---

## 1. Question

Is ufbx a sufficiently semantically faithful, inspectable, diagnosable and
general FBX reader for RigForge V1 ingest when tested against controlled
semantic fixtures and representative real humanoid / non-humanoid assets?

W0.4 preference: FBX read **ufbx v0.23.0**, class `PROVISIONAL_W0P_GATED`.
This PoC is allowed to reverse that preference. It does not.

---

## 2. Pin / environment

| Item | Value |
| --- | --- |
| ufbx | **v0.23.0** tag, commit `fcc5d6ba444cfd3eb80677dba5e37e493941abe5` |
| Archive | `https://github.com/ufbx/ufbx/archive/refs/tags/v0.23.0.zip` |
| Zip SHA-256 | `0436D21A0E4AD9B983E36D4CCD3F781C907F6A87514A18DDAA6FFA54E0A10CCE` |
| License | MIT or Unlicense (dual) |
| Retrieval | 2026-08-31, same extract as POC-CORE-01 |
| Newer release | Not switched. Pin remains v0.23.0. |
| Harness | C99 `inspect_fbx.c` around the documented **C API** |
| Compiler | MSVC 19.36.32548.0 / toolset 14.36.32532 / VS 2022 17.13.7 |
| Load opts | zeroed `ufbx_load_opts` (no `target_axes` / `target_unit_meters`) |

Evidence: `F:\NewResearch\rigforge_w0p_evidence\poc_fbx_01\versions.txt`.

---

## 3. Layers (mandatory)

| Layer | What |
| --- | --- |
| RAW SOURCE STRUCTURE | FBX properties (`Lcl *`, pivots, Pre/PostRotation, Geometric*), stacks, layers, names, hierarchy, skin vertex weights |
| UFBX EVALUATED RESULT | `local_transform`, `node_to_world`, `ufbx_evaluate_scene` / `ufbx_evaluate_transform` |
| RIGFORGE DERIVED TEST INTERPRETATION | sample-role labels, Walk stack selection, `derived_test_view` left `NOT_CREATED` |

`node_to_world` is **not** claimed to equal `ParentWorld × Lcl TRS`.

---

## 4. Controlled fixtures (Level 1)

Reused, not modified: `experiments/w0p/poc_core_01/fixture/minimal_chain.fbx`
(hierarchy / names).

Added:

| File | Purpose |
| --- | --- |
| `fixtures/ordinary_anim.fbx` | One stack, two layers (second empty), JointA `Lcl Rotation` Y 0→90 over 1s |
| `fixtures/authored_local.fbx` | PreRotation / RotationPivot / PostRotation / GeometricTranslation; two children of implicit root; Null helper; **Z-up** and `UnitScaleFactor 2.54` |

Observed:

- `ordinary_anim`: load success; stack `Take001` `time_begin=0` `time_end=1`; layers `BaseLayer` (weight 1, 1 anim prop) and `ExtraLayer` (weight 0, 0 props).
- `authored_local`: `coordinate_unit.up = +z`, `unit_meters ≈ 0.0254`. Two root children `PivotBone`, `SecondRoot`. PivotBone authored `Lcl Translation = [0,20,0]` vs evaluated `local_transform.translation ≈ [-5, 25, -7.07]`. GeometricTranslation `[5,0,0]` recorded separately. `has_geometry_transform = true`.
- Project LimbNode models without a separate NodeAttribute object report `attrib_type=unknown` (fixture incompleteness, not an L3 finding).

The two-layer fixture proves **layer inventory**. The second layer is empty, so
this PoC does **not** validate non-empty multi-layer composition, additive layer
semantics, or complex Maya layered animation. That remains
`UNVALIDATED / GUARD`, not a ufbx failure.

---

## 5. Level 2 (selective)

ufbx v0.23.0 upstream `maya_pivots_7500_ascii.fbx` (already on disk from
POC-CORE extract; **not Canonical**).

Bound input (Rev1; not only the inspection JSON hash):

```text
source_path:
  F:\NewResearch\rigforge_w0p_work\poc_core_01\deps\ufbx-0.23.0\misc\ufbx_testcases\maya_pivots_7500_ascii.fbx
origin: pinned ufbx v0.23.0 misc/ufbx_testcases/maya_pivots_7500_ascii.fbx
size: 13925
SHA-256: 6bedb4ebb449e42624a5b698f3f1c76d88e1790fe53b96158b00946c57ff6b2d
```

`pCube1` authored `RotationPivot = [1,2,3]`, `Lcl Translation = [-1,-2,-3]`,
evaluated local translation ≈ `[0.72, 1.83, -0.60]`. Confirms ufbx does not
collapse the Maya pivot recipe to plain Lcl TRS when those properties exist.

---

## 6. Real-asset manifest (Level 3)

Archives were **operator-placed** at the repo root, then moved out of git to
`F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\`. This agent did not fetch
cloud-drive bytes for L3. Rev1 did not download another real asset.

Frozen experiment variables live in
`experiments/w0p/poc_fbx_01/l3_run_config.json` and are passed by
`scripts/run_poc_fbx.py`. Effective values:
`F:\NewResearch\rigforge_w0p_evidence\poc_fbx_01\run_config.json`.

### Humanoid — F-L3-QCHAR

```text
TEST QUESTION
  Can ufbx load/inspect a representative humanoid animated FBX without a DCC?

ASSET
  Knight_Male.fbx
  pack-relative: Ultimate Animated Character Pack - Nov 2019/FBX/Knight_Male.fbx

SOURCE / LICENSE
  Pack page https://quaternius.com/packs/ultimatedanimatedcharacter.html
  Pack page License = CC0 1.0 [SOURCE_CONFIRMED] 2026-08-31
  Packaged License.txt: CC0 1.0 Universal

WHY SELECTED
  Bipedal character with skin + 17 animation stacks (Walk/Run/Idle/…).
  Not Chef_Hat / VikingHelmet accessory. Not pack extras Cow.fbx / Pug.fbx.

HASH
  archive SHA-256 b088fcbdae7b70b8a61eaea7771cfa6b1243506b4e7b7e5fdb3663c552cfaf65
  archive size 76553115
  file SHA-256 fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f
  file size 2270172

EXPERIMENT CONFIG (Rev1, repository runner)
  stack: CharacterArmature|Walk
  observed stack index: 15
  samples: Bone, Hips, UpperArm.L, Head_end

EXPECTED
  Load without DCC; hierarchy/names; axes/units; skin/bind including actual
  per-vertex weights; stacks/layers; deterministic Walk samples;
  non-deforming bones not collapsed into clusters.

OBSERVED
  Load success, warnings=[].
  FBX 7400 binary, exporter blender_binary, Blender 2.79 FBX IO 3.7.17.
  axes +x +y +z, unit_meters=0.01.
  nodes=35, meshes=1, bone attrib=32, skin clusters=23, poses=1 (is_bind_pose),
  anim_stacks=17, anim_layers=17, characters=0, constraints=0.
  Roots under implicit root: CharacterArmature (empty), Body (mesh) — siblings.
  Walk stack sampled: CharacterArmature|Walk, one layer, 72 anim props.
  Skin vertex weights: 1484 vertex records, 3093 weight entries, max 8/vertex,
  0 zero-influence vertices, structural validation PASS
  (invalid cluster refs=0, non-finite=0, negative=0).
  Observed weight-sum min≈0.893 max≈1.089 (not required to equal 1.0).
  Representative influences recorded (vertex → cluster → bone name → weight).
  3-run JSON SHA-256 8d492b294ff08c1cc59c82e115beb4f988b5956e6cbb73839dc99654ec54c0fc
  (changed vs original PoC because the harness now emits weight evidence)

PASS / FAIL / MIXED
  PASS for the inspectability questions on this file.
  MIXED vs a Maya-pivot oracle: this file has no nonzero PreRotation (source).

SOURCE FACTS
  Names include Bone, Hips, UpperArm.L, Head_end, PoleTarget.*, *_end.
  Duplicate source name Body (mesh node and bone node).
  Mesh not parented under the armature empty.

UFBX FACTS
  Authored props vs local_transform separated.
  Cluster bones vs non-deforming bone attributes distinguished (9 non-cluster bones).
  Explicit bind pose present (ufbx_pose.is_bind_pose).
  Actual per-vertex weights inspectable via ufbx_skin_deformer.vertices/weights.
  Evaluate Walk: UpperArm.L local quat changes; Hips world translation bobs;
  Bone local/world stable (root of this clip); Head_end local identity, world moves.

INDEPENDENT CROSS-CHECK
  fbx_name_scan.py: 34 Model records. Selected name/parent/root facts MATCH
  ufbx (CharacterArmature, Body×2, Bone, Hips, UpperArm.L, Head).
  Scanner is not Canonical.
  TOKEN-PRESENCE SECONDARY: FBX UTF-8 contains CharacterArmature, Bone, Hips,
  UpperArm.L, Head, CharacterArmature|Walk. Does not prove hierarchy/transform/skin/bind/eval.
  Headless Blender: not installed. glTF in pack: not used as FBX proof.

SEMANTIC GAPS
  SOURCE: Blender 2.79 FBX does not carry Maya pivot recipe.
  SOURCE AMBIGUITY: duplicate name Body.
  SOURCE: per-vertex weight sums are not identically 1.0.

HARNESS GAPS
  Walk stack index 15 and sample names are explicit in l3_run_config.json
  and passed by the repository runner (not first-stack Death).
  anim_props dump cap 200 (Walk had 72, not truncated).

LIMITATIONS
  One character from one Blender-exported pack. Not Maya/Max/MotionBuilder.

DECISION IMPACT
  Supports keeping ufbx as preferred reader for this class of file, with guards.
```

### Non-humanoid — F-L3-QANIMAL

```text
TEST QUESTION
  Same inspectability on a non-humanoid rigged/skinned/animated FBX?

ASSET
  Wolf.fbx
  pack-relative: Ultimate Animated Animals - July 2021/FBX/Wolf.fbx

SOURCE / LICENSE
  Pack page https://quaternius.com/packs/ultimateanimatedanimals.html
  Pack page License = CC0 1.0 [SOURCE_CONFIRMED] 2026-08-31
  Packaged License.txt: CC0 1.0 Universal

WHY SELECTED
  Quadruped, skinned (51 clusters), 12 stacks including Walk/Gallop/Idle.
  Not chosen because it is small. Distinct hierarchy from the humanoid file.

HASH
  archive SHA-256 4ea0244dab97ef5d35fcb75ff1d1a1c0453f6f9fcdacf3e477be35c776f901d1
  archive size 40126807
  file SHA-256 91d5c31678fa291571f82abb029fde5c14fdd5932a1569c3afaa5e9602bebf94
  file size 3501740

EXPERIMENT CONFIG (Rev1, repository runner)
  stack: AnimalArmature|Walk
  observed stack index: 11
  samples: Body, Torso3, FrontUpperLeg.L, Head_end

EXPECTED
  Load; non-humanoid names preserved; no silent “pelvis” labeling;
  skin/bind including actual weights; stacks; deterministic Walk samples.

OBSERVED
  Load success, warnings=[].
  Same exporter family as Knight (Blender 2.79 binary 7400).
  axes +x +y +z, unit_meters=0.01.
  nodes=70, bone attrib=67, clusters=51, stacks=12, bind pose present.
  Roots: AnimalArmature (empty), Wolf (mesh) — siblings.
  Walk: AnimalArmature|Walk, 156 anim props, not truncated.
  Skin vertex weights: 983 vertex records, 3267 weight entries, max 9/vertex,
  0 zero-influence vertices, structural validation PASS.
  Observed weight-sum min≈0.852 max≈2.006 (not required to equal 1.0;
  ufbx documents weights are not guaranteed normalized).
  Representative influences recorded.
  3-run JSON SHA-256 a7a058a0dfcf015b62791073fcc2f93a3ebb93b73b86ae2e5ad10f85f036ce87

PASS / FAIL / MIXED
  PASS for inspectability on this file.

SOURCE FACTS
  Names: Body, Torso/Torso2/Torso3, FrontUpperLeg.L, Tail*, Ear*, IK* / PoleTarget*.
  Central nodes are Body / Torso3 — not labelled pelvis.

UFBX FACTS
  Body local Z and world Y change over Walk (trajectory-like).
  FrontUpperLeg.L local quat changes.
  16 non-deforming bone nodes (ends / after IK helpers).
  Actual per-vertex weights inspectable.

INDEPENDENT CROSS-CHECK
  fbx_name_scan.py: 69 Model records. Selected name/parent/root facts MATCH
  ufbx (AnimalArmature, Wolf, Body, Torso3, FrontUpperLeg.L, Head).
  TOKEN-PRESENCE SECONDARY: FBX UTF-8 contains AnimalArmature|Walk and the
  selected names. Does not prove hierarchy/transform/skin/bind/eval.

SEMANTIC GAPS / HARNESS GAPS / LIMITATIONS
  Same exporter-family limit as QCHAR. No Maya HumanIK in this file.
  Non-normalized weight sums are SOURCE, not classified as a ufbx bug.

DECISION IMPACT
  Non-humanoid path is genuinely exercised. Supports WITH_GUARDS, not humanoid-only.
```

---

## 7. Q1–Q10

| Q | Answer |
| --- | --- |
| Q1 Load without DCC | **YES** for both L3 files. |
| Q2 Hierarchy without humanoid assumptions | **YES**. Wolf uses AnimalArmature / Body / Torso3. Sample roles are structural. Source name `Hips` on Knight is recorded as a source string, not Canonical pelvis. |
| Q3 Axes/units before normalization | **YES**. Recorded; `derived_test_view = NOT_CREATED`. |
| Q4 Avoid plain-TRS model | **YES** at the API: authored props vs evaluated transforms are separate. L1+L2 show evaluated local ≠ Lcl TRS when pivots/pre/post/geometric exist. L3 Blender files do not author that recipe ([SOURCE]). |
| Q5 Skin/bind without guessing | **YES**. Clusters, `geometry_to_bone`, `bind_to_world`, `ufbx_pose.is_bind_pose`, **and** actual per-vertex `vertices[]` / `weights[]` (cluster index, bone name, weight) with structural validation. Future rest pose not computed. Weight sums are observed, not assumed to be 1.0. |
| Q6 Stacks/layers/time before flatten | **YES** for inventory. Knight 17 stacks / Wolf 12; one layer each; time_begin/end recorded. Non-empty multi-layer composition is **not** validated (see guards). |
| Q7 Deterministic motion samples | **YES**. 3-run JSON SHA-256 stable through the repository runner. Walk world/quat samples change where the clip animates. |
| Q8 Require FBX SDK fallback for V1? | **Not on tested material.** No Character, constraint, NURBS, or load failure. Untested Maya HumanIK / non-empty layered Maya clips remain a **guard**, not a demonstrated SDK requirement. |
| Q9 Exposure | See §8. |
| Q10 Keep ufbx preferred? | **Yes, with guards.** Not `KEEP_UFBX_PREFERRED` (L3 exporter is one family; layered animation incomplete; V1 still needs explicit unsupported-feature policy). |

---

## 8. What ufbx exposed (this pin)

| Topic | Class |
| --- | --- |
| Load of Blender 7.4 binary FBX | fully exposed |
| Names, hierarchy, multiple roots | fully exposed |
| Axes / `unit_meters` / original unit | fully exposed |
| Authored Lcl / pivot / pre / post / geometric props | fully exposed when present |
| Evaluated local / world | fully exposed |
| Inherit mode | fully exposed (`normal` on L3) |
| Skin deformer / cluster / bind matrices | fully exposed |
| Per-vertex skin weights (`vertices[]` / `weights[]`) | fully exposed (Rev1) |
| FBX pose `is_bind_pose` | fully exposed |
| Anim stacks / layers / time range / anim props | fully exposed |
| `ufbx_evaluate_scene` / `evaluate_transform` | fully exposed |
| Warnings list | fully exposed (empty on these files) |
| Pose `bone_to_parent` | **partial** (ufbx: approximated from parent world) |
| Every anim prop on huge clips | **partial in harness** (cap 200; Walk under cap) |
| Maya HumanIK `Character`, constraints, NURBS | API exists; **not present** in L3 |
| Authored pivot recipe in Quaternius L3 | **absent at source** (not a ufbx hide) |
| Non-empty multi-layer composition | **unvalidated** on this corpus |
| Canonical joint identity | **not exposed** (correct; must not be invented) |

---

## 9. Semantic / implementation gaps

| Gap | Class |
| --- | --- |
| L3 files are Blender 2.79 exports only | SOURCE / RIGFORGE REQUIREMENT GAP (corpus diversity) |
| No nonzero PreRotation on L3 | SOURCE (expected; W0.4 warned not to assume Maya pivots) |
| Duplicate `Body` name on Knight | SOURCE AMBIGUITY |
| Mesh sibling of armature, not child | FBX FORMAT COMPLEXITY / Blender export convention |
| Per-vertex weight sums ≠ 1.0 (Wolf max ≈ 2.0) | SOURCE (ufbx documents weights are not guaranteed normalized) |
| `bone_to_parent` approximated | UFBX LIMITATION (documented) |
| Non-empty multi-layer / additive / Maya layered clips | UNVALIDATED / GUARD (not a demonstrated ufbx failure) |
| Walk selected by explicit config index | HARNESS (explicit in `l3_run_config.json`; first stack would have been Death/Attack) |
| No headless Blender eval | HARNESS / environment |
| glTF sitting in the animal pack | not used; POC-GLTF-01 NOT STARTED |
| Project LimbNode fixture without NodeAttribute | HARNESS / fixture gap (`attrib_type=unknown`) |

No jump from “unexpected” to “ufbx bug” without evidence. No ufbx bug claimed.

---

## 10. Guards (why not KEEP_UFBX_PREFERRED)

1. Zeroed load opts unless a labelled `DERIVED TEST VIEW` is created.
2. Never treat `local_transform` as the authored pivot/pre/post recipe.
3. Inventory **all** stacks and layers before any flatten (Blender: many stacks, one layer).
4. Distinguish bone attributes, skin-cluster bones, IK/pole/end non-deformers.
5. Do not assume an explicit bind pose; do not fabricate one; do not call cluster bind “rest pose”.
6. Source names are not Canonical IDs (duplicates happen).
7. Mesh↔armature deform relation is skin clusters, not necessarily node parent.
8. Diagnostics: do not suppress warnings.
9. V1 unsupported-feature policy still required for untested constructs (HumanIK Character, constraints, NURBS, exotic inherit, ASCII 6.1, etc.). Absence in this corpus is not proof they never appear.
10. Core language remains `PROVISIONAL_W0P_GATED`. This C harness is not L1/L2 evidence.
11. **Non-empty multi-layer FBX composition, additive layer semantics, and complex Maya layered animation are UNVALIDATED / GUARD.** This PoC proves stack inventory, layer inventory, and basic combined-stack evaluation. L3 has one layer per stack. The L1 fixture’s second layer is empty. Do not treat that as a ufbx failure.
12. **Do not assume per-vertex skin weights are normalized.** Inspect actual sums. ufbx documents they are not guaranteed to be normalized.
13. L3 exporter-family remains Blender 2.79 only. Do not claim cross-DCC exporter coverage from Knight+Wolf alone.

---

## 11. Generality audit

| Assumption | |
| --- | --- |
| humanoid-only | **NO** (Wolf exercised) |
| single root | **NO** (armature empty + mesh siblings) |
| one animation stack | **NO** (17 / 12 inventoried) |
| one layer | inventory **NO** (L1 ExtraLayer present but empty); **non-empty multi-layer UNVALIDATED** |
| root == pelvis | **NO** (Knight root sample is `Bone`; `Hips` is a source name only; Wolf uses `Body` / `Torso3`) |
| centimetres | **NO** (recorded; L1 used 2.54 scale factor) |
| Y-up | **NO** (recorded; L1 used Z-up) |
| one mesh | **NO** (counted, not assumed) |
| one skin | **NO** (counted) |
| every bone deforms | **NO** (ends / pole targets separated) |
| normalized skin weights | **NO** (sums observed; not required to be 1.0) |
| specific DCC required to load | **NO** |
| specific engine | **NO** |
| FBX as Canonical authority | **NO** |

---

## 12. Requirements-drift audit

Still serves: cross-format, cross-DCC, cross-engine, generic Character /
Skeleton / Motion infrastructure. This PoC validates **one Adapter
candidate**. FBX remains replaceable. Canonical contract not created.

---

## 13. Determinism / performance

Rev1 hashes are from `scripts/run_poc_fbx.py` itself (three runs per file).
They differ from the original PoC hashes because skin-weight JSON was added.

| File | 3-run JSON SHA-256 | load_ms (run 1) | peak WS |
| --- | --- | --- | --- |
| Knight_Male.fbx | `8d492b294ff08c1cc59c82e115beb4f988b5956e6cbb73839dc99654ec54c0fc` stable | 6.740 | 9973760 |
| Wolf.fbx | `a7a058a0dfcf015b62791073fcc2f93a3ebb93b73b86ae2e5ad10f85f036ce87` stable | 10.067 | 16056320 |

Performance is secondary. No optimization. No accept/reject on milliseconds.

---

## 14. Files

Repository (allowed):

- `experiments/w0p/poc_fbx_01/**`
- `docs/research/poc/POC_FBX_01.md`
- `docs/research/poc/README.md`
- `docs/research/R1_RESEARCH_BASELINE.md`
- `docs/development/ROADMAP.md`

External: work / evidence / assets trees. Rev1 review zip:
`F:\NewResearch\rigforge_w0_p_fbx_01_rev1_review.zip`.

No production `src/`. No `docs/research/decisions/*` edits.

---

## 15. Lifecycle (this report)

```text
W0: ACTIVE
W0-P: ACTIVE

POC-CORE-01: COMPLETE / PASS
Decision Impact: INCONCLUSIVE

POC-FBX-01: COMPLETE / PASS / BASELINED
Decision Impact: KEEP_UFBX_WITH_GUARDS

POC-GLTF-01: NOT STARTED
```

Do not start POC-GLTF-01.

---

## 16. Reproducible runner command

```text
python experiments/w0p/poc_fbx_01/scripts/run_poc_fbx.py
```

Equivalent explicit form (frozen Walk selections):

```text
python experiments/w0p/poc_fbx_01/scripts/run_poc_fbx.py ^
  --qchar-fbx "F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx" ^
  --qanimal-fbx "F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qanimal_extract\Ultimate Animated Animals - July 2021\FBX\Wolf.fbx" ^
  --qchar-stack-index 15 ^
  --qchar-sample-names Bone,Hips,UpperArm.L,Head_end ^
  --qanimal-stack-index 11 ^
  --qanimal-sample-names Body,Torso3,FrontUpperLeg.L,Head_end
```
