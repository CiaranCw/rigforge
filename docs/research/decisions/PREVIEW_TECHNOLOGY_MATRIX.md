# Preview Technology Matrix — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

Preview is a **required long-term product capability**. Technology is **not** selected as `DECIDE_NOW` except where noted.

Hard gate: Preview architecture must not require Unreal, Unity, Blender, Maya, or another specific engine/DCC.

```text
specific game engine required: NO
```

Dimensions are **independent**:

```text
Workbench host
        ≠
renderer
        ≠
Preview data path
        ≠
native bridge
```

WebGPU / WGSL status (do **not** call either a final W3C Recommendation). Distinguish **published W3C versions** from Editor's Draft / source-repository updates.

| Spec | Status | Published W3C evidence (access 2026-08-31) |
| --- | --- | --- |
| WebGPU | **Candidate Recommendation Draft** [SPEC] | Official history https://www.w3.org/standards/history/webgpu/ lists CR Drafts including **20 August 2026** (`CRD-webgpu-20260820`) and **14 July 2026** (`CRD-webgpu-20260714`). Latest published row this pass: 20 August 2026. Living TR: https://www.w3.org/TR/webgpu/ |
| WGSL | **Candidate Recommendation Draft** [SPEC] | Official W3C published version/history: **16 July 2026**. Living TR: https://www.w3.org/TR/WGSL/. History: https://www.w3.org/standards/history/WGSL/. **Not** a final Recommendation. Published TR ≠ Editor's Draft. |

Browser support ≠ spec maturity. CR Draft is implementable; it is not “done forever.”

---

## 1. Workbench host

| Candidate | Install | Local large assets | Native adapters | Offline | Launch workers | Embed Preview | CLI coexistence | License | Class |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **H1 Web application only** | Browser | Weak (file picker / OPFS limits) | Via WASM or remote | Partial | Weak | Native | Separate | MIT stacks | `KEEP_AS_ALTERNATIVE` (Preview-only / docs) |
| **H2 Native desktop UI** (e.g. Qt Widgets/Quick) | Installer | Strong | Strong | Strong | Strong | Native 3D or webview | Easy | Qt LGPL/commercial `HIGH` governance | `KEEP_AS_ALTERNATIVE` |
| **H3 Desktop shell + embedded web** | Installer | Strong (native I/O) | Strong (shell) | Strong | Strong | Webview Preview | Easy | Shell license + web MIT | **Preferred** `PROVISIONAL_W0P_GATED` |
| **H4 Game engine as host** | Engine | Strong | Too strong | Strong | Engine | Engine viewport | Engine | Vendor | **`REJECT`** |

Preferred **H3** because: browser-like Preview (product preference) + local files + adapter workers + no engine host. W0-P must show webview + large GLB/projection memory is acceptable.

**WHAT EVIDENCE COULD REVERSE H3:** webview cannot play/inspect required overlays or leaks memory on Level-3 assets → H2 native renderer. Or users only need a website → H1 for Preview, CLI for Domain.

Runner-up: **H2**.

---

## 2. Renderer

| Candidate | Engine-free | Cross-platform | Browser | Desktop embed | Offline | Skeleton/overlays | Large assets | WASM/native | Effort | License | Class |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **R1 Three.js `WebGPURenderer`** | Yes | Yes | Yes; **WebGL 2 fallback**; renderer still **experimental** (maturity improved) [OFFICIAL_DOC] https://threejs.org/manual/en/webgpurenderer.html. Pin formal GitHub Release at W0-P (candidate **r185**, 2026-07-01; re-check). Do not use `dev` | Via webview | Yes | Lines/gizmos/custom materials `FIT` for inspection | Medium | JS; Domain via WASM/bridge | Lower for web 3D | MIT | **Preferred** `PROVISIONAL_W0P_GATED` |
| **R2 Babylon.js WebGPUEngine** | Yes | Yes | Yes; WebGL side-by-side [OFFICIAL_DOC] https://doc.babylonjs.com/setup/support/webGPU | Via webview | Yes | Strong inspector ecosystem | Medium–high | JS | Higher engine surface | Apache-2.0 | `KEEP_AS_ALTERNATIVE` |
| **R3 Custom WebGPU/WebGL** | Yes | Yes | Yes | Yes | Yes | Full control | Unknown | Full | **High** | Own | `DEFER` |
| **R4 Qt Quick 3D** (RHI: D3D/Metal/Vulkan/OpenGL) [OFFICIAL_DOC] Qt 6.11 | Yes | Desktop | WASM limited | Native | Yes | Custom viz possible | Medium | C++ | Ties host to Qt | LGPL/commercial | `KEEP_AS_ALTERNATIVE` with H2 |
| **R5 wgpu / Dawn native** | Yes | Yes | Via wgpu-web | Native | Yes | Full control | Unknown | Excellent | High | BSD/Apache mix | `DEFER` |
| **R6 Unreal / Unity viewport** | **No** | Host | No | Host | Host | Host | Host | Host | — | EULA | **`REJECT` as Preview architecture** |

R1 preferred over R2 because inspection needs **small, explicit overlays** (joints, axes, mapping, QC), not a full game/material stack. Babylon’s inspector is valuable but heavier and easier to confuse with “the engine.” Popularity is **not** the criterion. Experimental status is a **visible risk**, not by itself a downgrade.

**WHAT EVIDENCE COULD REVERSE R1:** POC-PREVIEW-01B (or optional 02) cannot draw skeleton-only / QC overlays / unbound Motion UI without fighting Three.js → try R2 or R4.

PoC split: **POC-PREVIEW-01A** decides B vs D (same host/renderer/assets). **POC-PREVIEW-01B** smokes H3+R1 with one fixed payload. Do not confound those tests.

---

## 3. Preview data path (W0.3 A/B/C/D)

Rank for **semantic coverage**, QC, unbound Motion, skeleton-only, non-humanoid, transport, coupling, web/native share.

| Path | Coverage | QC / mapping overlays | Unbound Motion | Skeleton-only | Transport | Coupling | Interop | Rank | Class |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **A** Domain/View contract (not Canonical memory) | High if View is rich | High | High | High | Harder in browser | Medium if View ≈ W1 | Native-first | 3 | `KEEP_AS_ALTERNATIVE` |
| **B** Dedicated Preview Projection | High by design | High | High | High | Designed portable | Second schema | Best shared | **1** | **`PROVISIONAL_W0P_GATED`** |
| **C** Derived GLB only | Mesh/skin/clip | **Low** (glTF cannot own QC/mapping/recipe) | **Low** | Partial | Excellent | Low | Excellent | — | **Insufficient alone** `REJECT` as sole IR |
| **D** Hybrid: B + optional derived GLB | High + ecosystem | High via B | High via B | High via B | GLB for mesh | Must label GLB derived | Excellent | **2** | **`PROVISIONAL_W0P_GATED`** |

**Top two for PoC:** **B vs D**.

**WHAT EVIDENCE COULD REVERSE B-first:** if projection cost ≈ GLB and overlays can ride beside GLB without a second schema, prefer D. If even B is too heavy for V1, ship A for desktop-only Minimal Workbench and keep B as architecture.

Candidate A remains: **stable read-only Domain/View**, never renderer→struct/memory (W0.3 PV-DATA-001).

---

## 4. Native bridge

| Candidate | Role | Class |
| --- | --- | --- |
| **C ABI / WASM Domain** in Preview process | Headless projection + optional in-page WASM | `PROVISIONAL_W0P_GATED` |
| **HTTP/IPC to Domain worker** | Isolation; large assets | `KEEP_AS_ALTERNATIVE` / compose with H3 |
| **No bridge (JS reimplementation of Canonical)** | Drift risk | `REJECT` as authority |
| **Engine plugin bridge** | Optional later integration | Not Preview architecture |

---

## 5. Preferred combination (not a freeze)

```text
Host:     H3 desktop shell + embedded web     PROVISIONAL_W0P_GATED
Renderer: R1 Three.js WebGPURenderer
          + official WebGL 2 fallback         PROVISIONAL_W0P_GATED
Data:     B vs D (PoC); C never sole IR
Bridge:   Domain C ABI or local worker
          → Preview Projection payload
```

V1 may ship a **minimal inspection Preview** (skeleton + clip + diagnostics) without a polished Workbench. Long-term architecture stays the same.

---

## 6. Preview gate checklist

| Must remain possible | Status |
| --- | --- |
| Humanoid / quadruped / creature / robot / mechanical / generic | Yes (data path B/D) |
| Skeleton-only | Yes |
| Unbound Motion metadata | Yes (PV-MOTION-001) |
| Bound skeletal playback | Yes (PV-MOTION-004) |
| Retarget pair + QC overlays | Yes (display Domain/QC; no auto-fix) |
| No Unreal/Unity/DCC required | Yes |
