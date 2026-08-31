# Product Space Matrix — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

This is a **product necessity** comparison, not a feature-count contest.

Question:

> Does a mature product already own the exact cross-system infrastructure problem RigForge intends to solve?

The RigForge problem (accepted product + W0.1–W0.3) is:

```text
cross-format ingest (FBX / glTF / USD / others)
        → format-independent Canonical Character / Skeleton / Motion
        → inspect + loss/provenance
        → Semantic Mapping (not host slots)
        → Compatibility
        → Deterministic Retarget + QC
        → engine-independent Preview
        → headless Domain API / CLI
        → optional DCC / engine publish
```

without making any DCC, engine, format, or renderer Canonical authority.

---

## Legend

| Token | Meaning |
| --- | --- |
| `YES` | First-class for that product’s stated domain |
| `HOST` | Yes, but only inside that host / service |
| `PARTIAL` | Some cases, humanoid-only, or export-only |
| `NO` | Not the product’s job |
| `UNKNOWN` | Official text insufficient this pass |

Evidence tags follow [docs/research/README.md](../README.md).

---

## Matrix

| Criterion | MotionBuilder 2026 | Maya HumanIK 2026 | Unreal 5.x IK Rig / Retargeter | Unity 6 Humanoid | Rokoko Studio | Cascadeur | Character Creator / iClone | Mixamo | Blender (Rigify / retarget add-ons) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Cross-format ingest as **independent** Canonical | `NO` | `NO` | `NO` (Interchange → UE) | `NO` | `PARTIAL` (own + FBX) | `PARTIAL` | `PARTIAL` | `HOST` (service) | `PARTIAL` (Blender scene) |
| Generic / non-humanoid skeleton | `PARTIAL` (quad characterization) | `PARTIAL` | `PARTIAL` (chains) | `PARTIAL` (Generic ≠ Humanoid) | `NO` / humanoid mocap | `PARTIAL` | `NO` (CC humanoid) | `NO` | `YES` (any armature) |
| Semantic mapping as **portable identity** | `HOST` HumanIK slots | `HOST` | `HOST` chains | `HOST` Avatar | `HOST` | `HOST` | `HOST` | `HOST` Mixamo names | `HOST` / add-on |
| Compatibility analysis (layers, fail-closed) | `NO` | `NO` | `PARTIAL` logs | `PARTIAL` Avatar match | `NO` | `NO` | `NO` | `NO` | `NO` |
| Deterministic retarget (policy + evidence) | `HOST` solver | `HOST` | `HOST` op stack | `HOST` | `HOST` | `HOST` / AI pose | `HOST` | `HOST` service | add-on dependent |
| Retarget QC as first-class metrics | `PARTIAL` visual | `PARTIAL` | `PARTIAL` output log | `NO` | `PARTIAL` | `PARTIAL` | `NO` | `NO` | `NO` |
| Batch / headless | `PARTIAL` | `YES` (`mayapy`) | `PARTIAL` commandlet | `YES` `-batchmode` | `NO` typical | `NO` typical | `NO` typical | service API `PARTIAL` | `YES` `--background` |
| API / automation of **Canonical Domain** | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | service | `NO` (bpy is Blender) |
| Cross-DCC | `NO` | `NO` | `NO` | `NO` | export | export | export | export | export |
| Cross-engine | `NO` | `NO` | `NO` | `NO` | export | export | export | export | export |
| Standalone asset browser (engine-free) | `NO` (is a DCC) | `NO` | `NO` | `NO` | `HOST` app | `HOST` app | `HOST` app | web service | `NO` (is a DCC) |
| Engine-independent Preview of **Canonical** | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | web preview of Mixamo | `NO` |
| Provenance / loss diagnostics | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` | `NO` |
| Publish + runtime cook as **derived** | `NO` | `NO` | `HOST` cook | `HOST` cook | `NO` | `NO` | `NO` | `NO` | `NO` |
| Extensible open integration | Autodesk | Autodesk | Epic plugin | Unity package | closed core | closed core | Reallusion | Adobe service | GPL add-ons |
| License / business | Autodesk subscription | Autodesk subscription | UE license | Unity license | subscription | subscription | subscription | Adobe ToS | GPL / add-on mix |

Other serious products checked at lighter depth (same gap): AccuRIG / ActorCore (Reallusion auto-rig **service**, humanoid); Houdini KineFX (powerful, **Houdini-canonical**); Godot animation (engine-canonical). None own the infrastructure problem.

---

## What each product actually owns

| Product | What it solves | What it explicitly does not |
| --- | --- | --- |
| MotionBuilder | DCC mocap / characterization / plot | Format-independent Canonical; engine-free Domain API |
| Maya HumanIK | In-Maya characterization and retarget | DCC-independent ingest; portable mapping identity |
| Unreal IK Rig / Retargeter | In-UE chain retarget and animation ops | Any non-UE Canonical; Preview without UE |
| Unity Humanoid / Generic | In-Unity Avatar and clip playback | Cross-engine mapping; Humanoid ≠ Generic union |
| Rokoko Studio | Suit / body mocap capture and clean | Generic skeleton infrastructure; open Domain |
| Cascadeur | Physics-assisted key posing | Pipeline Canonical; batch Domain |
| CC / iClone | Character generation + cinematic animation | Non-humanoid infrastructure; open Canonical |
| Mixamo | Auto-rig + clip library for **named** humanoids | Offline Domain; non-Mixamo skeletons; provenance |
| Blender | Open DCC + Python + growing FBX (ufbx) | Being a **format-independent** workbench; GPL host lock |

Sources: W0.2 [REFERENCE_SYSTEM_MATRIX.md](../retargeting/REFERENCE_SYSTEM_MATRIX.md) (HumanIK / UE / Unity official docs). Product pages [OFFICIAL_DOC] / [PROJECT_CLAIM] 2026-08-31. No claim that any vendor “cannot retarget.”

---

## Architectural gap (not “no one has every feature”)

The unmet system gap is a **portable Character / Skeleton / Motion Domain**:

1. **Format independence.** Ingest FBX / glTF / USD (and later others) without treating any file format as Canonical (W0.1).
2. **Host independence.** Mapping / compatibility / retarget / QC live **outside** Unreal, Unity, Maya, and Blender object models (W0.2, AD-CAP-007).
3. **Engine-independent Preview** of humanoid **and** non-humanoid, skeleton-only, and unbound Motion (W0.3).
4. **Fail-closed diagnostics.** Explicit capability, loss, and determinism context (W0.3).
5. **Headless Domain API** that later CLI / Workbench / MCP all call (architecture).

Existing products each own a **host or service slice**. Combining them still leaves the user with incompatible mapping identities, no shared loss model, and Preview that requires that host.

---

## R1-G0 PRODUCT NECESSITY VERDICT

```text
BUILD
```

Confidence: **HIGH** for the gap existing; **MEDIUM** for commercial timing (no market-size study this pass).

`BUILD_CANONICAL_CORE_ADOPT_COMPONENTS` describes **implementation style** (write Domain; adopt parsers / validators), not the product verdict.

`RECONSIDER_BUILD` remains valid if a later product ships this **exact** infrastructure gap. That is a W0 Final Synthesis / later re-check trigger, not a current finding.

Re-check triggers: a new open, headless, engine-free Character/Skeleton/Motion Domain with explicit mapping + QC + Preview; or a vendor opening their Canonical contract under a usable license.
