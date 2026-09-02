# V1-8 Implementation Plan

Real-asset hardening, runtime/failure hardening, Workbench product-flow
hardening, and release qualification.

Status: `COMPLETE / PASS / BASELINED`

Gate D is **READY / NOT STARTED**. This plan is not a Gate D report.

Related:

- [V1_8_RELEASE_QUALIFICATION.md](V1_8_RELEASE_QUALIFICATION.md)
- [V1_8_REAL_ASSET_MATRIX.md](V1_8_REAL_ASSET_MATRIX.md)
- [V1_8_RUNTIME_RECOVERY.md](V1_8_RUNTIME_RECOVERY.md)
- [V1_8_RELEASE_DEPENDENCIES.md](V1_8_RELEASE_DEPENDENCIES.md)

## Baseline

```text
branch: main
HEAD:   9002a2d65b461faaac9f4cda956429ccd84bd1c9
```

Accepted lifecycle at closeout:

```text
V1-5: COMPLETE / PASS / BASELINED
Gate C: PASS / CLOSED
V1-6: COMPLETE / PASS / BASELINED
V1-7: SKIPPED / OPTIONAL
V1-8: COMPLETE / PASS / BASELINED
Gate D: READY / NOT STARTED
```

## V1-7 decision

```text
status:
SKIPPED / OPTIONAL

concrete consumer:
NONE IDENTIFIED

reason:
NO CONCRETE PRODUCT CONSUMER REQUIREMENT IDENTIFIED
No current V1 Product consumer requires a dedicated output/export
artifact contract. The V1-6 GLB Preview payload is a derived Preview
representation, not an export consumer. V1-7 is not called COMPLETE.
No Download GLB / Export FBX / Export GLB / Unity / Unreal / Maya
integration is implemented in V1-8.
```

## Mission

Evidence-driven qualification of already implemented V1. Fixes are limited
to observed V1 release risks. Deferred items may end as implemented,
explicitly unsupported, documented operational limitation, or post-V1.

## Architecture preserved

ADR-0001 through ADR-0006 remain Accepted. Native WorkbenchHost uses the
blender-worker adapter crate only to launch isolated processes for
user-requested inspect/Transfer. Widgets do not call `bpy`. Domain remains
backend-neutral. This is Host packaging, not a second DCC backend and not
Blender-as-Product-authority.

## Production changes in this stage

| Area | Change | Class |
| --- | --- | --- |
| Native Workbench Mapping / Compatibility / Transfer | Application-backed buttons, acknowledgement checkbox, Transfer action | REQUIRED_FOR_V1_RELEASE |
| Spawn-to-RUNNING crash window | Catalog-adjacent dispatch intent; DISPATCHABLE+intent fails closed on reopen | REQUIRED_FOR_V1_RELEASE |
| RUNNING after application restart | RUNNING JobRuns fail closed on catalog reopen; no publication | REQUIRED_FOR_V1_RELEASE |
| Worker script integrity | SHA-256 of `worker.py` + `preview_gen.py` via shared Application `verify_runtime_worker_package` at Transfer, Preview, QC inspect, and fresh reopen | REQUIRED_FOR_V1_RELEASE |
| Preview viewer integrity | SHA-256 of vendored model-viewer 4.3.1 | REQUIRED_FOR_V1_RELEASE |
| Relocatable runtime root | `RIGFORGE_RUNTIME_ROOT` or executable-relative layout; pin/SHA still enforced | REQUIRED_FOR_V1_RELEASE |
| Published RetargetPolicy selection | unique Published version, else explicit exact id; no arbitrary-first | REQUIRED_FOR_V1_RELEASE |
| PreviewHost sidecar lifecycle | Workbench owns active host; Drop/replace shuts down the listener thread | REQUIRED_FOR_V1_RELEASE |
| RUNNING cancellation | Not implemented | ACCEPTED_LIMITATION |
| Automatic retry | Not implemented; manual new attempt is sufficient | ACCEPTED_LIMITATION |
| Worker pool | Not implemented | ACCEPTED_LIMITATION |
| Polished installer | Not implemented | POST_V1 |
| V1-7 export | Not implemented | SKIPPED |

## GATE-C-OBS-001 disposition

```text
historical observation: remains historical Gate C evidence
current implementation: IMPLEMENTATION ISSUE ADDRESSED IN V1-8
```

Native Workbench `draw()` now calls:

- Propose Mapping → inspect + store draft
- Accept Mapping → `accept_mapping_version`
- Evaluate Compatibility → preflight + authorization
- warnings checkbox → `refresh_transfer_authorization`
- Transfer → `start_transfer` then isolated worker dispatch/collect/finalize

Tests exercise the same handler methods. Historical Gate C reports are not
edited.

## Non-goals

- Gate D report (ready, not started; independent Agent only)
- V1-7 export pipeline
- second DCC backend
- worker pool / process supervisor
- artistic QC metrics
- security/adversarial testing
- rewriting Gate A/B/C reports
