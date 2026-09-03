# Roadmap

Phase status and the **current** V1 sequence. Design details live in product,
research, and architecture docs.

Decision: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Replan authority: [../research/poc/W0P_REPLAN_PROPOSAL.md](../research/poc/W0P_REPLAN_PROPOSAL.md).

## Current lifecycle

```text
W0: COMPLETE / PASS / BASELINED
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-CORE-01: COMPLETE / PASS / BASELINED
POC-FBX-01: COMPLETE / PASS / BASELINED
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
W0-RS: COMPLETE / PASS / BASELINED
old original PoCs: PAUSED / REPLANNED
IA-1: COMPLETE / PASS / CLOSED
V1-1: COMPLETE / PASS / BASELINED
Gate A: PASS / CLOSED
V1-2: COMPLETE / PASS / BASELINED
V1-3: COMPLETE / PASS / BASELINED
Gate B: PASS / CLOSED
V1-4: COMPLETE / PASS / BASELINED
V1-5: COMPLETE / PASS / BASELINED
Gate C: PASS / CLOSED
V1-6: COMPLETE / PASS / BASELINED
V1-7: SKIPPED / OPTIONAL
NO CONCRETE PRODUCT CONSUMER REQUIREMENT
V1-8: COMPLETE / PASS / BASELINED
Gate D: PASS / CLOSED
Final independent Gate D result: GATE_D_PASS_CANDIDATE
R1-0: PASS / DESIGN COMPLETE / BASELINED
R1-1: COMPLETE / PASS / BASELINED
R1-2: COMPLETE / PASS / BASELINED
R1-V: READY / NOT STARTED
```

POC-BLENDER-E2E-01 is **COMPLETE / PASS / BASELINED**. Decision:
`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`. POC-PREVIEW-01R is **COMPLETE / PASS /
BASELINED**. Decision: `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`. W0-RS is
**COMPLETE / PASS / BASELINED**:
[synthesis](../research/decisions/W0_REVISED_SYNTHESIS.md) and
[traceability](../research/decisions/W0_RS_TRACEABILITY.md). IA-1 is
**COMPLETE / PASS / CLOSED**:
[independent audit](../research/audits/IA_1_INDEPENDENT_AUDIT.md) and
[external review](../research/audits/IA_1_EXTERNAL_REVIEW.md). W0 is
**COMPLETE / PASS / BASELINED**. V1-1 is **COMPLETE / PASS / BASELINED**.
Gate A is **PASS / CLOSED**. Core language is **Rust**
([ADR-0002](../architecture/decisions/ADR-0002-v1-core-language.md)
**Accepted**). V1-2 is **COMPLETE / PASS / BASELINED**.
V1-3 is **COMPLETE / PASS / BASELINED**. Gate B is **PASS / CLOSED**.
V1-4 is **COMPLETE / PASS / BASELINED**.
V1-5 is **COMPLETE / PASS / BASELINED**. Gate C is **PASS / CLOSED**.
V1-6 is **COMPLETE / PASS / BASELINED**. Preview viewer/payload/surface is
**Accepted** ([ADR-0006](../architecture/decisions/ADR-0006-v1-engine-independent-preview.md)).
V1-7 is **SKIPPED / OPTIONAL** (no concrete Product consumer for an
output/export artifact contract; the V1-6 GLB Preview payload is not that
consumer). V1-8 is **COMPLETE / PASS / BASELINED**. Gate D is
**PASS / CLOSED**. Final independent result: `GATE_D_PASS_CANDIDATE`.
Technical V1 release readiness is PASS. Public redistribution is not
automatically authorized. Blender redistribution / legal remains
HUMAN / LEGAL REVIEW REQUIRED. Do not start V1-7 export. Do not open
Gate D2. No additional V1 implementation stage is authorized.
Post-V1 R1-0 (First-Use UX Hardening design) is **PASS / DESIGN COMPLETE / BASELINED**.
R1-1 is **COMPLETE / PASS / BASELINED**. R1-2 is **COMPLETE / PASS / BASELINED**.
R1-V is **READY / NOT STARTED**.
The superseded R1-A / R1-B / R1-C execution sequence is not current authority.
Contract: [R1_ROADMAP.md](R1_ROADMAP.md).
Storage: SQLite **Accepted**
([ADR-0003](../architecture/decisions/ADR-0003-v1-local-catalog-storage.md)).
GUI: egui/eframe **Accepted**
([ADR-0004](../architecture/decisions/ADR-0004-v1-workbench-gui.md)).
Blender worker process: **Accepted**
([ADR-0005](../architecture/decisions/ADR-0005-v1-blender-worker-process.md)).
Preview viewer/payload/surface: **Accepted**
([ADR-0006](../architecture/decisions/ADR-0006-v1-engine-independent-preview.md)).

## Immediate sequence

```text
W0
COMPLETE / PASS / BASELINED
        ↓
IA-1
COMPLETE / PASS / CLOSED
        ↓
V1-1
COMPLETE / PASS / BASELINED
        ↓
Gate A
PASS / CLOSED
        ↓
V1-2
COMPLETE / PASS / BASELINED
Local-first Asset Catalog, persistence, orchestration,
and Workbench GUI shell
        ↓
V1-3
COMPLETE / PASS / BASELINED
Pinned isolated Blender worker integration
Gate B PASS / CLOSED
        ↓
V1-4
COMPLETE / PASS / BASELINED
Skeleton Mapping and compatibility workflow
        ↓
V1-5
COMPLETE / PASS / BASELINED
Transfer, QC, and Derived Variant lifecycle
Gate C PASS / CLOSED
        ↓
V1-6
COMPLETE / PASS / BASELINED
Engine-independent Preview
        ↓
V1-7
SKIPPED / OPTIONAL
No concrete Product consumer for an output/export artifact contract.
The V1-6 GLB Preview payload is not that consumer.
        ↓
V1-8
COMPLETE / PASS / BASELINED
Real-asset hardening and release qualification
        ↓
Gate D
PASS / CLOSED
Final independent result: GATE_D_PASS_CANDIDATE
Technical V1 release readiness: PASS
        ↓
R1-0
PASS / DESIGN COMPLETE / BASELINED
First-Use UX Hardening design
        ↓
R1-1
COMPLETE / PASS / BASELINED
First-Use Ingest Experience
        ↓
R1-2
COMPLETE / PASS / BASELINED
Responsive Execution Experience
        ↓
R1-V
READY / NOT STARTED
Integrated Validation + Human UAT
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
| W0 | Research Baseline | COMPLETE / PASS / BASELINED |

## New validation path

| Stage | Purpose | Status |
| --- | --- | --- |
| POC-BLENDER-E2E-01 | Minimal architecture slice: hidden Blender worker, thin domain, real cross-Skeleton pair | COMPLETE / PASS / BASELINED |
| POC-PREVIEW-01R | Click-to-preview from derived Preview Artifacts | COMPLETE / PASS / BASELINED |
| W0-RS | [Revised W0 synthesis](../research/decisions/W0_REVISED_SYNTHESIS.md) and [traceability](../research/decisions/W0_RS_TRACEABILITY.md) | COMPLETE / PASS / BASELINED |
| IA-1 | Independent architecture / research audit | COMPLETE / PASS / CLOSED |

POC-BLENDER-E2E-01 tests execution architecture only. Auto-Mapping quality,
full non-humanoid E2E, crash/timeout campaigns, worker pools, advanced
contact QC, packaging qualification, and multiple export profiles belong to
later V1 hardening. See [../research/poc/README.md](../research/poc/README.md).

## Future V1 implementation

V1-1 is **COMPLETE / PASS / BASELINED**. Gate A is **PASS / CLOSED**.
V1-2 is **COMPLETE / PASS / BASELINED**. V1-3 is **COMPLETE / PASS / BASELINED**.
Gate B is **PASS / CLOSED**. V1-4 is **COMPLETE / PASS / BASELINED**.
V1-5 is **COMPLETE / PASS / BASELINED**. Gate C is **PASS / CLOSED**.
V1-6 is **COMPLETE / PASS / BASELINED**. Names are planning
labels, not frozen lifecycle IDs.

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

## Independent gates

V1-4 is **COMPLETE / PASS / BASELINED**. V1-5 is **COMPLETE / PASS /
BASELINED**. Gate B is **PASS / CLOSED**. Gate C is **PASS / CLOSED**.
Do not open Gate B2. Do not open Gate C2, a post-Gate-C audit, or a
V1-5 release audit. V1-6 itself did not get a default independent Gate.

```text
Gate C:
PASS / CLOSED
Purpose: Product Core E2E Audit
Independent result: GATE_C_PASS_CANDIDATE
OPEN MAJOR: 0
OPEN MINOR: 0
OBSERVATION: 1 (GATE-C-OBS-001, NON-BLOCKING)
```

Fixed independent Gate D:

```text
Gate D:
PASS / CLOSED
Purpose: Final Release Readiness Audit
Independent result: GATE_D_PASS_CANDIDATE
OPEN MAJOR: 0
OPEN MINOR: 0
OBSERVATION: 11 (NON-BLOCKING)
```

Project acceptance: **Gate D PASS / CLOSED**. Do not open Gate D2.

## Deferred Gate B observations

These are accepted future hardening, not unresolved V1-3 blockers. They
belong primarily to later hardening / V1-8 unless a later owner stage needs
a narrower piece:

```text
spawn-to-RUNNING application crash recovery
  current: addressed in V1-8 (dispatch-intent fail-closed on reopen)
release worker-script/package integrity
  current: addressed in V1-8 (relocatable RuntimeLayout + pinned digests)
process reattachment
worker pool
running cancellation / retry policy
release packaging / installer
GPL/legal release qualification
  current: V1-8 inventoried notices/pins; distribution legal clearance remains open / POST-V1
broader asset coverage
  current: successful real non-humanoid retarget quality is POST-V1 / not a V1 guarantee
```

## Deferred Gate C observations

`GATE-C-OBS-001` remains **historical** Gate C evidence. Do not rewrite
Gate C audit reports.

```text
GATE-C-OBS-001 (historical)
Native Workbench controls / Application retention
are not fully event-wired.
```

V1-6 wired native Preview and did not close this observation.
V1-8 wires Mapping / Compatibility / Transfer acknowledgement / Transfer
through Application-backed Workbench handlers. Current disposition is
recorded in [V1_8_IMPLEMENTATION_PLAN.md](V1_8_IMPLEMENTATION_PLAN.md)
and review evidence. Historical Gate C text stays historical.

## Current next step

```text
Start R1-V — Integrated Validation + Human UAT.

Do not create another implementation stage unless R1-V finds a concrete issue.
V1 remains COMPLETE.
Gate D remains PASS / CLOSED.

Do not reopen R1-A or R1-B as separate stages.
Do not use the superseded old R1 lifecycle.
Do not reopen V1.
Do not reopen Gate D.
Do not create V1-9.
```

Do not open Gate D2. Do not start V1-7 export.

Historical GATE-C-OBS-001 remains historical Gate C evidence. V1-8 owns
current native Workbench event-wiring disposition.

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
| W6 | Domain API + CLI | later implementation, after V1-1 planning |
| W7 | Minimal Workbench | becomes Asset Browser / Transfer Tray after V1-1 |
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
