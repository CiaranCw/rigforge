# R1 Roadmap — First-Use UX Hardening

Post-V1 lifecycle. This is not V1-9 and does not reopen Gate D.

Status: `R1-1 COMPLETE / PASS / BASELINED`

Do not confuse this track with the historical W0 research file
[R1_RESEARCH_BASELINE.md](../research/R1_RESEARCH_BASELINE.md). That file
remains W0 research history. RigForge R1 here means First-Use UX Hardening.

## Current lifecycle (project)

```text
V1 technical baseline: COMPLETE
Gate D: PASS / CLOSED
Final independent Gate D result: GATE_D_PASS_CANDIDATE
R1-0: PASS / DESIGN COMPLETE / BASELINED
R1-1: COMPLETE / PASS / BASELINED
R1-2: READY / NOT STARTED
R1-V: NOT STARTED
R1-FIX: CONDITIONAL
R1 Gate: NOT STARTED
```

## Sequence

```text
R1-0
First-Use UX Contract & Architecture
PASS / DESIGN COMPLETE / BASELINED
        ↓
R1-1
First-Use Ingest Experience
COMPLETE / PASS / BASELINED
        ↓
R1-2
Responsive Execution Experience
READY / NOT STARTED
        ↓
R1-V
Integrated Validation + Human UAT
NOT STARTED
        ↓
R1-FIX
CONDITIONAL — only if R1-V finds concrete issues
        ↓
R1 Gate
Independent First-Use Readiness Audit
NOT STARTED
        ↓
R1 Baseline
NOT STARTED
```

Do not call the independent R1 gate `Gate E` or `Gate D2`.
V1 Gate D remains immutable and `PASS / CLOSED`.

Historical R1-0 design decomposition:

```text
R1-A / R1-B / R1-C
```

Current implementation batching:

```text
R1-1 / R1-2 / R1-V / optional R1-FIX / R1 Gate
```

```text
old R1-A + old R1-B  →  R1-1
old R1-C Transfer/QC/reopen responsiveness  →  R1-2
old R1-FR + old R1-UAT  →  R1-V
old R1-D  →  R1-FIX, CONDITIONAL ONLY
```

The superseded R1-A / R1-B / R1-C execution sequence is not current authority.

## Stage intent

| Stage | Intent |
| --- | --- |
| R1-0 | Design only. Freeze UX contract, discovery schema, TOCTOU, non-blocking Transfer ownership. No production mutation. |
| R1-1 | Native FBX browse, filename-derived names, unique-Armature inspect, strong clip association, rational FPS, inspection-backed registration, background ingest inspect. Combines historical R1-A + R1-B. |
| R1-2 | Transfer execute wait, QC inspection, fresh reopen, truthful running state. Historical R1-C responsiveness. |
| R1-V | Integrated source + UX review and real human first-use UAT. Historical R1-FR + R1-UAT. |
| R1-FIX | Only issues observed or clearly reproduced in R1-V. Conditional. Historical R1-D. |
| R1 Gate | Independent First-Use Readiness Audit by a new GPT-5.6 Sol High agent. |
| R1 Baseline | Commit/push only after design/implementation acceptance that is separately authorized. |

Historical R1-A/B/C names remain in R1-0 architecture text as design decomposition. They are not the current implementation sequence.

## Current next step

```text
Start R1-2 — Responsive Execution Experience.

R1-2 should address Transfer execute wait, QC inspection,
fresh reopen, truthful running state, and overall Workbench responsiveness.

Do not reopen R1-A or R1-B as separate stages.
Do not use the superseded old R1 lifecycle.
Do not reopen V1.
Do not reopen Gate D.
Do not create V1-9.
```

## R1 Gate

```text
Name: R1 Gate
Purpose: Independent First-Use Readiness Audit
Preferred agent: NEW clean Agent, GPT-5.6 Sol High
```

The auditor verifies:

- the frozen R1 Product requirement
- no V1 authority regression
- real UAT evidence
- the first-use workflow

This is not Gate D2 and not Gate E.

## Implementation authorization

R1-0 is `PASS / DESIGN COMPLETE / BASELINED`.

R1-1 is `COMPLETE / PASS / BASELINED`.
R1-2 is the next authorized implementation batch (historical R1-C
Transfer/QC/reopen responsiveness). Do not start Cancel, Retry, or a worker
pool unless R1-2 explicitly requires them.
