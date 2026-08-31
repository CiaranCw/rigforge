# POC-CORE-01 — Core language vertical-slice experiment

**Status:** `COMPLETE / PASS`

**Date:** 2026-08-31

**Revision:** Rev1 (integration fairness / semantic equivalence / evidence integrity)

**Classification:** RESEARCH ONLY / W0-P / NON-PRODUCTION

**Decision impact:** `INCONCLUSIVE`

**Core language selected:** NO (`PROVISIONAL_W0P_GATED`)

This is not production Core, not W1 Canonical, not a public ABI freeze, and
not a performance contest.

W0.4 documents are **not** rewritten by this report.

External review of the original run did **not** accept `KEEP_L1`. This Rev1
preserves the original primary raw-C evidence and adds the missing
sensitivity, equivalence, ABI-boundary, and source-binding evidence.

---

## 1. Question

Given accepted RigForge requirements, is the current preference

```text
L1: C++ Domain/Core + stable C ABI + Python + later Web/TS Preview
```

still justified over

```text
L2: Rust Domain/Core + stable C ABI + Python + later Web/TS Preview
```

after implementing the same representative vertical slice?

W0.4 class: `PROVISIONAL_W0P_GATED`. This experiment is allowed to reverse it.

Possible recorded outcomes: `KEEP_L1` | `SWITCH_TO_L2` | `INCONCLUSIVE` | `REDESIGN_SPLIT`.

---

## 2. CURRENT PREFERENCE (W0.4, unchanged)

```text
L1 C++ Domain + C ABI
```

Runner-up: L2 Rust Domain + C ABI.

This file records the PoC outcome. It does not freeze W0.4.

---

## 3. Held-constant table (PRIMARY CONTROLLED RUN)

| Variable | C++ | Rust raw-C | Held constant? |
| --- | --- | --- | --- |
| Logical model | same research Skeleton/Motion/Diagnostic | same | YES |
| C ABI header | `abi/rigforge_poc_core.h` | same | YES |
| Python caller | `python/run_poc.py` ctypes | same file | YES |
| Python assertions | same | same | YES |
| Worker JSON schema | `worker/protocol.md` | same | YES |
| Worker transport | stdin/stdout UTF-8 line | same | YES |
| Serialization | UTF-8 JSON | UTF-8 JSON | YES |
| ufbx version | v0.23.0 `fcc5d6ba…` | same `ufbx.lib` | YES |
| ufbx options | zeroed `ufbx_load_opts` | same via C accessor | YES |
| Input fixtures | `minimal_chain.fbx` + `long_name.fbx` | same | YES |
| Optimization intent | RelWithDebInfo | `opt-level=2` `debug=1` | YES |
| Implementation language | C++ | Rust | **NO — independent variable** |
| Compiler/toolchain | MSVC 19.36.32548 | rustc 1.98.0 + same MSVC linker | recorded, not a second test |

pybind11 / nanobind / PyO3 were **not** used and do **not** score L1 vs L2.

The primary run compares **direct raw-C integration of the same prebuilt
`ufbx.lib`**. That comparison remains valid. It does **not** by itself prove
that a realistic Rust RigForge Core needs the handwritten C shim.

---

## 4. Research data model (POC ONLY — NOT CANONICAL)

See [experiments/w0p/poc_core_01/ALGORITHM.md](../../experiments/w0p/poc_core_01/ALGORITHM.md).

- Skeleton: joints from ufbx (skip implicit `is_root`)
- Motion: one synthetic clip `poc_walk` (native FBX animation **not** consumed; that is POC-FBX-01)
- Diagnostics: 4 entries including a designed empty rotation-key collection on the last joint
- Nested variable-length arrays, strings, hierarchy, invalid-index and missing-file paths
- Long-name fixture: one joint whose name is 1105 ASCII bytes (string-boundary equivalence only)

---

## 5. PRIMARY RAW-C RESULT

This section is the original language-only controlled comparison, after the
Rev1 semantic-equivalence fix (two-pass node names) and ABI unwind wrappers.
The independent variable remains C++ vs Rust against the **same prebuilt
ufbx.lib / same raw C API**.

```text
C++ functional result:        PASS  (minimal_chain + long_name + worker)
Rust raw-C functional result: PASS  (minimal_chain + long_name + worker)
```

Same ctypes script. Worker JSON bytes identical on this host/fixture
(Rev1 `rev1_worker_cpp.txt` stdout SHA-256 == `rev1_worker_rust.txt` stdout
SHA-256 == `rev1_worker_rust_s.txt` stdout SHA-256).

Original primary-run worker files (`worker_cpp.txt` / `worker_rust.txt`)
remain identical to each other and are retained.

What this primary result **does** establish:

> Direct raw-C integration of ufbx is cheaper in C++ than in the current
> hand-written Rust raw-FFI path (198 physical lines of `ufbx_access.c` +
> `.h`, versus C++ `#include "ufbx.h"`).

What it **does not** establish:

> A realistic Rust RigForge Core necessarily needs that manual C shim.

---

## 6. OFFICIAL-RUST SENSITIVITY (POC-CORE-01S)

**Label:** `SENSITIVITY ANALYSIS` — **not** `PRIMARY CONTROLLED RUN`.

**Not** a new Mandatory W0-P PoC.

Changed variable only:

```text
Rust native-library integration strategy:
  primary:   manual raw-C shim over prebuilt ufbx.lib
  01S:       official crates.io ufbx =0.11.2
```

Pin [OFFICIAL_DOC]: ufbx v0.23.0 changelog (`misc/changelog.md` in tag
`v0.23.0`) associates that release with `ufbx-rust 0.11.2`. Do not use
floating latest (`0.11.3` existed on crates.io at run time and was **not**
used).

ufbx maintains an official Rust binding project (`ufbx/ufbx-rust`, crate
`ufbx`). That is current maintained pairing evidence, not a stale/historical
aside.

### 01S recorded facts

| Item | Observation |
| --- | --- |
| ufbx crate | `=0.11.2` (Cargo.lock checksum `163c9b35…b66f2`) |
| Underlying C | crate bundles ufbx **0.23.0** (`UFBX_HEADER_VERSION ufbx_pack_version(0, 23, 0)`) |
| Consumer bindgen/libclang | **not required**; crate ships `generated.rs` and compiles `ufbx/ufbx.c` via `cc` |
| Upstream generated.rs | 5825 lines — **not** RigForge handwritten LOC |
| RigForge handwritten C shim | **0** |
| RigForge Domain crate | `experiments/w0p/poc_core_01/rust_s/` (`lib.rs` 788 LOC) |
| RigForge-written `unsafe` | 48 `unsafe` tokens in `rust_s/src/lib.rs` (ABI belt + one `load_file_raw`; comments stripped) |
| Domain extraction | crate `Node` / `Transform` / `element.name.length` / parent `element_id` |
| Error mapping | `ErrorType::FileNotFound` → `RF_POC_ERR_IO`; else parse |
| Load opts | `RawLoadOpts::default()` ≡ zeroed C `ufbx_load_opts` |
| Functional | **PASS** same ctypes assertions (chain + long-name + worker) |
| Build/size vs raw-C | **not compared as if packaging were identical** |

### Sensitivity question

> Does using the official supported Rust ufbx binding materially remove the
> native-dependency integration tax that currently drives KEEP_L1?

**Observed answer:** **yes, for this library.** The 198-LOC manual FFI
disadvantage is an artifact of the raw-C controlled comparison, not a
necessary cost of a Rust Core that uses the supported binding.

Remaining integration differences that **do** exist on the supported path,
and are **not** treated as a 196-LOC-class tax:

- Cargo + `cc` compile of bundled C versus C++ `#include` + prebuilt lib
  (different integration strategies; do not score build milliseconds)
- Version coupling: crate `0.11.2` is changelog-paired with ufbx v0.23.0;
  C++ can pin the C tag independently
- Dual toolchain (rustc + MSVC) remains for any L2 Windows host used here

Those leftovers are real and modest. They do **not** preserve the original
claim that native-dependency fit **materially** favors L1 for ufbx.

---

## 7. EXPECTED

W0.4 expected L1 to remain if C/C++ native libraries stay materially easier
to integrate **and** this slice does not reveal a Core-level ownership/safety
problem that outweighs that.

SWITCH_TO_L2 would require comparable native/FFI cost **and** materially
better ownership/safety/maintainability on the same slice.

INCONCLUSIVE if those tradeoffs stay balanced without a decisive margin.

Rev1 rule: if the official Rust path removes most of the material
integration disadvantage, the statement “native dependency fit materially
favors L1” **must be downgraded**, then the final result reassessed. Do not
preserve `KEEP_L1` merely because the primary raw-C run favored C++.

---

## 8. OBSERVED — C++ (primary raw-C)

| Topic | Observation |
| --- | --- |
| Implementation | One `.cpp` Domain file includes `ufbx.h`, copies joints, attaches synthetic motion, exports the C ABI. |
| FFI | Direct C API. No shim. `ufbx_node` fields used as documented. Names copied with `node->name.length` (no fixed cap). |
| Ownership | `new`/`delete` opaque `Asset`; live `unordered_set`; caller buffers for strings. ufbx scene freed before return. |
| Errors | Load failures → NULL + `RF_POC_ERR_IO` / `PARSE` / `NULL`. Queries: invalid index / null / destroyed handle. |
| ABI unwind | **Every** `extern "C"` export is wrapped: status → `RF_POC_ERR_PARSE`; count → `-1`; destroy swallows. PoC-only, not a production exception policy. |
| ufbx | Include + link the shared static lib. Zeroed opts. FILE_NOT_FOUND mapped to IO. |
| Python | Same ctypes script; PASS (chain + long-name). |
| Worker | Hand-rolled JSON/Base64; stdin/stdout; PASS; output matches Rust raw-C and 01S. |
| Build | CMake + NMake RelWithDebInfo. Rev1 rebuild log in `rev1_build_cpp.txt` (command / exit / stdout / stderr / elapsed). |
| Safety exposure | Manual lifetime of the opaque object and the live-set invariant. No interior views returned. `reinterpret_cast` handle. |

---

## 9. OBSERVED — Rust raw-C (primary)

| Topic | Observation |
| --- | --- |
| Implementation | `lib.rs` Domain + `cdylib` ABI. Same algorithm. |
| FFI | Extra C file `ufbx_access.c` (~198 physical LOC with header) wraps the **same** `ufbx_load_file` and field reads. Two-pass name query (`NULL` buffer → exact length → allocate `nlen+1`). |
| Ownership | `Box` + `OnceLock<Mutex<HashSet>>` live-set; `Box::from_raw` on destroy; caller buffers. |
| Errors | Same status codes. Mutex poison → treat as not live. |
| ABI unwind | `catch_unwind` on **every** `extern "C"` export (load/destroy previously; queries added in Rev1). Panic → `RF_POC_ERR_PARSE` / `-1`. Do not treat abort-on-panic as the error contract. |
| Residual unsafe | Invalid pointers remain UB. `catch_unwind` does not make FFI memory-safe. |
| ufbx | Same `ufbx.lib`, same version, same fixtures, same opts (zeroed in the C accessor). |
| Python | Same ctypes script; PASS (chain + long-name). |
| Worker | Same protocol; PASS; JSON bytes identical to C++. |

---

## 10. Semantic equivalence (long-name)

Project-authored fixture `fixture/long_name.fbx` (ASCII FBX 7.5; **not** an
FBX-semantics oracle). Expected name is `Long_` + 1100 × `A` = **1105**
bytes (`fixture/long_name.expected.txt`).

This test only proves the string/lifetime boundary. It is not FBX validation.

```text
C++      load + exact name + exact byte length: PASS
Rust raw-C                                 PASS
POC-CORE-01S                               PASS
```

Rev1 root cause of the original gap: Rust used a fixed `[u8; 1024]` while
C++ copied `node->name.length`. That truncation is removed.

---

## 11. ABI boundary and shared harness limitation

Audited exports (header `rigforge_poc_core.h`): load, destroy, asset_id,
joint (count/parent/name/trs), motion (count/name/range), track
(count/joint/key counts/keys), diagnostic (count/get). **All** wrapped.

Shared limitation (**does not score L1 vs L2**):

```text
The pointer-address live registry is a research harness mechanism.
It validates immediate post-destroy misuse on the single-threaded tested path.
It is NOT a production stale-handle / ABA / concurrent query-destroy design.
```

This PoC was **not** expanded into concurrency testing.

Narrowed lifetime wording (replaces the original overclaim):

> No differentiating C++ ownership/lifetime failure was observed in the
> single-threaded tested slice.
>
> The shared research handle registry has known production-level limitations
> outside this PoC's scope.

This is **not** formal memory-safety validation.

---

## 12. Comparison evidence

Do not overclaim precision. Agent wall-clock implementation time is **not**
a scientific primary metric. Do **not** treat 01S build milliseconds or DLL
size as a language contest against the raw-C packaging.

| Item | C++ | Rust raw-C | 01S (sensitivity) | Material? |
| --- | --- | --- | --- | --- |
| Functional (chain) | PASS | PASS | PASS | no difference |
| Functional (long-name) | PASS | PASS | PASS | no difference after Rev1 fix |
| Worker JSON | identical bytes | identical | identical | no |
| Extra FFI glue | 0 | 198 C LOC | **0** (upstream crate) | **yes in primary; removed in 01S** |
| Generated bindings (RigForge-written) | 0 | 0 | 0 (crate-owned 5825-line `generated.rs`) | — |
| Domain LOC (`poc_core.cpp` / `lib.rs`) | 720 | 810 | 788 | small |
| `unsafe` tokens in Domain lib | n/a (whole TU manual) | 55 | 48 | ABI belt in both Rust paths |
| Python | identical | identical | identical | no |

Original primary-run build medians (retained, secondary): C++ ~8838 ms,
Rust raw-C ~5267 ms. Rev1 logs capture command/exit/stdout/stderr/elapsed
and are **not** a decision input.

---

## 13. Criteria table (after 01S)

| Criterion | C++ | Rust (supported path) | Material difference? | Decision impact |
| --- | --- | --- | --- | --- |
| FUNCTIONAL FITNESS | PASS | PASS | no | neither eliminated |
| NATIVE DEPENDENCY FIT (raw-C primary) | Direct `ufbx.h` | C shim required **in that design** | **yes** | favors L1 **only** for raw-C |
| NATIVE DEPENDENCY FIT (01S) | Direct `ufbx.h` | official crate; no shim; no consumer bindgen | modest (cc vs include; version coupling) | **downgraded** — not a KEEP_L1 driver |
| C ABI FIT | try/catch on all exports | cdylib + catch_unwind on all exports | small | both acceptable |
| PYTHON FIT | ctypes PASS | ctypes PASS | no | none |
| WORKER/PROCESS FIT | PASS | PASS | no | none |
| OWNERSHIP / SAFETY BURDEN | Manual new/delete + registry; no differentiating failure in the tested slice | Safer interiors; ABI still unsafe | real but not decisive at the **public C ABI** | not enough to switch |
| BUILD / DISTRIBUTION | CMake+MSVC | cargo+cc+MSVC linker+isolated rustup | dual toolchain for L2; 01S packaging ≠ raw-C | slight L1 ops cost if Rust is added; not decisive |
| DEBUGGING / TOOLING | MSVC | rustc + MSVC | qualitative | none |
| PORTABILITY | MSVC x64 this pass | windows-msvc this pass | not compared off-Windows | none this slice |

---

## 14. TRADEOFFS

- **Primary raw-C:** C++ reaches this C parser with an include. Rust needed
  extra C (or would need bindgen/libclang on the consumer).
- **Supported Rust path:** official `ufbx` 0.11.2 removes that handwritten
  shim. Consumer-side libclang is not inherently required.
- Rust makes *post-copy* Domain internals harder to alias incorrectly. The
  product-stable surface is still a C ABI, which every candidate implemented
  with a live-handle table and caller buffers.
- Official crate couples crate version to the bundled ufbx C sources. C++
  can pin `ufbx.h` / `ufbx.lib` without that crate mapping. That is
  maintenance coupling, not a 198-LOC integration wall.
- Build milliseconds and DLL sizes remain secondary and are not used to
  decide language.

---

## 15. GAPS

- OpenUSD, Autodesk FBX SDK, and engine plugin stubs are **not** in this
  slice (W0.4: USD is POC-USD-01). Inferring that those untested libraries
  would restore a **material** L1 native-fit advantage is
  [RIGFORGE_INFERENCE], not a result of this PoC. Rev1 does **not** keep
  `KEEP_L1` on that inference.
- bindgen-on-the-consumer was not measured (no libclang); 01S shows it is
  not required for this supported ufbx path.
- No Linux/macOS run.
- No ABI nanosecond microbenchmark (by contract: not decisive unless
  material to a real workload; this slice is not).
- Synthetic motion is Domain-side; FBX animation correctness is POC-FBX-01.
- Implementation-time as a human metric is confounded by Agent behavior.
- Handle registry is not a production lifetime design (shared limitation).

---

## 16. ROOT CAUSES

1. ufbx is a C API with a large typed scene graph. C++ consumes it as C.
   Rust cannot name those struct layouts without generated or handwritten
   glue **if** the experiment forbids the official binding.
2. The official binding **is** that generated glue, maintained upstream and
   changelog-paired with ufbx v0.23.0. Holding it out of the original
   decision made KEEP_L1 rest on an integration strategy the product would
   not be required to use.
3. The public surface is a C ABI and ctypes. That forces every language into
   manual lifetime at the boundary, which caps how much Rust safety can show
   on this slice.
4. The original 1023-byte Rust name buffer was an implementation bug, not a
   language property.

---

## 17. DECISION IMPACT

### PRIMARY RAW-C RESULT

```text
Favors L1 for direct raw-C ufbx.lib integration.
```

### SUPPORTED-RUST SENSITIVITY RESULT

```text
Official ufbx 0.11.2 removes the material handwritten-FFI tax for this library.
Native-dependency fit no longer materially favors L1 on the supported path.
```

### FINAL DECISION IMPACT

```text
INCONCLUSIVE
```

Reasoning (not a weighted score):

- The W0.4 guide was: keep L1 if native ecosystem integration stays
  **materially** simpler **and** this slice does not surface a Core-level
  ownership/safety problem that outweighs it.
- On the primary raw-C run, native integration **was** simpler in C++.
- On the supported Rust path required by Rev1, that material tax is gone
  for the library this PoC actually used. The native-fit KEEP_L1 driver is
  therefore **downgraded**.
- No differentiating C++ ownership/lifetime failure was observed in the
  single-threaded tested slice. The shared registry is not a production
  proof of C++ unsafety **or** of C++ safety.
- Rust interior safety still does not remove the C ABI unsafe belt that
  Core must keep (`PROVISIONAL_W0P_GATED` public C ABI).
- Remaining differences (dual toolchain, crate↔C version coupling, include
  vs `cc`) are real but not a decisive margin.

`KEEP_L1` is **not** recorded: it would ignore 01S and treat a raw-C-only
tax as a language law.

`SWITCH_TO_L2` is **not** recorded: native/FFI cost is now comparable on the
supported ufbx path, but the ownership/safety delta is still not material
at the stable C ABI, and this slice does not show L2 winning the original
joint test (comparable cost **and** materially better safety).

`REDESIGN_SPLIT` is not indicated: one Domain language + C ABI still fits
this slice. L3 workers remain a compose option from W0.4, not a result of
this PoC.

W0.4 preference is **neither confirmed nor contradicted with a freeze**.
Language remains `PROVISIONAL_W0P_GATED` until W0 synthesis after remaining
Mandatory PoCs. Documents under `docs/research/decisions/` are unchanged.

---

## 18. Rebuild / rerun

See [experiments/w0p/poc_core_01/README.md](../../experiments/w0p/poc_core_01/README.md).

External work tree: `F:\NewResearch\rigforge_w0p_work\poc_core_01\`

Evidence: `F:\NewResearch\rigforge_w0p_evidence\poc_core_01\`

Source-bound hashes: [experiments/w0p/poc_core_01/review_evidence/sha256_manifest.txt](../../experiments/w0p/poc_core_01/review_evidence/sha256_manifest.txt)

---

## 19. Result token

```text
POC-CORE-01: PASS
Decision Impact: INCONCLUSIVE
Core language selected: NO
```

External focused review accepted this Rev1 evidence.
W0.4 C++ preference remains historical/provisional and coexists with this
`INCONCLUSIVE` result until W0 Final Synthesis. This is not a Core language
implementation freeze.
