# W0.4 — Product / Tech-stack / Frontier / Engineering Decision Research

Access date: **2026-08-31**. Decision date: **2026-08-31**.

This is a **recommendation package**. It does not freeze implementation language, Preview renderer, format library, or GUI stack as product/architecture authority unless a row is explicitly `DECIDE_NOW`.

Do not read “preferred candidate” as “RigForge uses X.”

Companion files:

| File | Role |
| --- | --- |
| [PRODUCT_SPACE_MATRIX.md](PRODUCT_SPACE_MATRIX.md) | Competitive space + R1-G0 |
| [TECHNOLOGY_DECISION_MATRIX.md](TECHNOLOGY_DECISION_MATRIX.md) | Language / format / ABI / storage / cook / MCP |
| [PREVIEW_TECHNOLOGY_MATRIX.md](PREVIEW_TECHNOLOGY_MATRIX.md) | Host / renderer / data path / bridge |
| [LICENSE_DEPENDENCY_MATRIX.md](LICENSE_DEPENDENCY_MATRIX.md) | License + dependency governance |
| [FRONTIER_AI_MATRIX.md](FRONTIER_AI_MATRIX.md) | Learned systems |
| [V1_SCOPE_RECOMMENDATION.md](V1_SCOPE_RECOMMENDATION.md) | Scope buckets (does not edit `docs/product/V1_SCOPE.md`) |
| [W0_P_POC_SELECTION.md](W0_P_POC_SELECTION.md) | Mandatory / optional PoCs |

W0-P is **not** executed here. No experiments. No `src/`. No vendor install.

---

## 1. Scope

Execute W0.4 only:

```text
Given accepted W0.1–W0.3 requirements,
which product and engineering directions are credible,
which deserve W0-P PoCs,
which should be deferred,
and which should be rejected?
```

This stage **may** recommend. It **must not** falsely freeze decisions that need executable evidence.

Out of scope: W0-P execution, IA-1, Gate A, W1, product implementation, ADRs as frozen selections.

Accepted baselines were **not** rewritten. If a later review finds a conflict with W0.1–W0.3, classify evidence and route to review / W0-P. This pass found **no accepted-research conflict requiring rewrite**.

Rev1 (2026-08-31) corrects W0-P critical-path size, Core/Preview experimental design, I/O vs publish classification, retarget pair definition, source pins, MCP rationale, and Blender license wording. It does **not** restart W0.4 or execute W0-P.

Rev2 (2026-08-31) makes W0-P execution-ready: named license-cleared asset pools, Core FFI/worker constants, generic Engine stub as oracle, version-pin preflight, WGSL 2026-07-16 confirmed, cgltf MIT and Validator Apache-2.0 confirmed. Godot validation is RESEARCH ONLY, not a V1 SKU. W0-P is still **not** executed.

Rev3 (2026-08-31) corrects pack-specific license authority: QCHAR / QANIMAL / QANIM / KPETS are **CC0** from their official pack pages. Site-wide QAL does not override those declarations. W0-P is still **not** executed.

Evidence-only version notes (do not mutate W0.3 files):

| Topic | Rev1 official pin | Source |
| --- | --- | --- |
| TinyGLTF | **v3.0.1** released **2026-08-02**; latest GitHub Release as of 2026-08-31. v3 C runtime still experimental | https://github.com/syoyo/tinygltf/releases/tag/v3.0.1 [SOURCE_CONFIRMED] |
| AYON Core | Latest listed GitHub Release **1.9.10** (2026-08-05). Prior Release **1.9.9** (2026-07-28). Pins from Releases only | https://github.com/ynput/ayon-core/releases [SOURCE_CONFIRMED] |
| OpenUSD 26.08 | Released **2026-07-20** | Pixar CHANGELOG / GitHub tag v26.08 [SOURCE_CONFIRMED] |
| WebGPU | **Candidate Recommendation Draft**. Latest **published** W3C history row this pass: **20 August 2026**. Also published: 14 July 2026. Not a Recommendation. Not an Editor's Draft pin | https://www.w3.org/standards/history/webgpu/ [SPEC] |
| cgltf | **v1.15** published **2025-02-09** (not 2026-02-09) | https://github.com/jkuhlmann/cgltf/releases/tag/v1.15 [SOURCE_CONFIRMED] |

---

## 2. Accepted requirements

W0.1 answered: what Character / Skeleton / Motion semantics must preserve.

W0.2 answered: what Mapping / Compatibility / Retarget / QC must understand.

W0.3 answered: what Adapter / Infrastructure / Preview boundaries must support.

Permanent checks on every recommendation:

```text
PRODUCT REQUIREMENTS
GENERALITY
ENGINE INDEPENDENCE
FORMAT INDEPENDENCE
DCC INDEPENDENCE
REAL-ASSET EVIDENCE
MAINTAINABILITY
LICENSE / REDISTRIBUTION
```

RigForge remains cross-format, cross-DCC, cross-engine. Unreal, Unity, Blender, Maya, MotionBuilder remain **reference systems / adapter targets / optional integrations**, never architectural authority.

Preview is a **required long-term product capability**. A browser-like viewer is a preferred **direction**, not a selected implementation. Preview must not require a specific engine or DCC.

---

## 3. Decision methodology

Every major candidate receives exactly one of:

```text
DECIDE_NOW
PROVISIONAL_W0P_GATED
KEEP_AS_ALTERNATIVE
DEFER
REJECT
OPEN
```

`DECIDE_NOW` is used only when evidence is sufficient **and** W0-P cannot materially reverse the decision (architectural role, not a library pin).

Every `PROVISIONAL_W0P_GATED` row states **WHAT EVIDENCE COULD REVERSE THIS?**

Evidence tags: `[SPEC]` `[OFFICIAL_DOC]` `[SOURCE_CONFIRMED]` `[PROJECT_CLAIM]` `[PAPER]` `[EXPERIMENT]` `[RIGFORGE_INFERENCE]`.

Confidence: `HIGH` / `MEDIUM` / `LOW`.

No `[EXPERIMENT]` this stage.

Decision longevity (all technology rows):

| Field | Value |
| --- | --- |
| Decision date | 2026-08-31 |
| Evidence version | W0.4 package; W0.1–W0.3 accepted |
| Re-check triggers | major version; maintenance inactivity; license change; API break; W0-P failure; new platform constraint; new V1 requirement; a product that owns the exact Domain gap |

No “forever selection.”

Criteria are written. Popularity / star count is not a criterion.

---

## 4. Product necessity

See [PRODUCT_SPACE_MATRIX.md](PRODUCT_SPACE_MATRIX.md).

Compared at least: MotionBuilder, Maya HumanIK, Unreal IK Rig / IK Retargeter, Unity Humanoid, Rokoko Studio, Cascadeur, Reallusion Character Creator / iClone, Mixamo, Blender ecosystem. Also checked at lighter depth: AccuRIG / ActorCore, Houdini KineFX, Godot animation.

### R1-G0 verdict

```text
BUILD
```

Confidence: **HIGH** that the gap exists; **MEDIUM** for commercial timing (no market-size study).

The unmet **architectural** gap is a portable Character / Skeleton / Motion **Domain**:

1. Format-independent Canonical (W0.1).
2. Host-independent mapping / compatibility / retarget / QC (W0.2).
3. Engine-independent Preview of humanoid **and** non-humanoid, skeleton-only, and unbound Motion (W0.3).
4. Fail-closed capability / loss / determinism-context diagnostics (W0.3).
5. Headless Domain API that CLI / Workbench / later MCP all call.

Existing products own a **host or service slice**. Combining them still leaves incompatible mapping identities, no shared loss model, and Preview that requires that host.

`BUILD_CANONICAL_CORE_ADOPT_COMPONENTS` is the **implementation style** (write Domain; adopt parsers / validators), not a different product verdict.

`RECONSIDER_BUILD` remains valid if a later mature product ships this **exact** gap under a usable license. Do not defend the project merely because W0 has started.

Classification: `DECIDE_NOW` for **BUILD as current necessity**. Re-check trigger is product-space, not W0-P.

---

## 5. Core language / ABI

See [TECHNOLOGY_DECISION_MATRIX.md](TECHNOLOGY_DECISION_MATRIX.md).

Compared: C++ Domain (L1), Rust Domain (L2), hybrid workers (L3), Python-only Domain, TypeScript-only Domain.

| Role | Candidate | Class |
| --- | --- | --- |
| Preferred | **L1** C++ Domain + C ABI + Python bindings + Web/TS Preview frontend | `PROVISIONAL_W0P_GATED` |
| Runner-up | **L2** Rust Domain + C ABI + Python + WASM possibility + Web/TS Preview | `KEEP_AS_ALTERNATIVE` |
| Compose | **L3** native Domain library + host-specific adapter workers | `KEEP_AS_ALTERNATIVE` (compose with L1 or L2) |
| Rejected as Domain | Python-only; TypeScript-only; public C++ ABI | `REJECT` |

**Why L1 is preferred, not automatic:** mandatory-adjacent parsers and **all** engine / DCC native plugins are C/C++. Rust improves ownership safety and often WASM, but adds an FFI layer to OpenUSD, FBX SDK, and engine plugins. W0.3 already allows process isolation for heavy hosts. That offset is unproven without **POC-CORE-01**.

**WHAT EVIDENCE COULD REVERSE L1:** POC-CORE-01 equivalent vertical slice (same C ABI, Python caller, ufbx handoff, worker/serialized boundary) records `prefer L2` on observed FFI/ownership/ergonomics/build evidence — not “Rust feels safer.” Fallback: L2. OpenUSD deploy is **not** part of this language test.

Public surface: stable **C ABI** preferred (`PROVISIONAL_W0P_GATED`). Public C++ ABI `REJECT`.

In-process remains sufficient for small C libraries (ufbx, cgltf). Heavy / crashy / EULA hosts (USD, Maya, engines) should use **workers**, not a single topology for every adapter. Topology is per-adapter, not a monolith freeze.

---

## 6. Format implementation

### FBX

| Question | Recommendation | Class |
| --- | --- | --- |
| Preferred V1 ingest | **ufbx** v0.23.0 | `PROVISIONAL_W0P_GATED` |
| I/O asymmetry | V1 does **not** require symmetric import/export for every format. FBX write is not required because FBX read exists | `DECIDE_NOW` |
| Preferred V1 derived publish **candidate** | **glTF/GLB** | `PROVISIONAL_W0P_GATED` (POC-GLTF-01) |
| Is FBX write mandatory for V1? | **No** | `DECIDE_NOW` |
| First E2E candidate | FBX ingest → Domain → glTF publish | `PROVISIONAL_W0P_GATED` until POC-FBX-01 and POC-GLTF-01 succeed |
| USD as V1 publish | Not required | Placement `PROVISIONAL` / likely POST-V1 |

**WHAT EVIDENCE COULD REVERSE ufbx ingest:** representative real FBX corpus (skinned, animated, bind/rest, nontrivial pivots, helpers, layered if available) shows unrepresentable or incorrect semantics that cannot be fail-closed with explicit loss. Fallback: Autodesk FBX SDK **read**, isolated process, `HIGH` EULA risk.

ufbx-write mechanism exists; readiness **UNPROVEN**. Optional **POC-FBX-02**. Do not treat “API exists” as write readiness.

### glTF

Role split (`DECIDE_NOW` for **roles**, not library pin):

```text
native parse/write  →  cgltf (preferred lightweight) or fastgltf (C++17 + write + SIMD)
Khronos Validator   →  CI / tooling only, not sole import gate
glTF Transform      →  JS tooling / document ops only
Domain              →  Canonical + loss
```

Native library rank: `PROVISIONAL_W0P_GATED` (**POC-GLTF-01**). TinyGLTF **v3.0.1** is `KEEP_AS_ALTERNATIVE` (v3 C runtime still experimental).

**glTF library ≠ Preview renderer.** Separate decisions.

**WHAT EVIDENCE COULD REVERSE cgltf-first:** unknown-extension / IBM / skin semantics silently lost, or write path insufficient. Fallback: fastgltf, then TinyGLTF.

### VRM

VRM is **glTF + profile / specialization**, not a universal skeleton. `DECIDE_NOW` (composition). V1 OPTIONAL after glTF PoC. Must not become Canonical humanoid.

---

## 7. OpenUSD

Current version: **OpenUSD 26.08**, released **2026-07-20**, Apache-2.0. No official WASM target recorded.

| Placement | Class |
| --- | --- |
| Mandatory in-process Core | `REJECT` |
| Optional in-process adapter | `KEEP_AS_ALTERNATIVE` only if PoC size acceptable |
| Separate adapter worker | **Preferred if** V1 or V1-optional USD |
| External / later phase | Valid if POC-USD-01 cost too high |

Industry importance ≠ V1 Core. UsdSkel is valuable; composition / resolver / plugin discovery / binary size / version skew are the cost.

**WHAT EVIDENCE COULD REVERSE worker-if-any:** POC-USD-01 shows worker too large or unstable for a V1-optional adapter → **POST-V1** USD. If a tiny `usd-core` subset proves in-process-safe, in-process remains an alternative, not a Core mandate.

---

## 8. Preview / Workbench

See [PREVIEW_TECHNOLOGY_MATRIX.md](PREVIEW_TECHNOLOGY_MATRIX.md).

```text
specific game engine required: NO
```

Dimensions are independent.

| Dimension | Preferred | Runner-up | Class |
| --- | --- | --- | --- |
| Workbench host | **H3** desktop shell + embedded web | H2 native desktop | `PROVISIONAL_W0P_GATED` |
| Renderer | **R1** Three.js `WebGPURenderer` + official WebGL 2 fallback | R2 Babylon.js | `PROVISIONAL_W0P_GATED` |
| Preview data path | **B** dedicated Preview Projection | **D** hybrid B + optional derived GLB | `PROVISIONAL_W0P_GATED` |
| Native bridge | C ABI / WASM Domain or local worker | HTTP/IPC worker | `PROVISIONAL_W0P_GATED` |

Rejected: H4 game-engine host; R6 engine viewport as Preview architecture; **C derived GLB alone** as sole Preview IR; JS reimplementation of Canonical as authority.

WebGPU: **W3C Candidate Recommendation Draft** [SPEC]. Latest **published** history row loaded this pass: **20 August 2026** (`CRD-webgpu-20260820`). Also published: **14 July 2026**. Not a Recommendation. Published TR ≠ Editor's Draft / repo tip.

WGSL: **Candidate Recommendation Draft** [SPEC]. Official W3C published version/history: **16 July 2026**. https://www.w3.org/TR/WGSL/ and https://www.w3.org/standards/history/WGSL/. **Not** a final Recommendation. Published TR ≠ Editor's Draft.

Three.js `WebGPURenderer`: official manual states WebGPU primary + automatic WebGL 2 fallback, **and** that the renderer itself is still **experimental** (maturity improved). Classification remains `PROVISIONAL_W0P_GATED`.

Preview PoCs are deconfounded: **01A** = B vs D (same host/renderer/assets); **01B** = H3+R1 smoke (one fixed payload).

V1 Preview boundary: **minimal standalone inspection**, not a polished Workbench viewer. Long-term architecture stays engine-free.

**WHAT EVIDENCE COULD REVERSE H3 / R1 / B-first:** webview memory or overlay failure → H2 + R4; Three.js overlay fight → R2; projection cost ≈ GLB with overlays beside GLB → prefer D.

---

## 9. DCC priority

Priority is **product sequencing**, not Canonical convention.

| DCC | Class |
| --- | --- |
| File ingest without any DCC | `DECIDE_NOW` (required optionality) |
| **Blender** | `V1 FIRST` as **optional** thin adapter (headless / file / CLI). Highest open-tooling value |
| **Maya** | `V1 SECONDARY` / likely `POST-V1`. Highest professional pipeline value; EULA + version cost |
| MotionBuilder | `REFERENCE ONLY` |

Blender conventions ≠ Canonical. Maya conventions ≠ Canonical.

In-process Blender (`bpy` link) is `HIGH` GPL risk. Process/file isolation is an **engineering** candidate that avoids linking Blender into Core. Distributed `bpy` / add-on obligations remain **REQUIRES LEGAL REVIEW**. Isolation is not a legal safe harbor. Optional **POC-DCC-01**.

---

## 10. Engine priority

Which adapter ships first ≠ which engine defines RigForge.

| Engine | Role |
| --- | --- |
| Generic Engine Adapter contract | `DECIDE_NOW` (no UObject / AssetDatabase in Core) |
| **Unreal** | Intended **first productized** adapter (POST-V1 typical); highest industry handoff value |
| **Unity** | Intended **second** |
| **Godot** | Optional smoke only (**4.7.2-stable** candidate). **RESEARCH ONLY**, not a V1 SKU. Genericness oracle is POC-ENGINE-01A stub |
| Custom / generic engine | Must remain implementable |

A W0-P test that a **non-engine** host stub implements the generic contract is mandatory (POC-ENGINE-01A). A later W8 real-engine adapter remains optional product work.

---

## 11. Storage / runtime

| Topic | Recommendation | Class |
| --- | --- | --- |
| V1 asset management dependency | **No.** Minimal Domain AssetReference / Provenance | `DECIDE_NOW` |
| OpenAssetIO | Optional future adapter; v1.0.2; not a database | `DEFER` |
| AYON | Future pipeline adapter; latest GitHub Release **1.9.10** (2026-08-05); prior **1.9.9** (2026-07-28) | `DEFER` |
| Runtime cook implementation | **None in V1.** Keep Adapter contract | `DEFER` |
| ozz-animation 0.16.0 | Derived cook candidate later | `DEFER` |
| ACL 2.1.0 | Derived compression later | `DEFER` |

Canonical Motion ≠ runtime clip. Do not add a runtime system because a library exists. ozz offline tools may pull FBX SDK — do not pull that into Core.

---

## 12. MCP timing

```text
Domain API
    →
CLI / automation
    →
future MCP
```

MCP must not define Domain API.

Recommendation: **POST-V1**, because Domain API + CLI must stabilize first. Official maintainers published spec **2026-07-28** as a major/breaking revision after the **November 2025** stable release, and introduced a **formal deprecation policy** (twelve-month minimum window) [OFFICIAL_DOC] https://blog.modelcontextprotocol.io/posts/2026-07-28/. That evolution is a secondary reason to wait; it is **not** argued solely as a comparison to 2025-06-18.

Class: `DEFER`.

---

## 13. Frontier AI

See [FRONTIER_AI_MATRIX.md](FRONTIER_AI_MATRIX.md).

Deterministic Core is **not** overturned.

| Capability | Class |
| --- | --- |
| AI semantic suggestion | `POST-V1` (non-authoritative) |
| AI retarget | `POST-V1` / optional |
| AI auto-rig (SkinTokens / RigAnything class) | `POST-V1` / `RESEARCH ONLY` |
| Mandatory model in Core | `REJECT` |
| Mixamo / cloud auto-rig as authority | `REJECT` |

Papers and Hugging Face weights ≠ production dependency. Training-data ToS (VRoid / ModelsResource / ArticulationXL) **REQUIRES LEGAL REVIEW**.

Safety boundary `DECIDE_NOW`:

```text
proposal → diagnostics / confidence / evidence → Domain acceptance → Canonical mutation
```

AI suggestion ≠ Canonical truth.

---

## 14. License / dependency risks

See [LICENSE_DEPENDENCY_MATRIX.md](LICENSE_DEPENDENCY_MATRIX.md). Not legal advice.

Top blockers / unknowns:

1. Autodesk FBX SDK EULA — `HIGH`. Isolated optional only. `REQUIRES LEGAL REVIEW`.
2. Blender GPL — do not link Core in-process. File/CLI isolation is engineering only; distributed add-on/`bpy` use **REQUIRES LEGAL REVIEW**.
3. AI weights + dataset ToS — `HIGH` / `UNKNOWN`.
4. Qt LGPL static link — if H2/H3 uses Qt, prefer dynamic LGPL or commercial. `REQUIRES LEGAL REVIEW`.
5. TinyGLTF v3 experimental vs v2 sunset — engineering, not license.

Reject as **mandatory Core**: FBX SDK, full OpenUSD runtime, OpenAssetIO, AYON, ozz/ACL as Canonical, Qt, any engine/DCC SDK, any AI weight stack.

---

## 15. V1 scope recommendation

See [V1_SCOPE_RECOMMENDATION.md](V1_SCOPE_RECOMMENDATION.md). Does **not** edit `docs/product/V1_SCOPE.md`.

| Bucket | Count (table rows) |
| --- | --- |
| V1 CORE | 13 |
| V1 OPTIONAL | 5 |
| POST-V1 | 11 |
| RESEARCH ONLY | 10 |
| OUT OF SCOPE | 6 |
| W0-P GATED | 9 |

Intended V1 CORE still matches the working product list, plus explicit **loss/provenance** and a **minimal** engine-independent inspection Preview. Preferred first E2E **candidate**: FBX ingest → Domain → glTF publish (`PROVISIONAL_W0P_GATED`). I/O asymmetry (no mandatory FBX write) is `DECIDE_NOW`. FBX write, USD, Maya, Unreal/Unity product adapters, runtime cook, MCP, and AI are not V1 CORE.

---

## 16. Decisions we can make now (`DECIDE_NOW`)

These do not materially depend on W0-P:

| Decision | Why W0-P cannot reverse the **role** |
| --- | --- |
| R1-G0 **BUILD** (until product-space re-check) | Gap is architectural, not a library test |
| Canonical remains independent of runtime cook formats | Architecture + W0.3 |
| Preview architecture must not require a game engine or DCC | Product |
| Product value does not require symmetric import/export for every format; FBX write is not required because FBX read exists | Product. **Not** a freeze of the first publish format |
| Khronos Validator is tooling, not sole import gate | W0.3 AD-DIAG-002 |
| VRM is a glTF profile, not a universal skeleton | W0.1 |
| DCC / engine adapters are optional; file ingest without DCC | Product |
| Engine contract is generic; host types stay out of Core | W0.3 |
| V1 needs Domain AssetReference / Provenance, not an AMS | W0.3 + OAIO “not a database” |
| Keep runtime-cook **contract**; no V1 cook implementation | Scope |
| MCP is post-V1 over Domain API + CLI | Architecture + spec churn |
| AI suggestion ≠ Canonical; proposal → accept | Safety |
| Derived GLB is never sole Preview IR / never Canonical | W0.3 |
| Public C++ ABI is not the host surface | Industry ABI skew |
| Python / TypeScript are not sole Domain | W0.3 FFI |

---

## 17. Provisional W0-P gated decisions

| Preference | Test | Expected | Reversal | Fallback |
| --- | --- | --- | --- | --- |
| L1 C++ Domain + C ABI | POC-CORE-01 equivalent slice | Observed table prefers L1 | Observed table prefers L2 | L2 |
| ufbx V1 FBX ingest | POC-FBX-01 | Semantics + explicit loss on real files | Unrepresentable/incorrect, not diagnosable | Isolated FBX SDK read |
| Native glTF + preferred derived publish candidate glTF/GLB | POC-GLTF-01 | Extensions/skins preserved; write credible | Silent required-semantic loss or unusable write | Other native lib; other derived publish |
| USD optional worker, not Core | POC-USD-01 **CONDITIONAL** | Composed UsdSkel + context; size OK | Too heavy/unstable | POST-V1 USD |
| Preview data path B first | POC-PREVIEW-01A | Coverage without Canonical-as-GLB | B too costly; D sufficient | D, or A desktop-only |
| Preview host H3 + renderer R1 | POC-PREVIEW-01B | Webview + overlays + local I/O | Webview/memory fail; renderer fight | H2 / R4; optional POC-PREVIEW-02 |
| Retarget **method** remains OPEN | POC-RETARGET-01 | Diagnosable results on L3-A and compatible L3-B pairs | Baseline unusable on those pairs | More research; **not** host-as-Core |

---

## 18. Deferred / rejected

### Deferred

FBX write as V1 requirement; OpenUSD as V1 Core; OpenAssetIO / AYON runtime; ozz / ACL implementation; MCP; AI assistants; Maya product adapter; Unreal/Unity product SKUs; custom WebGPU engine; wgpu/Dawn native renderer; polished Workbench viewer.

### Rejected

Game engine as Workbench or Preview host; DCC as mandatory ingest; Mixamo as Canonical ingest; ozz/ACL as Canonical Motion; VRM as universal skeleton; public C++ ABI; Python/TS as sole Domain; derived GLB as sole Preview IR; JS Canonical reimplementation as authority; mandatory AI weights in Core.

---

## 19. Open questions

| Question | Route |
| --- | --- |
| Does ufbx cover the real FBX corpus? | W0-P |
| Which native glTF library ranks? | W0-P |
| USD V1-optional vs POST-V1? | W0 Final Synthesis; POC-USD-01 only if still V1 OPTIONAL |
| L1 vs L2 after ABI/worker cost? | W0-P |
| Preview B vs D; H3 vs H2? | W0-P |
| Which deterministic retarget **method**? | W0-P then W5 |
| Accept W0.4 V1 recommendation vs working `V1_SCOPE.md`? | W0 Final Synthesis |
| Record ADRs after gates? | W0 Final Synthesis / IA-1 |
| C++ language standard / Rust edition? | W1 |
| Desktop shell toolkit (Qt vs other)? | W1 / later if H3/H2 confirmed |
| Exact MCP version if ever adopted | Later |
| Commercial clearance of AI datasets | Later / legal |
| Market-size / pricing | Later (not a W0-P question) |

---

## 20. Mandatory W0-P PoCs

See [W0_P_POC_SELECTION.md](W0_P_POC_SELECTION.md).

```text
MANDATORY_W0P:     6
CONDITIONAL_W0P:   1
OPTIONAL_W0P:      3
```

| ID | Decision question |
| --- | --- |
| POC-CORE-01 | L1 vs L2 on an equivalent vertical slice |
| POC-FBX-01 | ufbx reader semantics on real assets |
| POC-GLTF-01 | Native read/write + whether glTF/GLB remains preferred derived publish |
| POC-RETARGET-01 | Real source–Motion–target pairs (L3-A humanoid+humanoid; L3-B compatible non-humanoid) |
| POC-PREVIEW-01 | Family: 01A B vs D; 01B H3+R1 smoke |
| POC-ENGINE-01 | 01A generic stub (Mandatory oracle); 01B Godot optional smoke |

Conditional: POC-USD-01 (only if Final Synthesis still considers USD V1 OPTIONAL). Optional: POC-FBX-02, POC-DCC-01, POC-PREVIEW-02.

Real assets are mandatory for POC-FBX-01 and POC-RETARGET-01. Level 1 synthetic + Level 2 curated public + Level 3 real-world where specified. No Mixamo corpus in-repo. No vendor engines in-repo.

---

## 21. W0.4 gate assessment

| Gate | Result | Note |
| --- | --- | --- |
| DG-PRODUCT | **YES** | `BUILD`; gap stated |
| DG-CORETECH | **YES** | Ranked L1 / L2 / L3; not frozen |
| DG-FORMATTECH | **YES** | Preferred paths + fallbacks |
| DG-PREVIEWTECH | **YES** | Preferred + fallback; engine-free |
| DG-INTEGRATION | **YES** | DCC/engine sequence without lock-in |
| DG-SCOPE | **YES** | Recommendation only |
| DG-LICENSE | **PARTIAL** | Blockers identified; several `REQUIRES LEGAL REVIEW` |
| DG-FRONTIER | **YES** | Classified; deterministic-first held |
| DG-W0P | **YES** | Six mandatory PoCs; USD conditional |

---

## Generality gate (every recommended stack)

| Question | Answer |
| --- | --- |
| Can new formats be added? | Yes — Adapter + C ABI |
| Can a custom DCC integrate? | Yes — optional thin adapter |
| Can a custom engine integrate? | Yes — generic contract; POC-ENGINE-01A stub |
| Can Preview run without a game engine? | Yes — required |
| Can non-humanoid assets flow through? | Yes — Canonical + Preview B/D |
| Can headless workflows exist? | Yes — Domain API + CLI first |

---

## Preview gate

Rejected any Preview architecture requiring Unreal, Unity, a specific DCC, or a humanoid-only runtime.

---

## Dependency gate

Rejected mandatory Core dependencies with incompatible/unclear redistribution, unacceptable maintenance, or platform violations. Isolated optional adapters remain allowed.

---

## Conflict log (W0.1–W0.3)

None requiring rewrite. W0.4 does not select a retarget algorithm, IK placement, Preview path as authority, or format library as authority.

---

## DECIDE_NOW audit (Rev1)

Question for every `DECIDE_NOW` row: can a W0-P result reasonably reverse this?

| Decision | Still `DECIDE_NOW`? | Why |
| --- | --- | --- |
| R1-G0 BUILD | Yes | Product-space, not a library test |
| Canonical ≠ runtime cook | Yes | Architecture |
| Preview must not require a game engine/DCC | Yes | Product |
| I/O asymmetry; FBX write not required for symmetry | Yes | Product. Publish **format** is separate |
| Khronos Validator role (tooling, not sole gate) | Yes | Role, not a library pin |
| VRM = glTF profile, not universal skeleton | Yes | Composition |
| DCC/engine adapters optional; file ingest without DCC | Yes | Product |
| Engine contract generic | Yes | Architecture |
| V1 needs Domain provenance, not an AMS | Yes | Scope |
| No V1 cook implementation (keep contract) | Yes | Scope |
| MCP POST-V1 over Domain+CLI | Yes | Timing; Domain-first |
| AI ≠ Canonical | Yes | Safety |
| Derived GLB never sole Preview IR / never Canonical | Yes | Architecture |
| No public C++ ABI; Python/TS not Domain | Yes | ABI/host |

**Moved off `DECIDE_NOW` in Rev1:** specific V1 derived publish format **glTF/GLB** → `PROVISIONAL_W0P_GATED` (POC-GLTF-01). First E2E pair is a candidate until POC-FBX-01 and POC-GLTF-01 succeed.

---

## Version evidence ledger (Rev1)

Pins from release/tag or W3C published-history only. Access date: **2026-08-31**. Do not infer “latest” from PR dates, commit activity, or Editor's Drafts.

| Component | Version / tag | Release / publish date | Official source | Type |
| --- | --- | --- | --- | --- |
| ufbx | v0.23.0 | Tag / changelog line associated with 2026-06-21 integration | https://github.com/ufbx/ufbx/tags (`v0.23.0`). GitHub **Releases** page has no Release objects | tag + changelog |
| TinyGLTF | **v3.0.1** | **2026-08-02** | https://github.com/syoyo/tinygltf/releases/tag/v3.0.1 | GitHub Release |
| TinyGLTF (prior) | v3.0.0 | 2026-03-23 | https://github.com/syoyo/tinygltf/releases/tag/v3.0.0 | GitHub Release |
| cgltf | v1.15 | **2025-02-09** | https://github.com/jkuhlmann/cgltf/releases/tag/v1.15 | GitHub Release |
| fastgltf | v0.9.0 | 2025-07-08 | https://github.com/spnda/fastgltf/releases/tag/v0.9.0 | GitHub Release |
| OpenUSD | v26.08 | 2026-07-20 | https://github.com/PixarAnimationStudios/OpenUSD/blob/v26.08/CHANGELOG.md ; tag v26.08 | CHANGELOG + tag |
| OpenAssetIO | v1.0.2 | 2026-04-21 | https://github.com/OpenAssetIO/OpenAssetIO/releases (v1.0.2 listed first) | GitHub Release |
| AYON Core | **1.9.10** (latest listed) | **2026-08-05** | https://github.com/ynput/ayon-core/releases/tag/1.9.10 | GitHub Release |
| AYON Core (prior) | 1.9.9 | 2026-07-28 | https://github.com/ynput/ayon-core/releases/tag/1.9.9 | GitHub Release |
| WebGPU | CR Draft | **Published** 20 August 2026 (also 14 July 2026) | https://www.w3.org/standards/history/webgpu/ ; dated TR `CRD-webgpu-20260820` | W3C published history |
| WGSL | CR Draft | **Published 16 July 2026** (confirmed official W3C published version/history) | https://www.w3.org/TR/WGSL/ ; https://www.w3.org/standards/history/WGSL/ | W3C TR / history |
| MCP spec | 2026-07-28 | 2026-07-28; previous official blog “November” release | https://blog.modelcontextprotocol.io/posts/2026-07-28/ ; https://modelcontextprotocol.io/specification/2026-07-28 | Official blog + spec |
| Three.js | **r185** (2026-07-01) latest formal GitHub Release this pass | `WebGPURenderer` experimental + WebGL 2 fallback | https://github.com/mrdoob/three.js/releases/tag/r185 ; https://threejs.org/manual/en/webgpurenderer.html | GitHub Release + official doc |
| Godot | **4.7.2-stable** (2026-08-18) | Optional 01B only | https://github.com/godotengine/godot/releases/tag/4.7.2-stable | GitHub Release |
| Babylon.js WebGPU | current official doc (no npm pin this pass) | — | https://doc.babylonjs.com/setup/support/webGPU | Official doc |

---

## Status

```text
W0:     ACTIVE
W0.1:   COMPLETE
W0.2:   COMPLETE
W0.3:   COMPLETE
W0.4:   COMPLETE
W0-P:   READY
```

W0 is not complete. IA-1 is not started. Gate A is not passed. W1 is not started.
