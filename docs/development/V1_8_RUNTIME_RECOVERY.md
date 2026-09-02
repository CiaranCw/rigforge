# V1-8 Runtime Recovery

Spawn-to-RUNNING, restart, retry, cancellation, and worker-pool disposition.

Status: `COMPLETE / PASS / BASELINED`

## Spawn-to-RUNNING

Known window:

```text
worker process successfully spawned
        ↓
application crashes before durable RUNNING state update
```

Characterization:

- Durable JobRun can remain `DISPATCHABLE` while a process exists.
- Publication cannot occur from `DISPATCHABLE`.
- Orphan workspace files are not Product records.

Smallest recovery (implemented):

1. Write `rigforge-dispatch-intents/<run_id>.intent.json` beside the Catalog
   before `dispatch_resolved`.
2. Clear the intent after durable `RUNNING` or after `fail_dispatch`.
3. On Catalog open, `DISPATCHABLE` + leftover intent → durable `FAILED`.

No schema migration. `DB_SCHEMA_VERSION` remains 1. Intent files are not
Product identity.

Tests: `app/tests/runtime_recovery.rs`.

## RUNNING after application restart

In-process `BlenderWorker` handles are not retained across process restart.
Collect cannot reattach without a supervisor.

On Catalog open, leftover `RUNNING` JobRuns become durable `FAILED` with a
diagnostic reason. Product publication remains blocked. A new user-requested
attempt may be created.

This is fail-closed recovery, not process reattachment.

## RUNNING cancellation

```text
status: NOT REQUIRED FOR V1
```

Not implemented. Native Transfer currently waits on collect on the UI
thread. Misleading "still RUNNING" after crash is addressed by fail-closed
reopen. Permanent Product unusability was not demonstrated for normal local
use once leftover RUNNING fails closed.

## Retry

```text
status: NOT REQUIRED FOR V1
```

Automatic retry is not implemented. Manual re-run is sufficient:

- failed attempt remains durable
- a new user-requested attempt can be created
- attempt IDs remain unique
- old WorkerResult cannot authorize a new JobRun
- publication remains exact

Evidence: existing orchestration tests, including
`one_worker_result_id_cannot_complete_two_runs` and
`same_jobspec_different_attempt_cannot_reuse_worker_result`.

## Worker pool

```text
status: NOT REQUIRED FOR V1
```

Sequential fresh-process execution remains the isolation invariant:

```text
one fresh Blender process
+
one isolated workspace
per attempt
```

Throughput pooling is not required by V1 Product evidence.

## What remains diagnosable without a supervisor

- Catalog JobRun terminal state after reopen
- dispatch intent files (if a crash hits the window)
- attempt workspace under the worker workspace root (`launch.json`, logs)
- no Product publication from orphan output

Orphan Blender processes may still exist until the OS reaps them. They
cannot authorize Catalog publication.
