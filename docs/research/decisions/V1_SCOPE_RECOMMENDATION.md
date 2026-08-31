# V1 Scope Recommendation — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

This is a **recommendation**. It does **not** edit [docs/product/V1_SCOPE.md](../../product/V1_SCOPE.md). External review and W0 Final Synthesis hold scope authority.

Working product V1 already listed Canonical objects, inspect/validate, mapping, compatibility, deterministic retarget, QC, one E2E ingest/publish, Domain API, CLI, Minimal Workbench. W0.4 re-opens each item against evidence.

---

## Counts (this recommendation)

```text
V1 CORE:        13
V1 OPTIONAL:     5
POST-V1:        11
RESEARCH ONLY:  10
OUT OF SCOPE:    6
W0-P GATED:      9
```

Counts are table rows in this file (Canonical objects are one CORE row). RESEARCH ONLY includes Motius and Godot **internal contract validation** (not a shipping SKU).

Items may appear in **W0-P GATED** and another bucket (implementation path gated).

---

## V1 CORE

| Item | Why |
| --- | --- |
| Canonical Character / Skeleton / Motion / Manifest | Product |
| Asset / Skeleton / Motion inspect (including unbound Motion metadata) | W0.3 Preview + Domain |
| Skeleton / Motion validation | Product |
| Semantic Skeleton Mapping | W0.2 |
| Skeleton Compatibility | W0.2 |
| Deterministic Retarget | W0.2; algorithm still W0-P |
| Retarget QC (no auto-fix) | W0.2 |
| At least one real ingest + **derived** publish E2E | Product. Preferred **candidate** pair: FBX ingest → Domain → glTF publish (`PROVISIONAL_W0P_GATED`). I/O asymmetry (no mandatory FBX write) is `DECIDE_NOW` |
| Loss / provenance / adapter facts | W0.3 |
| Domain API | Architecture |
| CLI / headless | Architecture; later MCP sits on this |
| Minimal Workbench **without** engine host | Product; Preview may be thin |
| Engine-independent Preview **architecture** (even if UI is thin) | Product vision |

---

## V1 OPTIONAL

| Item | Why |
| --- | --- |
| Blender headless / file adapter | Highest open DCC value; not ingest authority |
| Second format publish (FBX write) | Only if POC-FBX-02 passes |
| VRM profile on glTF (read + declared loss) | After glTF PoC; not universal skeleton |
| Desktop shell (H3) vs web-only Preview | Host gated |
| Python bindings | DCC/automation; not required to ship CLI |

---

## POST-V1

| Item | Why |
| --- | --- |
| Maya adapter | Pro pipelines; license + version cost |
| Unreal adapter (productized) | Highest engine value; editor ≠ Domain |
| Unity adapter | Second engine |
| OpenUSD ingest/publish (if PoC too heavy for V1) | 26.08 valuable; deployment cost |
| OpenAssetIO integration | Optional AMS bridge |
| AYON adapter | Pipeline |
| Runtime cook (ozz/ACL implementation) | Keep contract only in V1 |
| MCP server | Spec 2026-07-28 churn; Domain+CLI first |
| AI suggestion / auto-rig / learned retarget | License + authority |
| Full polished Workbench viewer | Long-term Preview ≠ V1 polish |
| Native Qt-only Preview | Fallback host |

---

## RESEARCH ONLY

SkinTokens, UniRig, RigAnything, AnyTop, SATA, UniMate, Motius, SOMA, GMR — see [FRONTIER_AI_MATRIX.md](FRONTIER_AI_MATRIX.md).

Godot (or a tiny stub) as **internal Engine Adapter contract validation** — RESEARCH ONLY. Not a V1 shipping adapter. A product Godot adapter would need a separate later product decision. Do not ship a third engine adapter merely to prove independence. POC-ENGINE-01A (generic stub) is the genericness oracle.

---

## OUT OF SCOPE (V1 and architecture)

| Item | Why |
| --- | --- |
| Game engine as Workbench or Preview host | Product + W0.3 |
| DCC as mandatory ingest | Product |
| Mixamo / cloud auto-rig as authority | Service + humanoid-only |
| ozz/ACL as Canonical Motion | Architecture |
| VRM as universal skeleton | V1_SCOPE + W0.1 |
| Complete DCC or engine replacement | Product |

---

## W0-P GATED (implementation path, not whether the product exists)

| Item | Gate |
| --- | --- |
| ufbx as V1 FBX reader | POC-FBX-01 |
| Native glTF library rank | POC-GLTF-01 |
| USD V1 OPTIONAL vs POST-V1 | POC-USD-01 (`CONDITIONAL_W0P`; not a W1 blocker) |
| Core language L1 vs L2 | POC-CORE-01 |
| Preview data path B vs D | POC-PREVIEW-01A |
| Preview host H3 + renderer R1 | POC-PREVIEW-01B |
| glTF/GLB as preferred derived publish | POC-GLTF-01 |
| FBX write in V1 OPTIONAL | POC-FBX-02 |
| Retarget **method** (not the requirement) | POC-RETARGET-01 |

---

## Mandatory scope questions (explicit)

| Topic | Recommendation |
| --- | --- |
| FBX ingest | `KEEP V1 CORE` (ufbx gated) |
| FBX export | `POST-V1` / `OPEN UNTIL W0-P` (not required) |
| glTF ingest | `KEEP V1 CORE` (library gated) |
| glTF as first derived publish | `OPEN UNTIL W0-P` — preferred candidate only |
| USD ingest/publish | `POST-V1` unless Final Synthesis keeps V1 OPTIONAL (`CONDITIONAL` PoC) |
| VRM | `MOVE V1 OPTIONAL` profile |
| Semantic Mapping / Compat / Retarget / QC | `KEEP V1 CORE` |
| Preview | `KEEP V1 CORE` as **minimal inspection**; full viewer `POST-V1` |
| Workbench | `KEEP V1 CORE` minimal; no engine |
| Blender adapter | `MOVE V1 OPTIONAL` first DCC |
| Maya adapter | `POST-V1` |
| Unreal / Unity adapters | `POST-V1` product; generic contract `KEEP` |
| Godot adapter | `RESEARCH ONLY` / internal contract validation — not V1 shipping |
| Runtime cook | `POST-V1` implementation |
| MCP | `POST-V1` |
| AI features | `POST-V1` / `RESEARCH ONLY` |

---

## Preview V1 boundary

```text
V1:  minimal standalone inspection
     (hierarchy, rest/bind diagnostics, clip list,
      unbound Motion metadata, bound playback if skeleton resolves,
      declared losses, no engine)

POST-V1: polished Workbench viewer, pair compare, rich QC overlays
```

Long-term Preview remains mandatory. V1 must not create an Unreal/Unity dependency.

---

## Workflow (UX, not pixels)

```text
Open / Ingest
        → Inspect (Preview + diagnostics)
        → Semantic Mapping
        → Compatibility
        → Retarget
        → QC
        → Compare / Preview
        → Publish derived
```

Workbench must show: provenance, adapter, loss, mapping evidence, compatibility, retarget policy, QC, derived status. Not a black box.

This matches cross-DCC/engine handoff better than “open in Maya and hope.”
