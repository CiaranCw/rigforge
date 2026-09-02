# V1-5 Implementation Plan

Transfer, QC, and Derived Variant lifecycle.

Status: `COMPLETE / PASS / BASELINED`

V1-4 remains `COMPLETE / PASS / BASELINED`. Gate C is `PASS / CLOSED`.
V1-6 is `READY / NOT STARTED`.

Related:

- [V1_5_TRANSFER_LIFECYCLE.md](V1_5_TRANSFER_LIFECYCLE.md)
- [V1_5_QC_CONTRACT.md](V1_5_QC_CONTRACT.md)
- [V1_5_DERIVED_VARIANT_PUBLICATION.md](V1_5_DERIVED_VARIANT_PUBLICATION.md)
- V1-4: [V1_4_COMPATIBILITY_CONTRACT.md](V1_4_COMPATIBILITY_CONTRACT.md)
- V1-3: [V1_3_WORKER_CONTRACT.md](V1_3_WORKER_CONTRACT.md)
- Domain: [V1_1_DOMAIN_CONTRACT.md](V1_1_DOMAIN_CONTRACT.md)
- ADR-0001 through ADR-0005 remain **Accepted**

## Baseline

```text
branch: main
HEAD:   d633932fbc4a33a2c86b8f16e84992cb4ffb2ee5
```

V1-4 remains `COMPLETE / PASS / BASELINED`. This stage does not reverse
accepted V1-1 / V1-2 / V1-3 / V1-4 semantics, Product contracts,
ADR-0001–0005, or Gate B corrections.

## Product transaction

```text
Character exact version
+ Motion exact version
+ Published MappingVersion
+ RetargetPolicyVersion
+ exact CompatibilityResult
        ↓
explicit Transfer authorization
        ↓
exact JobSpec
        ↓
JobRun
        ↓
pinned Blender Worker
        ↓
WorkerResult
        ↓
Derived Variant candidate
        ↓
durable Persistence Artifact
        ↓
independent non-mutating inspection / QC
        ↓
Persistence Verification
        ↓
publication graph validation
        ↓
Published DerivedVariantVersion
```

Invariant:

```text
Transfer request
!= Worker success
!= QC PASS
!= Persistence verification
!= Derived Variant publication
```

No step silently implies the next.

## Non-goals

Preview generation/viewer/payload (V1-6), Export profiles, engine
integration, second DCC backend, Auto-Rig, new Auto-Mapping, artistic
animation scoring, foot-contact quality, worker pool, distributed
scheduling, release installer, legal release qualification.

## Runtime reuse

Transfer execution reuses the accepted V1-3 surface:

```text
JobSpec
JobRun
WorkerDispatchRequest
WorkerPort
WorkerCompletionPort
BlenderWorker
WorkerResult
```

Preserve: Blender 5.2.1 LTS build `9e2066aef7ef`, launch ≠ collect, one
process per attempt, attempt-bound WorkerResult, source digest verification,
fresh execute-process reopen, KeepTargetRestScale, rational-frame fail-closed.

No new execution backend.

## Source layout

| Path | Role |
| --- | --- |
| `domain/` | JobSpec compatibility authorization, DerivedVariant Draft-before-QC, write-once evidence, derived QC verdict, publication graph |
| `app/` | Transfer API, artifact promotion, Product QC evaluator, Catalog graph guards, atomic publication transaction. Raw Catalog SQL mutation is not a public Application API. |
| `blender-worker/` | Read-only `inspect_qc` and durable-artifact reopen. Execute/reopen semantics unchanged |
| `workbench/` | Transfer tray, warning acknowledgement, publication result. No Preview |

## Lifecycle after closeout

```text
V1-4: COMPLETE / PASS / BASELINED
V1-5: COMPLETE / PASS / BASELINED
Gate C: PASS / CLOSED
V1-6: READY / NOT STARTED
```

Gate C independent audit remains `GATE_C_PASS_CANDIDATE` in
[GATE_C_PRODUCT_CORE_AUDIT.md](audits/GATE_C_PRODUCT_CORE_AUDIT.md).
Project acceptance is **PASS / CLOSED**. `GATE-C-OBS-001` is deferred /
non-blocking. Do not open Gate C2. Next ordinary work is V1-6.
