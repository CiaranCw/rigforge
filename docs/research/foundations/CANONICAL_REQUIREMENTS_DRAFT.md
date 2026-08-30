# Canonical Requirements Draft — W0.1

Closeout: MUST audit corrected to 16 → 17; CR-MOTION-007 identity-key membership left OPEN. Time-domain provenance remains MUST.

**This is not W1. This is not a schema.**

No structs, fields, serialization, or language choices.

Classification: `MUST` / `SHOULD` / `OPEN`.

Evidence tags follow [docs/research/README.md](../README.md). Access date: 2026-08-30.

---

## CR-ASSET

### CR-ASSET-001 — MUST

The future Canonical layer must represent a **provenance-bearing ingest unit** distinct from any one file format’s top-level object.

Evidence: glTF `asset` is metadata, not a character; USD is a composed stage; FBX is a scene. [SPEC] / [OFFICIAL_DOC]

Reason: RigForge product objects include Character + Manifest. Format containers are adapters.

### CR-ASSET-002 — SHOULD

Canonical ingest should record source format, source version/generator when present, and original unit/axis metadata.

Evidence: glTF `asset.generator`; ufbx `metadata.exporter` and scene axes/units; USD `metersPerUnit` / `upAxis`. [SPEC] / [OFFICIAL_DOC]

### CR-ASSET-003 — OPEN

Whether one Canonical Character may bind multiple skeletons / multiple skins as first-class V1.

Evidence: all four formats allow multiple skins or multiple bound prims. Product V1 scope is unfrozen. [SPEC] / [OFFICIAL_DOC]

---

## CR-TRANSFORM

### CR-TRANSFORM-001 — MUST

Canonical processing must **not** assume that an FBX node’s evaluated world transform can be reconstructed as `ParentWorld × simple(LclTranslation, LclRotation, LclScaling)` while ignoring the remaining authored transform recipe.

The Autodesk relation is conceptually `WorldTransform = ParentWorldTransform × authored local FBX transform recipe`, where that local recipe includes `T`, rotation offset, rotation pivot, pre-rotation, rotation, post-rotation inverse, scale offset, scale pivot, and scale, plus inheritance behavior. Geometric T/R/S are not inherited.

Do not confuse authored Lcl T/R/S properties, the authored local recipe, evaluated local transform, and evaluated world transform.

Evidence: Autodesk documents the FBX/Maya product `T * Roff * Rp * Rpre * R * Rpost⁻¹ * Rp⁻¹ * Soff * Sp * S * Sp⁻¹`. Geometric T/R/S are not inherited. Inherit types exist. [OFFICIAL_DOC] [Computing transformation matrices](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/nodes_and_scene_graph/fbx_nodes/computing_transformation_matrix.html)

### CR-TRANSFORM-002 — MUST

Canonical design must distinguish **authored transform recipe** from **evaluated local/world TRS**.

Evidence: Autodesk `ConvertPivotAnimationRecursive` exists to bake pivot stack into T/R/S. ufbx documents that using `local_transform` / `evaluate_transform` / `bake_anim` hides internal complexity and is lossy relative to the recipe. [OFFICIAL_DOC]

### CR-TRANSFORM-003 — MUST

Coordinate convention (handedness, up, forward, units) must be an explicit Canonical concern, not an implicit “whatever the first file used”.

Evidence: glTF fixes right-handed Y-up, +Z forward, meters. [SPEC] FBX axes/units vary; ufbx conversion methods are not unique. [OFFICIAL_DOC] USD `upAxis` / `metersPerUnit`. [OFFICIAL_DOC]

### CR-TRANSFORM-004 — OPEN

The actual RigForge storage convention (axis, units, quaternion layout, matrix layout) remains **OPEN for W1**.

Reason: no repository contract freezes Y-up / meters / centimeters.

### CR-TRANSFORM-005 — SHOULD

Canonical should be able to record negative / non-uniform scale as a **detected** property, even if V1 retarget refuses it.

Evidence: glTF allows scale including negative; UsdSkel warns that non-uniform joint scale is poorly portable. [SPEC] / [OFFICIAL_DOC]

### CR-TRANSFORM-006 — OPEN

Whether V1 Canonical stores FBX inherit modes and geometric transforms natively, or only evaluated results plus a loss report.

Requires W0-P on real FBX files.

---

## CR-SKELETON

### CR-SKELETON-001 — MUST

Canonical Skeleton must preserve a **stable hierarchy** independent of display names.

Evidence: glTF source-local key is node index; UsdSkel source-local key is token path; FBX names are not specified unique; VRM maps a role onto a node index. None of these is a stable Canonical ID. [SPEC] / [OFFICIAL_DOC]

### CR-SKELETON-002 — MUST

Joint identity must be distinct from (a) display name, (b) semantic role, (c) skin-influence index.

Evidence: UsdSkel animation `joints` order need not match Skeleton `joints`. glTF `skin.joints` order is a skin contract, not a name. VRM maps roles onto nodes that still have indices/names. [SPEC] / [OFFICIAL_DOC]

### CR-SKELETON-003 — MUST

Canonical must allow **generic skeletons** with no humanoid taxonomy.

Evidence: glTF, FBX, UsdSkel all encode arbitrary joint graphs. VRM humanoid is a profile, not a universal model. [SPEC] / [OFFICIAL_DOC] Survives [docs/product/V1_SCOPE.md](../../product/V1_SCOPE.md) (“VRM Humanoid as universal schema” is non-core).

### CR-SKELETON-004 — MUST

Canonical must allow **multiple roots** or an explicit “forest” statement.

Evidence: UsdSkel permits multiple root joints. FBX scenes commonly have multiple roots. [OFFICIAL_DOC]

### CR-SKELETON-005 — SHOULD

Canonical should retain source node identifiers (FBX node, glTF index, USD path) as provenance, not as the only identity.

### CR-SKELETON-006 — OPEN

Concrete identity algorithm (hash, UUID, stable path) is **not** designed in W0.1.

---

## CR-SKIN

### CR-SKIN-001 — MUST

Canonical must treat **rest**, **bind**, **inverse bind**, and **geometry bind** as separable facts.

Evidence: UsdSkel states rest may differ from bind. [OFFICIAL_DOC] glTF has no rest-pose or bind-pose object; it stores IBM and default node properties, and a Bind Shape Matrix may be premultiplied into mesh data or IBMs. [SPEC] FBX has Pose objects **and** cluster Transform/TransformLink. [OFFICIAL_DOC]

### CR-SKIN-002 — MUST

Canonical must record joint-influence **order** as used by each skin, separately from hierarchy order.

Evidence: glTF `joints` MUST match IBM order. [SPEC] UsdSkel may remap via `skel:joints`. [OFFICIAL_DOC]

### CR-SKIN-003 — SHOULD

Canonical should record whether weights were normalized at source, and whether dual-quaternion (or blended) skinning was requested.

Evidence: ufbx: FBX weights not guaranteed normalized; skinning_method includes DQ. [OFFICIAL_DOC] glTF core is LBS only. [SPEC]

### CR-SKIN-004 — OPEN

V1 skinning evaluation completeness (DQ, in-between blend shapes, geometry caches) vs inspect-only.

---

## CR-MOTION

### CR-MOTION-001 — MUST

Canonical Motion must not assume a single clip model. It must be able to name the source clip analogue (glTF animation, FBX stack, UsdSkelAnimation).

Evidence: glTF `animations[]`; FBX animation stacks; UsdSkelAnimation prims. [SPEC] / [OFFICIAL_DOC]

### CR-MOTION-002 — MUST

Immediate reduction of all source animation to sampled joint TRS must be treated as a **lossy cook**, not as silent Canonical equality.

Loss surface: FBX cubic/Euler/layers/pivots; glTF CUBICSPLINE tangents and non-transform extras; USD composition/time-code domain. [OFFICIAL_DOC] / [SPEC]

### CR-MOTION-003 — MUST

Missing animation data must follow the **source format’s defined fallback/evaluation semantics**. RigForge must never silently substitute identity unless the source semantics explicitly require identity.

Source examples (not a single shared fallback object):

- **glTF:** a non-animated property remains at the authored/default node property. [SPEC]
- **UsdSkel:** unmapped / unanimated joints fall back to `restTransforms` through the resolved query. [OFFICIAL_DOC]
- **FBX:** layer/curve absence must respect FBX stack / layer / property evaluation. Do not force glTF or UsdSkel rest-fallback semantics onto FBX. Detailed layered transfer remains W0.3 / W0-P.

Evidence: glTF “Non-animated properties MUST keep their values during animation.” [SPEC] UsdSkel rest as sparse-animation fallback. [OFFICIAL_DOC] FBX animation is stack → layer → curve evaluation, not a rest-pose object. [OFFICIAL_DOC]

### CR-MOTION-004 — SHOULD

Canonical should retain interpolation mode when the source defines it (glTF sampler interpolation).

### CR-MOTION-005 — OPEN

Whether V1 Canonical stores unevaluated curves vs evaluated samples vs both (authoritative vs derived). Aligns with existing architecture principle Canonical ≠ Runtime. [RIGFORGE_INFERENCE]

### CR-MOTION-006 — OPEN

Do not assume one universal source-independent additive / loop / root-motion / event model.

Evidence split:

- FBX: native **layer-level** additive and blend (`FbxAnimLayer::EBlendMode`, rotation/scale accumulation). [OFFICIAL_DOC]
- glTF 2.0.1, UsdSkel, VRM: no portable **clip-level** additive, loop, root-motion, or event contract. [SPEC] / [OFFICIAL_DOC]

If V1 needs a cross-format additive model, it is an explicit future RigForge (or adapter) contract, not a recovered shared format fact. [RIGFORGE_INFERENCE]

### CR-MOTION-007 — MUST

**Priority change (Rev2): SHOULD → MUST.**

Source time domain, and any conversion into a normalized Canonical time domain, must be **explicit and provenance-bearing**. Silent “frame / fps” or “USD is 24 fps” conversion is a semantic error.

This time-semantics requirement is MUST. The exact membership of a Canonical Motion identity key remains **OPEN for W1**.

Future Motion identity / provenance design must **account for** these candidate distinguishers (not a frozen key):

- source asset
- source skeleton binding
- source time-domain metadata
- conversion rule
- duration derivation rule
- track set
- source clip name
- provenance/version

Canonical MUST NOT assume:

- FBX key times are integer frames divided by fps;
- FBX `EMode` is required to decode `FbxTime` into elapsed seconds (`GetSecondDouble()` is the documented seconds mapping; `EMode` is frame/time-code addressing provenance);
- USD TimeCode ordinates are seconds without resolved stage `timeCodesPerSecond` (and composition rescale) context;
- glTF stores an explicit `animation.duration` field.

The final Canonical time representation (float seconds, integer ticks, rational time, nanoseconds, or other) remains **OPEN for W1**.

Evidence: `FbxTime` has a fixed internal time-to-seconds mapping (`GetSecondDouble()` / documented unit). `EMode` is separate display/frame addressing, including drop-frame ≠ clock seconds. [OFFICIAL_DOC] `UsdStage::GetTimeCodesPerSecond` vs advisory `framesPerSecond`, fallback chain, composition rescale. [OFFICIAL_DOC] glTF sampler input is seconds with required accessor `min`/`max` and no duration property. [SPEC]

Reason for MUST: after F8, time-domain conversion is a foundation of Motion ingest, not optional identity polish. Without recorded conversion, FBX and USD times are not comparable to glTF seconds.

---

## CR-SEMANTICS

### CR-SEMANTICS-001 — MUST

Semantic humanoid roles are an **optional mapping layer**, not the Skeleton itself.

Evidence: only VRM (of the four) natively encodes humanoid roles. [SPEC] Product vision already lists Semantic Mapping as a separate object.

### CR-SEMANTICS-002 — MUST

Canonical must not require VRM-required bones for non-humanoid assets.

Evidence: VRM schema `required` humanBones are a humanoid profile. [SPEC] [humanBones schema](https://raw.githubusercontent.com/vrm-c/vrm-specification/master/specification/VRMC_vrm-1.0/schema/VRMC_vrm.humanoid.humanBones.schema.json)

### CR-SEMANTICS-003 — OPEN

Helper / twist / IK classification has **no** interchange standard among FBX/glTF/UsdSkel. Deferred to W0.2 mapping research.

---

## CR-PROVENANCE

### CR-PROVENANCE-001 — MUST

Derived representations (baked TRS, space-converted scenes, helper-node rewrites) must be marked derived.

Evidence: ufbx load options insert helper nodes or bake geometry/pivots. [OFFICIAL_DOC] Autodesk pivot bake. [OFFICIAL_DOC]

### CR-PROVENANCE-002 — SHOULD

Record conversion choices (axis conversion method, unit bake vs root scale, IBM identity-fill, source time-domain → any normalized Canonical time-domain conversion).

---

## Transform Convention Decision Inputs (not a decision)

Inputs W1 must weigh:

1. glTF’s fixed right-handed Y-up, +Z forward, meters. [SPEC]
2. FBX’s unstable exporter conventions and cm default. [OFFICIAL_DOC] ufbx
3. USD stage `upAxis` / `metersPerUnit`. [OFFICIAL_DOC]
4. Engine destinations (Unreal cm Z-up; Unity Y-up) — **not researched in W0.1**; W0.3/W0.4.
5. Quaternion XYZW (glTF) vs other layouts.
6. Whether Canonical stores one convention and adapters convert, vs storing source convention + a declared Canonical convention.

**No axis/unit is selected.**

---

## Skeleton / Motion identity — requirements only

Identity **needs** (not algorithms):

Skeleton distinguishers: source asset id, source node/path/index set, hierarchy, rest, bind, skin order, optional semantic map id.

Motion candidate distinguishers (exact identity-key membership OPEN): source asset, bound skeleton, clip name/stack, source time domain + conversion provenance, track coverage, version/provenance.

Name collisions and retarget reuse are why names are insufficient. [RIGFORGE_INFERENCE]

---

## Rev2 MUST audit (closeout-corrected)

CR-MOTION-007 was a SHOULD in Rev1 and became MUST in Rev2. It is counted only as an upgrade, not also as a rewritten existing MUST.

| Metric | Count |
| --- | --- |
| MUST before Rev2 | 16 |
| MUST after Rev2 | 17 |

| Change | IDs |
| --- | --- |
| Retained unchanged MUST | 14 — CR-ASSET-001; CR-TRANSFORM-002, 003; CR-SKELETON-001–004; CR-SKIN-001, 002; CR-MOTION-001, 002; CR-SEMANTICS-001, 002; CR-PROVENANCE-001 |
| Rewritten existing MUST | CR-TRANSFORM-001 (local/world wording); CR-MOTION-003 (source-defined fallback) |
| Upgraded SHOULD → MUST | CR-MOTION-007 |
| Added | none |
| Removed | none |
| Downgraded | none |

Actual headings matching `### CR-... — MUST`: **17**.

No MUST was added without official evidence. Final Canonical time representation remains OPEN. Exact Motion identity-key membership remains OPEN.
