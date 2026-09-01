# ADR-0005: V1 production Blender worker process

Status: **Accepted**

Date: 2026-09-01

Accepted at: **V1-3 / Gate B Runtime Foundation Audit** (`PASS / CLOSED`).

Owner stage: V1-3

This decision is process packaging and isolation for the accepted Blender
backend. It does not reopen ADR-0001 (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`)
and it does not select a Preview viewer or payload.

## Context

V1-2 defined a backend-neutral `WorkerPort` and `FakeWorker`. V1-3 must run
a real pinned Blender process per attempt without importing the research
harness `experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py`.

## Decision

- Production adapter lives in crate `blender-worker` (`rigforge_blender_worker`).
- Python entrypoint lives at `blender-worker/python/worker.py`.
- One job attempt = one fresh Blender process + one isolated workspace.
- Command always includes `--background --factory-startup --disable-autoexec --python-exit-code 1`.
- Pin: official Blender 5.2.1 LTS, build `9e2066aef7ef`.
- Launch (`dispatch_resolved`) is separate from terminal collection (`collect`).
- Reopen verification is a second fresh process.

## Alternatives considered

- Import the PoC `run_e2e.py` as production runtime: rejected. Research-only campaign harness.
- Complete Blender inside `dispatch()`: rejected. That would invert RUNNING semantics.
- User-installed Blender / add-ons: rejected. Isolation and pin would be lost.

## Consequences

Workbench and Domain still do not call `bpy`. GUI does not depend on this
crate. Crash-before-RUNNING remains a diagnosable workspace, not a
supervisor platform.

## Evidence

POC-BLENDER-E2E-01 (`COMPLETE / PASS / BASELINED`) plus V1-3 production
checkpoint on the frozen Knight / UAL2 pair.

## Open questions

These remain deferred later hardening / V1-8 work. They are not unresolved
V1-3 blockers:

- worker pool
- running cancellation / retry policy
- restart / process reattachment
- spawn-to-RUNNING application crash recovery
- release packaging / installer
- release worker-script / package integrity
- GPL / legal release qualification
- broader asset coverage
