# W0-P research validation PoCs

Execution evidence for W0-P experiments. These documents are **not** product
implementation.

Current V1 direction: [ADR-0001](../../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Replan authority: [W0P_REPLAN_PROPOSAL.md](W0P_REPLAN_PROPOSAL.md).

## Active lifecycle

```text
W0-P: REPLANNED
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-CORE-01: COMPLETE / PASS
POC-FBX-01: COMPLETE / PASS
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
old original PoCs: PAUSED / REPLANNED
```

POC-BLENDER-E2E-01 is **COMPLETE / PASS / BASELINED**. Decision impact:
`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`. POC-PREVIEW-01R is **COMPLETE / PASS /
BASELINED**. Decision impact: `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`.
Do not start product implementation. Next is W0-RS.

| ID | Class | Status |
| --- | --- | --- |
| W0-P | — | **REPLANNED** |
| POC-CORE-01 | Mandatory (executed) | **COMPLETE / PASS** |
| POC-FBX-01 | Mandatory (executed) | **COMPLETE / PASS** |
| POC-BLENDER-E2E-01 | Mandatory (executed) | **COMPLETE / PASS / BASELINED** |
| POC-PREVIEW-01R | Mandatory (after E2E) | **COMPLETE / PASS / BASELINED** |
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

## POC-BLENDER-E2E-01 (COMPLETE / PASS / BASELINED)

Report: [POC_BLENDER_E2E_01.md](POC_BLENDER_E2E_01.md).

```text
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_BLENDER_BACKEND_WITH_GUARDS
Core language: NOT SELECTED
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
```

Harness: [experiments/w0p/poc_blender_e2e_01/](../../experiments/w0p/poc_blender_e2e_01/).

This is a W0 research/architecture validation result. It does not authorize
product implementation. Blender is not product authority.

## POC-PREVIEW-01R (COMPLETE / PASS / BASELINED)

Report: [POC_PREVIEW_01R.md](POC_PREVIEW_01R.md).

```text
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
viewer library: RESEARCH HARNESS ONLY / NOT SELECTED
payload candidate: GLB (not a permanent format selection)
W0-RS: READY / NOT STARTED
```

Harness: [experiments/w0p/poc_preview_01r/](../../experiments/w0p/poc_preview_01r/).

This is a W0 research/architecture validation result. It does not authorize
product implementation. It does not select a viewer library or Preview
payload format. Next is W0-RS. Do not start IA-1 before W0-RS is externally
accepted/baselined.

Contract: [W0P_REPLAN_PROPOSAL.md](W0P_REPLAN_PROPOSAL.md).
