# V1-3 Worker Contract

Production Blender adapter contract. Storage is not Product authority.
Blender object names are execution diagnostics, not Product identity.

Status: `COMPLETE / PASS / BASELINED`

Process lifecycle: [V1_3_PROCESS_LIFECYCLE.md](V1_3_PROCESS_LIFECYCLE.md).

## Boundary

```text
Workbench
   ↓
Application / Catalog / Orchestrator
   ↓
WorkerDispatchRequest   (exact Catalog projection)
   ↓
WorkerPort::dispatch_resolved   (launch only)
   ↓
Blender adapter
   ↓
python/worker.py
   ↓
WorkerCompletionPort::collect
   ↓
Validated<WorkerResult>
```

Never: GUI → Blender / bpy. Domain → bpy. SQLite schema → bpy types.

`JobSpec` stays immutable intent. Runtime belongs to `JobRun`,
`WorkerResult`, and attempt-local execution evidence.

## Dispatch projection

`WorkerDispatchRequest` is assembled from exact Catalog records before the
worker boundary. It is backend-neutral execution input, not a new Product
identity.

It carries:

- JobSpec ID and orchestrator-owned `attempt_id`
- exact Character / Motion versions plus filesystem location + digest
- Source Skeleton reference
- exact `BoneMappingVersion` and `RetargetPolicyVersion`
- requested clip / frame range from Motion time provenance
- determinism / isolation text from JobSpec

Forbidden inside the worker: `latest` Character, `latest` Motion, current
Mapping, current Policy. Missing exact data fails before Blender mutation.

Path overlay may change operational location; Product IDs and Domain digests
do not change.

## Mapping / Policy

Bone Mapping remains RigForge truth. The adapter projects exact entries
(source joint, target joint, role, required). It does not auto-map, rename
correspondence, or author new rows.

Supported Policy is the V1-1 proven set only:

```text
copy_world_translation_delta
rotation_only_mapped_non_root
normalize_before_key
consecutive_hemisphere
every_source_frame
rest_relative_world_delta
remain_at_target_rest / fail_closed_do_not_invent
keep_target_rest_scale
explicit_no_ik
```

Unknown / unsupported Policy: **FAIL BEFORE MUTATION**.

Domain `PolicyContractStatus.executed` stays `false`. Worker execution does
not rewrite that Product field.

## WorkerResult

Use existing Domain `WorkerResult` / `BackendExecutionContext` /
`NamedMeasurement` / diagnostics / `staged_artifact_digests`.

A real V1-3 `WorkerResult` also carries backend-neutral execution
correlation (`attempt_id`, `worker_execution_ref`). These are runtime
execution correlation, not Product IDs. Before `SUCCEEDED`, Catalog
requires:

```text
WorkerResult.job_spec_id == JobRun.job_spec_id
WorkerResult.attempt_id == JobRun.attempt_id
WorkerResult.worker_execution_ref == JobRun.worker_execution_ref
WorkerResult.worker_success == true
```

One WorkerResult ID cannot authorize two JobRuns.

Every real result records:

```text
backend kind: Blender
version:        5.2.1 LTS
build:          9e2066aef7ef
adapter:        rigforge-blender-worker/0.1.0
```

Successful envelopes must match those identities. A mismatched envelope
fails. `WorkerResult` records the envelope adapter version when present and
does not substitute an unexecuted adapter identity.

Production `BlenderWorker` seals pin verification, source digest
verification, fresh-process reopen, and the crate-bundled worker script.
Successful collection requires the reopen envelope to contain an explicit
`root_scale_audit` of `PASS`. Missing, empty, `FAIL`, or unknown values are
reopen failure. Test-only fake executables live behind `for_fake_executable`
and cannot weaken the production constructor.

`worker SUCCESS ≠ QC PASS ≠ publication`. V1-3 does not create or publish a
`DerivedVariantVersion`.

Staged `.blend` bytes are attempt-local and non-authoritative. Digests may
be recorded on success. Failure must not register staged bytes as Product
truth.

## Failure classes

Distinguish at least:

```text
launch_failure
worker_script_exception
structured_worker_fail
missing_result_envelope
invalid_result_envelope
job_spec_mismatch
backend_build_mismatch
backend_version_mismatch
adapter_version_mismatch
source_digest_mismatch
unsupported_policy
reopen_failure
```

A process that never launched has no `WorkerResult`. A launched terminal
failure should persist a failed `WorkerResult` when an envelope or
diagnostics exist, then `JobRun FAILED`. If collection returns `Err` after
launch, Catalog still persists `JobRun FAILED` and does not fabricate a
`WorkerResult` without evidence.

Success requires structured envelope and process status to agree. File
existence alone is not success.
