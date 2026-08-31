# Format Adapter Matrix — W0.3

Engineering comparison. **Not** a library selection. Access date: **2026-08-30**.

W0.1 semantics remain authoritative. This matrix covers **implementation suitability**, not transform/skin law.

## Cell legend

| Token | Meaning |
| --- | --- |
| `CONFIRMED` | Official doc/source states the capability |
| `PARTIAL` | Some cases, bake, optional stage, or subset |
| `UNCONFIRMED` | Claimed, typed, or not production-proven here |
| `UNSUPPORTED` | Official source **explicitly** absent or contradicted |
| `NO OFFICIAL WASM TARGET CONFIRMED` | Official platform lists do not include WASM; **not** a claim of technical impossibility |
| `NOT APPLICABLE` | Outside role |

---

## Matrix

| Topic | ufbx | Autodesk FBX SDK 2020.3.x | ufbx-write | cgltf | tinygltf v3 | fastgltf | glTF Transform | OpenUSD / UsdSkel | VRM (as glTF+ext) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Role | C99 single-file **reader** + evaluators | Official C++ read/write SDK | C99 single-file **writer** (WIP) | C99 parse/write | C11 parse/write | C++17 parse/write | JS/TS document SDK | C++/Python composed stage | Profile on glTF |
| Read | `CONFIRMED` [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | `NOT APPLICABLE` | `CONFIRMED` parse; buffers optional [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | `CONFIRMED` NodeIO/WebIO [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | via glTF + VRMC_* [SPEC] |
| Write | `UNSUPPORTED` (separate project) | `CONFIRMED` | `PARTIAL` mechanism; WIP [OFFICIAL_DOC] | `CONFIRMED` JSON write; buffers not auto-written [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | `CONFIRMED` Exporter [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | `CONFIRMED` [OFFICIAL_DOC] | write = glTF + extensions |
| Skin | `CONFIRMED` clusters (W0.1) | `CONFIRMED` | **API `CONFIRMED`** (`ufbxw_create_skin_deformer` / cluster / bind pose) [SOURCE_CONFIRMED] `ufbx_write.h`; semantic completeness **UNPROVEN** | `CONFIRMED` core skins | `CONFIRMED` | `CONFIRMED` | `CONFIRMED` | `CONFIRMED` UsdSkel | inherits glTF |
| Skeleton | FBX nodes + optional bone attrib (W0.1) | FBX nodes | node + bind-pose **API `CONFIRMED`** [SOURCE_CONFIRMED]; end-to-end **UNPROVEN** | node joints | node joints | node joints | node joints | `CONFIRMED` Skeleton prim | humanoid slots [SPEC] |
| Animation | stacks/layers/curves; bake resamples (W0.1) | stacks/layers | **API `CONFIRMED`** (curve / prop / layer / stack + node helpers) [SOURCE_CONFIRMED]; semantic completeness **UNPROVEN** | core animations | core animations | `CONFIRMED` | `CONFIRMED` | TimeCode / UsdSkel anim | VRMC_vrm_animation [SPEC] |
| Curve preservation | `PARTIAL` if not baked | `CONFIRMED` native | API exists; interoperable curve fidelity **UNPROVEN** | glTF samplers | glTF samplers | glTF samplers | glTF samplers | `PARTIAL` composition | samplers + ext |
| Metadata / extensions | FBX custom props `PARTIAL` | `CONFIRMED` | `UNCONFIRMED` | extras + `extensions` member; listed KHR/EXT subset [OFFICIAL_DOC] | extras/ext `PARTIAL` | extras callbacks; extensions `PARTIAL` | **must register** extensions [OFFICIAL_DOC] | schemas/plugins | VRMC_* required for meaning |
| Diagnostics | `ufbx_error` [OFFICIAL_DOC] | SDK status | `ufbxw_error` | `cgltf_result` | structured error stack [OFFICIAL_DOC] | Expected/Error | exceptions/result | Tf/Usd notices | via glTF + profile |
| Loss control | load opts change bake vs recipe (W0.1) | pivot bake APIs (W0.1) | `UNPROVEN` | unknown ext via raw member | `PARTIAL` | extras write callback [OFFICIAL_DOC] | unregistered ext dropped | composition/resolve | drop VRMC_* = profile loss |
| Headless | `CONFIRMED` (no GUI) | `CONFIRMED` library | `CONFIRMED` | `CONFIRMED` | `CONFIRMED` | `CONFIRMED` | Node CLI `CONFIRMED` | headless converter common [OFFICIAL_DOC] Exchange deploy | `CONFIRMED` if glTF is |
| WASM | CI WASI/Clang WASM [OFFICIAL_DOC] | `NO OFFICIAL WASM TARGET CONFIRMED` (APS platform list: Win/macOS/Linux; not a proof of impossibility) | `UNCONFIRMED` | feasible (C, no deps) | official Emscripten demo [OFFICIAL_DOC] | `UNCONFIRMED` (simdjson) | native JS/Web [OFFICIAL_DOC] | `NO OFFICIAL WASM TARGET CONFIRMED` for full runtime (Exchange deploy does not list WASM) | via glTF/JS |
| Language / API | C / C++ / Rust bindings [OFFICIAL_DOC] | C++ | C / C++ | C | C | C++17 | TypeScript/JS | C++ / Python | spec + host impl |
| Dependency footprint | two files | large binary SDK | two files | one/two headers | three C files + optional stb | simdjson embed [OFFICIAL_DOC] | npm packages | large (plugins, TBB, optional Python) | glTF lib + VRM schemas |
| License | MIT **or** Unlicense [SOURCE_CONFIRMED] | Autodesk LSA; SDK not redistributable as source kit [OFFICIAL_DOC] APS / historical LSA | MIT **or** Unlicense [SOURCE_CONFIRMED] | MIT (typical; confirm LICENSE) | MIT [OFFICIAL_DOC] | MIT; simdjson Apache-2.0 [OFFICIAL_DOC] | MIT (confirm package) | Apache-2.0 [OFFICIAL_DOC] | VRM spec + impl licenses |
| Maintenance | recorded tag **v0.23.0** (2026-06-21) [SOURCE_CONFIRMED] changelog; 0.Y.Z may break [OFFICIAL_DOC] | vendor 2020.3.x line [OFFICIAL_DOC] APS | WIP; breaking expected [OFFICIAL_DOC] | widely used [PROJECT_CLAIM] | v3 rewrite is mainline [OFFICIAL_DOC] | active docs 0.9.x [OFFICIAL_DOC] | active [OFFICIAL_DOC] | Pixar release train | VRM Consortium |
| Testability | 592 cases / fuzz / public+private sets [OFFICIAL_DOC] | SDK samples | writer used in ufbx v0.23 casegen [SOURCE_CONFIRMED]; interop suite **UNPROVEN** | sample-model script [OFFICIAL_DOC] | unit + libFuzzer [OFFICIAL_DOC] | CI x64/ARM [OFFICIAL_DOC] | own tests | OpenUSD tests; license per path (W0.1) | official samples |
| Production readiness | high as **reader** candidate; **not selected** | high as official R/W; **deployment/license cost** | **WIP / UNPROVEN** — not V1 exporter | high as parser | v3 young vs historic C++ attic | high as C++ parser/writer | high in JS/TS pipelines | high in DCC/film; **heavy** for Core | profile, not Core |

---

## FBX strategy candidates (not selected)

| Option | Semantic coverage | Deploy / license | Headless | Determinism | Maintenance | Cross-platform | Testability | Vendor lock |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A. ufbx read + ufbx-write write | Read strong; write **API present, semantics UNPROVEN** | Permissive files | Yes | Reader CI bit-exact claim; writer unknown | Writer WIP | C portable | Reader strong; writer interop UNPROVEN | Low |
| B. ufbx read + Autodesk write | Read strong; write official | Dual stack; SDK EULA | Yes if both linked | Writer vendor | Two owners | SDK platform matrix | Split | Medium write lock |
| C. Autodesk read/write | Official full | EULA, no SDK vendoring, binary size | Yes as library | Vendor | Vendor version coupling | Win/macOS/Linux | SDK samples | High |
| D. ufbx read only in V1; no FBX export | Read only | Permissive | Yes | Reader | One owner | High | High | Low |
| E. other | None found that displaces A–D without new evidence | — | — | — | — | — | — | — |

**Disposition:** no option frozen. Writer **mechanism** is confirmed; **semantic completeness, interoperability, and production readiness** remain UNPROVEN. **REQUIRES W0-P**. License of Autodesk SDK is a **W0.4** full-matrix item. Semantic reference authority of Autodesk docs ≠ product dependency suitability. [RIGFORGE_INFERENCE]

### ufbx-write classification (Rev1)

Do **not** collapse skin/animation to a single `UNCONFIRMED`.

| Layer | Classification |
| --- | --- |
| Skin/animation element and construction API | `SOURCE_CONFIRMED` / **MECHANISM CONFIRMED** — `ufbx_write.h` (access 2026-08-30): `ufbxw_create_skin_deformer`, `ufbxw_create_skin_cluster`, `ufbxw_create_bind_pose`, `ufbxw_anim_curve_*`, `ufbxw_animate_prop`, `ufbxw_create_anim_layer`, `ufbxw_create_anim_stack`, `ufbxw_node_animate_*` |
| End-to-end semantic completeness | **UNPROVEN** |
| Interoperability with major FBX producers/consumers | **UNPROVEN** |
| Production readiness | **WIP / UNPROVEN** [OFFICIAL_DOC] README: work-in-progress; breaking expected |
| V1 selection | **NOT SELECTED**; **REQUIRES W0-P** |

**ufbx relationship (engineering evidence, not readiness):** ufbx **v0.22.0** (2026-05-18) added `UFBX_EXPORTER_UFBX_WRITE` [SOURCE_CONFIRMED] https://github.com/ufbx/ufbx/blob/v0.22.0/misc/changelog.md. Recorded current tag **v0.23.0** (2026-06-21) [SOURCE_CONFIRMED] changelog. This strengthens the reason to **test** the path. It does **not** establish production readiness.

---

## glTF notes

Parser ≠ validator ≠ transform ≠ serializer.

- **cgltf:** parse success ≠ buffers loaded [OFFICIAL_DOC]. Unknown extensions via `extensions` member; listed KHR/EXT subset first-class.
- **tinygltf v3:** C POD + arena + structured errors; WASM demo exists [OFFICIAL_DOC]. Legacy C++ attic deprecated.
- **fastgltf:** SIMD parse; official Exporter/FileExporter including extras callback [OFFICIAL_DOC] v0.9 guides.
- **glTF Transform:** JS/TS; Node + Web; excellent for CLI/browser **glTF document** work; **not** a native Core by itself. Unregistered extensions are not round-tripped [OFFICIAL_DOC].

Unknown extension requirement: see main report §11. Policy among preserve-opaque / declare-loss / refuse / strip-under-policy is **OPEN**.

---

## OpenUSD notes

OpenUSD is far larger than ufbx/cgltf: plugin `plugInfo.json`, asset resolver, layer composition, optional Python, TBB [OFFICIAL_DOC] OpenUSD Exchange deployment.

**Not** a mandatory Core dependency on present evidence. Credible placements: **optional adapter** and/or **separate process** (strong candidate when ABI/plugin/crash isolation warrants it). Final in-process vs subprocess/service placement remains **OPEN** (`DEFER W0.4`, `REQUIRES W0-P`). Direct Core link is a size/ABI/plugin-path risk. [RIGFORGE_INFERENCE] Compose vs authored-layer import modes also `REQUIRES W0-P`.

---

## VRM notes

VRM 1.0 is glTF + standardized extensions (W0.1). Engineering composition candidates: glTF adapter + VRM **profile/specialization**; or a separate adapter facade over the same parser. Decision is not “code reuse only.” Must preserve humanoid, expressions, lookAt, VRM animation, extension provenance **without** making VRM universal (V1_SCOPE).
