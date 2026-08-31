# W0.3 Reality Check Plan

Preparation for **W0-P** (RV-1 Format & Canonical Reality Check, Preview PoCs, adapter probes).

No assets are downloaded in this pass. No product `src/`.

Access date: **2026-08-30**.

License tokens reuse W0.1: `REPO_FIXTURE_CANDIDATE`, `DOWNLOAD_DURING_TEST_ONLY`, `REFERENCE_ONLY`, `LICENSE_UNCLEAR`, `SERVICE_RESTRICTED`.

W1 Canonical schema does **not** exist yet. PoCs use **research projections** and semantic assertions only.

Every executed experiment later must record: `EXPECTED` / `OBSERVED` / `GAP` / `ROOT CAUSE` / `DECISION IMPACT` and `CONTINUE` / `ADJUST` / `RESEARCH MORE` / `REDESIGN` / `DROP / DEFER`.

---

## RV-1 — Format & Canonical Reality Check (future)

Use Khronos SimpleSkin / InterpolationTest (`REPO_FIXTURE_CANDIDATE`), Fox (`DOWNLOAD_DURING_TEST_ONLY` + attribution), CesiumMan (trademark caution), license-cleared UsdSkel paths (W0.1), VRM samples when licensed.

Projection target: temporary research structs, **not** Core.

---

## Format experiments

### F-FBX-01 Ingest evaluate vs recipe

| Field | Value |
| --- | --- |
| QUESTION | Does a ufbx load with default vs recipe-preserving options change reported local transforms on a pivot-heavy file? |
| CANDIDATE | ufbx current master |
| ASSET | Autodesk sample or producer FBX (`REFERENCE_ONLY` / `LICENSE_UNCLEAR` until checked) |
| LICENSE | verify per file |
| METHOD | Load twice; compare evaluated local vs authored Lcl; record `ufbx_error` |
| EXPECTED | Difference when pivots/pre-rotate exist (W0.1) |
| OBSERVE | which option hid the recipe |
| DECISION IMPACT | FBX adapter loss policy; option A vs C |

### F-FBX-02 Layered animation bake

| Field | Value |
| --- | --- |
| QUESTION | What does `ufbx_bake_anim` drop vs stack/layer curves? |
| CANDIDATE | ufbx |
| ASSET | layered take (`LICENSE_UNCLEAR` until found) |
| METHOD | Bake vs evaluate layers; count curves |
| EXPECTED | resample / layer flatten (W0.1) |
| DECISION IMPACT | PRESERVE_LAYERING capability; loss class |

### F-FBX-03 ufbx-write skin/anim interoperability

Construction APIs exist [SOURCE_CONFIRMED] `ufbx_write.h`. This experiment is **not** “does an API exist?”

| Field | Value |
| --- | --- |
| QUESTION | Does the existing ufbx-write skin/bind/anim **mechanism** produce **correct interoperable semantics**? |
| CANDIDATE | ufbx-write WIP + ufbx (detect `UFBX_EXPORTER_UFBX_WRITE`) + optional Autodesk FBX SDK |
| ASSET | synthetic or SimpleSkin analogue in FBX |
| LICENSE | project-owned synthetic preferred |
| METHOD | Write skinned + animated FBX (binary 7500); reload with ufbx; reload with FBX SDK if available; compare semantic assertions (joint count, bind/IBM analogue, curve/stack presence, evaluated pose samples) |
| EXPECTED | files load; semantic completeness and producer/consumer interop remain **UNPROVEN** until observed |
| DECISION IMPACT | Strategy A vs D; **FBX write in V1?** Mechanism confirmed ≠ production readiness |

### F-GLTF-01 Parse vs buffers vs validity

| Field | Value |
| --- | --- |
| QUESTION | Does parse-only success differ from buffer load + skin check on SimpleSkin? |
| CANDIDATE | cgltf and/or tinygltf v3 and/or fastgltf |
| ASSET | Khronos SimpleSkin CC0 |
| METHOD | parse; load buffers; assert joints/IBM per W0.1 caveat |
| EXPECTED | three-stage outcomes differ if buffers omitted |
| DECISION IMPACT | AD-DIAG-002; library shortlist (not select) |

### F-GLTF-02 Unknown extension round-trip

| Field | Value |
| --- | --- |
| QUESTION | Does re-export preserve an unknown `extensions` object and `extensionsUsed`? |
| CANDIDATE | cgltf / tinygltf v3 / fastgltf / glTF Transform (registered vs not) |
| ASSET | synthetic glTF with dummy vendor ext |
| METHOD | import/export; JSON diff |
| EXPECTED | Transform drops unregistered [OFFICIAL_DOC]; cgltf may keep member |
| DECISION IMPACT | preserve-opaque vs refuse vs strip policy |

### F-GLTF-03 Validator vs adapter

| Field | Value |
| --- | --- |
| QUESTION | Can Khronos Validator pass while Domain import still reports semantic loss? |
| CANDIDATE | glTF-Validator + chosen parser |
| ASSET | InterpolationTest |
| METHOD | validate; import projection |
| EXPECTED | validator ≠ Canonical validity |
| DECISION IMPACT | do not use Validator as sole import gate |

### F-USD-01 Composed vs authored

| Field | Value |
| --- | --- |
| QUESTION | Does opening a stage flatten references differently from reading the root layer only? |
| CANDIDATE | OpenUSD (separate process is a **strong isolation candidate**; placement **OPEN**) |
| ASSET | license-cleared UsdSkel test or synthetic USDA |
| METHOD | Dump authored vs composed joint counts / timeCodesPerSecond (W0.1); record resolver + composition **context** as declared inputs (AD-DET-001) |
| EXPECTED | composition can rescale TimeCodes [OFFICIAL_DOC]; results depend on declared context |
| DECISION IMPACT | import mode; process vs in-process remains OPEN |

### F-USD-03 Determinism context

| Field | Value |
| --- | --- |
| QUESTION | If resolver/composition context is omitted or mutated (latest alias, different search path), does output change, and is the Adapter forbidden from claiming full determinism? |
| CANDIDATE | OpenUSD adapter research projection |
| ASSET | stage with a versioned/aliased reference |
| METHOD | Run twice with declared vs undeclared/changed resolver context; compare projection |
| EXPECTED | undeclared mutable resolve → must **not** claim full deterministic reproducibility (AD-DET-005) |
| DECISION IMPACT | provenance fields; fail-closed vs warn |

### F-USD-02 Export/publish derived

| Field | Value |
| --- | --- |
| QUESTION | Can a research projection write USDA that reopens? |
| CANDIDATE | OpenUSD write APIs |
| ASSET | synthetic |
| METHOD | write; reopen; compare restTransforms |
| DECISION IMPACT | USD export in V1 vs later |

### F-VRM-01 Profile on glTF

| Field | Value |
| --- | --- |
| QUESTION | Does a glTF-only import lose `humanBones` / expressions while parse succeeds? |
| CANDIDATE | glTF parser + VRM schema check |
| ASSET | licensed VRM sample |
| METHOD | parse glTF; detect VRMC_*; map slots |
| EXPECTED | VRM meaning requires extension layer (W0.1) |
| DECISION IMPACT | specialization vs separate adapter facade |

---

## DCC experiments (optional host — not ingest proof)

### D-BLENDER-01 Headless file export

| Field | Value |
| --- | --- |
| QUESTION | Can `--background --python` export glTF/FBX without GUI? |
| CANDIDATE | Blender current LTS |
| ASSET | simple .blend (`LICENSE_UNCLEAR` / project-owned) |
| METHOD | official CLI [OFFICIAL_DOC] |
| EXPECTED | yes |
| DECISION IMPACT | topology C vs D; **not** required for ingest |

### D-MAYA-01 mayapy standalone

| Field | Value |
| --- | --- |
| QUESTION | Does `maya.standalone` export FBX without UI? |
| CANDIDATE | Maya 2026 if licensed |
| METHOD | mayapy script |
| EXPECTED | yes for non-UI commands [OFFICIAL_DOC] |
| DECISION IMPACT | optional Maya adapter; never ingest authority |

### D-DET-01 Uncontrolled DCC session

| Field | Value |
| --- | --- |
| QUESTION | Does an interactive or prefs-polluted DCC session change export vs factory-startup / standalone, and must the Adapter refuse a full-determinism claim? |
| CANDIDATE | Blender `--background` and/or Maya `standalone` |
| ASSET | project-owned simple scene |
| METHOD | Export twice: factory-startup vs leftover session/prefs; compare derived files; record whether context was declared |
| EXPECTED | session state can change output; uncontrolled state → no full-repro claim (AD-DET-005) |
| DECISION IMPACT | DCC adapter capability; determinism class |

---

## Engine experiments (optional)

### E-UE-01 File vs Interchange

| Field | Value |
| --- | --- |
| QUESTION | Can a Canonical-derived FBX/glTF exist without running Interchange? |
| CANDIDATE | none / file only |
| METHOD | produce file outside UE; optionally later import with `is_automated` [OFFICIAL_DOC] |
| EXPECTED | conversion independent of editor |
| DECISION IMPACT | editor ≠ Domain |

### E-UNITY-01 Batchmode vs Domain

| Field | Value |
| --- | --- |
| QUESTION | Same as UE: Domain file vs `-batchmode` import |
| CANDIDATE | Unity 6 if licensed |
| METHOD | official CLI [OFFICIAL_DOC] |
| DECISION IMPACT | batch is host automation, not Core |

### E-GODOT-01 Generic contract

| Field | Value |
| --- | --- |
| QUESTION | Can Godot import a derived glTF without RigForge knowing `Skeleton3D`? |
| CANDIDATE | Godot 4 `GLTFDocument` [OFFICIAL_DOC] |
| ASSET | SimpleSkin or Fox |
| EXPECTED | yes |
| DECISION IMPACT | engine-agnostic gate remains YES |

---

## Runtime cook experiments

### R-OZZ-01 Derived cook

| Field | Value |
| --- | --- |
| QUESTION | What tracks survive glTF → ozz offline tool? |
| CANDIDATE | ozz tools (may pull tinygltf) |
| ASSET | SimpleSkin |
| EXPECTED | sampled runtime clip; loss declared |
| DECISION IMPACT | cook adapter; not Canonical |

### R-ACL-01 Compress research projection

| Field | Value |
| --- | --- |
| QUESTION | Can ACL compress a synthetic clip and report error metric? |
| CANDIDATE | ACL headers |
| ASSET | synthetic tracks |
| EXPECTED | yes; rebuildable with policy id |
| DECISION IMPACT | cook vs Canonical separation |

---

## Preview experiments

Do **not** score visual polish. Question: can Preview expose RigForge needs **without an engine**?

| ID | QUESTION | CANDIDATE | ASSET | METHOD | EXPECTED | DECISION IMPACT |
| --- | --- | --- | --- | --- | --- | --- |
| P1 | Skinned humanoid + anim inspectable | projection and/or derived GLB | SimpleSkin or licensed VRM | load outside UE/Unity | motion visible | A vs B vs C |
| P2 | Non-humanoid skeleton | same | Khronos Fox | no humanoid slots | still previews | generality |
| P3 | Multiple roots | synthetic | project-owned | overlay roots | no silent merge | W0.1 multi-root |
| P4 | Twist/helper-heavy | licensed DCC export | classify overlays | helpers ≠ equal deform | W0.2 |
| P5 | Source/target side-by-side | two projections | W0.2 pair idea | two viewports or dual pose | mapping display |
| P6 | Skeleton-only | synthetic no mesh | joints only | **must work** | PV-SKELETON-001 |
| P7-A | Unbound/detached Motion: inspect metadata/tracks/time/binding diagnostics **without** pose reconstruction | projection and/or Domain/View | motion-only or broken bind | no pose invented | Can Preview stay useful before complete asset resolution? PV-MOTION-001 |
| P7-B | Resolve/bind Skeleton, then reconstruct/play skeletal poses | same + compatible skeleton | clip + resolved skel | poses only after bind | PV-MOTION-004 |
| P8 | Root trajectory overlay | loco clip | draw path from Domain/policy | not new root law | W0.2 |
| P9 | QC overlay | synthetic slide | consume QC numbers | no auto-fix | W0.2 |

Compare **Candidate A (stable read-only Domain/View contract — not Canonical struct/memory)** vs **Preview Projection** vs **derived GLB** vs **hybrid** on: semantic coverage, payload complexity, browser/native portability, coupling, load cost, debuggability, round-trip confusion. A/B/C/D remain **OPEN**.

---

## Decision authority

W0-P results may change: FBX write strategy; glTF library shortlist; USD process placement; VRM composition; Preview data path; **not** Canonical-first or engine-independence.
