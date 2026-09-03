# V1 Scope

This is the current V1 scope contract.

It supersedes the earlier working baseline that treated Canonical Character /
Skeleton / Motion plus native adapters as V1 foundation. Historical research
remains intact; it is not a V1 implementation mandate.

Decision: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Architecture: [../architecture/README.md](../architecture/README.md).

Blender as execution backend is **`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`**,
validated by POC-BLENDER-E2E-01. Derived Preview is
**`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`**, validated by POC-PREVIEW-01R.
These are W0 research/architecture validation results. At the time this
scope file first recorded that statement, Product implementation remained
unauthorized until W0-RS and IA-1 completed the remaining required gates.

Current disposition: W0-RS and IA-1 are complete. V1-1 through V1-8 are
implemented and baselined. Core language is **Rust** (ADR-0002 Accepted).
Catalog is **SQLite / rusqlite bundled** (ADR-0003 Accepted). GUI is
**egui/eframe** (ADR-0004 Accepted). Blender worker process is **Accepted**
(ADR-0005). Preview is **GLB + `@google/model-viewer` 4.3.1** (ADR-0006
Accepted). Gate D is **PASS / CLOSED**. Final independent result:
`GATE_D_PASS_CANDIDATE`.

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
| Blender-backed transfer | Hidden worker: **IN V1 DIRECTION / ACCEPTED WITH GUARDS**; implemented through V1-8 |
| Derived Variant | First-class result with versions and provenance |
| Version / provenance | Inputs, Mapping, policy, backend, QC, artifacts |
| Basic structural QC | First-class; must not silently mutate the subject |
| Engine-independent Preview | **Accepted** (ADR-0006): GLB payload + `@google/model-viewer` 4.3.1 sidecar viewer. GLB is a Preview payload only, not a Product format and not a V1-7 export. |

Happy path:

```text
register local Character + Motion (FBX ingest evidence; FBX is not Product authority)
        ↓
Asset Browser lists exact Published versions
        ↓
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

Exact Domain field schemas are implemented in `domain/`. UI toolkit, catalog
storage, Blender worker packaging, and Preview viewer/payload are Accepted
in ADR-0002 through ADR-0006. They are not reopened by this scope file.

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
| Core language selection | Historical `DEFER` at scope adoption. Current: ADR-0002 **Accepted** (Rust). POC-CORE-01 remains `INCONCLUSIVE` evidence. |

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
Authoritative Product Object
        ↓
Preview Generator
        ↓
Preview Artifact     (DERIVED / REBUILDABLE / NON-AUTHORITATIVE)
        ↓
binding / integrity validation
        ↓
engine-independent Viewer
```

Derived Preview architecture is `VALIDATED_WITH_GUARDS` by POC-PREVIEW-01R
for one research Character / Motion / Derived Variant set. Decision:
`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`. Current production Preview payload
and viewer are **Accepted** in ADR-0006: GLB + `@google/model-viewer` 4.3.1.
GLB is a Preview payload only, not a Product format and not a V1-7 export.
The research Motion hierarchy proxy is not a production-mandatory Canonical
model. Preview must not require Unreal, Unity, or Blender as the user-facing
viewer.

## Non-humanoid

Non-humanoid Characters are **not prohibited** by Product. Contracts must
not assume humanoid-only slots. Non-humanoid input may be assessed by
Mapping and Compatibility. Unsupported pairs must fail closed honestly.

V1 technical release requirement:

```text
backend-neutral / general Product semantics
no mandatory humanoid role table
non-humanoid input may be assessed by Mapping / Compatibility
unsupported pairs must fail closed honestly
```

Not a V1 release guarantee:

```text
successful Retarget / QC / Preview for arbitrary real non-humanoid pairs
```

Current accepted evidence: generic non-humanoid architecture is supported;
the real Horse + UAL2 pair is Unsupported / honest reject; successful real
non-humanoid retarget quality is **not proven**. Broader successful
non-humanoid retarget quality is **POST-V1 HARDENING / FUTURE QUALIFICATION**.
Do not read this as "non-humanoid unsupported by Product".
