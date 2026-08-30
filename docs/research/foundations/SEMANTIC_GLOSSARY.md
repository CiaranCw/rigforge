# Semantic Glossary — W0.1

Rev2: time-domain terms expanded. F1–F7 glossary corrections remain.

Source-aware terminology. Do not treat these as a frozen Canonical vocabulary.

Access date for all cited pages: 2026-08-30.

Evidence tags follow [docs/research/README.md](../README.md).

## How to read

Each term records:

- a **working sense** used in this research pass;
- **source-specific meanings** where they differ;
- **equivalence** only when evidence supports it.

`UNKNOWN` means the official text does not settle the term.

---

## Asset / Character

### asset

Working sense: a named bundle of scene objects that can be ingested as one provenance unit.

| Source | Meaning |
| --- | --- |
| glTF | Top-level `asset` object is metadata (version, generator, copyright). The file is the delivery unit. [SPEC] [Khronos glTF 2.0 Specification, version 2.0.1, 2021-10-11](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html) |
| USD | A layer / stage composition; not one “character file” contract. [OFFICIAL_DOC] |
| FBX | An `FbxScene` / file. No public ISO definition of “asset”. [OFFICIAL_DOC] |
| VRM | A glTF file plus `VRMC_vrm` (and related) extensions. [SPEC] |

### character

Working sense: a product-level grouping of skeleton + optional skins + optional motion + provenance.

No researched format defines a normative “Character” object equivalent to RigForge’s intended product object. [RIGFORGE_INFERENCE]

---

## Skeleton / joint / bone

### skeleton

| Source | Meaning |
| --- | --- |
| glTF | Not a first-class object. Optional `skin.skeleton` is a node that is the common root or an ancestor of the joint hierarchy. [SPEC] |
| UsdSkel | `UsdSkelSkeleton`: vectorized joint tokens + `bindTransforms` + `restTransforms`. [OFFICIAL_DOC] [UsdSkel Schema Overview](https://openusd.org/release/api/_usd_skel__schema_overview.html) |
| FBX | No single skeleton object. Joints are nodes, often with a bone attribute; skin clusters reference link nodes. [OFFICIAL_DOC] ufbx: bone attribute is **not** required for skinning. [OFFICIAL_DOC] [ufbx Deformers](https://ufbx.github.io/elements/deformers/) |
| VRM | Humanoid bone map over glTF nodes. Not a generic skeleton schema. [SPEC] |

### joint

Working sense: a hierarchical transform used for skinning and/or posing.

| Source | Meaning |
| --- | --- |
| glTF | A node listed in `skin.joints`. [SPEC] |
| UsdSkel | An entry in `Skeleton.joints` (token path). Not necessarily a scene-graph prim. [OFFICIAL_DOC] |
| FBX | Commonly a node used as a cluster link. “Joint” is DCC language, not a unique FBX class name. [OFFICIAL_DOC] |
| VRM | A glTF node mapped to a named humanoid bone. [SPEC] |

### bone

Often used interchangeably with joint in DCC talk. In FBX/ufbx, `ufbx_bone` is an optional node attribute used for visualization / discovery, not the skinning contract. [OFFICIAL_DOC] [ufbx Deformers](https://ufbx.github.io/elements/deformers/)

### joint hierarchy

Parent/child relation used to concatenate local transforms.

UsdSkel encodes topology in joint token paths; intermediate missing tokens skip to the next authored ancestor. Multiple root joints are valid. [OFFICIAL_DOC]

glTF hierarchy is the node `children` graph. Joints must be nodes in that graph. [SPEC]

FBX hierarchy is the node parent/child graph. Scene may have multiple roots. [OFFICIAL_DOC]

### joint identity vs joint name

These are **source-local reference keys**, not a future RigForge Canonical ID.

| Source | Source-local key | Name |
| --- | --- | --- |
| glTF | Node index within one asset instance; `name` is optional metadata. [SPEC] |
| UsdSkel | Joint token / relative path in `joints`; order is significant for arrays. [OFFICIAL_DOC] |
| FBX | Node object in the scene; names are not guaranteed unique. [OFFICIAL_DOC] / [RIGFORGE_INFERENCE] |
| VRM | Humanoid role (`hips`, …) **plus** the mapped glTF node index in that file. [SPEC] |

**Joint name alone is not identity in any of these standards.** A source-local key is not a stable Canonical identity across import/export/versioning. [RIGFORGE_INFERENCE] Canonical ID algorithm remains OPEN.

### joint order

glTF: `skin.joints` order MUST match `inverseBindMatrices` order. [SPEC]

UsdSkel: `bindTransforms` / `restTransforms` / skin indices follow `joints` (or an explicit `skel:joints` remap). [OFFICIAL_DOC]

FBX: cluster list order is an implementation/file order; not a normative “skeleton order”. [OFFICIAL_DOC]

### root / multiple roots

UsdSkel: multiple root joints are allowed; the Skeleton prim is the “true root”. [OFFICIAL_DOC]

glTF: `skin.skeleton` hints at a common root; multiple disconnected joint trees are not a first-class concept. [SPEC] / [RIGFORGE_INFERENCE]

FBX: multiple scene roots are normal. [OFFICIAL_DOC]

### semantic joint / helper / twist / deform / IK / end

None of FBX, glTF, or UsdSkel define a normative taxonomy for helper / twist / IK / end joints. [SPEC] / [OFFICIAL_DOC]

VRM defines **humanoid roles** only (required + optional human bones). Extra glTF nodes may exist outside that map. [SPEC]

---

## Transforms

### local transform

Transform relative to the parent.

glTF: node `matrix` **or** TRS; composed `T * R * S` (scale, then rotate, then translate). If `matrix` is present it MUST be decomposable to TRS. Valid glTF matrices cannot skew or shear. Animated nodes: TRS only; `matrix` MUST NOT be present. [SPEC] glTF 2.0.1 § transformations / node.

UsdSkel joint animation / `restTransforms`: joint-local. [OFFICIAL_DOC]

FBX evaluated local TRS is **not** the full authored node formula. [OFFICIAL_DOC] See [W0_1_CANONICAL_FOUNDATIONS.md](W0_1_CANONICAL_FOUNDATIONS.md).

### global / world transform

Concatenation through parents (plus format-specific extra terms).

UsdSkel world joint = local * parent-skel * skeleton world. [OFFICIAL_DOC] [UsdSkel Intro](https://openusd.org/release/api/_usd_skel__intro.html)

### translation / rotation / scale / TRS / matrix

glTF rotation is unit quaternion XYZW. [SPEC]

UsdSkel animation stores `translations`, `rotations` (quat), `scales`; local basis is expected orthogonal; non-uniform scale is discouraged except reflections. [OFFICIAL_DOC]

FBX rotation is typically Euler with a rotation order; pre/post rotation are extra Euler terms. [OFFICIAL_DOC]

### joint orientation / pre-rotation / post-rotation / pivot / geometric transform

**FBX-native.** Official node formula (FBX/Maya):

`World = ParentWorld * T * Roff * Rp * Rpre * R * Rpost⁻¹ * Rp⁻¹ * Soff * Sp * S * Sp⁻¹`

[OFFICIAL_DOC] Autodesk FBX Developer Help 2018, [Computing transformation matrices](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/nodes_and_scene_graph/fbx_nodes/computing_transformation_matrix.html)

Geometric T/R/S apply to the node **attribute** only and are **not inherited**. [OFFICIAL_DOC]

glTF / UsdSkel: these terms are **NOT DEFINED** as first-class authored channels. [SPEC] / [OFFICIAL_DOC]

### inherit modes

FBX: `eInheritRrSs`, `eInheritRSrs`, `eInheritRrs`. [OFFICIAL_DOC] `FbxNode::SetTransformationInheritType`

ufbx exposes `inherit_mode` and load-time conversion options. [OFFICIAL_DOC] [ufbx Nodes](https://ufbx.github.io/elements/nodes/)

glTF / UsdSkel: standard parent × local composition. Inherit modes **NOT DEFINED**. [SPEC] / [OFFICIAL_DOC]

---

## Rest / bind / inverse bind

These are **not synonyms**.

### default / reference transform

glTF: if a node has no TRS/matrix, transform is identity; missing animation channels leave the node at its authored default. [SPEC]

FBX: node default property values (Lcl T/R/S plus pivot stack) before evaluation. [OFFICIAL_DOC]

### rest pose / rest transform

UsdSkel: `restTransforms` = **local** fallback when animation is missing or sparse. Officially **not required** to match bind. [OFFICIAL_DOC]

FBX: `FbxPose` may store “Rest Pose” data, distinct from “Bind Pose”. Rest pose matrices may be marked local or global. [OFFICIAL_DOC] [FbxPose](https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_pose.html)

glTF: does **not** define a rest-pose object. When a property is not animated, the authored/default node property remains in effect. Treating that default as RigForge rest/reference is a later interpretation, not a glTF definition. [SPEC] / [RIGFORGE_INFERENCE]

### bind pose / bind transform

UsdSkel: `bindTransforms` = **world** joint transforms at bind time. [OFFICIAL_DOC]

FBX: (1) optional `FbxPose` bind-pose tables; (2) per-cluster `Transform` / `TransformLink` matrices. These can disagree or be missing. [OFFICIAL_DOC]

glTF: no bind-pose object. Bind is implied by inverse bind matrices + mesh in its stored positions. [SPEC]

### inverse bind matrix

glTF [SPEC]: `skin.inverseBindMatrices[i]` corresponds to `skin.joints[i]` and is used to bring coordinates being skinned into the same space as that joint. If the accessor is omitted, each matrix is identity.

glTF [SPEC] implementation note: a Bind Shape Matrix may be premultiplied into mesh data **or** into inverse bind matrices.

glTF tutorial [OFFICIAL_DOC]: in the SimpleSkin-style example, IBM is the inverse of that joint’s initial global transform. That is a **controlled-example** relationship, not a universal invariant.

[RIGFORGE_INFERENCE] `IBM == inverse(default-global-joint)` is only a validation formula when bind-shape/conversion assumptions are known.

UsdSkel: consumers derive IBM from world `bindTransforms` (and geom bind). Skinning uses `inv(bind) * jointSkel`. [OFFICIAL_DOC]

FBX/ufbx: cluster `geometry_to_bone` / TransformLink relationship is the practical IBM analogue. [OFFICIAL_DOC] [ufbx Deformers](https://ufbx.github.io/elements/deformers/)

### geometry bind

UsdSkel: `primvars:skel:geomBindTransform` = world transform of the skinned prim at bind; points are transformed by it **before** skinning. Distinct from `UsdGeomXformable`. [OFFICIAL_DOC]

COLLADA: `bind_shape_matrix` (historical analogue). [SPEC] COLLADA 1.4/1.5

glTF: no separate geom-bind matrix; mesh positions are already in node/mesh space. [SPEC]

FBX: cluster `Transform` is the global transform of the **geometry node** at bind. [OFFICIAL_DOC] [FbxCluster](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/cpp_ref/class_fbx_cluster.html)

### T-pose / A-pose

Authoring conventions. Not defined by glTF, UsdSkel, or FBX SDK as normative pose types. VRM discussion of T-pose appears in animation compatibility notes; hierarchy may have arbitrary rest rotation. [SPEC] `VRMC_vrm_animation` 1.0

---

## Skin

### skin / skin cluster / skin weight / joint influence

glTF: `skins[]` + `JOINTS_n` / `WEIGHTS_n` on primitives. Linear blend skinning. Multiple skins allowed. [SPEC]

UsdSkel: `jointIndices` / `jointWeights` primvars; `elementSize` = influences per point. [OFFICIAL_DOC]

FBX: `FbxSkin` + `FbxCluster` (control points + weights). Skinning method may be linear or dual-quaternion. Weights are **not** guaranteed normalized. [OFFICIAL_DOC] ufbx. [OFFICIAL_DOC]

---

## Motion

### motion / animation clip / stack / layer / take

| Source | Clip analogue | Layering |
| --- | --- | --- |
| glTF | `animations[]` entries | NOT DEFINED [SPEC] |
| FBX | Animation stack / take | Layers inside a stack [OFFICIAL_DOC] ufbx |
| UsdSkel | `UsdSkelAnimation` bound via `skel:animationSource` | Sparse **joint subset**, not layered blending [OFFICIAL_DOC] |
| VRM | glTF animations + `VRMC_vrm_animation` map | inherits glTF [SPEC] |

### track / channel / curve / keyframe

glTF: `animation.channels` target a node path (`translation`/`rotation`/`scale`/`weights`); `samplers` hold input times + outputs. [SPEC]

FBX: curve nodes + curves on properties (including Euler axes). [OFFICIAL_DOC]

UsdSkel: vectorized T/R/S arrays, optionally time-sampled. [OFFICIAL_DOC]

### sample time / sample rate / variable timestamps / interpolation

These are **not** one time domain.

**glTF [SPEC] 2.0.1 §3.11 / `animation.sampler.input`:** floating-point scalar **seconds**; `time[0] >= 0.0` and strictly increasing; relative to `t = 0` at the parent animation; output clamps to the nearest endpoint outside the sampler range; samplers in one animation MAY use different inputs; input accessors MUST define `min`/`max`. There is **no** `animation.duration` field. An application MAY derive an interval from those ranges. Timestamps may be uneven. Interpolation: `LINEAR` / `STEP` / `CUBICSPLINE`. Quaternion LINEAR uses slerp (SHOULD).

**FBX [OFFICIAL_DOC]:** curve-key time is `FbxTime`, stored as a 64-bit internal integer (`FbxLongLong` `Set`/`Get`), **not** an integer frame number. Autodesk documents the `FbxTime` unit as 1/46,186,158,000 second. [FbxAnimCurve](https://help.autodesk.com/cloudhelp/2020/ENU/FBX-API-Reference/cpp_ref/class_fbx_anim_curve.html) Elapsed seconds come from that fixed internal mapping (`GetSecondDouble()`); `EMode` is **not** required for that decode. Display / frame / time-code addressing is `FbxTime::EMode` (including drop-frame / NTSC modes where time-code address is not clock seconds) and remains source provenance. File/global settings may carry that mode (`FbxGlobalSettings::GetTimeMode`). Curves may be cubic with tangents.

**UsdSkel / USD [OFFICIAL_DOC]:** animation uses USD **TimeCode** ordinates. `UsdStage::GetTimeCodesPerSecond()` maps those ordinates to real seconds (example: `timeCodesPerSecond = 24` ⇒ TimeCode 24 is one second after TimeCode 0). `framesPerSecond` is primarily advisory playback/display rate and is only a fallback when `timeCodesPerSecond` is unset (session TCPS → root TCPS → session FPS → root FPS → 24). Composed layers with different `timeCodesPerSecond` can have TimeCodes automatically rescaled into the containing stage. Interpolation follows USD value resolution, not a glTF-style enum on `UsdSkelAnimation`.

**VRM [SPEC]:** inherits glTF animation timing.

The future Canonical storage unit (float seconds, integer ticks, rational time, …) is **not** chosen here.

### clip duration / playback range

glTF: **NOT DEFINED** as a property. Derived interval ≠ authored duration field. [SPEC]

FBX: stacks/curves expose `FbxTime` intervals via the SDK (`FbxTimeSpan`); this is not a portable clip-duration field shared with glTF. [OFFICIAL_DOC]

USD: `startTimeCode` / `endTimeCode` are primarily client playback-range metadata and do not limit value resolution. [OFFICIAL_DOC]

### missing animation data / missing channel

Working sense: a joint or property with no authored samples for some or all transform channels.

Fallback is **source-defined**. Do not silently write identity unless that source says identity.

| Source | Defined fallback |
| --- | --- |
| glTF | Non-animated property remains at the authored/default node property. [SPEC] |
| UsdSkel | Unmapped / unanimated joints fall back to `restTransforms` through the resolved query. [OFFICIAL_DOC] |
| FBX | Absence of a layer/curve must respect FBX stack / layer / property evaluation. Do not force glTF or UsdSkel fallback onto FBX. [OFFICIAL_DOC] / [RIGFORGE_INFERENCE] |
| VRM | Inherits glTF. [SPEC] |

### root motion / additive / loop / events

Root motion, loop, events: **NOT DEFINED** as interchange semantics in glTF 2.0.1 core, UsdSkel, or VRM 1.0 humanoid. [SPEC] / [OFFICIAL_DOC]

Additive: FBX has **native layer-level** additive/blend (`FbxAnimLayer::EBlendMode`, including `eBlendAdditive`, plus rotation/scale accumulation modes). [OFFICIAL_DOC] https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_anim_layer.html

There is **no** portable clip-level additive contract shared by FBX, glTF, UsdSkel, and VRM. [RIGFORGE_INFERENCE]

FBX may store custom properties / markers; no researched normative “root motion” or “additive clip” contract. [UNKNOWN] / [RIGFORGE_INFERENCE]

---

## Coordinates

| | glTF | USD stage | FBX |
| --- | --- | --- | --- |
| Handedness | right-handed [SPEC] | typically right-handed; not restated as a Skel rule [OFFICIAL_DOC] | file-dependent [OFFICIAL_DOC] ufbx |
| Up | +Y [SPEC] | `upAxis` metadata (example uses Y) [OFFICIAL_DOC] | file-dependent [OFFICIAL_DOC] |
| Forward | +Z, asset faces +Z [SPEC] | not a Skel normative [OFFICIAL_DOC] | file-dependent |
| Units | meters [SPEC] | `metersPerUnit` (example `.01` = cm) [OFFICIAL_DOC] | often cm de-facto; `unit_meters` in ufbx [OFFICIAL_DOC] |

VRM inherits glTF coordinates. [SPEC]

---

## Provenance

glTF `asset.generator` / `copyright` / `extras`. [SPEC]

USD layer metadata / composition arcs. [OFFICIAL_DOC]

FBX scene info / exporter metadata (ufbx `metadata.exporter`). [OFFICIAL_DOC]

None of these are a RigForge publish manifest. [RIGFORGE_INFERENCE]
