# W0.1 — Canonical Foundations

Accepted research baseline. Rev1 closed F1–F7. Rev2 closed F8–F10. Final closeout corrected bookkeeping / wording only (MUST audit, `FbxTime` vs `EMode`, Motion identity-key boundary). Status: **COMPLETE**.

Research report. Not a schema. Not an implementation.

Access date for cited sources: **2026-08-30**.

Related: [SEMANTIC_GLOSSARY.md](SEMANTIC_GLOSSARY.md), [FORMAT_SEMANTIC_MATRIX.md](FORMAT_SEMANTIC_MATRIX.md), [CANONICAL_REQUIREMENTS_DRAFT.md](CANONICAL_REQUIREMENTS_DRAFT.md), [REAL_ASSET_VALIDATION_PLAN.md](REAL_ASSET_VALIDATION_PLAN.md).

---

## 1. Scope

Establish an evidence-grounded semantic baseline for Character / Skeleton / Motion **representation** across FBX, glTF 2.0, OpenUSD UsdSkel, and VRM 1.0.

Question:

> What must RigForge understand and preserve so Canonical is not accidentally a copy of one format, DCC, or engine?

Out of scope: W0.2 mapping/retarget, W0.3 adapter/library selection, W1 field freeze, any parser.

---

## 2. Method

1. Read repository authorities (product, architecture, R1 plan).
2. Prefer normative specs, then official docs, then official source/headers.
3. Classify every material claim.
4. Record conflicts with working assumptions instead of overwriting them.
5. No local asset experiments in this pass.

Confidence: `HIGH` = explicit spec/doc sentence. `MEDIUM` = official doc + consistent secondary official page. `LOW` = incomplete docs or WIP project.

---

## 3. Source hierarchy

### Normative / specification

| Source | Org | Version | URL |
| --- | --- | --- | --- |
| glTF 2.0 Specification | Khronos | specification version **2.0.1**, date **2021-10-11** | https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html |
| VRMC_vrm 1.0 + humanoid schema | VRM Consortium | 1.0 | https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm-1.0 |
| VRMC_vrm_animation 1.0 | VRM Consortium | 1.0 | https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm_animation-1.0 |
| ISO/IEC 19774:2019 HAnim | ISO / Web3D | 2019 | https://www.web3d.org/standards/hanim |
| COLLADA Digital Asset Schema | Khronos | 1.4 (spec PDF reviewed at overview level) | https://www.khronos.org/files/collada_spec_1_4.pdf |

### Official documentation (not a public ISO for FBX)

| Source | Org | Version / note | URL |
| --- | --- | --- | --- |
| Computing transformation matrices | Autodesk | FBX Developer Help 2018 | https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/nodes_and_scene_graph/fbx_nodes/computing_transformation_matrix.html |
| Transformation data / pivot bake | Autodesk | FBX Developer Help 2018 | https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/nodes_and_scene_graph/fbx_nodes/transformation_data.html |
| FbxNode / inherit / pivots | Autodesk | FBX Developer Help 2019 | https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_node.html |
| FbxPose | Autodesk | FBX Developer Help 2019 | https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_pose.html |
| FbxSkin / FbxCluster | Autodesk | FBX Developer Help 2018 | https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/cpp_ref/class_fbx_cluster.html |
| FbxAnimLayer | Autodesk | FBX Developer Help 2019 | https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_anim_layer.html |
| Using the Blend Modes | Autodesk | FBX Developer Help 2018 | https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/animation/blending_animation/using_blend_modes.html |
| FbxTime | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_time.html |
| FbxAnimCurve | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_anim_curve.html |
| FbxGlobalSettings | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_global_settings.html |
| UsdStage TimeCode API | Pixar / OpenUSD | release / current API | https://openusd.org/release/api/class_usd_stage.html |
| Time and Animated Values | Pixar / OpenUSD | release user guide 26.08 | https://openusd.org/release/user_guides/time_and_animated_values.html |
| UsdSkel Introduction | Pixar / OpenUSD | release docs, generated 2026-07-17 | https://openusd.org/release/api/_usd_skel__intro.html |
| UsdSkel Schema Overview | Pixar / OpenUSD | release | https://openusd.org/release/api/_usd_skel__schema_overview.html |
| UsdSkelSkeleton API | Pixar / OpenUSD | 25.05 docs also reviewed | https://openusd.org/25.05/api/class_usd_skel_skeleton.html |
| ufbx Nodes / Animation / Deformers | ufbx | current site | https://ufbx.github.io/ |
| glTF Tutorial: Skins | Khronos | tutorial (official educational) | https://github.com/KhronosGroup/glTF-Tutorials/blob/main/gltfTutorial/gltfTutorial_020_Skins.md |

### Source / headers inspected (no vendor)

| Source | Note | URL |
| --- | --- | --- |
| VRMC_vrm.humanoid.humanBones.schema.json | required bone list | raw GitHub |
| ufbx_write.h / README | element types; save format/version example | https://github.com/ufbx/ufbx-write |

### Papers

None.

### Experiments

None.

**FBX has no public ISO-equivalent specification.** Autodesk SDK documentation is the highest available official authority. That is a permanent evidence-quality limit. [RIGFORGE_INFERENCE]

---

## 4. FBX findings

### CONFIRMED — HIGH

1. **Evaluated FBX world is not `ParentWorld × simple(LclTranslation, LclRotation, LclScaling)`.**
   Official relation is conceptually `WorldTransform = ParentWorldTransform × authored local FBX transform recipe`. The local recipe includes `T`, rotation offset, rotation pivot, pre-rotation, rotation, post-rotation inverse, scale offset, scale pivot, and scale, plus inheritance behavior. [OFFICIAL_DOC] Autodesk 2018 computing-transformation-matrix.
   Rotation `R` embeds rotation **order** (`R = Rz*Ry*Rx` for default XYZ). [OFFICIAL_DOC]
   Do not confuse authored Lcl T/R/S properties, the authored local recipe, evaluated local transform, and evaluated world transform.

2. **Geometric T/R/S are not inherited.** They offset the node attribute (e.g. mesh) after the node transform. [OFFICIAL_DOC] same page; FbxNode class notes.

3. **Inherit types exist** (`eInheritRrSs`, `eInheritRSrs`, `eInheritRrs`). [OFFICIAL_DOC] FbxNode.

4. **There is no single universally authoritative FBX bind pose.**
   - `FbxPose` can hold Bind Pose **or** Rest Pose. Bind pose is described as global matrices of geometry + links + ancestors at bind. Rest pose may mix local/global flags. [OFFICIAL_DOC] FbxPose.
   - Each `FbxCluster` stores `Transform` (geometry global at bind), `TransformLink` (link/bone global at bind), plus associate/parent variants. [OFFICIAL_DOC] FbxCluster.
   Official samples **create** a bind pose from evaluated globals after setting cluster matrices — i.e. pose tables can be reconstructed or omitted. [OFFICIAL_DOC] SwitchBinding example. Whether producer output can leave pose-table data **disagreeing** with cluster bind matrices is **OPEN / REQUIRES_POC**, not an established official fact.

5. **Skinning is cluster-based**, not a UsdSkel-style token skeleton. Bone attribute is optional. [OFFICIAL_DOC] ufbx Deformers (interpreting FBX as loaded).

6. **Animation is stack → layer → curve.** ufbx: a stack is a clip/take; layers composite; curves may be cubic; rotation is Euler + order; evaluation should go through official-style evaluators. [OFFICIAL_DOC] ufbx Animation.

7. **FBX layer-level additive/blend is native.** `FbxAnimLayer::EBlendMode` includes `eBlendAdditive`, `eBlendOverride`, `eBlendOverridePassthrough`. Rotation accumulation and scale accumulation (`eScaleMultiply`, `eScaleAdditive`) are also defined. [OFFICIAL_DOC] [FbxAnimLayer](https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_anim_layer.html), [Using the Blend Modes](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/animation/blending_animation/using_blend_modes.html). This is **not** a portable clip-level additive contract shared with glTF / UsdSkel / VRM.

8. **FBX animation time is `FbxTime`, not an integer frame number.**
   - `FbxTime` stores a moment in an **internal integer format** accessed as `FbxLongLong` via `Set` / `Get`. [OFFICIAL_DOC] [FbxTime](https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_time.html)
   - Autodesk documents the FBX (`FbxTime`) time unit as **1/46,186,158,000 of one second**. [OFFICIAL_DOC] [FbxAnimCurve](https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_anim_curve.html)
   - Elapsed seconds from that internal value do **not** require `EMode`: `GetSecondDouble()` is the documented seconds mapping. [OFFICIAL_DOC] `FbxTime`
   - Display / frame addressing is a separate `FbxTime::EMode` (120, 100, 60, 50, 48, 30, 30-drop, NTSC drop, NTSC full frame, PAL, 24, 1000, film full frame, custom, 96, 72, 59.94, 119.88, … depending on SDK version). [OFFICIAL_DOC] `FbxTime`
   - Drop-frame / NTSC modes show that **frame-address / time-code semantics are not trivially elapsed seconds**. Official remark: `eNTSCDropFrame` is used so clock time stays almost in sync with time-code (drops 2 frames per minute except every 10 minutes); `eNTSCFullFrame` is a time address **not in sync** with clock time (example: time-code `01:00:00:00` equals clock `01:00:03:18`). [OFFICIAL_DOC] `FbxTime`
   - File/global settings carry a time mode: `FbxGlobalSettings::SetTimeMode` / `GetTimeMode`. Record `EMode` as frame/time-code provenance; do not treat it as the seconds decoder. [OFFICIAL_DOC] [FbxGlobalSettings](https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_global_settings.html)
   Converting FBX animation into any normalized Canonical seconds therefore requires an **explicit, recorded time-domain conversion** of `FbxTime` (not “frame / fps”). The final RigForge time representation remains **OPEN for W1**.

### OPEN / MEDIUM

- Exact numeric relationship IBM = `inverse(TransformLink) * Transform` (or similar) is the usual cluster reading; Autodesk text defines the three matrices but this pass did not re-derive the skinning equation from a single normative page. **REQUIRES_POC**.
- Whether markers/events have a stable official semantic.

### REQUIRES_POC

- Can pose-table data disagree with cluster bind matrices in real producer output, and if so what authority policy should RigForge apply?
- Inherit-mode and geometric-transform files vs ufbx conversion options.
- Layered animation vs `ufbx_bake_anim` error.

### ufbx (reader) — not FBX-the-standard

[OFFICIAL_DOC] https://ufbx.github.io/ and https://github.com/ufbx/ufbx

| Topic | Finding | Class |
| --- | --- | --- |
| Role | Single-file C loader + evaluators | Adapter candidate, not Canonical |
| License | MIT **or** Unlicense | [SOURCE_CONFIRMED] README |
| Maintenance | Active; integration v0.23.0 merged 2026-06-21 | [PROJECT_CLAIM] PR #261 |
| Exposes | nodes, bone attrib, skins, clusters, poses, local/world/geometry transforms, inherit_mode, anim stacks/layers/curves, evaluate/bake, axes, units | [OFFICIAL_DOC] |
| Read/write | Read + evaluate. Write is a **separate** project | [OFFICIAL_DOC] |
| Diagnostics | Structured load error (`error.description`); malformed handling claimed | [OFFICIAL_DOC] getting-started |
| Space conversion | `target_axes` / `target_unit_meters`; methods TRANSFORM_ROOT / ADJUST_TRANSFORMS / MODIFY_GEOMETRY are **not** equivalent across Blender vs Maya files | [OFFICIAL_DOC] Nodes |
| Bake | `ufbx_bake_anim` → linear T + quat R + S; cubic/Euler **must** resample | [OFFICIAL_DOC] |

ufbx **mitigates** FBX complexity; it does not redefine FBX. Treating `local_transform` as the FBX authored recipe is a semantic error. [RIGFORGE_INFERENCE]

### ufbx-write

Canonical repository: https://github.com/ufbx/ufbx-write
License: MIT **or** Unlicense. [SOURCE_CONFIRMED] README.

**WIP:** official README: work-in-progress; issues and breaking changes expected. [PROJECT_CLAIM] / [OFFICIAL_DOC] README accessed 2026-08-30.

Distinguish **mechanism/API** from **semantic completeness** and **production readiness**.

| Feature | Mechanism | Semantic completeness | Note |
| --- | --- | --- | --- |
| Scene / nodes / mesh | `CONFIRMED` | `UNPROVEN` | README example |
| Binary save | `CONFIRMED` | `UNPROVEN` | `save_opts.format = UFBXW_SAVE_FORMAT_BINARY` |
| FBX version field | `CONFIRMED` | `UNPROVEN` | example `save_opts.version = 7500` |
| ASCII formatting | `CONFIRMED` exists | `UNPROVEN` | README: internal ASCII uses `snprintf`; `extra/` faster formatters |
| Skin / cluster / bone / bind pose / anim stack | `CONFIRMED` types in header | `UNPROVEN` | round-trip quality not shown |
| Production-ready export | — | `UNPROVEN` / high-risk | WIP disclaimer |

Not a selected V1 exporter. **DEFER TO W0.3. REQUIRES POC.**

---

## 5. glTF findings

Authority: [SPEC] Khronos glTF 2.0 Specification, **version 2.0.1**, **2021-10-11**, https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html , accessed 2026-08-30.

### CONFIRMED — HIGH

**Coordinates:** right-handed; +Y up; +Z forward; −X right; front faces +Z; linear units **meters**. §3.4.

**Node transform:** a node MAY use `matrix` **or** any combination of TRS. Composition of TRS is `T * R * S`. Defaults: rotation `[0,0,0,1]`; identity if nothing authored. **When `matrix` is defined, it MUST be decomposable to TRS.** Implementation note: transformation matrices cannot skew or shear. When a node is an animation target, only TRS MAY be present; `matrix` MUST NOT be present. [SPEC]

**Skin:** required `joints` (node indices); optional `inverseBindMatrices` matching that order (or longer); optional `skeleton` = common root or ancestor. Missing IBM → each matrix **identity**. IBM fourth row MUST be `[0,0,0,1]`. Core skinning is linear blend. An asset MAY contain multiple `skins[]` and multiple skinned nodes. A **node** has at most one `skin` reference. A skinned mesh MAY have multiple primitives.

**Default node properties:** glTF does **not** define a rest-pose object. Un-animated properties remain at their authored/default values. Using those defaults as RigForge rest/reference is a later interpretation. [SPEC] / [RIGFORGE_INFERENCE]

**IBM [SPEC]:** `inverseBindMatrices[i]` corresponds to `joints[i]` and brings skinned coordinates into the same space as that joint.

**Bind Shape Matrix [SPEC] implementation note:** should be premultiplied into mesh data **or** into inverse bind matrices. Therefore `IBM == inverse(initial/global joint transform)` is **not** a universal invariant.

**IBM [OFFICIAL_DOC] tutorial:** SimpleSkin-style examples treat IBM as the inverse of the joint’s initial global transform.

**IBM [RIGFORGE_INFERENCE]:** that inverse formula is a validation check only when bind-shape/conversion assumptions are known.

**Animation:** `animations[]` of channels + samplers. Paths: translation, rotation (xyzw quat), scale, weights. Interpolation `LINEAR` (quat slerp SHOULD), `STEP`, `CUBICSPLINE` (in/out tangents; 3× outputs). Appendix C requires normalizing interpolated quaternions.

**Sampler time domain [SPEC] §3.11 / `animation.sampler.input`:**
- Input is a floating-point **scalar** accessor: linear time in **seconds**.
- `time[0] >= 0.0`, and values are **strictly increasing** (`time[n+1] > time[n]`).
- Inputs are relative to `t = 0`, the beginning of the parent `animations` entry.
- Before and after a sampler’s input range, output MUST clamp to the nearest endpoint.
- Samplers inside one animation **MAY have different inputs** (different input accessors / ranges).
- Animation input accessors MUST define `min` and `max`.

**Duration [SPEC]:** there is **no** `animation.duration` property. An application MAY derive an interval from sampler input ranges (`min`/`max`). That derivation is an application policy, not a glTF field. W1 must not pretend the spec stores an explicit clip duration.

**Missing channels:** non-animated properties MUST keep their authored/default node values. [SPEC]

### What glTF does **not** define — HIGH

Loop, playback policy, root motion, events, **portable clip-level additive**, semantic bone roles, retarget, helper-bone taxonomy. [SPEC] silence. (FBX layer additive is a different, source-specific fact.)

### Extensions (only motion-relevant)

| Extension | Kind | W0.1 note |
| --- | --- | --- |
| `VRMC_vrm` / `VRMC_vrm_animation` | vendor / consortium (VRM) | Track E |
| `KHR_animation_pointer` | Khronos `KHR` | Can animate arbitrary pointers; **not** deep-dived; DEFER W0.3 if ingest needs non-TRS targets |
| morph weights (core) | core | Blend-shape weights, not skeleton |

Ratification status of every KHR extension was **not** exhaustively audited. [OPEN]

### OPEN / REQUIRES_POC

- Negative scale + IBM interaction.
- Multiple `skins[]` / multiple skinned nodes / multi-primitive skinned meshes in the wild.
- Duration **derivation policy** for multi-sampler clips (union of sampler `min`/`max` vs extras). Spec fact is settled: no `animation.duration` field. Policy remains OPEN.

---

## 6. UsdSkel findings

Authority: [OFFICIAL_DOC] OpenUSD UsdSkel Intro + Schema Overview (release docs).

### CONFIRMED — HIGH

UsdSkel is an **interchange encoding** of simple skeletons, blend shapes, and skin bindings for games and crowds. **“What UsdSkel Is Not”:** it does **not** provide general rigging/execution as core USD. [OFFICIAL_DOC] Intro.

**Skeleton:** `joints` token paths = topology; parent by path; omitted intermediates skip; **multiple roots allowed**.

**`bindTransforms`:** world-space bind, joint order. Chosen to match DCC world bind.

**`restTransforms`:** **local** fallback for missing/sparse animation. Optional only if a **complete** animation is bound. Officially: rest often matches bind **but is not required to**.

**Animation:** separate `UsdSkelAnimation`; `joints` may be a **subset** and **reordered**; T/R/S vectorized; timesamples allowed. Binding `skel:animationSource` is **namespace-inherited**. Sparse **layering** of joint transforms is **not** supported. [OFFICIAL_DOC] Intro trade-offs.

**USD time domain (stage, not a UsdSkel-only object):**
- Animated values are authored on **TimeCode** ordinates. Those ordinates are **not** portable seconds without stage time-domain context. [OFFICIAL_DOC]
- `UsdStage::GetTimeCodesPerSecond()` scales TimeCode ordinates to real seconds. Example: if `timeCodesPerSecond = 24`, TimeCode 24 is one second after TimeCode 0. [OFFICIAL_DOC] [UsdStage](https://openusd.org/release/api/class_usd_stage.html)
- `framesPerSecond` is primarily an **advisory playback/display rate**. It normally does not affect TimeCode scaling when `timeCodesPerSecond` is set. [OFFICIAL_DOC] [Time and Animated Values](https://openusd.org/release/user_guides/time_and_animated_values.html)
- Fallback used to resolve `timeCodesPerSecond`: session `timeCodesPerSecond` → root `timeCodesPerSecond` → session `framesPerSecond` → root `framesPerSecond` → fallback **24**. [OFFICIAL_DOC] `UsdStage::GetTimeCodesPerSecond`
- Composition: when layers with different `timeCodesPerSecond` participate through supported arcs (sublayer, reference, payload), OpenUSD can **automatically rescale** TimeCode values into the containing stage’s time domain. [OFFICIAL_DOC] Time and Animated Values, “Automatic Scaling of timeCodesPerSecond”.
- `startTimeCode` / `endTimeCode` are primarily client playback-range metadata; they do **not** clip value resolution. [OFFICIAL_DOC]
Do **not** reduce USD to “24 fps”. A raw UsdSkel time-sample ordinate is not elapsed seconds without the resolved stage time domain. Final RigForge time representation remains **OPEN for W1**.

**Skin:** `jointIndices` / `jointWeights`; optional `skel:joints` remap; `geomBindTransform` world mesh bind, applied **before** skinning; xformable transform is **not** the skin bind.

**Spaces:** joint-local, skeleton space (no root world), world = local * parentSkel * skelWorld. Skinning transform `inv(bind)*jointSkel`. [OFFICIAL_DOC]

### Scope it does not provide — HIGH

Retarget ontology, DCC constraint networks, general rig controls, animation clip looping/additive policy.

### OPEN

USD spline / future SkelJoint-prim encoding (forum, 2024+) is **not** current interchange baseline. [PROJECT_CLAIM] AOUSD forum — ignore for V1.

---

## 7. VRM findings

### CONFIRMED — HIGH

VRM 1.0 is **glTF 2.0 plus extensions**. Humanoid is a **node→role map**. [SPEC] VRMC_vrm 1.0.

Required humanBones (schema `required`):
`hips, spine, head, leftUpperLeg, leftLowerLeg, leftFoot, rightUpperLeg, rightLowerLeg, rightFoot, leftUpperArm, leftLowerArm, leftHand, rightUpperArm, rightLowerArm, rightHand`. [SPEC] schema JSON.

Optional roles include chest, upperChest, neck, eyes, jaw, toes, shoulders, fingers (full list in schema). [SPEC]

`VRMC_vrm_animation` 1.0: maps humanoid / expressions / lookAt to glTF nodes; **animation data is core glTF animation** and therefore **inherits glTF sampler timing** (seconds, `t = 0` at the parent animation, clamp, no explicit duration field). Same clip intended to apply to **any** VRM humanoid. Expression weight = translation.x of a dedicated node, clamped [0,1]. LookAt uses local rotation as yaw/pitch (extrinsic ZXY). `leftEye`/`rightEye` **cannot** be defined in the animation humanoid map. [SPEC]

Companion `how_to_transform_human_pose.md` states **“This document is non-normative.”** Treat pose-compatibility / rest-rotation conversion guidance as [OFFICIAL_DOC] non-normative companion guidance, not [SPEC]. T-pose hierarchy may carry rest rotation according to that companion text. Normative humanoid/schema requirements above are unchanged.

### SPEC FACT vs inference

| [SPEC FACT] | [RIGFORGE_INFERENCE] |
| --- | --- |
| Humanoid bone set is finite and human-shaped | VRM cannot be the universal RigForge skeleton |
| Required bones are those 15 names | Quadrupeds, robots, mechanical rigs, helper-heavy DCC skeletons are out of profile |
| Extra glTF nodes may exist | Helpers are untyped unless another extension says so |
| Animation interoperability is humanoid-map based | Deterministic retarget still needs RigForge mapping/QC (W0.2) |

---

## 8. Newly discovered standards / projects

| Name | Why relevant | Class | In original R1 inventory? |
| --- | --- | --- | --- |
| ISO/IEC 19774:2019 HAnim (+ X3D HAnim component) | Mature **semantic** humanoid joint naming / LOA, motion data | `USEFUL_REFERENCE` for W0.2 ontology; not a V1 ingest target | **NEW DISCOVERY** |
| COLLADA 1.4/1.5 skin (`bind_shape_matrix`, `INV_BIND_MATRIX`) | Historical interchange; geom-bind analogue | `USEFUL_REFERENCE` / `HISTORICAL_REFERENCE` | **NEW DISCOVERY** (not listed) |
| BVH | Motion-only hierarchy + Euler channels | `HISTORICAL_REFERENCE` | **NEW DISCOVERY** |
| ASF / AMC | Academic mocap split skeleton/motion | `HISTORICAL_REFERENCE` | **NEW DISCOVERY** |
| KHR_animation_pointer | Non-TRS animation targets on glTF | `USEFUL_REFERENCE` — W0.3 if needed | Partial (glTF extensions not itemized) |
| Autodesk `ConvertPivotAnimationRecursive` | Official FBX bake path | `MUST_DEEP_DIVE` in W0-P / W0.3 adapter work | Implicit in FBX SDK, not named in R1 |

HAnim is **not** selected as Canonical. It is evidence that semantic bone names have an ISO home distinct from VRM and from interchange TRS.

---

## 9. Cross-format conclusions

1. **Rest ≠ bind ≠ IBM ≠ default node xform.** Only UsdSkel states the rest/bind split normatively. glTF has no rest-pose object; un-animated defaults remain in effect and IBM may include premultiplied bind-shape. FBX may have both pose tables and cluster matrices.
2. **Name ≠ Canonical identity.** Source-local keys (indices, token paths, node objects, humanoid roles) identify a joint inside one asset instance only.
3. **Joint order is a skin-binding contract**, not a universal skeleton property.
4. **Humanoid semantics are optional profile data** (VRM, later HAnim/engine maps), not Skeleton.
5. **Portable clip-level** loop / additive / root motion / events are **NOT DEFINED** across glTF, UsdSkel, and VRM. FBX **does** define layer-level additive/blend. Do not collapse those two facts.
6. **FBX is the transform-recipe outlier.** Canonical processing must not assume an FBX node’s evaluated world equals `ParentWorld × simple(LclTranslation, LclRotation, LclScaling)` while ignoring the remaining authored recipe. A Canonical that can only store simple parent-relative TRS is already an FBX-lossy cook unless the recipe or a loss report is retained.
7. **Time domains are not interchangeable.** glTF sampler input is already seconds. FBX keys are `FbxTime` plus a file/global frame/time mode. USD TimeCodes need resolved `timeCodesPerSecond` (and may have been rescaled by composition). Missing animation data must follow **source** fallback/evaluation semantics; identity is not a silent default. The final Canonical time representation is **OPEN**.

These conclusions **support** existing RigForge principles: Canonical-first; formats are adapters; VRM is not a universal schema; Canonical ≠ Runtime. [docs/architecture/README.md](../../architecture/README.md)

---

## 10. Semantic gaps

If an asset is forced into:

```text
joint hierarchy
+ local rest TRS
+ inverse bind matrices
+ per-joint sampled animation TRS
```

**Gap families:**

| ID | Family | Primary source | Lost information |
| --- | --- | --- | --- |
| G-FBX-PIVOT | Pivot / pre-post / offsets | FBX | Authored recipe; joint-orient; animatable pivot components |
| G-FBX-INHERIT | Inherit modes | FBX | Non-standard scale inheritance |
| G-FBX-GEOM | Geometric transform | FBX | Mesh-only offset; children unchanged |
| G-FBX-LAYER | Animation layers | FBX | Per-layer contribution; non-baked blend |
| G-FBX-CURVE | Curve math | FBX | Cubic tangents, Euler order, quaternion-interp flags |
| G-FBX-SKIN | Cluster extras | FBX | Associate model, DQ method, unnormalized weights, pose-table vs cluster conflict |
| G-GLTF-MAT | Matrix vs TRS authoring | glTF | Authored `matrix` vs TRS form; decomposition/canonicalization; negative-determinant policy. Valid glTF `matrix` MUST already be TRS-decomposable (no shear). |
| G-GLTF-CUBIC | CUBICSPLINE | glTF | Authored tangents if resampled to LINEAR |
| G-GLTF-RT | Runtime policy | glTF | loop / root motion / events (never present) |
| G-USD-BIND | Bind vs rest vs geomBind | UsdSkel | If IBM-only stored, rest/bind/geomBind collapse |
| G-USD-COMP | Composition | USD | Layering, instancing, inherited bindings |
| G-VRM-HUM | Humanoid map | VRM | Roles, expression encoding, lookAt, meta |
| G-AXIS | Units/axes | FBX/USD | If converted without recording method |
| G-TIME | Time domain | FBX / USD / glTF | Treating FBX keys as frame/fps, USD TimeCodes as seconds, or a derived glTF interval as an authored `duration` field |
| G-ID | Identity | all | Names as sole keys; skin order vs hierarchy |

---

## 11. Surviving assumptions

From product/architecture (not overwritten):

- Canonical is format-independent.
- Adapters wrap FBX/glTF/USD/VRM.
- Authoritative vs derived (bakes must be marked).
- Canonical vs runtime (sampled playback is a cook).
- DCC is not a mandatory ingest path (SDK/docs/ufbx path exists for FBX **research**; implementation language still unfrozen).
- VRM Humanoid is not a universal schema.
- Deterministic-first: portable runtime policies must not be smuggled in as shared format facts. FBX layer additive remains a source fact.

---

## 12. Challenged assumptions / keep OPEN

| Working idea | Status |
| --- | --- |
| “Bind pose” is one universal object | **Challenged** — keep OPEN as a product term; store the four facts separately |
| Simple parent-relative TRS is a lossless interchange core | **Challenged** for FBX authored form |
| Joint name is a stable id | **Challenged** |
| Rest pose = bind pose | **False as universal**; sometimes true in assets |
| Sampled joint TRS is a safe Canonical Motion | **Challenged** — allowed as **derived** |
| ufbx-write can publish FBX in V1 | **Unproven**; WIP |

No architecture ADR accepted.

---

## 13. Open questions

See also the eighteen numbered questions in §16 of the execution prompt — answers in §16 below.

Remaining:

- W0-P: cluster-vs-pose bind on real FBX; inherit/geometric files.
- W0.2: mapping/retarget inputs given these gaps.
- W0.3: reader/writer selection (ufbx vs FBX SDK vs write WIP).
- W1: axis/unit convention; whether authored FBX recipe is in-V1 Canonical or cook+report.

---

## 14. Decision implications (not decisions)

| Finding | Disposition |
| --- | --- |
| FBX pivot stack | `CONFIRMED FOUNDATION` + `CANONICAL REQUIREMENT CANDIDATE` (CR-TRANSFORM-001) |
| Rest/bind/IBM/geomBind split | `CONFIRMED FOUNDATION` + CR-SKIN-001 |
| Name ≠ identity | `CONFIRMED FOUNDATION` + CR-SKELETON-001/002 |
| VRM = profile | `CONFIRMED FOUNDATION` + CR-SEMANTICS-001 |
| Portable clip-level loop/additive/root motion | `OUT OF V1` as shared format facts; `OPEN FOR W1` as optional RigForge policy. FBX layer additive is a source-specific `CONFIRMED FOUNDATION`. |
| Sampled TRS as only Motion | `OPEN FOR W1` / `REQUIRES W0-P POC` |
| ufbx as reader | `DEFER TO W0.3` (capable, not selected) |
| ufbx-write | `DEFER TO W0.3` / high risk |
| HAnim | `DEFER TO W0.2` (semantics reference) |
| Final axes/units | `OPEN FOR W1` |
| Final Canonical time representation | `OPEN FOR W1` (seconds / ticks / rational / other — not selected) |
| Identity hash | `OPEN FOR W1` |

---

## 15. W0.1 result

Status for process: **COMPLETE** (accepted research baseline for later W0 stages; not Gate A / IA-1 / W1 ready).

Decision-gate inputs:

| Gate | Ready? |
| --- | --- |
| DG-1 Semantic coverage | Yes — surface identified; FBX bind equation needs PoC |
| DG-2 Transform readiness | Yes — major loss risks listed |
| DG-3 Rest/bind readiness | Yes — terms non-equivalent with citations |
| DG-4 Motion readiness | Yes — glTF seconds, FBX `FbxTime`/frame modes, USD TimeCode/`timeCodesPerSecond`, source-specific fallback, variable timestamps, interpolation, and layering are understood enough to ask W1 questions. **Canonical time representation remains OPEN.** |
| DG-5 Real-asset PoC readiness | Yes — SimpleSkin + InterpolationTest are `REPO_FIXTURE_CANDIDATE` (CC0 model files); Fox is `CC0-1.0 AND CC-BY-4.0`; OpenUSD HTML is `REFERENCE_ONLY` |

---

## 16. Required questions (short answers)

1. **Arbitrary FBX → simple TRS lossless?** No. [OFFICIAL_DOC] formula. Confidence HIGH.
2. **Default / rest / bind / IBM / reference?** See glossary. Not one thing. HIGH.
3. **Rest ≡ bind universally?** No. UsdSkel says not required. FBX has two pose kinds. glTF has no bind object. HIGH.
4. **Bind semantics differ?** FBX: pose table + clusters. glTF: IBM + mesh positions. UsdSkel: world bind + local rest + geomBind. HIGH.
5. **Name = identity?** No. HIGH.
6. **Joint order?** Skin IBM / influence index / UsdSkel array order. Hierarchy is separate. HIGH.
7. **Who encodes humanoid roles?** VRM natively. Others: not defined (HAnim/engines are later). HIGH.
8. **Generic skeletons?** FBX, glTF, UsdSkel yes. VRM profile no. HIGH.
9. **Coordinates?** glTF implicit-fixed. USD metadata. FBX per-file. HIGH/MEDIUM.
10. **Variable timestamps / time domain?** glTF sampler input = seconds (uneven allowed; no `animation.duration`). FBX keys are `FbxTime` (internal high-resolution integer), distinct from `EMode` frame addressing. UsdSkel uses USD TimeCodes scaled by resolved `timeCodesPerSecond`, not “24 fps” by default. HIGH.
11. **Interpolation precise?** glTF yes. FBX curves/tangents (complex). UsdSkel via USD, not a Skel enum. HIGH/MEDIUM.
12. **Layering?** FBX yes. glTF no. UsdSkel no (sparse subset ≠ layers). HIGH.
13. **Loop / root motion / events?** Not defined in glTF / UsdSkel / VRM cores. **Additive:** FBX layer-level native; no portable clip-level contract. HIGH.
14. **Safe immediate sample to joint TRS?** No, not without calling it derived/lossy. HIGH.
15. **Authoritative but likely V1-irrelevant?** FBX NURBS, geometry caches, selection sets, full constraint rigs; USD crowds instancing scale; VRM toon materials. MEDIUM.
16. **Needed later for deterministic retarget?** Hierarchy, rest, bind, IBM, units/axes, optional semantic roles, clip time domain, explicit missing-channel rule. [RIGFORGE_INFERENCE] — W0.2.
17. **Surviving assumptions?** §11.
18. **Revise / OPEN?** §12.

---

## Sources record (compact)

Each important source used above includes: name, type, URL, org, version, access date 2026-08-30, topic, claim, classification — expanded in-section rather than duplicated as a dump.

### Rev2 time-domain ledger

| Source | Authority | Version / context | URL | Access | Supported claim | Class |
| --- | --- | --- | --- | --- | --- | --- |
| `FbxTime` class reference | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_time.html | 2026-08-30 | Internal `FbxLongLong` Get/Set; `EMode` frame/time modes including drop vs full NTSC; `IsDropFrame`; used by global settings and animation classes | [OFFICIAL_DOC] |
| `FbxAnimCurve` class reference | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_anim_curve.html | 2026-08-30 | “The time unit in FBX (`FbxTime`) is 1/46186158000 of one second.” Curve keys are `FbxTime`. | [OFFICIAL_DOC] |
| `FbxGlobalSettings` class reference | Autodesk | FBX C++ API Reference 2020 | https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_global_settings.html | 2026-08-30 | Scene/file global `SetTimeMode` / `GetTimeMode` (`FbxTime::EMode`) | [OFFICIAL_DOC] |
| `UsdStage::GetTimeCodesPerSecond` | Pixar / OpenUSD | UsdStage API | https://openusd.org/release/api/class_usd_stage.html | 2026-08-30 | TimeCodes scale to seconds; example 24 → TimeCode 24 is one second after 0; fallback precedence including `framesPerSecond` then 24 | [OFFICIAL_DOC] |
| `UsdStage::GetFramesPerSecond` | Pixar / OpenUSD | UsdStage API | https://openusd.org/release/api/class_usd_stage.html | 2026-08-30 | Advisory playback/presentation rate; default 24 | [OFFICIAL_DOC] |
| Time and Animated Values | Pixar / OpenUSD | release user guide 26.08 | https://openusd.org/release/user_guides/time_and_animated_values.html | 2026-08-30 | `framesPerSecond` vs `timeCodesPerSecond`; same fallback order; automatic TimeCode rescale across composition arcs; `startTimeCode`/`endTimeCode` are playback-range metadata | [OFFICIAL_DOC] |
| glTF 2.0 Specification §3.11 / `animation.sampler.input` | Khronos | spec 2.0.1, 2021-10-11 | https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html | 2026-08-30 | Input = seconds; `time[0] >= 0`; strictly increasing; relative to parent animation `t = 0`; endpoint clamp; samplers MAY have different inputs; input accessor `min`/`max` MUST be defined; no `animation.duration` property | [SPEC] |
