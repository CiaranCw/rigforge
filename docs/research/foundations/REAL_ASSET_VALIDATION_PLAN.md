# Real-Asset Validation Plan — W0.1

Rev2: time-domain W0-P questions added/replaced. No assets downloaded.

Preparation for later **W0-P**. No assets are downloaded into this repository.

Access date: 2026-08-30.

Classification:

| Token | Meaning |
| --- | --- |
| `REPO_FIXTURE_CANDIDATE` | License understood well enough to consider vendoring later |
| `DOWNLOAD_DURING_TEST_ONLY` | Use in CI/local test; do not copy into git without a later license review |
| `REFERENCE_ONLY` | Official example text / non-redistributable SDK content |
| `LICENSE_UNCLEAR` | Do not add to a permanent corpus |
| `SERVICE_RESTRICTED` | Marketplace / ToS-bound assets; local tests only |

None of these assets imply that RigForge “supports” a format.

---

## Coverage goals

Future W0-P should exercise:

- simple hierarchy
- skin + IBM
- animation (including multiple clips where possible)
- nontrivial transforms (pivot / pre-rotation / non-uniform scale)
- different units / axes
- helper / extra bones
- humanoid semantics
- non-humanoid structure
- missing channels / sparse animation
- source time domain (glTF sampler ranges, FBX `FbxTime` / time mode, USD `timeCodesPerSecond` vs `framesPerSecond`)

---

## Candidates

### A. Khronos SimpleSkin

| Field | Value |
| --- | --- |
| Source | KhronosGroup/glTF-Sample-Assets |
| URL | https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/SimpleSkin |
| License | Model files: SPDX `CC0-1.0`. Metadocumentation: CC-BY-4.0. [OFFICIAL_DOC] Models/SimpleSkin/LICENSE.md |
| Redistribution | Model files permissive (CC0) |
| Class | `REPO_FIXTURE_CANDIDATE` |
| Characteristics | Minimal two-joint skin; IBM; animation |
| Why | Smallest official glTF skin contract |
| Features | simple hierarchy, skin, animation, IBM |
| Future question | For this fixture, whose bind-shape/conversion assumptions are those of the official SimpleSkin tutorial, verify the **documented** skinning relationship. Do **not** treat `IBM[i] == inverse(default-global-joint-i)` as a universal glTF invariant. Also record how duration is derived. |

### B. Khronos InterpolationTest

| Field | Value |
| --- | --- |
| Source | KhronosGroup/glTF-Sample-Assets `InterpolationTest` |
| URL | https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/InterpolationTest |
| License | Model files: SPDX `CC0-1.0`. Metadocumentation: CC-BY-4.0. [OFFICIAL_DOC] Models/InterpolationTest/LICENSE.md |
| Redistribution | Model files permissive (CC0) |
| Class | `REPO_FIXTURE_CANDIDATE` |
| Why | LINEAR / STEP / CUBICSPLINE side by side |
| Future question | Does sampled TRS cook preserve CUBICSPLINE extrema? |

### C. Khronos Fox

| Field | Value |
| --- | --- |
| Source | KhronosGroup/glTF-Sample-Assets `Fox` |
| URL | https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox |
| License | Model files: SPDX expression **`CC0-1.0 AND CC-BY-4.0`**. [OFFICIAL_DOC] Models/Fox/LICENSE.md + README. PixelMannen model: CC0-1.0. tomkranis rigging/animation: CC-BY-4.0. @AsoboStudio / @scurest glTF conversion: CC-BY-4.0. Attribution required for the CC-BY portions. |
| Class | `DOWNLOAD_DURING_TEST_ONLY` until attribution text is prepared; not a no-credit repo fixture |
| Characteristics | Multiple animation clips (Survey / Walk / Run) |
| Future question | How are multiple `animations[]` named independently? For **each** animation, inspect **all** sampler input ranges: what union/start/end interval do their `min`/`max` imply? Do samplers begin/end at different times? How should a future duration-derivation policy represent that **without** pretending glTF has an `animation.duration` field? |

### D. CesiumMan

| Field | Value |
| --- | --- |
| Source | KhronosGroup/glTF-Sample-Assets `CesiumMan` |
| URL | https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/CesiumMan |
| License | CC-BY 4.0 **with trademark limitations** (Cesium mark). [OFFICIAL_DOC] |
| Redistribution | Restricted by trademark terms |
| Class | `DOWNLOAD_DURING_TEST_ONLY` / not a logo-bearing repo fixture |
| Characteristics | Skinned humanoid-like character, animation |
| Future question | Joint count vs mesh bind; node names vs indices |

### E. OpenUSD UsdSkel samples (docs vs source)

Do **not** re-type or “re-author” the copyrighted HTML “Skinning an Arm” example into the repository. Copying authored numbers/names from the docs does not create redistribution rights.

| Path | Value |
| --- | --- |
| A — official source file | Candidate: `pxr/usd/usdSkel/testenv/testUsdSkelRoot/root.usda` in [PixarAnimationStudios/OpenUSD](https://github.com/PixarAnimationStudios/OpenUSD/blob/dev/pxr/usd/usdSkel/testenv/testUsdSkelRoot/root.usda). Repo-root license is Apache-2.0; **path-specific NOTICE/LICENSE still unverified**. Class: `DOWNLOAD_DURING_TEST_ONLY` until that path is license-confirmed. Not a `REPO_FIXTURE_CANDIDATE` yet. |
| B — synthetic later | During W0-P/W3, an independently designed minimal USDA (new joint names, transforms, values) from the **abstract public semantics**. Do not copy the example’s authored content. |
| Docs HTML | https://openusd.org/release/api/_usd_skel__schema_overview.html — `REFERENCE_ONLY` |

Future question (on a license-cleared or synthetic file): if only a subset of joints is animated, do the others stay on `restTransforms`? Are rest and bind independent after space conversion?

Future time-domain question (synthetic or license-cleared): author `timeCodesPerSecond != framesPerSecond`, or two composed layers with differing `timeCodesPerSecond` if practical. Does the resolved stage time map to seconds as documented, and can an importer preserve enough provenance to explain the conversion?

### F. ufbx public testdata (selected)

| Field | Value |
| --- | --- |
| Source | https://github.com/ufbx/ufbx testdata / fbx corpus |
| License | Confirm per-file; library is MIT OR Unlicense, **test assets may differ** |
| Class | `LICENSE_UNCLEAR` until each file’s header/license is read |
| Why | Real FBX pivot / exporter / animation-stack cases |
| Future question | Does evaluated TRS match Autodesk SDK evaluation on the same file? Also record `FbxTime` / file time mode and convert keys to elapsed seconds (see planned question 11). |

### G. Autodesk FBX SDK sample scenes

| Field | Value |
| --- | --- |
| Source | Autodesk FBX SDK installer samples |
| License | Autodesk SDK / sample license (proprietary) |
| Class | `DOWNLOAD_DURING_TEST_ONLY` — do not vendor |
| Why | Official cluster / bind-pose / animation-stack examples |
| Future question | When `FbxPose` is absent, do cluster Transform/TransformLink still define a consistent bind? |

### H. VRM official / UniVRM sample models

| Field | Value |
| --- | --- |
| Source | vrm-c samples / UniVRM sample VRM 1.0 |
| URL | https://github.com/vrm-c/vrm-specification and related sample repos |
| License | Confirm each model (often VRM public license / CC variants) |
| Class | `DOWNLOAD_DURING_TEST_ONLY` until license file is attached to the candidate |
| Characteristics | Required humanoid bones; expressions; look-at |
| Future question | Are required bones present? Can a valid VRM still contain extra non-humanoid nodes? Does `.vrma` map apply to a second VRM? |

### I. Mixamo / marketplace FBX

| Field | Value |
| --- | --- |
| Source | Adobe Mixamo and similar |
| License | Service ToS; typically **no** unrestricted redistribution |
| Class | `LICENSE_UNCLEAR` / `SERVICE_RESTRICTED` — local machine tests only, never repo |
| Why | Humanoid + extra bones + baked animation common in the wild |
| Future question | Are Mixamo “Humanoid” names a reliable semantic map? (expected: no)

### J. BVH public mocap (CMU or similar)

| Field | Value |
| --- | --- |
| Source | Historical mocap dumps |
| License | Varies by corpus |
| Class | `LICENSE_UNCLEAR` / `REFERENCE_ONLY` for W0.1 |
| Why | Motion-only, no skin; tests Motion-without-Character |
| Future question | Out of V1 ingest unless a later gate adds BVH |

### K. COLLADA official examples (Khronos)

| Field | Value |
| --- | --- |
| Source | Khronos COLLADA test / examples |
| License | Confirm per archive |
| Class | `REFERENCE_ONLY` for W0.1 (format is discovery, not V1 ingest) |
| Why | `bind_shape_matrix` vs UsdSkel `geomBindTransform` analogy |

---

## Planned W0-P questions (not executed)

### Transform

1. FBX file with non-zero pre-rotation: does `ParentWorld × simple(LclTranslation, LclRotation, LclScaling)` match SDK/`ufbx_evaluate_transform` world? (Expected: no when the remaining authored recipe is non-identity.)
2. FBX geometric transform: does child inherit it? (docs say no — confirm)
3. glTF unanimated `matrix` node: record authored form vs TRS decomposition; confirm the matrix is TRS-decomposable (valid glTF cannot carry shear). Negative-determinant policy is OPEN.

### Rest / bind

4. License-cleared or synthetic UsdSkel file: rest local vs bind world independence.
5. glTF SimpleSkin: verify the **documented tutorial** skinning relationship under that fixture’s known bind-shape assumptions. Do not generalize `IBM == inverse(default-global)` to all glTF.
6. FBX OPEN: can pose-table data disagree with cluster bind matrices in real producer output, and what authority policy should RigForge apply?

### Motion

7. glTF CUBICSPLINE vs LINEAR resample error.
8. FBX layered stack vs flattened bake.
9. UsdSkel sparse joint list fallback.
10. glTF Fox (do **not** assume one sampler): for each animation, inspect all sampler input ranges. What union/start/end interval is implied by their `min`/`max`? Do different samplers begin/end at different times? How should a future RigForge duration-derivation policy represent that without pretending glTF contains an explicit duration field?
11. FBX time domain: record `EMode` / file time mode and inspect frame/time-code addressing (prefer a drop/non-drop or non-integer-rate case where practical). Independently convert the same `FbxTime` keys to elapsed seconds via `GetSecondDouble()`. Do not treat `EMode` as required to decode ticks into seconds.
12. USD time domain: fixture with `timeCodesPerSecond != framesPerSecond`, or two composed layers with differing `timeCodesPerSecond` if practical. Does resolved stage time map to seconds as `UsdStage::GetTimeCodesPerSecond` documents, and can an importer preserve conversion provenance?

### Semantics

13. VRM required-bone set vs extra helper nodes.
14. Name-identical joints in two FBX files: are they the same joint?

### Coordinates

15. Blender-exported FBX cm/Y-up vs Maya cm: ufbx `ADJUST_TRANSFORMS` vs `MODIFY_GEOMETRY`.

---

## Experiments in this W0.1 pass

None. No local file inspection, no parser, no dependency.

Any later run must record: QUESTION / ASSET / TOOL+VERSION / METHOD / OBSERVED / LIMITATION and tag `[EXPERIMENT]`.
