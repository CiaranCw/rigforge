# W0-P PoC Selection — W0.4 Rev3

Access date: **2026-08-31**. Decision date: **2026-08-31**. Rev2: **2026-08-31**. Rev3: **2026-08-31** (pack-specific license authority).

This file is the **execution basis** for W0-P. W0-P is **not** executed here. No assets are downloaded in W0.4.

Rule: smallest set that can **reverse** architecture or **V1-Core** decisions.

Priority tokens: `MANDATORY_W0P` / `CONDITIONAL_W0P` / `OPTIONAL_W0P` / `DEFER`.

If a preferred candidate fails: record failure → root cause → apply the reversal rule. Do not rewrite an observation to protect the current preference.

---

## Mandatory set (6)

```text
POC-CORE-01
POC-FBX-01
POC-GLTF-01
POC-RETARGET-01
POC-PREVIEW-01
POC-ENGINE-01
```

`POC-PREVIEW-01` remains one Mandatory **family** (`01A` data path, `01B` host/renderer smoke).

`POC-ENGINE-01` remains one Mandatory **family** (`01A` generic stub, `01B` optional Godot smoke).

## Conditional / optional set (4)

```text
POC-USD-01      CONDITIONAL_W0P
POC-FBX-02      OPTIONAL_W0P
POC-DCC-01      OPTIONAL_W0P
POC-PREVIEW-02  OPTIONAL_W0P
```

USD does **not** block IA-1, Gate A, or W1.

---

## Global preflight (before any PoC)

```text
A. Verify git / research baseline (W0.1–W0.3 accepted; this file is the W0-P plan).

B. Resolve exact tool/library versions:
   1. resolve current intended stable release/tag
   2. record exact immutable version/tag/commit
   3. record release/source URL
   4. keep that version fixed across all runs of that PoC
   5. do not silently upgrade mid-experiment
   If a dependency is tested across versions, version is an explicit independent variable.

C. Verify asset official source + **pack-specific** license **before** download.
   Record exact URL, capture pack-page license evidence, record download date.
   Inspect packaged LICENSE if present. If the current official pack license
   page has changed since this document: STOP that asset; re-evaluate.
   Do not silently use stale W0.4 license assumptions.

D. After download, record file hashes. Do not commit corpus into RigForge unless separately authorized.

E. Record environment (OS, compiler/toolchain, Python version).

F. Record PoC-specific HELD CONSTANT / INDEPENDENT VARIABLES.

G. Do not mutate the RigForge product source tree (`src/`, tests, build, deps).

H. Write evidence outside the production tree or under a later authorized research-evidence location.

I. If a required Level-3 asset cannot be legally **and** semantically selected:
   BLOCK that PoC. Do not substitute an easier synthetic asset.
```

**Asset authority:** legal/available ≠ semantically suitable. Both must pass.

**Download policy (all named pools):** `DOWNLOAD_DURING_TEST_ONLY`. Do not vendor engines. Do not commit downloaded packs.

Floating labels forbidden in a recorded run: `latest`, `master` (except a documented WIP SHA), `4.x`, `dev`.

---

## Candidate version pins (re-check at W0-P start)

| Component | Current pin (2026-08-31) | Source | Note |
| --- | --- | --- | --- |
| ufbx | **v0.23.0** | https://github.com/ufbx/ufbx/tags | Tag, not GitHub Release object |
| TinyGLTF | **v3.0.1** (2026-08-02) | https://github.com/syoyo/tinygltf/releases/tag/v3.0.1 | v3 C runtime still experimental |
| cgltf | **v1.15** (2025-02-09) | https://github.com/jkuhlmann/cgltf/releases/tag/v1.15 | LICENSE = MIT |
| fastgltf | **v0.9.0** (2025-07-08) | https://github.com/spnda/fastgltf/releases/tag/v0.9.0 | |
| OpenUSD (if conditional) | **v26.08** (2026-07-20) | Pixar CHANGELOG / tag | |
| Three.js | **r185** (2026-07-01) latest formal GitHub Release this pass | https://github.com/mrdoob/three.js/releases/tag/r185 | Re-check; do not use `dev` |
| Godot (01B only) | **4.7.2-stable** (2026-08-18) | https://github.com/godotengine/godot/releases/tag/4.7.2-stable | Re-check |
| ufbx-write (optional) | commit SHA (no release) | repo at execution | Record SHA |
| Khronos Validator | exact tag at preflight | https://github.com/KhronosGroup/glTF-Validator | LICENSE = Apache-2.0 |
| Python | exact CPython version at preflight | python.org | Same for L1 and L2 |
| H3 shell / WebView | exact runtime once chosen | vendor | PoC **configuration**, not Workbench authority |

---

## Named asset pools (no download in W0.4)

### License evidence (official pages, 2026-08-31)

| ID | Pool | Official source | Formats / animated | License recorded | Use |
| --- | --- | --- | --- | --- | --- |
| **F-L3-QCHAR** | Quaternius Ultimate Animated Character Pack | https://quaternius.com/packs/ultimatedanimatedcharacter.html | FBX / OBJ / Blend; animated YES [OFFICIAL_DOC] | Pack page **License = CC0** (link to CC0 1.0) [SOURCE_CONFIRMED] 2026-08-31. Classification: **CC0** | Primary humanoid FBX L3. `DOWNLOAD_DURING_TEST_ONLY`. Hash after download. Do not commit |
| **F-L3-QANIMAL** | Quaternius Ultimate Animated Animal Pack | https://quaternius.com/packs/ultimateanimatedanimals.html | FBX / OBJ / Blend / glTF; animated YES [OFFICIAL_DOC] | Pack page **License = CC0** (link to CC0 1.0) [SOURCE_CONFIRMED] 2026-08-31. Classification: **CC0** | Primary non-humanoid animated FBX L3. Same download policy. Compatibility not assumed |
| **F-L3-QANIM** | Quaternius Universal Animation Library | https://quaternius.com/packs/universalanimationlibrary.html | FBX / OBJ / glTF; 120+ clips [OFFICIAL_DOC] | Pack page **License = CC0** [SOURCE_CONFIRMED] 2026-08-31. Classification: **CC0**. Optional on **semantic** usefulness, not license uncertainty | Optional humanoid motion pool |
| **F-L3-KCHAR** | Kenney Blocky Characters | https://kenney.nl/assets/blocky-characters | 3D + animation | **Creative Commons CC0** [OFFICIAL_DOC] | CC0 humanoid backup / retarget L3-A candidate |
| **F-L3-KPETS** | Kenney Cube Pets | https://kenney.nl/assets/cube-pets | 3D animals + animations | Official table: **Creative Commons CC0** (link to CC0 1.0) [SOURCE_CONFIRMED] 2026-08-31. Classification: **CC0**. Capture zip LICENSE at W0-P as immutable evidence | CC0 non-humanoid backup. Semantic preflight still required |
| **F-L2-KHR** | Khronos glTF-Sample-Assets SimpleSkin / InterpolationTest | Khronos GitHub | glTF | Per-model LICENSE.md (SimpleSkin model files CC0-1.0 per W0.1) | L2 format fixtures |

### Pack-specific license vs site-wide QAL

Quaternius also publishes **QAL v1.0** (updated 2026-08-28) at https://quaternius.com/license.html. QAL governs assets **released under that license**. Its change clause says future license versions do not retroactively alter assets obtained under an earlier version [OFFICIAL_DOC].

Research rule (source/evidence classification, not legal advice):

```text
PACK-SPECIFIC LICENSE DECLARATION
takes precedence as evidence for that named asset candidate.

QAL applies where the asset/pack is explicitly released under QAL.
A pack explicitly marked CC0 must not automatically be relabeled QAL.
A pack explicitly marked QAL must be treated under QAL.
```

Example that **both regimes exist**: Bestiary - Dungeon Monsters Kit pack page shows **License = QAL** (link to `/license.html`) [SOURCE_CONFIRMED] https://quaternius.com/packs/bestiarydungeonmonsterskit.html. That pack is **not** in the W0-P corpus.

QCHAR / QANIMAL / QANIM are **not** QAL on their current pack pages.

**Do not claim before inspection** that Quaternius FBX contains animation layers, complex Maya pivots, or MotionBuilder characterization. Those edge semantics stay on **L1 synthetic** fixtures.

Producer Maya / MotionBuilder files: `OPTIONAL` / `USER_PROVIDED` / `DOWNLOAD_DURING_TEST_ONLY` if a license can be cleared later. They are **not** the Mandatory L3 plan.

**Why representative:** Quaternius packs are real shipped animated characters/animals in interchange FBX (and glTF for animals), not two-joint toys. They are still not Canonical authority.

**Selection rule:** pick specific files **after** download + hash + joint/clip inspection. Legal availability ≠ semantic suitability.

---

## Retarget pair preflight (required before L3 runs)

Every benchmark item is:

```text
source character/skeleton
+ source Motion
+ target character/skeleton
+ expected mapping
+ expected compatibility
+ QC questions
```

For each selected pair record:

```text
SOURCE CHARACTER / SKELETON
SOURCE MOTION
TARGET CHARACTER / SKELETON
LICENSE
SOURCE FORMAT
TARGET FORMAT
ROOT COUNT
JOINT COUNT
REST/BASE POSE
SEMANTIC PROFILE / CHAIN EXPECTATION
WHY EXPECTED COMPATIBLE
WHAT DIFFERENCE MAKES THIS PAIR USEFUL
```

Do **not** choose a pair only because both files are in the same pack.

| Pair | Requirement |
| --- | --- |
| **L3-A** | Humanoid source + Motion + **compatible** humanoid target. Prefer a real challenge if present: different proportions, rest pose, or helper/twist structure |
| **L3-B** | Non-humanoid source + Motion + **compatible** non-humanoid target. Compatibility is established in **preflight**, not assumed. Two arbitrary animals are **not** automatically compatible |

If no compatible license-clean L3-B pair exists:

```text
L3-B ASSET_BLOCKED
```

Select another license-clean pool or a user-provided representative asset **before** executing that part. **Do not invent compatibility.**

L3-A / L3-B are **not** humanoid → creature unless a later row is explicitly labeled incompatible.

---

### POC-CORE-01 Equivalent L1 / L2 vertical slice

| Field | Value |
| --- | --- |
| ID | POC-CORE-01 |
| CLASS | `MANDATORY_W0P` |
| WHY MANDATORY | Core language is `PROVISIONAL_W0P_GATED`. W1 cannot choose a Domain language without equivalent-function evidence. |
| CURRENT PREFERENCE | L1 C++ Domain + C ABI |
| ALTERNATIVES | L2 Rust Domain + C ABI |
| CONTROLLED TEST QUESTION | Given the same slice, prefer L1 or L2? |
| INDEPENDENT VARIABLES | **Implementation language only** (C++ vs Rust). Toolchain differs by language and is **recorded**, not a second test. |

**HELD CONSTANT**

```text
C ABI header/contract and function signatures
lifetime / create-destroy flow
diagnostic flow
parser input (same tiny FBX) and ufbx v0.23.0
test data
Python version
primary Python FFI: ctypes   (same .py test code for both)
worker serialization: UTF-8 JSON
worker message schema: research-only {op, payload_b64, diag[]}
worker transport: stdin/stdout of one child process
compiler optimization intent: RelWithDebInfo / release-with-debug, recorded
```

Primary comparison is **C++ implementation vs Rust implementation**, not C++/pybind11 vs Rust/PyO3.

Optional **secondary** observation only (must not drive `prefer L1` / `prefer L2`): C++ pybind11/nanobind vs Rust PyO3.

**Same functional slice:** tiny research Skeleton/Motion object; nested variable-size joints/tracks; strings/ids; diagnostics; read-only C ABI query; one ufbx handoff; ctypes Python caller; one JSON/stdio worker handoff; explicit create/destroy.

Do not build final Canonical classes. Do not commit production implementation. OpenUSD is not part of this test.

**Compare:** FFI glue, ownership/lifetime, nested data, errors, diagnostics, buffers, parser friction, worker friction, build, package size, tooling, platform, memory-safety exposure, implementation complexity. ABI nanoseconds secondary.

| Field | Value |
| --- | --- |
| OBSERVED RESULT FORMAT | `OBSERVED` / `TRADEOFF` / `DECISION IMPACT`. Final sentence: `prefer L1` or `prefer L2` |
| REVERSAL | Equivalent slice records `prefer L2` on those observations. Feelings are insufficient |
| FALLBACK | The other language |
| DECISION IMPACT | Core language recommendation |
| ASSETS | L1 synthetic only |
| COST | Medium |

---

### POC-FBX-01 Reader semantics

| Field | Value |
| --- | --- |
| ID | POC-FBX-01 |
| CLASS | `MANDATORY_W0P` |
| WHY MANDATORY | V1 Core needs a real ingest path |
| CURRENT PREFERENCE | ufbx V1 ingest |
| CONTROLLED TEST QUESTION | Does ufbx represent bind/rest, nontrivial L1 pivots/helpers, and **representative real animated FBX** with explicit loss? |
| INDEPENDENT VARIABLES | Reader (ufbx; isolated SDK only if reversal) |
| L1 | Project-owned synthetic semantic-edge fixtures (pivots, helpers, layered **if constructed**) |
| L2 | Curated/public format fixtures if a license-cleared FBX exists; else skip L2 |
| L3 | **At least one** animated humanoid FBX from **F-L3-QCHAR** (CC0) **and at least one** animated non-humanoid FBX from **F-L3-QANIMAL** (CC0). Backup humanoid: **F-L3-KCHAR** (CC0). CC0 ≠ compatible / representative / suitable. Semantic preflight still required |
| MEASUREMENTS | Joint count; bind vs rest; IBM analogue; bake vs layers if present; `ufbx_error`; fail-closed loss |
| REVERSAL | Unrepresentable/incorrect semantics on the named L3 pool that cannot be loss-diagnosed |
| FALLBACK | Isolated FBX SDK read |
| DECISION IMPACT | FBX ingest strategy |

Must still include on L1 and, where the L3 files actually contain them: skinned humanoid; animated file; bind/rest; nontrivial transform; helpers. Layered animation remains L1 / optional producer file.

---

### POC-GLTF-01 Native read/write + extensions

| Field | Value |
| --- | --- |
| ID | POC-GLTF-01 |
| CLASS | `MANDATORY_W0P` |
| CURRENT PREFERENCE | Role split; cgltf first; fastgltf if write+SIMD needed |
| L2 | Khronos SimpleSkin / InterpolationTest (**F-L2-KHR**) |
| L3 | One file from **F-L3-QANIMAL** glTF **or** a license-cleared VRM — only after license preflight |
| TOOLS | Pinned cgltf / fastgltf / TinyGLTF / Validator tags from global pins |
| REVERSAL | Silent required-semantic loss, or write not a credible V1 derived publish |
| FALLBACK | Other native lib; other derived publish (not forced FBX write) |
| DECISION IMPACT | glTF library **and** whether glTF/GLB remains preferred derived publish |

---

### POC-USD-01 (`CONDITIONAL_W0P`)

Unchanged class. Run only if Final Synthesis still considers USD **V1 OPTIONAL**. Pin OpenUSD **v26.08** at start. Does not block W1.

---

### POC-RETARGET-01 Real source–Motion–target pairs

| Field | Value |
| --- | --- |
| ID | POC-RETARGET-01 |
| CLASS | `MANDATORY_W0P` |
| CURRENT PREFERENCE | Deterministic Core; method OPEN |
| L1 | Two-joint synthetic pair |
| L2 | Only if license-cleared public pair exists (Mixamo remains **REJECT** as dependency) |
| Humanoid pool | **F-L3-QCHAR** (CC0) primary; **F-L3-KCHAR** (CC0) backup; **F-L3-QANIM** (CC0) optional if semantically useful |
| Non-humanoid pool | **F-L3-QANIMAL** (CC0) primary; **F-L3-KPETS** (CC0) backup. Do not invent compatibility |
| L3-A / L3-B | See pair preflight above |
| REVERSAL | Baseline unusable on a **preflight-passed** L3-A or compatible L3-B. Not “use Unreal as Core.” If L3-B is `ASSET_BLOCKED`, do not fake a pair |
| DECISION IMPACT | Retarget method still not Canonical |

---

### POC-PREVIEW-01 Family

#### 01A — Data path B vs D

Unchanged Rev1 control: same host, same renderer, same assets, same overlays. Decides B vs D only.

L2 may use Khronos SimpleSkin **as a Preview fixture**, not as the Engine contract oracle.

#### 01B — H3 + R1 smoke

| Field | Value |
| --- | --- |
| HELD CONSTANT | One fixed payload (winner or interim of 01A). Does **not** compare B vs D |
| Three.js | Exact GitHub Release tag chosen at preflight (candidate **r185**; re-check) |
| H3 shell | Exact WebView/desktop-shell version recorded as **PoC configuration**. H3 remains an architecture candidate, not a selected Workbench product |
| Renderer risk | `WebGPURenderer` still **experimental** [OFFICIAL_DOC] |
| REVERSAL | Webview/memory fail → H2/R4. Renderer-specific fight → optional 02 |

```text
specific game engine required: NO
```

---

### POC-ENGINE-01 Family

#### 01A — Mandatory generic host stub

| Field | Value |
| --- | --- |
| ID | POC-ENGINE-01A |
| CLASS | `MANDATORY_W0P` |
| WHY MANDATORY | Generic Engine Adapter must not secretly require Unreal/Unity (or Godot) types |
| CONTROLLED TEST QUESTION | Can a completely new host implement the generic contract without pretending to be Unreal or Unity? |
| HOST | Tiny **research-only** stub. **No** Unreal, Unity, Godot, or game-engine SDK |
| FORBIDDEN IN DOMAIN / CONTRACT SURFACE | `UObject`, `AssetDatabase`, `Skeleton3D`, `HumanBodyBones` |
| STUB CONCEPTS (research names only) | derived asset reference; host asset id; host joint/bone id; host clip id; host binding; loss/diagnostics |
| INPUT | In-memory or file **research** payload produced by the stub or Domain. **Must remain executable if glTF publish fails** |
| MEASUREMENTS | Contract checklist; no host type leaked into Domain |
| REVERSAL | Contract requires engine-shaped types → **redesign contract** |
| DECISION IMPACT | Genericness oracle |

```text
mandatory generic stub: YES
Godot required for genericness: NO
specific engine required by Core: NO
```

A stub failure caused by Unreal/Unity assumptions **invalidates** the contract. A later Godot failure does **not**.

#### 01B — Optional real-engine smoke

| Field | Value |
| --- | --- |
| ID | POC-ENGINE-01B |
| CLASS | `OPTIONAL_W0P` (same family, not the genericness oracle) |
| WHEN | Only after 01A passes |
| HOST | Godot **4.7.2-stable** (re-check). `GLTFDocument` is a **host** API, not Domain |
| INPUT | Derived GLB **only if** POC-GLTF-01 validates that path. If glTF publish fails, **skip 01B**; 01A still stands |
| SEMANTICS | `ONE REAL ENGINE INTEGRATION SMOKE` ≠ `CONTRACT GENERICNESS` |
| SCOPE | **RESEARCH ONLY / INTERNAL CONTRACT VALIDATION**. Not a V1 shipping SKU |

---

### Optional remaining

POC-FBX-02, POC-DCC-01, POC-PREVIEW-02: unchanged class. DCC-01: process isolation ≠ legal safe harbor. PREVIEW-02: same payload/host; renderer only. FBX-02: ufbx-write **commit SHA**.

---

## Asset levels (policy)

| Level | Use |
| --- | --- |
| 1 Synthetic | Always, project-owned |
| 2 Curated public | Khronos / other license-cleared only |
| 3 Real-world | **Required** for POC-FBX-01 and POC-RETARGET-01 from **named** pools above |

Legal + semantic gates both required. No Mixamo in-repo. No vendor engines in-repo.
