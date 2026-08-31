# DCC and Engine Adapter Boundary — W0.3

Architectural boundary only. Access date: **2026-08-30**.

Hard rule (accepted product / W0.1): file ingest of FBX / glTF / USD must work **without** Blender or Maya installed when a format adapter exists. DCC/engine integration is an **optional workflow surface**.

Core must never assume: `UObject`, Unity `AssetDatabase`, HumanIK slots, `HumanBodyBones`, Blender `bpy` types, or Maya DAG nodes as Canonical objects.

---

## What Core must never assume

| Forbidden Core concept | Why |
| --- | --- |
| Unreal `UObject` / Interchange pipeline | Engine-specific [OFFICIAL_DOC] |
| Unity `AssetDatabase` / Avatar | Engine-specific (W0.2) |
| Maya `MObject` / HumanIK node | DCC-specific |
| Blender `bpy.types` / Rigify | DCC-specific |
| Godot `Skeleton3D` | Engine-specific [OFFICIAL_DOC] |
| “Import through Blender first” | Violates DCC-independent ingest |

A Godot or custom-engine adapter **must** be able to implement the Engine Adapter contract by consuming Canonical-derived files or a documented bridge **without pretending to be Unreal or Unity**.

**Engine-agnostic gate:** YES — the contract below is file + Domain + optional host plugin, not a two-engine abstraction.

---

## Process topologies (not one universal choice)

| Pattern | Crash isolation | Version / ABI / Python isolation | GPU / UI | Deploy | Debug | Headless | Determinism |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A. In-process plugin → Core FFI | Low | Poor if DCC Python ≠ Core | Shares host GPU/UI thread | Single binary | Harder | Only if host is headless | Host-session noise |
| B. Thin in-process plugin → external RigForge process | High | High | Host keeps GPU | Two processes | Easier Core | Core headless | Better |
| C. DCC/engine headless batch | Medium | Host version still required | Usually no UI | Host install required | Script logs | Yes (host-dependent) | Session/prefs risk |
| D. File interchange only | Highest | Highest | None | No plugin | File diffs | Yes | Highest |
| E. Hybrid | Mixed | Mixed | Mixed | Mixed | Mixed | Mixed | Mixed |

Different hosts **may** use different topologies under **one** capability/loss/diagnostics contract.

Final in-process vs subprocess/service placement for OpenUSD, Blender, Maya, and engine editors remains **OPEN**, pending W0.4 packaging analysis and W0-P feasibility. Separate process is a **strong candidate** when ABI, dependency, or crash isolation warrants it — not a frozen topology.

---

## Blender

Official headless: `--background` / `-b` and `--python` [OFFICIAL_DOC] Blender 5.2 LTS Command Line Arguments https://docs.blender.org/manual/en/latest/advanced/command_line/arguments.html

`bpy` as a Python module exists but is **not** the default official download; official docs describe it as a build/PIP option [OFFICIAL_DOC] https://docs.blender.org/api/current/info_advanced_blender_as_bpy.html

| Topic | Finding |
| --- | --- |
| Embedded Python | Yes (`bpy`) |
| Native plugin | Add-ons; C/C++ via Python or extensions |
| Headless / batch | Official `--background` |
| Scene / undo | Session-owned; factory-startup recommended for automation [OFFICIAL_DOC] |
| UI thread | Interactive add-ons are UI-process bound |
| File/session | `.blend` is not Canonical |
| Version | Major Blender versions break add-on APIs historically |
| External process | Documented CLI invocation |

**Implication:** file interchange or subprocess `--background` is a **strong isolation candidate** versus embedding Core inside `bpy`. In-process FFI remains allowed. Placement is **OPEN** (W0.4 / W0-P).

---

## Maya

Official/adjacent: `mayapy` + `maya.standalone.initialize()` for batch without UI [OFFICIAL_DOC] Autodesk PyMEL standalone (Maya Tech Docs) https://help.autodesk.com/cloudhelp/2023/ENU/Maya-Tech-Docs/PyMel/standalone.html — UI commands and some plugins are unavailable; scriptJobs caveat.

Also: `maya -batch` as a product batch path (companion to mayapy).

| Topic | Finding |
| --- | --- |
| Embedded Python | `mayapy` / Maya Python |
| Native plugin | Maya API (C++) |
| Headless | standalone / `-batch` |
| UI thread | Interactive plugins are Maya-main-thread |
| Undo / transaction | Host-owned |
| Version | Yearly Maya + Python version coupling |
| External process | `mayapy script.py` |

**Implication:** treat Maya as an optional host. Do not require Maya for FBX ingest (W0.1: SDK/ufbx path exists). Process topology is **OPEN** (same contract as Blender).

---

## Unreal Engine (adapter target only)

Editor integration ≠ Canonical conversion.

| Path | Role |
| --- | --- |
| In-editor plugin | Optional UX |
| Commandlet / `-run=` Python | Headless project mutation [OFFICIAL_DOC] community + Epic Python commandlet pattern |
| Interchange `UInterchangeManager` / `is_automated` | Engine **import into Content** [OFFICIAL_DOC] UE 5.5/5.7 Interchange API |
| File handoff (FBX/glTF/USD on disk) | Conversion can happen **outside** the editor |

Canonical conversion must remain usable **without** Unreal Editor. Interchange is an engine ingest pipeline, not RigForge Core.

---

## Unity (adapter target only)

| Path | Role |
| --- | --- |
| Editor plugin | Optional UX |
| `-batchmode -quit -executeMethod` | Official automation [OFFICIAL_DOC] Unity 6 Command-line interface https://docs.unity3d.com/6/Documentation/Manual/CommandLineArguments.html |
| `Application.isBatchMode` | Detect batch [OFFICIAL_DOC] Unity 6.6 Scripting API |
| Humanoid Avatar / Generic | Engine mapping (W0.2) — **not** Core |
| File handoff | Preferred for Domain conversion |

One Editor instance per project in batch mode [OFFICIAL_DOC]. Not a RigForge preview host.

---

## Godot / generic-engine check

Godot 4 documents `GLTFDocument` / `GLTFDocumentExtension` / `GLTFState` for import/export and custom extensions [OFFICIAL_DOC] https://docs.godotengine.org/en/stable/classes/class_gltfdocument.html

A generic engine can: consume derived glTF/USD/FBX from RigForge; optionally register a host importer; map to **its** skeleton class.

**Could Godot implement the Engine Adapter contract without pretending to be Unreal/Unity?** **Yes**, if the contract is:

```text
Canonical (or Canonical-derived interchange)
        ↓
Engine Adapter (host-specific)
        ↓
Host identity + derived cooked assets + loss/diagnostics
```

and not:

```text
Canonical
        ↓
UObject / AssetDatabase
```

---

## Engine Adapter requirements (generic)

An Engine Adapter should be able to express:

- import Canonical-derived asset into the host
- export/source from the host **where supported**
- host asset identity (opaque to Core)
- host-side **derived** artifacts
- **host binding / host projection** facts (Canonical joint → host bone id; Canonical Motion → host clip/asset id; Domain Semantic Mapping **translated** into a host representation)
- runtime/cooked representation as **derived**
- version compatibility
- diagnostics and loss

```text
DOMAIN SEMANTIC MAPPING
        ≠
HOST BINDING / HOST PROJECTION
```

The Adapter **translates** Domain Semantic Mapping. It must not author Canonical semantic roles. Host metadata on import is **evidence / provenance**, not Mapping authority (AD-CAP-002, AD-CAP-007).

Editor integration is optional. Canonical conversion is not.

---

## DCC Adapter requirements (generic)

Same capability/loss/diagnostics contract as format adapters, plus:

- host scene ownership remains the DCC’s
- undo/UI thread constraints stay in the host process
- RigForge Core remains callable headless without the DCC
