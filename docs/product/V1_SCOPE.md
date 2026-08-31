# V1 Scope

This is the current V1 scope contract.

It supersedes the earlier working baseline that treated Canonical Character /
Skeleton / Motion plus native adapters as V1 foundation. Historical research
remains intact; it is not a V1 implementation mandate.

Decision: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Architecture: [../architecture/README.md](../architecture/README.md).

Blender as execution backend is **`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`**,
validated by POC-BLENDER-E2E-01. This is a W0 research/architecture
validation result. Product implementation remains unauthorized until
POC-PREVIEW-01R, W0-RS, and IA-1 complete the remaining required gates.

## In scope

| Capability | Notes |
| --- | --- |
| Character Asset management | Already-rigged mesh + skeleton/armature + skin weights |
| Motion Asset management | Clip + source Skeleton context + time-domain provenance |
| Source Skeleton context | Shared across Motions; not duplicated per clip |
| Asset Browser | Character, Motion, Derived Variant |
| Asset preview | Click-to-preview source and derived assets |
| Selection / Transfer Tray | One Character + one Motion |
| Skeleton Summary | Thin, portable, derived inspection evidence |
| Mapping | First-class; automatic preflight; editor only when needed |
| Compatibility | First-class gate; not a single boolean |
| Retarget Policy | Product-owned intent; worker may execute |
| Blender-backed transfer | Hidden worker: **IN V1 DIRECTION / ACCEPTED WITH GUARDS**; subject to remaining W0-RS / IA-1 gates before implementation |
| Derived Variant | First-class result with versions and provenance |
| Version / provenance | Inputs, Mapping, policy, backend, QC, artifacts |
| Basic structural QC | First-class; must not silently mutate the subject |
| Engine-independent Preview | Derived Preview Artifact + independent viewer |

Happy path:

```text
select Character + select Motion
        ↓
automatic Mapping / Compatibility preflight
        ↓
Ready
        ↓
Transfer
        ↓
Derived Variant
        ↓
Preview / Version
```

Exact field schemas, enum names, UI toolkit, database, and viewer library
remain implementation work.

## Out of scope / deferred

| Item | Class |
| --- | --- |
| Auto-Rig / automatic skeleton generation | `DROP_FROM_V1` |
| Automatic skinning | `DROP_FROM_V1` |
| Raw unrigged mesh as transfer target | `DROP_FROM_V1` |
| Raw mesh topology repair for rig generation | `DROP_FROM_V1` |
| Full DCC editing environment | `DROP_FROM_V1` |
| Heavy Canonical asset runtime (complete mesh/skin/curve/evaluator copies) | not V1 critical path |
| Custom animation evaluator | `DROP_FROM_V1` |
| Custom retarget solver / runtime | `DROP_FROM_V1` |
| Mandatory native FBX stack | `DEFER` (POC-FBX-01 remains valid evidence) |
| Mandatory native glTF stack | `DEFER` |
| Mandatory OpenUSD stack | `DEFER` |
| Multiple DCC backends in V1 | `DROP_FROM_V1` |
| Direct Unreal / Unity / Godot integrations | `DROP_FROM_V1` |
| Multiple versioned Export Profiles | `DEFER` / implementation-gated |
| Advanced production QC (foot contact, sliding, artistic acceptance) | later hardening |
| AI as mandatory authority | `DEFER`; suggestions cannot silently accept Mapping |
| MCP as V1 shaping requirement | `DEFER` |
| Core language selection | `DEFER` (POC-CORE-01 remains `INCONCLUSIVE`) |

Export Artifact is an **optional derived concept**. Direct engine integration
is out of V1. One practical output artifact may still be used for
persistence, reopen, or preview where necessary.

## Character input

```text
REQUIRED:  mesh + skeleton / armature + skin weights
FORBIDDEN as V1 requirement: raw unrigged mesh, Auto-Rig, skeleton
generation, automatic skinning, raw mesh topology repair
```

## Motion input

```text
Motion Asset =
  animation data
  + Source Skeleton reference / context
  + time-domain provenance
  + fallback / missing-channel semantics as needed
```

Missing source Skeleton context is diagnosable. Generic playback and retarget
must not invent it.

## Compatibility (product-facing)

```text
MAPPING_COMPLETE
  != RETARGET_COMPATIBLE
  != RESULT_ACCEPTABLE
```

Illustrative UX (names not frozen): Ready, Ready with warnings, Mapping
confirmation required, Unsupported.

## QC

```text
execution success
  != structural validity
  != QC quality
  != production acceptability
```

QC **MUST NOT** silently mutate or repair its subject.

V1 structural examples: NaN/Inf, required mapped bones, duration/time-domain
sanity, expected baked animation, gross invalid scale/transform, basic
root-trajectory sanity.

## Preview

```text
Authoritative Asset / Derived Variant
        ↓
Preview Generator
        ↓
Derived Preview Artifact     (DERIVED / REBUILDABLE / NON-AUTHORITATIVE)
        ↓
engine-independent Viewer
```

A GLB/browser path is a candidate, not a frozen technology choice. Preview
must not require Unreal, Unity, or Blender as the user-facing viewer.

## Non-humanoid

Non-humanoid Characters are **not prohibited**. Contracts must not assume
humanoid-only slots. Real non-humanoid E2E remains a release hardening
requirement, not the first architecture PoC.
