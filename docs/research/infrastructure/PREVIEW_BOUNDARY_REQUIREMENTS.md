# Preview Boundary Requirements — W0.3

Access date: **2026-08-30**.

**This is not a renderer selection.** WebGPU, WebGL, Three.js, Babylon.js, Qt, native Vulkan, WASM core, and embedded browsers remain **OPEN** for W0.4.

Accepted product requirement: Canonical Character / Skeleton / Motion must be inspectable **without** Unreal, Unity, Blender, Maya, or another specific engine [docs/product/PRODUCT_VISION.md](../../product/PRODUCT_VISION.md).

A browser-like viewer is a **preferred product direction**, not a technology decision.

---

## What Preview is

A **derived, read-only inspection surface** for Domain objects (and QC/mapping overlays). It exists so humans and later Workbench UI can see hierarchy, skinning, motion, and diagnostics.

## What Preview is not

- Not Canonical Authority
- Not a game-engine final renderer
- Not Unreal Editor viewport
- Not Unity Game View
- Not a substitute for Domain validation
- Not a silent editor of Canonical state
- Not required to reproduce full materials, lighting, or post-process

```text
asset inspection preview
        ≠
game-engine final rendering
```

---

## Who owns Preview state

| State | Owner |
| --- | --- |
| Canonical Character / Skeleton / Motion | Domain |
| Preview Projection / Preview Scene / Preview Motion | **Derived** Preview layer |
| Renderer GPU objects | Preview frontend (throwaway) |
| Camera, scrub time, overlay toggles | Preview session (ephemeral) |

The renderer must not silently modify Canonical state.

If later editing exists:

```text
user edit
        ↓
explicit Domain command
        ↓
Canonical mutation
        ↓
Preview rebuild (derived)
```

not:

```text
renderer object mutation
        ↓
Canonical changed implicitly
```

---

## What Preview consumes (candidates — not selected)

### A. Preview reads Canonical through a Domain/View contract

```text
Canonical → stable read-only Domain/View contract → Preview API → Renderer
```

Candidate A does **not** mean the renderer reads internal Canonical struct layout or memory. That would violate PV-DATA-001.

If A remains in W0-P, it means a **stable read-only Domain/View contract** that shields renderer code from Canonical implementation details.

Benefits: no extra published projection schema. Risks: transport/version coupling if the View contract is too close to W1 fields; accidental mutation if the contract is not strictly read-only.

### B. Dedicated Preview Projection (requirement **candidate**)

```text
Canonical → Preview Projection → Preview Scene / Preview Motion → Renderer
```

Benefits: stable read-only surface; smaller payload; renderer independence; web/native share one projection. Risks: second schema to maintain; projection loss must be declared.

### C. Derived glTF/GLB Preview

```text
Canonical → derived GLB → Viewer
```

Benefits: ecosystem; browser readiness (glTF Transform WebIO [OFFICIAL_DOC]). Risks: glTF cannot express all inspection semantics (FBX recipe, mapping overlays, QC, helper classification, multi-root diagnostics); **preview falsely treated as Canonical**; skeleton-only and unknown-ext gaps.

### D. Hybrid

Projection for inspection semantics + optional derived GLB for mesh/skin/playback. GLB remains **derived**.

**W0.3 does not select A/B/C/D.** W0-P must compare coverage, payload, portability, coupling, load cost, debuggability, and round-trip confusion. Convenience must not freeze “GLB is the Preview format.”

---

## Preview generality gate

Every PV MUST below was checked against: no Unreal; no Unity; no Blender; no Maya; no humanoid-only; no mesh-required; no one-renderer.

Future support must remain possible for: humanoid, quadruped, creature, robot, mechanical, generic skeleton, **skeleton-only**, motion + skeleton, and **unbound / detached Motion** (metadata and track inspection without pose reconstruction).

---

## PV-AUTH

### PV-AUTH-001 — MUST

Preview output is **DERIVED**. It must not reverse-authorize Canonical.

### PV-AUTH-002 — MUST

Preview must not require an Engine Adapter or DCC Adapter to exist.

Forbidden architecture:

```text
Canonical → Unreal Adapter → Unreal viewport
Canonical → Unity → Game View
```

Those may exist later as **optional integrations**, not the RigForge preview path.

### PV-AUTH-003 — MUST

Preview must remain **read-only** with respect to Canonical. Session camera/time are not Canonical writes.

### PV-AUTH-004 — SHOULD

Later Workbench edits must go through Domain API, then refresh Preview.

### PV-AUTH-005 — OPEN

Whether Preview Projection is a first-class published object or an ephemeral session artifact is OPEN for W1/W7.

---

## PV-DATA

### PV-DATA-001 — MUST

The Preview layer must be able to consume a representation that is **independent of both** Canonical internal implementation details **and** any game engine.

Evidence: product vision; architecture Canonical-first. [RIGFORGE_INFERENCE]

### PV-DATA-002 — MUST

Derived Preview data must carry provenance (source Canonical identity/version, projection version, declared losses vs Canonical).

### PV-DATA-003 — SHOULD

A dedicated Preview Projection should be evaluable as a W0-P candidate even if V1 ships a thinner path.

### PV-DATA-004 — MUST

Derived GLB, if used, must be labeled derived and must not be the only way to inspect semantics glTF cannot express.

### PV-DATA-005 — OPEN

Exact projection schema is OPEN. Do not freeze glTF as Preview IR.

---

## PV-RENDER

### PV-RENDER-001 — MUST

Preview architecture must not require a specific GPU API or engine renderer. Frontends bind to a stable consumption contract.

### PV-RENDER-002 — SHOULD

Minimum visual fidelity for inspection: geometry presence; skin deformation; joint hierarchy/axes; motion playback. Not Unreal/Unity lighting parity.

### PV-RENDER-003 — OPEN

WebGPU vs WebGL vs native vs embedded browser is W0.4.

---

## PV-PLAYBACK

### PV-PLAYBACK-001 — SHOULD

CORE PREVIEW (long-term): animation playback and scrubbing of one or more clips.

### PV-PLAYBACK-002 — SHOULD

Root trajectory overlay as a **display** of Domain/QC facts, not a new root-motion law.

### PV-PLAYBACK-003 — MUST

Preview data preparation must be **separable** from interactive viewer UI where practical (headless projection for automation/MCP).

### PV-PLAYBACK-004 — OPEN

V1 ships a full viewer? Subject to V1_SCOPE note — not frozen here.

---

## PV-SKELETON

### PV-SKELETON-001 — MUST

Preview must allow skeleton hierarchy inspection **without** a mesh.

### PV-SKELETON-002 — MUST

Preview must not assume a humanoid slot set.

### PV-SKELETON-003 — SHOULD

Joint axes, rest pose, and bind-pose **diagnostics** (W0.1 rest ≠ bind) as overlays.

### PV-SKELETON-004 — SHOULD

Semantic labels / mapping overlay as display of Mapping objects (W0.2), not as Preview-owned semantics.

### PV-SKELETON-005 — OPEN

How multi-root graphs are laid out visually is OPEN.

---

## PV-MOTION

### PV-MOTION-001 — MUST

Preview must be able to inspect **Motion-level structural and metadata** information without a resolved Skeleton binding when the source representation exposes it.

Examples (not a field freeze): asset/provenance; clip list/name; time domain; time interval; track/channel inventory; sampling/key information; interpolation; root/trajectory channels where identifiable; diagnostics; binding state.

This is required so a Workbench can diagnose a motion-only asset, a missing skeleton, a broken reference, or an unresolved binding instead of refusing Preview entirely.

Evidence: W0.1 motion-only formats exist as ingest cases; parser/semantic/Canonical outcomes are distinct (AD-DIAG-002). [RIGFORGE_INFERENCE]

### PV-MOTION-002 — SHOULD

Side-by-side source/target retarget preview as a **USEFUL** long-term capability (W0.2 pair tests). Classify V1 vs post-V1 later.

### PV-MOTION-003 — OPEN

Skinning GPU path vs CPU debug skin is W0.4/W0-P.

### PV-MOTION-004 — MUST

**Skeletal pose playback** — pose reconstruction, joint transforms in a hierarchy, skeletal animation playback, and skinning — **requires** a compatible, resolved Skeleton binding.

Do not claim that a Motion can produce skeletal poses without a Skeleton model. Binding/resolution failure is a diagnosable Preview state (PV-MOTION-001), not a silent pose invention.

---

## PV-QC

### PV-QC-001 — SHOULD

QC overlays (foot slide, discontinuities, unmapped joints) consume **QC results**, not invent thresholds (W0.2).

### PV-QC-002 — MUST

QC visualization must not auto-fix motion (W0.2).

### PV-QC-003 — OPEN

Which overlays ship in V1 is OPEN.

---

## PV-PORTABILITY

### PV-PORT-001 — MUST

If browser-like Preview remains preferred, the consumption contract must admit a **portable payload** (not an Unreal/Unity project).

### PV-PORT-002 — SHOULD

Browser-like constraints to treat as requirements **if** that direction is kept: portable asset payload; load/stream boundary; possible WASM Domain or projection; GPU API abstraction; large-asset memory; animation playback; offline/local use; desktop embedding possibility.

Do not perform security probing. Sandbox implications stay high-level (untrusted files; no engine process).

### PV-PORT-003 — OPEN

WASM core vs native core + web bridge vs projection-only in JS is W0.4.

---

## Capability classification (not V1 freeze)

| Capability | Class |
| --- | --- |
| Mesh display | USEFUL |
| Skinning | USEFUL / CORE long-term |
| Skeleton hierarchy | CORE PREVIEW REQUIREMENT |
| Joint axes | USEFUL |
| Rest / bind diagnostics | USEFUL |
| Motion metadata / track / time / binding diagnostics | CORE PREVIEW REQUIREMENT |
| Skeletal pose playback (requires resolved Skeleton) | USEFUL / CORE long-term |
| Playback / scrub | USEFUL / CORE long-term |
| Multiple clips | USEFUL |
| Root trajectory overlay | USEFUL |
| Semantic labels | USEFUL |
| Mapping overlay | USEFUL |
| Retarget pair compare | USEFUL / POST-V1 |
| QC overlays | USEFUL / POST-V1 |
| Full PBR/lighting | OPTIONAL / OUT OF V1 |
| Contact viz | OPTIONAL / POST-V1 |

---

## How to preview semantics GLB cannot express

Declare loss; show Domain/QC text or overlay channels **beside** or **instead of** GLB; or use Preview Projection fields. Do not pretend GLB extras are Canonical.

## How user editing later re-enters Domain

Preview gestures → Domain commands → Canonical mutation → rebuild derived Preview. Renderer objects are never authoritative.

## Questions remaining for W0.4

Renderer/stack comparison; WASM vs native split; embedding; packaging. Not Preview **authority**.
