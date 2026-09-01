# V1-2 Orchestration Contract

Product-side job orchestration and worker-dispatch *boundary*. This stage
does not execute Blender.

Status: `COMPLETE / PASS / BASELINED`

Catalog persistence: [V1_2_CATALOG_CONTRACT.md](V1_2_CATALOG_CONTRACT.md).

## Intent vs runtime

| Object | Mutability | Role |
| --- | --- | --- |
| `JobSpec` | immutable Domain record | Product execution intent; exact version IDs only |
| `JobRun` | mutable orchestration row | runtime state, attempt identity, failure diagnostics |

Do not mutate `JobSpec` to represent progress, worker PID, or retry count.

A JobRun never stores `latest` / `current` Character, Motion, Mapping, or
Policy. Exact IDs are already inside the validated `JobSpec`.

## Lifecycle

```text
QUEUED
  → DISPATCHABLE
  → RUNNING
  → SUCCEEDED   (requires matching successful WorkerResult)
  → FAILED

QUEUED         → CANCELLED
DISPATCHABLE   → CANCELLED
DISPATCHABLE   → FAILED     (dispatch/start failure)
```

`RUNNING → SUCCEEDED` is not a generic state write. It requires
`complete_success` with a `WorkerResult` whose `job_spec_id` equals the
run's `JobSpec` and `worker_success == true`. `SUCCEEDED` without a
WorkerResult is not a legal V1 runtime state; that gap is `RUNNING`.

A `WorkerResult` with `worker_success == false` cannot mark the run
`SUCCEEDED`. Use `complete_failure` (from `RUNNING`) or the dispatch-start
failure path.

Terminal states: `SUCCEEDED`, `FAILED`, `CANCELLED`.

No other writes are valid. Arbitrary `UPDATE job_runs SET state=` is not
part of the API.

Negative tests cover illegal transitions (for example `QUEUED → SUCCEEDED`,
`RUNNING → CANCELLED`, `SUCCEEDED → RUNNING`).

## Dispatch

```text
Catalog / Orchestrator
        ↓
validated exact JobSpec
        ↓
WorkerPort::dispatch
        ↓
V1-2: FakeWorker
V1-3: Blender worker (not implemented here)
```

The dispatcher **must not** resolve latest/current at execution time.

Enqueue requires the exact Catalog graph to pass V1-1
`validate_job_inputs` (Character, Motion, Source Skeleton, Mapping, Policy,
JobSpec). Invalid graphs never persist a `JobSpec`/`JobRun` pair from
`enqueue_job`. Dispatch repeats that check defensively; it does not invent
a second rule set.

`WorkerPort` is backend-neutral. It accepts `&Validated<JobSpec>` plus the
orchestrator-owned `attempt_id` and must echo that same attempt identity.
It does not import `bpy`, spawn Blender, or generate Preview.

A WorkerPort error is durable: `DISPATCHABLE → FAILED` with a persisted
failure reason. The run is not left `DISPATCHABLE` after a failed start.

V1-2 ships `FakeWorker` for tests. There is no Blender implementation in
this stage.

## Crash / retry boundary (representation only)

V1-2 does **not** implement worker crash/retry policy (V1-3 / V1-8).

A `JobRun` can represent:

- `attempt_id` (orchestration UUIDv7; not a Product ID; owned by the
  orchestrator, generated once per JobRun, stable across dispatch)
- `worker_execution_ref` (opaque; fake worker uses a stable token)
- `failure_reason`
- `created_at` / `updated_at` (unix milliseconds, ordering)

A later retry is a **new attempt** (new `JobRun` and/or new `attempt_id`),
not mutation of a historical `WorkerResult`. `WorkerResult` rows are
append-only Domain records.

## Worker success is not publication

Unchanged from V1-1:

```text
worker SUCCESS ≠ QC PASS ≠ publication
```

Orchestration `SUCCEEDED` means the *run* reached a successful worker
receipt according to the port. It does not publish a Derived Variant.

## Application API (minimal)

- create `JobSpec` from exact version IDs (Domain)
- persist `JobSpec`
- enqueue → `JobRun` in `QUEUED`
- mark dispatchable
- dispatch through `WorkerPort` (exact spec)
- succeed / fail with diagnostics
- cancel from `QUEUED` or `DISPATCHABLE`
- load job status without mutating Product records

GUI reads this API. GUI does not own the state machine.
