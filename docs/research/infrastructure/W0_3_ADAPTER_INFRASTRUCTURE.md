# W0.3 — Adapter / Infrastructure / Preview Boundary

Research report. Not a schema. Not an implementation. Not a language or renderer decision.

Access date: **2026-08-30**.

Related: [ADAPTER_CAPABILITY_AND_LOSS_MODEL.md](ADAPTER_CAPABILITY_AND_LOSS_MODEL.md), [FORMAT_ADAPTER_MATRIX.md](FORMAT_ADAPTER_MATRIX.md), [DCC_ENGINE_BOUNDARY.md](DCC_ENGINE_BOUNDARY.md), [RUNTIME_STORAGE_INFRASTRUCTURE.md](RUNTIME_STORAGE_INFRASTRUCTURE.md), [PREVIEW_BOUNDARY_REQUIREMENTS.md](PREVIEW_BOUNDARY_REQUIREMENTS.md), [W0_3_REALITY_CHECK_PLAN.md](W0_3_REALITY_CHECK_PLAN.md).

W0.1 and W0.2 remain accepted. Conflicts, if any, are recorded here and are **not** applied back to those trees.

Status: **COMPLETE**.

Rev1 closed focused review findings R1-MAJOR-001 (determinism context), R1-MAJOR-002 (Domain Semantic Mapping ≠ Host Binding), R1-MAJOR-003 (Motion Preview split), R1-MODERATE-004 (ufbx-write mechanism vs readiness), R1-MODERATE-005 (ACL UE version; WASM classification). External focused review: **TECHNICAL PASS**. No format library, language, process topology, Preview path, renderer, or engine is selected.

Permanent checks: product requirements; generality; engine independence; **format independence**; **DCC independence**; real-asset evidence.

---

## 1. Scope

Establish evidence-grounded requirements that keep:

```text
Formats / DCCs / Engines / Storage / Runtime / Preview
        ↓
      Adapters
        ↓
Engine-independent RigForge Domain
```

from collapsing into a mandatory host or Canonical format.

Out of scope: W0.4 tech selection, W0-P execution, W1/W2 field freeze, product code.

No experiments in this pass. Documentation and official source/READMEs were sufficient for a requirements baseline.

---

## 2. Accepted product constraints

- Formats and engines are adapters / references (architecture + product vision).
- DCC is never ingest authority when a format adapter exists (W0.1 / V1_SCOPE).
- Engine-independent Preview is a long-term product requirement; browser-like is **preferred**, not selected.
- ozz / ACL / VRM-as-universal / OpenAssetIO-as-mandatory-runtime remain non-core (V1_SCOPE).
- W0.2: mapping ≠ compatibility ≠ QC; Preview must not auto-fix.

No W0.1/W0.2 contradiction requiring a rewrite. W0.1 already deferred ufbx vs SDK vs ufbx-write to W0.3; this stage compares **engineering** paths and leaves selection OPEN.

---

## 3. Research method

1. Read repository authorities.
2. Official current docs / READMEs / specs (versions recorded).
3. Classify claims. HIGH = explicit official sentence.
4. Generality gate on every MUST: still valid if FBX, Unreal, Unity, Blender, or Maya disappeared.

---

## 4. Generic adapter architecture

Provisional categories (not frozen classes): Importer; Exporter; DCC Adapter; Engine Adapter; Storage / Resolver Adapter; Runtime Cook Adapter; Preview Adapter (frontend bind).

| Concern | Requirement direction |
| --- | --- |
| Input / output | External bytes/session ↔ Domain objects or derived |
| Authoritative side | Domain for Canonical; host for host-native scene |
| Derived side | Export, cook, Preview |
| Capabilities | Discoverable, versioned, contextual |
| Loss | Explicit records |
| Diagnostics | Common envelope |
| Version | Adapter + source/target + config |
| Determinism | Semantic, only under a **declared** context; not necessarily byte |
| Thread/process | Per-host; not one topology |
| Headless | Core workflows without Workbench |
| External runtime | Declared; never implicit Core |
| Failure | Preflight then mutate |

**CONFIRMED:** Adapter translates; it does not own Canonical truth. Domain Semantic Mapping ≠ Host Binding (AD-CAP-007). **OPEN:** final class names. **REQUIRES_POC:** real loss tables per library.

---

## 5. Capability model

See AD-CAP. Caller must not guess. Capabilities vary by version, format subset, config, and platform (ufbx load opts; glTF Transform registration; USD plugins).

---

## 6. Loss model

See AD-LOSS. Silent loss is forbidden. Families: supported / supported-with-loss / unsupported / ambiguous (names provisional).

---

## 7. Import / export lifecycle

**Import:** file parse → source semantic extraction → normalization → Canonical construction → validation → provenance → diagnostics.

Parser success ≠ semantic import success ≠ Canonical validity (cgltf parse without buffers [OFFICIAL_DOC]; glTF `extensionsRequired` [SPEC]; USD compose/resolve [OFFICIAL_DOC]).

**Export:** Canonical validate → target capability negotiate → loss preflight → convert → serialize → post-validate → publish manifest.

Exporter must be able to refuse, warn+continue under policy, or require approval. Exact names OPEN.

Fail-closed: no irrecoverable write before preflight.

---

## 8. FBX

W0.1 semantics accepted. Engineering:

**ufbx** (https://github.com/ufbx/ufbx, https://ufbx.github.io/) [OFFICIAL_DOC] README 2026-08-30: C99/C++11 single file (`ufbx.h`/`ufbx.c`); allocator/load opts; `ufbx_error`; graceful invalid/OOM; FBX ≥ 3000 ASCII/binary; skins, blendshapes, evaluate/bake; CI bit-exact on Win/macOS/Linux + **WASI WASM**; MIT or Unlicense; 0.Y.Z may break; pre-C11 atomics caveat for thread safety. **Not selected.**

**Autodesk FBX SDK 2020.3.x** [OFFICIAL_DOC] https://aps.autodesk.com/developer/overview/fbx-sdk : C++ read/write; Windows/macOS/Linux (iOS listed). Semantic **reference authority**. Product dependency: proprietary LSA; SDK headers/libs not redistributable as a kit; runtime redistributables + acknowledgement (historical Autodesk legal posts / LSA). Re-read **current** EULA at adoption (W0.4). Large binary; compiler-flavor coupling. **No official WASM target confirmed** (APS list does not include WASM; that is not a proof of impossibility).

**ufbx-write** [OFFICIAL_DOC] README: WIP, breaking expected; scene/node/mesh/save binary 7500 **mechanism confirmed**; ASCII slow path documented; optional C++17 threads extra; MIT or Unlicense.

Skin/animation **element and construction API: MECHANISM CONFIRMED** [SOURCE_CONFIRMED] `ufbx_write.h` — skin deformer/cluster, bind pose, animation curve/prop/layer/stack, node animation helpers. **End-to-end semantic completeness: UNPROVEN. Interoperability: UNPROVEN. Production readiness: WIP / UNPROVEN.** Not selected as V1 exporter. **REQUIRES W0-P**.

ufbx **v0.22.0** (2026-05-18) added `UFBX_EXPORTER_UFBX_WRITE`; recorded current tag **v0.23.0** (2026-06-21) [SOURCE_CONFIRMED] https://github.com/ufbx/ufbx/blob/v0.23.0/misc/changelog.md. This strengthens the reason to test the write path. It does **not** establish production readiness.

Strategies A–E in [FORMAT_ADAPTER_MATRIX.md](FORMAT_ADAPTER_MATRIX.md). **No choice.** V1 FBX write is **unproven**.

---

## 9. glTF / GLB

Parser / validator / transform / serializer need not be one component.

| Library | Engineering |
| --- | --- |
| **cgltf** | C99 stb-style parse+write; no deps; extras; listed extensions; unknown via `extensions` member; parse ≠ load buffers [OFFICIAL_DOC] |
| **tinygltf v3** | C11 POD + arena + structured errors; WASM demo; MIT; v3 is mainline, C++ attic deprecated [OFFICIAL_DOC] |
| **fastgltf** | C++17 SIMD; official Exporter; extras write callback; MIT + simdjson Apache-2.0 [OFFICIAL_DOC] v0.9 |
| **glTF Transform** | JS/TS Node+Web; CLI; extension **registration required** [OFFICIAL_DOC] https://gltf-transform.dev/ — strong for JS preview/CLI, not a native Core |

**Unknown extensions:** spec lists `extensionsUsed` / `extensionsRequired`; extras are application metadata [SPEC] Khronos extensions README. Policy OPEN: preserve opaque / declare loss / refuse round-trip / strip under policy. Transform will drop unregistered extensions [OFFICIAL_DOC].

Khronos Validator is a **validator**, not an importer.

---

## 10. USD / UsdSkel

C++ and Python APIs; plugin architecture; layer composition; asset resolver; threading/TBB; large deploy; Apache-2.0 [OFFICIAL_DOC] OpenUSD / Exchange deployment docs. Headless converters are a documented use [OFFICIAL_DOC].

**Mandatory Core dependency?** **No** on present evidence. **Optional adapter** and/or **separate process** are credible placements. Separate process is a **strong candidate** when ABI/dependency/crash isolation warrants it. Final in-process vs subprocess/service placement remains **OPEN** (`DEFER W0.4`, `REQUIRES W0-P`). Direct Core link is a size/ABI/plugin-path risk.

**Composition:** distinguish authored layer, composed stage, resolved references, UsdSkel schema. Import-from-composed vs preserve-authored vs multi-mode is **OPEN**. Must record composition provenance, resolver context, TimeCode scaling (W0.1), layer source, and loss.

---

## 11. VRM

VRM is glTF + standardized extensions (W0.1). Prefer conceptual **glTF adapter + VRM profile/specialization** so VRM is not a second parser universe — but a **facade adapter** is allowed if it clarifies provenance. Do not decide on code reuse alone.

Must preserve: humanoid slots, expressions, lookAt, VRM animation, extension provenance. Must not make VRM universal.

---

## 12. DCC boundary

See [DCC_ENGINE_BOUNDARY.md](DCC_ENGINE_BOUNDARY.md).

Blender: official `--background --python` [OFFICIAL_DOC]. Maya: `mayapy` / `maya.standalone` [OFFICIAL_DOC]. Neither is ingest authority.

In-process FFI vs external process vs headless host vs file-only: **no universal topology**. Separate process is a **strong candidate** when ABI, Python version, GPU, or crash domains conflict. Placement remains **OPEN**.

---

## 13. Engine boundary

Unreal Interchange and Unity `-batchmode` are **host import/automation** [OFFICIAL_DOC]. Canonical conversion must work without those editors.

Godot `GLTFDocument` proves a third engine can consume derived glTF without UObject/AssetDatabase [OFFICIAL_DOC].

Engine Adapters may emit **host binding / host projection** facts. They must not author Domain Semantic Mapping (AD-CAP-007).

**Specific engine required by Core: NO.**

---

## 14. Storage / resolver

OpenAssetIO: host↔manager bridge; not a schema/database/storage [OFFICIAL_DOC]. Optional integration; not V1 mandatory runtime.

AYON: folder/product/version/representation [OFFICIAL_DOC]. Optional pipeline adapter; taxonomy stays out of Core.

---

## 15. Runtime cook

ozz (MIT, C++17, WASM CI) and ACL (MIT, header-only, WASM CI) are **derived cook** candidates [OFFICIAL_DOC]. Neither is Canonical. Cooks must be rebuildable with policy/version provenance.

---

## 16. Preview boundary

See [PREVIEW_BOUNDARY_REQUIREMENTS.md](PREVIEW_BOUNDARY_REQUIREMENTS.md).

Preview is derived and read-only. Must not go Canonical → Unreal/Unity viewport as the architecture.

Candidates A/B/C/D compared; **not selected**. Candidate A means a **stable read-only Domain/View contract**, not renderer access to Canonical struct/memory. GLB is insufficient as the only inspection IR. Skeleton-only, non-humanoid, and **unbound Motion metadata** inspection must work. Skeletal pose playback requires a resolved Skeleton. Browser-like constraints listed without choosing WebGPU/WebGL.

**preview technology selected: NO. specific engine required: NO.**

---

## 17. Language / ABI constraints (inputs, not a decision)

Do **not** choose C++, Rust, Python, or TypeScript here.

| Pressure | Implication |
| --- | --- |
| ufbx / cgltf / tinygltf / ACL | Easy **native or C ABI** ingest/cook |
| OpenUSD / FBX SDK | Heavy **C++** (and often Python for USD) |
| DCC hosts | **Python** versions owned by the host |
| Browser-like Preview | **WASM** and/or JS frontend; portable payload |
| CLI / headless / later MCP | Domain without UI |
| FFI | Opaque handles, explicit ownership, versioned errors, large buffers |

Plausible structures (constraints only): native core + C ABI; native + Python bindings; Rust + C ABI; C++ core + WASM; hybrid process boundary. They can **coexist as research options**. Final split is W0.4.

---

## 18. Process isolation / headless

Separate process is a **strong candidate** for Maya, Blender (when using the app), full OpenUSD, engine editors, unstable third-party libraries, and future AI backends **when** ABI, dependency, or crash isolation warrants it. In-process remains valid for small C libraries.

Final in-process vs subprocess/service placement remains **OPEN**, pending W0.4 packaging and W0-P evidence. DCCs may use different topologies under one contract.

Headless **MUST** cover: import, validate, map, compatibility, retarget, export, publish — without Workbench. Preview **preparation** separable from interactive viewer.

---

## 19. Diagnostics

Common envelope (AD-DIAG). Do not let each library invent the only user-visible model.

---

## 20. Determinism

A determinism claim requires a **declared determinism context** (AD-DET-001): logical input, adapter/dependency versions, configuration, resolution/composition context, required host/environment state, Domain contract version, and policy.

If required external state cannot be captured, fixed, or reconstructed, the Adapter **must not** claim full deterministic reproducibility and must report the limitation (AD-DET-005).

Distinguish semantic / structural / byte determinism. No bit-identical promise for USD/DCC/GPU.

---

## 21. Versioning

Describe-capabilities-before-mutate (name OPEN). Record adapter identity/version and environment.

---

## 22. Real-asset PoC implications

See [W0_3_REALITY_CHECK_PLAN.md](W0_3_REALITY_CHECK_PLAN.md). RV-1 uses research projections. Preview PoCs P1–P9 (P7 split into unbound metadata vs bound pose) test engine-free inspection, not polish.

---

## 23. Confirmed foundations

| Finding | Class |
| --- | --- |
| Adapter ≠ Canonical | `CONFIRMED FOUNDATION` |
| Derived export/cook/Preview cannot reverse-authorize | `CONFIRMED FOUNDATION` |
| Domain Semantic Mapping ≠ Host Binding | `CONFIRMED FOUNDATION` |
| Determinism claim requires declared context | `ADAPTER REQUIREMENT CANDIDATE` |
| Silent loss forbidden | `ADAPTER REQUIREMENT CANDIDATE` |
| Parser ≠ semantic import ≠ Canonical valid | `ADAPTER REQUIREMENT CANDIDATE` |
| DCC/engine optional | `CONFIRMED FOUNDATION` |
| USD not mandatory Core | `ENGINEERING CONSTRAINT` |
| Process placement OPEN | `DEFER W0.4` / `REQUIRES W0-P` |
| OpenAssetIO/AYON/ozz/ACL not Canonical | `CONFIRMED FOUNDATION` |
| Preview derived + engine-free | `PREVIEW REQUIREMENT CANDIDATE` |
| Unbound Motion inspectable; pose needs Skeleton | `PREVIEW REQUIREMENT CANDIDATE` |
| Language unfrozen | `DEFER W0.4` |

---

## 24. Open questions

- W0.4: language/ABI split; renderer; full license matrix; FBX SDK EULA current text; process packaging.
- W0-P: ufbx-write **semantic/interop** (API exists); extension round-trip; USD compose modes + declared resolver context; Preview A/B/C/D; unbound vs bound Motion.
- W1: provenance/manifest fields; Preview object identity.
- W2: adapter class names; capability tokens; diagnostic schema.

---

## 25. W0.3 gate assessment

| Gate | Ready? | Meaning |
| --- | --- | --- |
| DG-ADAPTER | **YES** | Boundary enough for later W2 questions |
| DG-LOSS | **YES** | Families + required record contents; names not frozen |
| DG-FORMAT | **PARTIAL** | Paths compared; writer/USD/extension policy need PoC |
| DG-DCC | **YES** | Optional thin adapters; ingest without DCC |
| DG-ENGINE | **YES** | Generic contract; Godot check passed; Core needs no engine |
| DG-INFRA | **YES** | FFI/process/headless/diagnostics/versioning constraints recorded |
| DG-PREVIEW | **YES** | Authority + candidates without renderer pick; Motion split Rev1 |
| DG-REAL | **PARTIAL** | Experiments specified; none executed |

Rev1 re-audited every `AD-* MUST` and `PV-* MUST`. None requires FBX, Unreal, Unity, Blender, Maya, OpenUSD, OpenAssetIO, ozz, ACL, a humanoid-only structure, or a specific Preview renderer. Unbound Motion and skeleton-only remain expressible. Browser-like Preview can be replaced by native Preview without breaking PV MUST statements.

---

## Required questions (short answers)

1. Adapter: translate/negotiate external ↔ Domain; report loss. 2. Never: Canonical identity, Domain Semantic Mapping / compat / retarget / QC authority. 3. Discover capabilities in context before mutate. 4. Explicit loss records. 5. Fail when required semantics unsupported or `extensionsRequired` unmet without policy. 6. Adapter id/version/config **and declared execution context** in provenance. 7. Parser success **insufficient**. 8. Credible FBX: ufbx read; SDK official R/W; ufbx-write **mechanism confirmed**, semantics UNPROVEN. 9. FBX write **not** V1-credible yet. 10. cgltf / tinygltf v3 / fastgltf credible C/C++; Transform credible JS. 11. Unknown ext: policy OPEN (preserve/declare/refuse/strip). 12. OpenUSD: optional adapter and/or separate-process **candidate**; **not** mandatory Core; placement **OPEN**. 13. Preserve composition provenance, resolver context, time scaling, layer source. 14. VRM: glTF + profile/specialization (facade allowed). 15. DCC in-process **not** required. 16. Isolation is a **strong candidate** when ABI/Python/GPU/crash domains clash; topology **OPEN**. 17. Yes — file ingest without Blender/Maya. 18. Engine contract: derived interchange + **host binding/projection** + loss; no UObject; Adapter does not author Mapping. 19. Godot: **yes**. 20. OAIO/AYON: identity/resolve/publish + explicit resolution context. 21. Not Canonical schema; do not copy OAIO Context into Core. 22. Cook adapter: policy → derived runtime clip. 23. ozz/ACL derived because they sample/compress. 24. Preview consumes a renderer-independent derived surface (path OPEN). 25. Candidate A: **stable read-only Domain/View contract**, not Canonical memory. 26. Projection: stable read-only payload. 27. Derived GLB **not** sufficient for all inspection. 28. Overlays / projection / declared loss. 29. Skeleton-only must work. Unbound Motion metadata must work. 30. Preview read-only: **yes**. 31. Edits via Domain API. 32. Portable payload, load boundary, possible WASM, memory, offline, embed. 33. Native/FFI: ufbx/cgltf/USD/SDK. 34. Python: DCC hosts, USD scripts. 35. WASM/browser: Preview + ufbx CI + ozz/ACL CI + tinygltf demo; FBX SDK/full USD WASM **not officially listed**. 36. Yes — constraints coexist without a language pick. 37. Semantic determinism = same Domain meaning **under a declared context**, not bytes; uncontrolled state forbids a full-repro claim. 38. See reality-check plan.

---

## Sources (compact)

| Source | Class | Version / note |
| --- | --- | --- |
| ufbx README / site | [OFFICIAL_DOC] | master; WASM CI; MIT/Unlicense |
| ufbx changelog | [SOURCE_CONFIRMED] | v0.22.0 `UFBX_EXPORTER_UFBX_WRITE` (2026-05-18); v0.23.0 (2026-06-21) |
| ufbx-write README + `ufbx_write.h` | [OFFICIAL_DOC] / [SOURCE_CONFIRMED] | WIP; skin/anim construction APIs present |
| OpenAssetIO Context | [OFFICIAL_DOC] | resolution may need explicit Context / Manager State |
| Autodesk APS FBX SDK | [OFFICIAL_DOC] | 2020.3.x |
| Autodesk FBX LSA / legal posts | [OFFICIAL_DOC] | redistribution; confirm current EULA in W0.4 |
| cgltf / tinygltf v3 / fastgltf READMEs | [OFFICIAL_DOC] | 2026-08-30 |
| glTF Transform docs | [OFFICIAL_DOC] | NodeIO/WebIO; extensions |
| Khronos glTF extensions README | [SPEC] | extensionsUsed/Required; extras |
| OpenUSD intro / Exchange deploy | [OFFICIAL_DOC] | Apache-2.0; plugins |
| OpenAssetIO intro | [OFFICIAL_DOC] | not a database/schema |
| AYON glossary / addon intro | [OFFICIAL_DOC] | product/version/representation |
| ozz-animation README | [OFFICIAL_DOC] | MIT; WASM |
| ACL README | [OFFICIAL_DOC] | MIT; header-only; WASM; `5.13` wording = [PROJECT_CLAIM] typo |
| ACL UE plugin README | [SOURCE_CONFIRMED] | in-engine from **UE 5.3**; not a Core reason to adopt ACL |
| Blender CLI args 5.2 LTS | [OFFICIAL_DOC] | `--background` |
| Maya PyMEL standalone | [OFFICIAL_DOC] | mayapy |
| Unreal Interchange API | [OFFICIAL_DOC] | 5.5–5.7 docs |
| Unity 6 CLI / isBatchMode | [OFFICIAL_DOC] | 6000.x |
| Godot GLTFDocument | [OFFICIAL_DOC] | stable 4.x |
