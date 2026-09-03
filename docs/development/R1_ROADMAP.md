# R1 Roadmap — First-Use UX Hardening

Post-V1 lifecycle. This is not V1-9 and does not reopen Gate D.

Status: `R1-0 PASS / DESIGN COMPLETE / BASELINED`

Do not confuse this track with the historical W0 research file
[R1_RESEARCH_BASELINE.md](../research/R1_RESEARCH_BASELINE.md). That file
remains W0 research history. RigForge R1 here means First-Use UX Hardening.

## Current lifecycle (project)

```text
V1 technical baseline: COMPLETE
Gate D: PASS / CLOSED
Final independent Gate D result: GATE_D_PASS_CANDIDATE
R1-0: PASS / DESIGN COMPLETE / BASELINED
R1-A: READY / NOT STARTED
R1-B: NOT STARTED
R1-C: NOT STARTED
```

## Sequence

```text
R1-0
First-Use UX Contract & Architecture
PASS / DESIGN COMPLETE / BASELINED
        ↓
R1-A
Native Asset Selection
NEXT / NOT STARTED
        ↓
R1-B
Motion Metadata Discovery
NOT STARTED
        ↓
R1-C
Responsive Long Operations
NOT STARTED
        ↓
R1-FR
Focused Source + UX Review
NOT STARTED
        ↓
R1-UAT
Real Human First-Use Checkpoint
NOT STARTED
        ↓
R1-D
UAT-driven Corrections
NOT STARTED
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

## Stage intent

| Stage | Intent |
| --- | --- |
| R1-0 | Design only. Freeze UX contract, discovery schema, TOCTOU, non-blocking Transfer ownership. No production mutation. |
| R1-A | Native FBX browse for Character and Motion. Filename-derived editable names. No typed path as the normal path. |
| R1-B | Backend-neutral inspect. Unique Armature 0/1/fail-closed. Strong Action-to-Armature association only (`direct_action` / `nla_strip` / `pose_channels`). Multi-clip on that Armature. Exact clip identity into `TimeDomainProvenance`. Exact `Fraction(str(fps_base))` FPS. No Product subobject schema. |
| R1-C | Blender waits via existing `WorkerCompletionPort::collect` on a `Send` BlenderWorker thread. QC/reopen waits use existing pinned inspect/reopen functions. One live Application. |
| R1-FR | Focused source + UX review of implemented R1-A/B/C before UAT. Not an independent Gate. |
| R1-UAT | Real human checkpoint on Knight + UAL2 without being told clip/FPS/skeleton fields. |
| R1-D | Only issues observed or clearly reproduced in R1-UAT. Not a speculative feature bucket. |
| R1 Gate | Independent First-Use Readiness Audit by a new GPT-5.6 Sol High agent. |
| R1 Baseline | Commit/push only after design/implementation acceptance that is separately authorized. |

## Current next step

```text
Start R1-A — Native Asset Selection.

Do not start R1-B or R1-C in the same implementation task.
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

R1-0 is `PASS / DESIGN COMPLETE / BASELINED`. The accepted design now
authorizes R1-A implementation as a separate task.

R1-A may add Workbench `rfd` 0.17.2 and native Browse. Do not start R1-B
or R1-C in the same implementation task as R1-A.
