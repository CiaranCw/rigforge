# Technology Decision Matrix — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

Re-check triggers (all rows): major version; license change; W0-P failure; new V1 requirement; maintenance inactivity.

Classification: `DECIDE_NOW` / `PROVISIONAL_W0P_GATED` / `KEEP_AS_ALTERNATIVE` / `DEFER` / `REJECT` / `OPEN`.

Do **not** read “preferred” as “RigForge uses X” unless class is `DECIDE_NOW`.

---

## Rows

| Topic | Preferred candidate | Version | Benefits | Risks | License | Generalization | Engine independence | Evidence | PoC? | Class | Fallback |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Core language** | L1: C++ Domain + C ABI + Python + Web/TS Preview | ISO C++ (version OPEN) | Native ufbx/cgltf/fastgltf; OpenUSD/FBX SDK/engine plugins are C++; DCC plugins C++/Python; mature bindings | Memory unsafety; build complexity | Toolchain + deps | New formats via C ABI | Yes | W0.3 FFI constraints; ecosystem [RIGFORGE_INFERENCE] | **POC-CORE-01** | `PROVISIONAL_W0P_GATED` | L2 Rust + C ABI |
| **Runner-up language** | L2: Rust Domain + C ABI | edition OPEN | Ownership; WASM story | Extra FFI to USD/FBX SDK/UE/Unity; bindgen cost | Apache-2.0/MIT toolchain | Same if C ABI held | Yes | Offset vs C++ ecosystem **unproven** | Same PoC | `KEEP_AS_ALTERNATIVE` | L1 |
| **Hybrid workers** | L3: native Domain lib + per-host workers | — | Isolation for USD/DCC/engines | IPC cost; two runtimes | — | Yes | Yes | W0.3 process OPEN | With USD/DCC | `KEEP_AS_ALTERNATIVE` (compose with L1/L2) | In-process for small C libs |
| **Python as Domain** | — | — | DCC familiarity | Host Python version wars; Preview/WASM; not USD-light | PSF | Weak for engines | Yes but slow | W0.3 | No | `REJECT` as sole Core | Bindings only |
| **TypeScript as Domain** | — | — | Preview | Weak FBX/USD/engine plugins | — | Weak | Preview-only | W0.3 | No | `REJECT` as sole Core | Frontend |
| **ABI / bindings** | Stable **C ABI** + generated Python | — | Hosts, WASM, versioning | Manual ownership | N/A | New hosts | Yes | W0.3 AD-VERSION | POC-CORE-01 | `PROVISIONAL_W0P_GATED` | IPC JSON/binary for workers |
| **C++ ABI as public** | — | — | Convenience | Compiler skew | — | Poor | N/A | Industry | No | `REJECT` as public surface | C ABI |
| **FBX read** | **ufbx** | v0.23.0 | Single-file; diagnostics; bake; WASM CI; MIT/Unlicense | 0.Y breaks; recipe vs evaluate | MIT/Unlicense | Format adapter | Yes | W0.1/W0.3 HIGH | **POC-FBX-01** | `PROVISIONAL_W0P_GATED` | FBX SDK read (isolated) |
| **FBX write** | **Not mandatory V1**; if later: ufbx-write then SDK | WIP / 2020.3.x | Product does not need symmetric write | Semantics UNPROVEN; SDK EULA | See license matrix | Yes | Yes | W0.3 | **POC-FBX-02** optional | `DEFER` V1 write; I/O asymmetry `DECIDE_NOW` | Other derived publish — **not** a frozen glTF pin |
| **glTF read** | **Role split**: native parser (rank: **cgltf** preferred lightweight; **fastgltf** if C++17 + write+SIMD) | cgltf v1.15 (2025-02-09); fastgltf v0.9.0 (2025-07-08); TinyGLTF **v3.0.1** (2026-08-02) | Spec-aligned; extras/extensions | One lib ≠ validator; TinyGLTF v3 still experimental | MIT (+ Apache simdjson) | Yes | Yes | Official GitHub Releases | **POC-GLTF-01** | `PROVISIONAL_W0P_GATED` | The other native parser |
| **glTF write / preferred V1 derived publish candidate** | Same native writer **or** fastgltf Exporter; Transform for JS. **Candidate** first publish: glTF/GLB | same | Interchange publish | Extension drop if unregistered; write may fail PoC | MIT | Yes | Yes | W0.3 | **POC-GLTF-01** | `PROVISIONAL_W0P_GATED` | Other native writer; other derived publish format |
| **Validation** | **Khronos Validator** in CI/tooling, **not** sole import gate | pin tag at W0-P | Spec validity ≠ Canonical | False confidence | **Apache-2.0** [SOURCE_CONFIRMED] repo LICENSE | Yes | Yes | W0.3 AD-DIAG-002 | In POC-GLTF | `DECIDE_NOW` (role) | Domain validators |
| **USD** | **Optional adapter + separate-process candidate**; **not** in-process Core | OpenUSD **26.08** (2026-07-20) | UsdSkel + composition | Size, plugins, resolver, no official WASM | Apache-2.0 | Yes as adapter | Yes | W0.3 + AOUSD | **POC-USD-01** `CONDITIONAL_W0P` | `PROVISIONAL_W0P_GATED` (not V1 Core; does not block W1) | POST-V1 file-only / skip V1 ingest |
| **VRM** | **glTF + profile/specialization** | VRM 1.0 spec | Preserve humanoid/expr/lookAt | Must not become universal skeleton | Spec + impl | Profile only | Yes | W0.1/W0.3 | After glTF PoC | `DECIDE_NOW` (composition) | Facade adapter same parser |
| **DCC boundary** | Optional thin adapter; file ingest without DCC | — | Matches product | Process OPEN | Host EULA/GPL | New DCCs | Yes | W0.3 | Optional POC-DCC | `DECIDE_NOW` (optionality) | File interchange only |
| **Engine boundary** | Generic contract; no UObject / AssetDatabase / Skeleton3D / HumanBodyBones in Core | — | Custom stub must implement; Godot optional smoke only | Host EULA if a real engine is used later | — | Yes | **Yes** | W0.3 | **POC-ENGINE-01A** stub Mandatory; 01B Godot optional | `DECIDE_NOW` (genericness) | File handoff |
| **Storage** | **Minimal AssetReference / Provenance in Domain** | — | No AMS required | Users with AYON still need adapter later | N/A | Yes | Yes | W0.3 + OAIO 1.0.2 | No | `DECIDE_NOW` | OpenAssetIO POST-V1 |
| **OpenAssetIO** | Optional future adapter | 1.0.2 | Resolve/publish concepts | Not a schema | Apache | Yes | Yes | Official “not a database” | No | `DEFER` | None |
| **AYON** | Future pipeline adapter | Latest GitHub Release **1.9.10** (2026-08-05); prior **1.9.9** (2026-07-28) | Publish taxonomy reference | Server lock if mandatory | Apache core | Yes | Yes | GitHub Releases | No | `DEFER` | None |
| **Runtime cook** | **Keep contract; no V1 implementation** | ozz 0.16.0; ACL 2.1.0 | Preview/engines have own playback | Scope creep | MIT | Yes | Yes | V1_SCOPE; W0.3 | No | `DEFER` | ozz or ACL later |
| **MCP** | **Post-V1** over Domain API + CLI | Spec **2026-07-28** after November 2025 release; formal deprecation policy [OFFICIAL_DOC] blog | Agent automation | Protocol can still evolve; must not define Domain | MCP spec | Yes | Yes | Domain+CLI first; 2026-07-28 shows substantial evolution **and** a deprecation policy | No | `DEFER` | CLI first |

---

## Language scoring (written rationale, not star-count)

Criteria from the W0.4 prompt. Score: `FIT` / `COST` / `RISK` only.

| Criterion | C++ Domain (L1) | Rust Domain (L2) |
| --- | --- | --- |
| ufbx | `FIT` C99 drop-in | `FIT` via cc/bindgen (official Rust bindings exist historically [OFFICIAL_DOC] ufbx) |
| OpenUSD | `FIT` first-class C++ | `COST` cxx/FFI or worker |
| Engine plugins | `FIT` UE/Unity native C++ | `COST` C ABI then C++ plugin stub |
| DCC | `FIT` Maya C++; Blender via process | Same process story |
| WASM | `FIT` (ufbx CI WASI) | `FIT` often stronger story |
| Browser Preview | Via C ABI / WASM | Via WASM / C ABI |
| FFI | C ABI well-known | C ABI required anyway |
| Memory safety | `RISK` | `FIT` |
| Performance | `FIT` | `FIT` |
| Build complexity | Familiar, heavy if USD in-tree | Dual toolchain if USD/C++ hosts |
| Ecosystem | Parsers + USD + engines | Growing; less USD |
| Win/Linux/macOS | `FIT` | `FIT` |
| Python bindings | Ecosystem: pybind11 / nanobind. **POC-CORE-01 primary caller is ctypes for both languages** | Ecosystem: PyO3. Must **not** drive L1 vs L2 |
| Velocity | Faster to glue C++ libs | Faster for safe Domain internals |
| Distribution | So/dll familiar | Same if C ABI |

**Why L1 is preferred, not automatic:** most **mandatory-adjacent** libraries and **all** engine/DCC native plugins are C/C++. Rust wins safety and possibly WASM, but W0.3 already allows process isolation for USD. That offset is exactly what **POC-CORE-01** must measure.

**WHAT EVIDENCE COULD REVERSE L1:** POC-CORE-01 equivalent vertical slice records `prefer L2` on observed FFI/ownership/ergonomics/build evidence. Not “Rust feels safer.” Fallback: L2. USD deploy is POC-USD-01, not this test.

---

## FBX product questions (answers)

| Question | Answer |
| --- | --- |
| Preferred V1 FBX ingest? | **ufbx** (`PROVISIONAL_W0P_GATED`) |
| I/O asymmetry? | `DECIDE_NOW`: V1 does not require symmetric import/export for every format |
| Is FBX write mandatory for V1? | **No.** Not required because FBX read exists |
| Preferred V1 derived publish **candidate**? | **glTF/GLB** (`PROVISIONAL_W0P_GATED`, POC-GLTF-01) |
| First E2E candidate? | FBX ingest → Domain → glTF publish — **not frozen** until POC-FBX-01 and POC-GLTF-01 succeed |
| USD as V1 publish? | **Not required** |
| What W0-P changes this? | FBX corpus fails ufbx → isolated SDK read. glTF write/read fails → other derived publish. ufbx-write interop **passes** → may promote FBX write to V1 OPTIONAL |

---

## glTF role split (recommendation)

```text
native parse/write  →  cgltf or fastgltf (W0-P rank)
Khronos Validator   →  CI / tooling only
glTF Transform      →  JS CLI / Preview document ops
Domain              →  Canonical + loss
```

Do **not** equate native glTF library with Preview renderer.

TinyGLTF: **v3.0.1** released **2026-08-02**, latest GitHub Release as of 2026-08-31 [SOURCE_CONFIRMED] https://github.com/syoyo/tinygltf/releases/tag/v3.0.1. v3 C runtime remains **experimental**; v2 maintenance / sunset after mid-2026 [OFFICIAL_DOC]. v3.0.0 (2026-03-23) is the prior major tag.

---

## OpenUSD placement (recommendation)

```text
mandatory in-process Core:     REJECT
optional in-process adapter:   KEEP_AS_ALTERNATIVE (only if PoC size acceptable)
separate adapter worker:       PREFERRED placement if V1/V1-optional USD
external / later phase:        valid if POC-USD-01 (CONDITIONAL) cost too high or Final Synthesis drops V1 OPTIONAL USD
```

Industry importance ≠ V1 Core dependency.
