# Gate B Runtime Foundation Findings

Audit target: committed V1-2 at
`18c76c521c86c9d31382ece5c03d431c38b4df08` plus the uncommitted V1-3
candidate reviewed on 2026-09-01.

## Summary

```text
OPEN MAJOR:       4
OPEN MINOR:       2
OPEN OBSERVATION: 3
```

Gate B cannot pass while the MAJOR findings remain open.

## GATE-B-MAJOR-001 — WorkerResult is not bound to its JobRun attempt

**Severity:** MAJOR

**Affected files:**

- `domain/src/execution.rs:170-300`
- `app/src/catalog.rs:457-481`
- `app/src/application.rs:170-175`

**Reproduction:**

1. Enqueue the same validated `JobSpec` twice, producing two different
   `JobRun.attempt_id` values.
2. Dispatch both runs to `RUNNING`.
3. Create one successful `Validated<WorkerResult>` for that `JobSpec`.
4. Call `complete_success(run_a, result)`, then
   `complete_success(run_b, result)`.
5. Both calls succeed and both runs point to the same WorkerResult even
   though the result cannot evidence both attempts.

`WorkerResult` carries `job_spec_id` but no attempt identity or opaque worker
execution reference. `complete_success` checks only the JobSpec and
`worker_success`, and it remains a public Catalog/Application API after V1-3
added the guarded collection path.

**Contract violated:**

- V1-2 orchestration attempt identity ownership.
- V1-3 one fresh process per attempt and matching terminal receipt.
- Gate B orchestration requirement that a WorkerResult from one attempt
  cannot complete another.

**Impact:**

An ordinary public API call can durably mark an attempt `SUCCEEDED` using a
stale or fabricated successful result from a different attempt. This is false
runtime success even though Product publication remains separately gated.

**Required correction:**

Make successful completion inseparable from the collected attempt. Either
persist and validate attempt/execution-reference binding in WorkerResult
evidence, or seal arbitrary `complete_success` access and complete only from a
validated receipt/outcome tied to the durable `JobRun`. Also prevent one
WorkerResult ID from authorizing multiple attempts.

## GATE-B-MAJOR-002 — Rejected failed outcomes can leave a terminal attempt RUNNING

**Severity:** MAJOR

**Affected files:**

- `app/src/catalog.rs:532-583`
- `app/src/catalog.rs:494-529`

**Reproduction:**

1. Put a JobRun into `RUNNING`.
2. Supply a normal `WorkerCompletionPort` implementation that returns
   `TerminalOutcome::Failed` with a failed WorkerResult bound to another
   JobSpec (or with `worker_success == true`).
3. `collect` calls `complete_terminal_failure`.
4. Result validation rejects the WorkerResult and the error propagates.
5. Reloading the JobRun shows `RUNNING`, despite collection having returned a
   terminal failure.

The success branch catches completion failure and attempts to mark the run
failed. The failed-outcome branch uses `?` directly and has no equivalent
fallback.

**Contract violated:**

- `V1_3_WORKER_CONTRACT.md`: collection failure after launch must persist
  JobRun `FAILED`.
- `V1_3_PROCESS_LIFECYCLE.md`: collection errors must not leave an attempt
  `RUNNING`.
- Gate B orchestration requirement for terminal failure binding.

**Impact:**

A malformed or mismatched terminal failure receipt leaves durable state
claiming execution is still active. This is an unsafe ordinary state-machine
outcome and blocks credible failure recovery.

**Required correction:**

Treat a rejected terminal WorkerResult as collection failure evidence:
atomically persist `FAILED` without the rejected result, retain the rejection
diagnostic, and add tests for mismatched and success-valued WorkerResults on
the failed-outcome branch.

## GATE-B-MAJOR-003 — Rational frame provenance is silently changed at dispatch

**Severity:** MAJOR

**Affected files:**

- `domain/src/time.rs:13-97`
- `domain/src/time.rs:150-196`
- `app/src/dispatch.rs:73-96`

**Reproduction:**

1. Ingest a valid frame `TimePoint` through the public validated JSON boundary
   with `value_num = 3`, `value_den = 2`, and a valid FPS rational.
2. Domain validation accepts the rational frame point.
3. Assemble `WorkerDispatchRequest`.
4. The request exports `frame_start = 3`; `value_den` is neither carried nor
   rejected.
5. The Blender worker executes from frame 3 rather than the recorded 3/2
   frame point.

The same loss applies to the end point. The constructor currently creates
integer frames, but validated serialized ingress explicitly remains a normal
Domain boundary and the V1-1 contract defines rational `TimePoint` values.

**Contract violated:**

- V1-1 explicit rational time provenance and “frames are not seconds”.
- V1-3 exact Catalog projection before Blender.
- Gate B exact-provenance and dispatch-resolution requirements.

**Impact:**

A Domain-valid exact Motion version can execute a different interval from the
one durably recorded while still producing worker success.

**Required correction:**

Carry the full rational frame points through the execution projection and
implement them, or reject non-integral frame points before Blender mutation.
Add validated-ingress and dispatch tests covering non-unit denominators.

## GATE-B-MAJOR-004 — Root execution can violate KeepTargetRestScale

**Severity:** MAJOR

**Affected files:**

- `blender-worker/python/worker.py:446-476`
- `blender-worker/python/worker.py:487-512`
- `blender-worker/python/worker.py:526-539`

**Reproduction:**

1. Use the accepted Policy with a mapped root whose source animation has a
   non-rest scale.
2. In `apply_entries`, the root branch assigns the complete
   `tgt_rest @ (src_rest^-1 @ src_now)` matrix to the target pose bone.
3. Unlike the non-root branch, it never restores target pose scale to
   `(1, 1, 1)`.
4. Stored keys contain root location and rotation only; scale is neither
   normalized nor audited.
5. The final source scale delta can remain as unkeyed target pose state in the
   saved `.blend`, while rotation-only/root audits still report PASS.

**Contract violated:**

- Structured `ScalePolicy::KeepTargetRestScale`.
- V1-3 requirement to execute the exact recognized Policy.
- Gate B target-rest-scale and false-worker-success checks.

**Impact:**

The production worker can return success and fresh-reopen PASS for output
that silently violates accepted Product Policy. The frozen Knight/UAL2 pair
does not expose this because its observed root scale remains benign.

**Required correction:**

Decompose root translation/rotation from the rest-relative delta without
applying source scale, explicitly restore target pose scale, and audit the
root together with non-root scale. Add a deterministic root-scale fixture.

## GATE-B-MINOR-001 — Completed attempts remain in adapter memory

**Severity:** MINOR

**Affected files:**

- `blender-worker/src/adapter.rs:64-72`
- `blender-worker/src/adapter.rs:577-804`

**Reproduction:**

Run and collect multiple attempts through one long-lived `BlenderWorker`.
Every dispatch inserts a `LaunchedAttempt`; no terminal branch removes it.

**Contract violated:**

No Product-authority contract is violated. This conflicts with proportional
long-lived worker resource hygiene expected by Gate B.

**Impact:**

Memory and bookkeeping grow with every completed attempt. Child handles are
released, but JobSpecs and workspace/path state are retained indefinitely.

**Required correction:**

Remove terminal attempts after constructing the outcome, retaining durable
diagnostics in the workspace/Catalog. Owner: V1-3 correction or V1-8 worker
reliability hardening.

## GATE-B-MINOR-002 — Unicode stderr can panic terminal collection

**Severity:** MINOR

**Affected files:**

- `blender-worker/src/adapter.rs:855-864`
- `blender-worker/src/adapter.rs:331-344`

**Reproduction:**

Capture Blender stderr longer than 1200 bytes where byte offset
`len - 1200` falls inside a multibyte UTF-8 character. `tail` slices the
string at that byte index and panics.

**Contract violated:**

V1-3 terminal collection should convert ordinary process diagnostics into a
durable outcome rather than crash.

**Impact:**

A normal localized traceback or Unicode path can crash collection and leave
the JobRun in the acknowledged recoverability window. It does not create
false Product success.

**Required correction:**

Truncate at a valid character boundary (or by characters/bytes before UTF-8
decoding) and add a Unicode diagnostic test. Owner: V1-3 correction.

## GATE-B-OBS-001 — Spawn-to-RUNNING crash window remains

**Severity:** OBSERVATION

**Affected files:**

- `blender-worker/src/adapter.rs:505-572`
- `app/src/catalog.rs:428-449`

**Reproduction:**

Terminate the application after `cmd.spawn()` succeeds and before the
Catalog writes `RUNNING`.

**Contract violated:**

None at this Gate. The process lifecycle explicitly acknowledges this window.

**Impact:**

The run may remain `DISPATCHABLE` while a diagnosable process/workspace
exists. `launch.json` records command, attempt, worker reference, and best-
effort PID. No false success is recorded.

**Required correction:**

Deferred owner: V1-8 worker reliability. Add reconciliation/reattachment or a
supervised launch protocol before release qualification.

## GATE-B-OBS-002 — Worker script binding is development-tree binding

**Severity:** OBSERVATION

**Affected files:**

- `blender-worker/src/adapter.rs:809-816`

**Reproduction:**

Move an installed binary away from the Cargo source tree or omit
`blender-worker/python/worker.py`; the compile-time
`CARGO_MANIFEST_DIR` path no longer identifies a packaged script.

**Contract violated:**

None for the current repository/dev runtime. Release packaging is explicitly
deferred.

**Impact:**

Current tests execute the reviewed script and are trustworthy for Gate B.
Installer layout and script integrity are not yet established.

**Required correction:**

Deferred owner: V1-8 packaging. Bind packaged script bytes/location and
integrity in the release artifact.

## GATE-B-OBS-003 — Packaged test breakdown miscounts app tests

**Severity:** OBSERVATION

**Affected files:**

- `review_evidence/test_results.txt` inside
  `rigforge_v1_3_rev1_review.zip`

**Reproduction:**

Sum the rerun output: app has 7 unit plus 47 integration tests = 54, not the
stated 7 plus 49 = 56. The overall total remains correctly stated as 173.

**Contract violated:**

None in runtime implementation.

**Impact:**

The evidence summary contains an arithmetic inconsistency, but the independent
rerun established 173 PASS / 0 FAIL.

**Required correction:**

Correct the evidence breakdown in the next review package.
