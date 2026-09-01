# Roadmap

Phase status and the **current** V1 sequence. Design details live in product,
research, and architecture docs.

Decision: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Replan authority: [../research/poc/W0P_REPLAN_PROPOSAL.md](../research/poc/W0P_REPLAN_PROPOSAL.md).

## Current lifecycle

```text
W0: ACTIVE
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-CORE-01: COMPLETE / PASS / BASELINED
POC-FBX-01: COMPLETE / PASS / BASELINED
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
W0-RS: COMPLETE / PASS / BASELINED
old original PoCs: PAUSED / REPLANNED
IA-1: READY / NOT STARTED
product implementation: NOT STARTED
```

POC-BLENDER-E2E-01 is **COMPLETE / PASS / BASELINED**. Decision:
`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`. POC-PREVIEW-01R is **COMPLETE / PASS /
BASELINED**. Decision: `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`. W0-RS is
**COMPLETE / PASS / BASELINED**:
[synthesis](../research/decisions/W0_REVISED_SYNTHESIS.md) and
[traceability](../research/decisions/W0_RS_TRACEABILITY.md). Remaining
pre-implementation sequence is IA-1 (`READY / NOT STARTED`). Do not begin
product implementation. Product implementation remains unauthorized until
IA-1 is PASS / CLOSED.

## Immediate sequence

```text
W0-RS
COMPLETE / PASS / BASELINED
        ↓
IA-1
READY / NOT STARTED
        ↓
V1 implementation
ONLY IF IA-1 PASS / CLOSED
        ↓
Mapping / Compatibility validation
        ↓
Worker reliability
        ↓
Transfer / QC validation
        ↓
Real-asset + non-humanoid hardening
        ↓
Release qualification
```

## Historical research — COMPLETE

| Stage | Name | Status |
| --- | --- | --- |
| — | Repository Initialization | COMPLETE |
| W0.1 | Canonical Foundations | COMPLETE |
| W0.2 | Mapping / Compatibility / Retarget | COMPLETE |
| W0.3 | Adapter / Infrastructure | COMPLETE |
| W0.4 | Product / Tech-stack / Frontier | COMPLETE |
| POC-CORE-01 | Equivalent language slice | COMPLETE / PASS; impact `INCONCLUSIVE` |
| POC-FBX-01 | ufbx real-asset validation | COMPLETE / PASS; impact `KEEP_UFBX_WITH_GUARDS` |

These remain accepted history. They may be still applicable, partially
applicable, risk knowledge, superseded for V1 scope, or deferred. Original
findings are not rewritten.

## Current scope revision — COMPLETE / ADOPTED

| Stage | Purpose | Status |
| --- | --- | --- |
| W0-SR | Reconcile clarified workflow with W0 evidence | `COMPLETE / ADOPTED` |
| W0 | Research Baseline (open until revised synthesis + IA-1) | ACTIVE |

## New validation path

| Stage | Purpose | Status |
| --- | --- | --- |
| POC-BLENDER-E2E-01 | Minimal architecture slice: hidden Blender worker, thin domain, real cross-Skeleton pair | COMPLETE / PASS / BASELINED |
| POC-PREVIEW-01R | Click-to-preview from derived Preview Artifacts | COMPLETE / PASS / BASELINED |
| W0-RS | [Revised W0 synthesis](../research/decisions/W0_REVISED_SYNTHESIS.md) and [traceability](../research/decisions/W0_RS_TRACEABILITY.md) | COMPLETE / PASS / BASELINED |
| IA-1 | Independent architecture / research audit | READY / NOT STARTED |

POC-BLENDER-E2E-01 tests execution architecture only. Auto-Mapping quality,
full non-humanoid E2E, crash/timeout campaigns, worker pools, advanced
contact QC, packaging qualification, and multiple export profiles belong to
later V1 hardening. See [../research/poc/README.md](../research/poc/README.md).

## Future V1 implementation

Start only after IA-1 acceptance. Names are planning labels, not frozen
lifecycle IDs.

| Stage | Purpose |
| --- | --- |
| V1-1 | Workflow Domain and version/provenance contract |
| V1-2 | Local-first Asset Catalog and job orchestration |
| V1-3 | Pinned isolated Blender worker integration |
| V1-4 | Skeleton Mapping and compatibility workflow |
| V1-5 | Transfer, QC, and Derived Variant lifecycle |
| V1-6 | Engine-independent Preview |
| V1-7 | Optional artifact/output support (only if a concrete consumer requires it) |
| V1-8 | Real-asset hardening and release audit |

## Historical planned phases (superseded as the active plan)

The previous W1–W10 sequence assumed a heavy Canonical Domain Contract, I/O
Adapter Contract, Canonical I/O MVP, and engine adapters as the
implementation path. That is **not** the current V1 plan.

| Former phase | Former name | Current reading |
| --- | --- | --- |
| W1 | Canonical Domain Contract | superseded as active plan; thin Workflow Domain instead |
| W2 | I/O Adapter Contract | superseded as active plan; worker import path first |
| W3 | Canonical I/O MVP | superseded as active plan |
| W4 | Validation + Semantic Mapping | retained as capability; sequenced under V1-4 / V1-5 |
| W5 | Compatibility + Deterministic Retarget | retained as policy/UX; execution is worker-backed |
| W6 | Domain API + CLI | later implementation, after IA-1 |
| W7 | Minimal Workbench | becomes Asset Browser / Transfer Tray after IA-1 |
| W8 | DCC / Engine Adapters | multiple DCC backends and direct engine integrations are out of V1 |
| W9 | Automation / MCP-ready API | MCP deferred |
| W10 | CI / Structural Validation / Final Audit | absorbed into V1-8 release qualification |

## Stage invariants (from W0.2 Rev1; still in force)

Every later stage must explicitly check:

```text
PRODUCT REQUIREMENTS
GENERALITY
ENGINE INDEPENDENCE
FORMAT INDEPENDENCE
DCC INDEPENDENCE
REAL-ASSET EVIDENCE
```

DCC independence is a **product-semantics and replaceability** property.
V1 may still have one managed Blender operational dependency
(`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`). Reference formats, DCCs, engines, runtimes, and
renderers are not product authority.
