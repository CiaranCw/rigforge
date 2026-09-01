# V1-4 Implementation Plan

Skeleton Mapping and Compatibility workflow.

Status: `COMPLETE / PASS / BASELINED`

V1-5 is `READY / NOT STARTED`. Gate C is `NOT STARTED`.

Related:

- [V1_4_MAPPING_WORKFLOW.md](V1_4_MAPPING_WORKFLOW.md)
- [V1_4_COMPATIBILITY_CONTRACT.md](V1_4_COMPATIBILITY_CONTRACT.md)
- V1-3: [V1_3_WORKER_CONTRACT.md](V1_3_WORKER_CONTRACT.md)
- Domain: [V1_1_DOMAIN_CONTRACT.md](V1_1_DOMAIN_CONTRACT.md)
- ADR-0001 through ADR-0005 remain **Accepted**

## Baseline

```text
branch: main
HEAD:   85ffc0dac8597196c1173e26a7863e701378dc62
```

V1-3 remains `COMPLETE / PASS / BASELINED`. Gate B remains `PASS / CLOSED`.
This stage does not reverse accepted V1-1 / V1-2 / V1-3 semantics, Product
contracts, ADR-0001–0005, or Gate B corrections.

Focused review is **PASS**. V1-4-MAJOR-001 through V1-4-MAJOR-005 and
V1-4-MINOR-001 / V1-4-MINOR-002 are **CLOSED**. OPEN blocking findings: 0.

Rev1 correction: publication provenance, typed unmapped disposition,
CompatibilitySummary invariant, SkeletonSummary binding, Workbench Product
accept/override.

Rev2 correction: unmapped source/target side containers, Published Mapping
as the Ready gate, Workbench selection-bound reopen.

Rev3 correction: Catalog persistence graph-validates CompatibilityResult.
Ready / ReadyWithWarnings cannot be stored against a Draft Mapping.

## Goals

```text
Character exact version + Motion exact version
        ↓
SkeletonSummary derived evidence (source + target)
        ↓
deterministic Mapping candidates
        ↓
explicit acceptance / manual override
        ↓
published BoneMappingVersion
        ↓
Compatibility preflight (five dimensions)
        ↓
Ready | ReadyWithWarnings | MappingConfirmationRequired | Unsupported
```

V1-4 stops before Transfer execution.

## Non-goals (owned later)

Actual Product Transfer, Product QC, Derived Variant creation/publication,
Preview, Export, Auto-Rig, second DCC backend, new Retarget solver, worker
pool, release hardening.

```text
Compatibility PASS
!= Worker SUCCESS
!= QC PASS
!= Derived Variant publication
```

## Source layout

| Path | Role |
| --- | --- |
| `domain/` | Additive Mapping/Compatibility/SkeletonSummary accessors only |
| `app/` | Inspector boundary, candidates, preflight, Catalog workflow |
| `blender-worker/` | Optional Blender-backed inspect (not retarget) |
| `workbench/` | Minimal Mapping / Compatibility surface |

## Review

Ordinary focused review **PASS**. Not an independent Gate.

Gate C remains after V1-4 + V1-5:

```text
Gate C
Purpose: Product Core E2E Audit
Model: NEW independent GPT-5.6 Sol High Agent
Status: NOT STARTED
```
