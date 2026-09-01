# V1-3 Implementation Plan

Pinned isolated Blender worker integration.

Status: `COMPLETE / PASS / BASELINED`

Gate B is **PASS / CLOSED**. V1-4 is `READY / NOT STARTED`.

Related:

- [V1_3_WORKER_CONTRACT.md](V1_3_WORKER_CONTRACT.md)
- [V1_3_PROCESS_LIFECYCLE.md](V1_3_PROCESS_LIFECYCLE.md)
- [ADR-0005](../architecture/decisions/ADR-0005-v1-blender-worker-process.md)
  (`Accepted` at V1-3 / Gate B Runtime Foundation Audit)
- V1-2: [V1_2_ORCHESTRATION_CONTRACT.md](V1_2_ORCHESTRATION_CONTRACT.md)
- Decision: [ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md)

## Baseline

```text
branch: main
HEAD:   18c76c521c86c9d31382ece5c03d431c38b4df08
```

V1-2 remains `COMPLETE / PASS / BASELINED`. This stage does not reverse
accepted V1-1 / V1-2 semantics, Product contracts, ADR-0001–0004, or W0 / IA-1
evidence.

## Goals

```text
validated exact RigForge Job
        ↓
real isolated Blender process
        ↓
real transfer/bake execution
        ↓
backend-neutral WorkerResult
        ↓
durable JobRun terminal state
```

## Non-goals (owned later)

Auto-Mapping, Compatibility UX, Product QC, Derived Variant publication,
Preview generation/viewer, Export pipeline, worker pool, running-job
cancellation, retry framework, release packaging / legal clearance.

## Shape

```text
blender-worker/
    Rust adapter / process integration
    python/worker.py
```

Research harness remains at `experiments/w0p/poc_blender_e2e_01/`. Production
runtime does not import `run_e2e.py`.

## Review

Gate B Runtime Foundation Audit is **PASS / CLOSED**. Do not open a second
independent Gate for V1-3. Next stage is V1-4.
