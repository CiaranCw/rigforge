# V1-3 Process Lifecycle

Launch is not completion. One attempt is one fresh Blender process and one
isolated workspace.

Status: `COMPLETE / PASS / BASELINED`

## Lifecycle

```text
resolve exact Catalog graph
        ↓
WorkerDispatchRequest
        ↓
create isolated attempt workspace
        ↓
WorkerPort::dispatch_resolved → spawn Blender
        ↓
DispatchReceipt { attempt_id, worker_execution_ref }
        ↓
JobRun RUNNING
        ↓
WorkerCompletionPort::collect
        ↓
wait + stdout/stderr + envelope
        ↓
fresh-process reopen (success path)
        ↓
Validated<WorkerResult>
        ↓
JobRun SUCCEEDED / FAILED
```

`RUNNING` is written only after a matching receipt. `SUCCEEDED` still
requires `complete_success` with `worker_success == true` and matching
`job_spec_id`.

`RUNNING` cannot be cancelled (unchanged from V1-2). Retry is a new
attempt.

## Command

```text
blender.exe
  --background
  --factory-startup
  --disable-autoexec
  --python-exit-code 1
  --python <blender-worker/python/worker.py>
  --
  <execute|reopen> <job.json>
```

Do not weaken background, factory startup, disabled autoexec, or explicit
Python exception exit code.

## Isolation

Every attempt gets `.../attempts/<attempt_id>/` with at least:

```text
TEMP
TMP
TMPDIR
BLENDER_USER_CONFIG
BLENDER_USER_SCRIPTS
BLENDER_USER_DATAFILES
```

redirected into that workspace. Do not use interactive user Blender state,
user startup files, installed add-ons, or a shared mutable scene.

Reopen uses a **second** Blender process. It must not reuse the execute
scene.

`attempt_id` is orchestrator-owned. `worker_execution_ref` is
`blender-worker:<attempt_id>` and is not a second attempt identity.

## Crash window

If the application crashes after spawn and before durable RUNNING / receipt
update, the attempt workspace and `launch.json` (command, attempt, optional
pid) remain diagnosable under `attempts/<attempt_id>/`. Full reattachment,
supervisor, and pool recovery are later hardening. Gate B reviews this
boundary.

## Pin

```text
Blender:  5.2.1 LTS
Build:    9e2066aef7ef
Archive:  blender-5.2.1-windows-x64.zip
SHA-256:  0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c
```

Prefer the already extracted official PoC toolchain. Do not redownload when
present. Executable location may use `RIGFORGE_BLENDER_EXECUTABLE`; pin
verification remains mandatory on the production constructor. The worker
script is bound to `blender-worker/python/worker.py` and is not caller-
overridable on the production API.

If `WorkerCompletionPort::collect` returns `Err` after launch, Catalog
persists `JobRun FAILED` and does not leave the attempt `RUNNING`.

If collection returns `TerminalOutcome::Failed` but the supplied
`WorkerResult` cannot be bound, Catalog still persists `JobRun FAILED`
without attaching the rejected result, and records the rejection in
`failure_reason`. The run must not remain `RUNNING`.

Release packaging, installer, and GPL/legal clearance remain V1-8.
