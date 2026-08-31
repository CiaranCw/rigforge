# License / Dependency Matrix — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

This is **dependency governance**, not legal advice. Where text is ambiguous: `REQUIRES LEGAL REVIEW`.

Risk: `LOW` / `MEDIUM` / `HIGH` / `UNKNOWN`.

No security testing was performed.

---

## Code / SDK candidates

| Component | Version / tag (recorded) | Code license | Binary redistribution | Static / dynamic notes | Commercial use | Attribution | Source disclosure | Patent / EULA | Model / data | Account / service | Risk |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **ufbx** | v0.23.0 (2026-06-21) [SOURCE_CONFIRMED] changelog | MIT **or** Unlicense [SOURCE_CONFIRMED] | Yes under chosen license | Single-file; static typical | Yes (MIT/Unlicense) | MIT notice if MIT | No (permissive) | None documented | N/A | None | `LOW` |
| **ufbx-write** | WIP master [OFFICIAL_DOC] | MIT **or** Unlicense [SOURCE_CONFIRMED] | Same | Single-file | Yes | Same | No | None documented | N/A | None | `MEDIUM` (WIP API, not license) |
| **Autodesk FBX SDK** | 2020.3.x [OFFICIAL_DOC] APS | Autodesk LSA / EULA | SDK kit **not** redistributable as source kit [OFFICIAL_DOC]; runtime redistributables historically + acknowledgement | Dynamic vendor binaries typical | Per EULA | Per EULA | No source | **Vendor EULA** — **REQUIRES LEGAL REVIEW** at adoption | N/A | Autodesk account for SDK download | `HIGH` |
| **cgltf** | v1.15 (**2025-02-09**) [SOURCE_CONFIRMED] https://github.com/jkuhlmann/cgltf/releases/tag/v1.15 | **MIT** [SOURCE_CONFIRMED] https://raw.githubusercontent.com/jkuhlmann/cgltf/v1.15/LICENSE | Yes | Header | Yes | MIT notice | No | None in LICENSE | N/A | None | `LOW` |
| **TinyGLTF** | **v3.0.1** (2026-08-02) latest GitHub Release as of 2026-08-31 [SOURCE_CONFIRMED] https://github.com/syoyo/tinygltf/releases/tag/v3.0.1. v3.0.0 = 2026-03-23. v3 C runtime still **experimental** [OFFICIAL_DOC] | MIT [SOURCE_CONFIRMED] repo | Yes | Header / C files | Yes | MIT notice | No | None documented | N/A | None | `MEDIUM` (v2 sunset; v3 experimental) |
| **fastgltf** | v0.9.0 (2025-07-08) [SOURCE_CONFIRMED]; last C++17-oriented major [PROJECT_CLAIM] README | MIT; simdjson **Apache-2.0** [OFFICIAL_DOC] | Yes | Static + simdjson | Yes | MIT + Apache-2.0 NOTICE | No | Apache patent grant on simdjson | N/A | None | `LOW`–`MEDIUM` (C++17 + SIMD) |
| **glTF Transform** | npm `@gltf-transform/*` — pin tag at W0-P | Package LICENSE **UNKNOWN** this pass (do not treat MIT as fact until the chosen package LICENSE is read) | Yes if permissive | npm | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | N/A | npm registry | `UNKNOWN` until LICENSE read |
| **Khronos glTF Validator** | pin tag at W0-P | **Apache-2.0** [SOURCE_CONFIRMED] https://raw.githubusercontent.com/KhronosGroup/glTF-Validator/main/LICENSE | Tooling | CLI | Yes | NOTICE | No | Apache patent grant | N/A | None | `LOW` as **tool** |
| **OpenUSD** | **v26.08** (2026-07-20) [SOURCE_CONFIRMED] Pixar CHANGELOG / AOUSD | Apache-2.0 [OFFICIAL_DOC] | Yes | Large; plugins; TBB; optional Python | Yes | NOTICE | No | Apache patent | N/A | None | `MEDIUM` (size/ABI), not license |
| **OpenAssetIO** | **v1.0.2** (2026-04-21) [SOURCE_CONFIRMED] GitHub Releases | Repo LICENSE **UNKNOWN** this Rev2 pass (do not treat Apache-2.0 as confirmed until LICENSE is re-read) | UNKNOWN | C++/Python | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | N/A | Manager plugin optional | `UNKNOWN` as optional until LICENSE re-read |
| **AYON Core** | Latest listed GitHub Release **1.9.10** (2026-08-05) [SOURCE_CONFIRMED] https://github.com/ynput/ayon-core/releases/tag/1.9.10. Prior Release **1.9.9** (2026-07-28). Pins from Releases/tags only | Apache-2.0 [SOURCE_CONFIRMED] repo | Pipeline, not embed-in-Core | Python addons | Yes (Apache) | NOTICE | No | Server product is separate | N/A | AYON **server** is a service | `MEDIUM` if treated as runtime; `LOW` as future adapter (engineering only) |
| **ozz-animation** | **0.16.0** (2025-01-19) [SOURCE_CONFIRMED] | MIT [OFFICIAL_DOC] | Yes | C++17 runtime small; offline tools may need FBX SDK [OFFICIAL_DOC] | Yes | MIT | No | None | N/A | None | `LOW` runtime; `HIGH` if offline pulls FBX SDK |
| **ACL** | **v2.1.0** (2023-12-07) [SOURCE_CONFIRMED] | MIT [SOURCE_CONFIRMED] | Yes (header-only) | Header | Yes | MIT | No | None | N/A | None | `LOW` |
| **Three.js** | Formal GitHub Release **r185** (2026-07-01) this pass; re-check at W0-P. `WebGPURenderer` **experimental** + WebGL 2 fallback [OFFICIAL_DOC] https://threejs.org/manual/en/webgpurenderer.html | MIT [OFFICIAL_DOC] | Yes | npm | Yes | MIT | No | None | N/A | None | `LOW` license; **MEDIUM** engineering (experimental renderer) |
| **Babylon.js** | pin tag at W0-P if POC-PREVIEW-02 runs | Apache-2.0 stated on official site historically [OFFICIAL_DOC]; re-read LICENSE at pin time | Yes | npm | Yes if Apache | NOTICE | No | Apache patent if Apache | N/A | None | `LOW` if Apache LICENSE confirmed at pin |
| **Qt (Quick 3D / Widgets)** | Qt 6.11 docs line [OFFICIAL_DOC] | **LGPLv3** and/or **commercial** [OFFICIAL_DOC] Qt Licensing | LGPL dynamic-link obligations; static **REQUIRES LEGAL REVIEW** | Prefer dynamic if LGPL | Commercial OK under LGPL terms **or** paid | LGPL notices | LGPL: provide relink path | Qt Company commercial alternative | N/A | None | `HIGH` if Core-mandated; `MEDIUM` as optional desktop shell |
| **Blender `bpy` / CLI** | Blender 5.x LTS docs [OFFICIAL_DOC] | GPL-2.0-or-later for Blender | **Do not link Core into Blender in-process without GPL analysis** | Process/file isolation is an **engineering** candidate that avoids linking Blender into Core. Official guidance: distributed Python scripts/add-ons that use the Blender Python API are subject to GPL. Isolation is **not** a legal safe harbor | Yes as **external** host | GPL if combined work | Distributed `bpy`/add-on: **REQUIRES LEGAL REVIEW** | GPL | N/A | Blender install optional | `HIGH` in-process; file/CLI engineering `MEDIUM` pending legal review — **not** automatic `LOW` |
| **Maya API / mayapy** | Maya 2026 Tech Docs [OFFICIAL_DOC] | Autodesk EULA | Plugin for licensed Maya | Host process | Requires Maya license | EULA | No | **Vendor EULA** | N/A | Autodesk license | `HIGH` as Core; `LOW` as optional adapter |
| **Unreal Engine plugin** | UE 5.x [OFFICIAL_DOC] | Unreal EULA / source license | Engine redistribution rules | In-editor | Per Epic | EULA | Engine source access | **Vendor EULA** | N/A | Epic account | `HIGH` as Core; `LOW` as optional adapter |
| **Unity Editor plugin** | Unity 6 [OFFICIAL_DOC] | Unity EULA | Editor/runtime split | Editor | Per Unity | EULA | No | **Vendor EULA** | N/A | Unity account | `HIGH` as Core; `LOW` as optional adapter |
| **Godot** | Candidate **4.7.2-stable** (2026-08-18) [SOURCE_CONFIRMED] https://github.com/godotengine/godot/releases/tag/4.7.2-stable. Optional smoke only | MIT [OFFICIAL_DOC] | Yes | **Not** a V1 SKU; optional 01B host | Yes | MIT | No | None | N/A | None | `LOW` as optional research host |

---

## Frontier / model (see [FRONTIER_AI_MATRIX.md](FRONTIER_AI_MATRIX.md))

| Component | Version | Code | Weights | Dataset | Risk |
| --- | --- | --- | --- | --- | --- |
| SkinTokens | 2026 repo + HF | MIT code | HF ckpts; commercial **UNCONFIRMED** | ArticulationXL / VRoid / ModelsResource **REQUIRES LEGAL REVIEW** | `HIGH` as dependency |
| UniRig | SIGGRAPH 2025 line | MIT code | HF | Same family | `HIGH` as dependency |
| RigAnything | TOG 2025 | UNKNOWN | HF | UNKNOWN | `HIGH` / `UNKNOWN` |
| Mixamo | service | Closed | Adobe | Adobe ToS | `HIGH` (service lock) |

---

## Governance scores (non-license)

| Component | Maintenance | Bus factor | Cadence | API stability | Platforms | Binary size | Build complexity | Vendor lock |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ufbx | Active 0.Y.Z | Low–medium (small team) | Frequent | 0.Y may break [OFFICIAL_DOC] | Wide + WASM CI | Tiny | Trivial | Low |
| FBX SDK | Vendor 2020.3.x | Vendor | Slow | Compiler-flavor coupling | Win/macOS/Linux listed | Large | High | High |
| TinyGLTF v3.0.1 | Active; v2 maintenance | Medium | Major rewrite | v3 experimental | Wide | Small | Low | Low |
| fastgltf | Active docs 0.9 | Medium | 0.9 last C++17 note | 0.x | Desktop; WASM UNCONFIRMED | Medium | C++17 + SIMD | Low |
| OpenUSD 26.08 | Pixar + AOUSD | Institutional | Quarterly-ish | Plugin/version skew | Desktop; no official WASM | **Very large** | **Very high** | Medium (ecosystem) |
| Qt | Corporate | Institutional | Regular | LGPL/commercial | Desktop | Large | High | Medium–high |
| ozz | Maintained; 0.16.0 2025 | Low–medium | Slow | 0.16 config break | Wide + WASM | Small runtime | Medium | Low |
| ACL | MIT; 2.1.0 2023 tag; develop active | Medium | Slow tags | Header-only | Wide + WASM CI | Tiny | Low | Low |

---

## Blockers / unknowns (top)

1. **Autodesk FBX SDK EULA** — `HIGH`. Do not make it a mandatory Core link. `REQUIRES LEGAL REVIEW` before any write-path adoption.
2. **Blender GPL** — do not link Core in-process. File/CLI/`--background` is an engineering isolation candidate only. Distributed `bpy` / add-on obligations **REQUIRES LEGAL REVIEW**. Do not treat subprocess as an automatic legal safe harbor.
3. **AI weights + training-data ToS** — `HIGH` / `UNKNOWN`. Not V1 Core.
4. **Qt LGPL static link** — if a native shell is chosen, prefer dynamic LGPL **or** commercial; `REQUIRES LEGAL REVIEW`.
5. **TinyGLTF v3 experimental vs v2 sunset** — engineering risk, not license.

---

## Dependency gate (this stage)

Reject as **mandatory Core**: FBX SDK, OpenUSD full runtime, OpenAssetIO, AYON, ozz/ACL as Canonical, Qt, any engine/DCC SDK, any AI weight stack.

Allow as **optional adapter / tooling / frontend**: all of the above except treating them as Canonical.

Permissive MIT/Apache/Unlicense parsers (ufbx, cgltf, fastgltf, ACL, ozz runtime) are **license-credible** for Core-adjacent use after W0-P engineering gates.

---

## W0-P asset pools (not Core dependencies)

| Pool | Official source | License recorded 2026-08-31 | Redistribute as repo corpus | Risk |
| --- | --- | --- | --- | --- |
| **F-L3-QCHAR** Ultimate Animated Character Pack | https://quaternius.com/packs/ultimatedanimatedcharacter.html | Pack page **License = CC0** [SOURCE_CONFIRMED] | **No** — `DOWNLOAD_DURING_TEST_ONLY` | `LOW` (CC0). Capture zip LICENSE at W0-P |
| **F-L3-QANIMAL** Ultimate Animated Animal Pack | https://quaternius.com/packs/ultimateanimatedanimals.html | Pack page **License = CC0** [SOURCE_CONFIRMED] | **No** — `DOWNLOAD_DURING_TEST_ONLY` | `LOW` (CC0). Semantic pair preflight still required |
| **F-L3-QANIM** Universal Animation Library | https://quaternius.com/packs/universalanimationlibrary.html | Pack page **License = CC0** [SOURCE_CONFIRMED] | **No** — `DOWNLOAD_DURING_TEST_ONLY` | `LOW` (CC0). Optional on semantics |
| Kenney Blocky Characters | https://kenney.nl/assets/blocky-characters | **Creative Commons CC0** [OFFICIAL_DOC] | Prefer download-only unless later authorized | `LOW` |
| Kenney Cube Pets | https://kenney.nl/assets/cube-pets | Official page **Creative Commons CC0** [SOURCE_CONFIRMED] | Prefer download-only | `LOW`. Capture zip LICENSE at W0-P |

Quaternius **QAL v1.0** (https://quaternius.com/license.html, updated 2026-08-28) applies to packs **explicitly released under QAL**. It does **not** globally override pack pages that declare CC0. Example QAL pack (not in W0-P corpus): Bestiary - Dungeon Monsters Kit, pack page **License = QAL** [SOURCE_CONFIRMED] https://quaternius.com/packs/bestiarydungeonmonsterskit.html.
