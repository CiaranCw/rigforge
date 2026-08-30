# Format Semantic Matrix — W0.1

Rev2: time-domain rows refined. F1–F7 matrix cells remain.

Compare interchange representations. Cells are **not** product capabilities.

Access date: 2026-08-30.

## Cell legend

| Token | Meaning |
| --- | --- |
| `NATIVE` | First-class authored concept in the format |
| `REPRESENTABLE` | Can be stored using other native constructs without a dedicated object |
| `PARTIAL` | Only some cases, or only after evaluation/bake |
| `NOT DEFINED` | Spec/docs do not define the concept |
| `NOT APPLICABLE` | Outside the format’s stated scope |
| `UNKNOWN` | Official text is insufficient |

VRM column is **VRM 1.0 (`VRMC_vrm` + glTF 2.0)**, not a generic skeleton format.

---

## Matrix

| Topic | FBX | glTF 2.0 | UsdSkel | VRM 1.0 |
| --- | --- | --- | --- | --- |
| Scene hierarchy | `NATIVE` node graph; multiple roots | `NATIVE` node graph | `NATIVE` USD prims; joints are tokens on Skeleton, not required as prims | inherits glTF |
| Source-local joint key | `PARTIAL` node object; name not unique | `NATIVE` node index within one asset instance; `name` optional | `NATIVE` joint token path + array index | `NATIVE` humanoid role + node index within one asset |
| Joint naming | `NATIVE` node name | `REPRESENTABLE` optional `name` | `REPRESENTABLE` last path component | `NATIVE` role names; node `name` still optional |
| Joint hierarchy | `NATIVE` parent/child nodes | `NATIVE` `children` | `NATIVE` token-path topology; skipped intermediates allowed | inherits glTF; humanoid hierarchy rules extra |
| Rest pose | `PARTIAL` default TRS + optional Rest `FbxPose` | `NOT DEFINED` as an object; un-animated/default node properties remain in effect | `NATIVE` local `restTransforms` (optional if animation complete) | inherits glTF defaults |
| Bind pose | `PARTIAL` `FbxPose` bind + cluster matrices; not one authority | `NOT DEFINED` as an object | `NATIVE` world `bindTransforms` | inherits glTF (no bind object) |
| Inverse bind | `REPRESENTABLE` cluster link/geometry matrices; ufbx `geometry_to_bone` | `NATIVE` `inverseBindMatrices` (optional → identity) | `REPRESENTABLE` derived from bind | inherits glTF |
| Geometry bind | `NATIVE` cluster `Transform` (mesh global at bind) | `NOT DEFINED` (mesh positions as stored) | `NATIVE` `geomBindTransform` | inherits glTF |
| Local transform | `PARTIAL` evaluated TRS ≠ full pivot stack | `NATIVE` matrix XOR TRS; if `matrix` is present it MUST be decomposable to TRS (no valid shear) | `NATIVE` local matrices / T+R+S | inherits glTF |
| Global transform | `NATIVE` evaluated | `REPRESENTABLE` parent walk | `NATIVE` defined spaces (joint-local / skel / world) | inherits glTF |
| Pre/post rotation | `NATIVE` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Pivot / offset stack | `NATIVE` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Joint orientation (Maya-style) | `NATIVE` via pre/post | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Inherit modes | `NATIVE` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Geometric (non-inherited) xform | `NATIVE` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Handedness | `PARTIAL` file setting | `NATIVE` right-handed | `PARTIAL` stage convention | inherits glTF |
| Up axis | `PARTIAL` file setting | `NATIVE` +Y | `NATIVE` `upAxis` | inherits glTF |
| Forward convention | `PARTIAL` / `UNKNOWN` | `NATIVE` +Z forward | `NOT DEFINED` at Skel layer | inherits glTF |
| Units | `PARTIAL` file; cm common | `NATIVE` meters | `NATIVE` `metersPerUnit` | inherits glTF |
| Skin weights | `NATIVE` clusters; not guaranteed normalized | `NATIVE` `JOINTS_n`/`WEIGHTS_n` | `NATIVE` primvars | inherits glTF |
| Multiple skins | `NATIVE` multiple deformers/clusters | `NATIVE` multiple `skins[]` objects / multiple skinned nodes; a node has at most one `skin` | `NATIVE` (multiple bound prims / remaps) | inherits glTF |
| Dual-quaternion skin | `NATIVE` method enum | `NOT DEFINED` in core | `NOT DEFINED` | `NOT DEFINED` |
| Animation clip | `NATIVE` stack/take | `NATIVE` `animations[]` | `NATIVE` `UsdSkelAnimation` | glTF clips + map |
| Animation layering | `NATIVE` layers in a stack | `NOT DEFINED` | `NOT DEFINED` (explicitly no sparse layering) | `NOT DEFINED` |
| Time coordinate unit/domain | `NATIVE` `FbxTime` (64-bit internal ticks; not a frame number) | `NATIVE` sampler input = seconds | `NATIVE` USD TimeCode ordinates | inherits glTF seconds |
| Time-to-seconds metadata | `NATIVE` `FbxTime` → seconds is a fixed internal mapping (`GetSecondDouble()`). `EMode`/custom rate is separate frame/time-code addressing provenance, not required to decode ticks into elapsed seconds | `NATIVE` already seconds; no extra scale metadata | `NATIVE` stage `timeCodesPerSecond` (fallback may use `framesPerSecond`) | inherits glTF |
| Playback/frame-rate metadata | `NATIVE` `FbxGlobalSettings` time mode / custom rate | `NOT DEFINED` as playback fps | `NATIVE` `framesPerSecond` (advisory; distinct from time-to-seconds) | inherits glTF (`NOT DEFINED`) |
| Variable timestamps | `NATIVE` curve keys on `FbxTime` | `NATIVE` uneven sampler inputs | `NATIVE` USD time samples | inherits glTF |
| Explicit duration field | `NOT DEFINED` as a portable clip-duration property | `NOT DEFINED` (no `animation.duration`; interval MAY be derived from sampler `min`/`max`) | `PARTIAL` stage `startTimeCode`/`endTimeCode` are playback-range metadata, not a Skel clip-duration object | inherits glTF |
| Interpolation specified | `PARTIAL` curve/tangent; bake recommended | `NATIVE` LINEAR/STEP/CUBICSPLINE | `PARTIAL` USD resolution, not a Skel enum | inherits glTF |
| Translation tracks | `NATIVE` | `NATIVE` | `NATIVE` | inherits glTF |
| Rotation tracks | `NATIVE` Euler (+ order) | `NATIVE` quaternion | `NATIVE` quaternion arrays | inherits glTF |
| Scale tracks | `NATIVE` | `NATIVE` | `NATIVE` (non-uniform discouraged) | inherits glTF |
| Missing channels | `PARTIAL` follow FBX stack/layer/property evaluation; not glTF/UsdSkel rest fallback | `NATIVE` remain at authored/default node property | `NATIVE` fall back to `restTransforms` | inherits glTF |
| Root motion | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Loop semantics | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Layer additive / blend | `NATIVE` `FbxAnimLayer` `EBlendMode` (`eBlendAdditive`, override, passthrough) + rotation/scale accumulation | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Portable clip-level additive contract | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Events / markers | `PARTIAL` / `UNKNOWN` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Semantic humanoid roles | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` | `NATIVE` |
| Helper/twist/IK class | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` (unmapped nodes only) |
| Multiple skeleton roots | `NATIVE` | `PARTIAL` | `NATIVE` | constrained by humanoid |
| Provenance / version metadata | `PARTIAL` scene/exporter info | `PARTIAL` `asset` + extras | `PARTIAL` layer metadata | `NATIVE` `VRMC_vrm.meta` + glTF asset |
| Lossless simple-TRS round-trip | `PARTIAL` — pivot/inherit/layers/curves lost if reduced | `PARTIAL` — matrix-vs-TRS authoring form, CUBIC tangents, extras | `PARTIAL` — composition, sparse USD features | `PARTIAL` — same as glTF + humanoid map |

---

## How to use this matrix

Source-local keys (glTF node index, VRM role+index, USD joint token, FBX node object) identify a joint **inside one source asset instance**. They are not a stable RigForge Canonical identity across import/export/versioning.

- `NATIVE` in one column and `NOT DEFINED` in another is a **Canonical coverage demand**, not a bug in the poorer format.
- Do not “fix” FBX by assuming glTF TRS, or “fix” VRM by treating it as a universal skeleton.
- `UNKNOWN` cells are W0-P / W1 questions, not silent defaults.
