# W0-P research validation PoCs

Execution evidence for W0-P experiments. These documents are **not** product
implementation.

Current V1 direction: [ADR-0001](../../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Replan authority: [W0P_REPLAN_PROPOSAL.md](W0P_REPLAN_PROPOSAL.md).

## Active lifecycle

```text
W0-P: REPLANNED
W0-SR: COMPLETE / ADOPTED
POC-CORE-01: COMPLETE / PASS
POC-FBX-01: COMPLETE / PASS
POC-BLENDER-E2E-01: READY / NOT STARTED
POC-PREVIEW-01R: NOT STARTED
old original PoCs: PAUSED / REPLANNED
```

POC-BLENDER-E2E-01 must be executed only under its dedicated experiment
prompt / contract. Do not execute it from this file.

| ID | Class | Status |
| --- | --- | --- |
| W0-P | — | **REPLANNED** |
| POC-CORE-01 | Mandatory (executed) | **COMPLETE / PASS** |
| POC-FBX-01 | Mandatory (executed) | **COMPLETE / PASS** |
| POC-BLENDER-E2E-01 | Mandatory (current next) | **READY / NOT STARTED** |
| POC-PREVIEW-01R | Mandatory (after E2E) | **NOT STARTED** |
| POC-GLTF-01 | original Mandatory | PAUSED / REPLANNED (`MERGE`) |
| POC-RETARGET-01 | original Mandatory | REDEFINED / REPLANNED (`MERGE`) |
| POC-PREVIEW-01 | original Mandatory | REDEFINED (`REDEFINE` → POC-PREVIEW-01R) |
| POC-ENGINE-01 | original Mandatory | DROP_FROM_V1 / DEFERRED (`DOWNGRADE`) |
| POC-USD-01 | Conditional | DEFERRED (`DEFER`) |
| POC-FBX-02 | Optional | DEFERRED (`DEFER`) |
| POC-DCC-01 | Optional | PAUSED / REPLANNED (`MERGE` into E2E first-slice observations only) |
| POC-PREVIEW-02 | Optional | DEFERRED (`DEFER`) |

## Completed

POC-CORE-01 report: [POC_CORE_01.md](POC_CORE_01.md).

```text
POC-CORE-01: COMPLETE / PASS
Decision Impact: INCONCLUSIVE
Core language: NOT SELECTED
```

Harness: [experiments/w0p/poc_core_01/](../../experiments/w0p/poc_core_01/).

POC-FBX-01 report: [POC_FBX_01.md](POC_FBX_01.md).

```text
POC-FBX-01: COMPLETE / PASS
Decision Impact: KEEP_UFBX_WITH_GUARDS
Core language: NOT SELECTED (C harness is not a language selection)
```

Harness: [experiments/w0p/poc_fbx_01/](../../experiments/w0p/poc_fbx_01/).

POC-FBX-01 remains valid technical evidence. W0-SR changes its architectural
placement: ufbx may later be a fast inspector, an optimization, or a
specialized adapter — or may not ship in V1 if Blender import is sufficient.
That choice is **not decided**. Do not delete or rewrite POC-FBX evidence.

W0.4 C++ preference remains historical/provisional and coexists with
POC-CORE-01 `INCONCLUSIVE` evidence. Core language is not selected.

## Next: POC-BLENDER-E2E-01

Primary question:

> Can Blender serve as a hidden execution backend for the real RigForge
> Character + Motion → Derived Variant workflow without forcing Blender
> semantics into RigForge's durable product model?

Minimal path (not executed here):

```text
one rigged Character A
+ one Motion B with a genuinely different Source Skeleton
        ↓
thin Skeleton Summaries
        ↓
reviewed / frozen Mapping
        ↓
basic Compatibility
        ↓
frozen Retarget Policy
        ↓
backend-neutral Job Spec
        ↓
isolated Blender worker
        ↓
import / retarget / bake
        ↓
basic structural QC
        ↓
Derived Variant
        ↓
candidate Preview / Persistence Artifact
        ↓
fresh-process reopen
+ clean-process consistency
```

This is a minimal architecture slice. Auto-Mapping quality, full
non-humanoid E2E, identity/negative-pair suites, crash/timeout campaigns,
worker pools, advanced contact QC, Blender upgrade qualification, complete
shipping qualification, and multiple export profiles are **out of this PoC**.

Contract: [W0P_REPLAN_PROPOSAL.md](W0P_REPLAN_PROPOSAL.md).
