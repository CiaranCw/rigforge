# V1-5 Transfer Lifecycle

Product Transfer authorization, JobSpec construction, worker execution, and
candidate creation. Not QC authority and not publication.

Status: `COMPLETE / PASS / BASELINED`

## Eligibility

Transfer requires an exact persisted `CompatibilityResult` bound to the same
exact:

```text
CharacterAssetVersion
MotionAssetVersion
BoneMappingVersion
RetargetPolicyVersion
```

Only `Ready` and `ReadyWithWarnings` may authorize Transfer.

```text
Ready                         → eligible; no warning acknowledgement
ReadyWithWarnings             → eligible only with explicit durable acknowledgement
MappingConfirmationRequired   → rejected before JobSpec / JobRun
Unsupported                   → rejected before JobSpec / JobRun
```

Acknowledgement is recorded on the Transfer `JobSpec` as
`compatibility_warnings_acknowledged`. It is not a transient GUI Boolean.

## Exact graph

Before a V1-5 JobSpec is persisted or enqueued:

```text
CompatibilityResult.character == CharacterAssetVersion
CompatibilityResult.motion    == MotionAssetVersion
CompatibilityResult.mapping   == Published BoneMappingVersion
CompatibilityResult.policy    == RetargetPolicyVersion
Mapping.target_character      == Character
Mapping.source_skeleton       == Motion.source_skeleton
```

No `latest`, `current`, or logical-only version resolution.

Historical JobSpecs without `compatibility_result_id` remain enqueueable for
V1-1/V1-2/V1-3 compatibility. A new V1-5 Transfer JobSpec must bind an exact
CompatibilityResult.

## Application API

```text
authorize_transfer(compatibility_result_id, warnings_acknowledged)
start_transfer(compatibility_result_id, warnings_acknowledged, existing_derived_variant_id, display_name)
ingest_worker_success_candidate(run_id, staged_path)
finalize_transfer(run_id, staged_path)
```

Workbench must not assemble arbitrary JobSpecs. Transfer eligibility is
Application/Product truth.

Production `finalize_transfer` acquires QC inspect and persistence reopen
evidence only through the sealed pinned production executable path. It does
not accept a caller-supplied inspector or reopener, and it does not honor
`RIGFORGE_BLENDER_EXECUTABLE` for publication-critical evidence.
Test injection exists only behind `#[cfg(test)]` / the non-default
`test-support` feature (`finalize_transfer_for_test`).

`start_transfer` loads the exact CompatibilityResult, validates eligibility
and the exact graph, resolves or creates the exact `DerivedVariant`, persists
it if new, binds `JobSpec.target_derived_variant_id`, and enqueues a JobRun.
`display_name` is used only when creating a new logical. Historical JobSpecs
without `target_derived_variant_id` remain enqueueable. V1-5 Transfer always
sets the target. Ingest fails if it is missing.

Ingest and finalize must not accept another caller-supplied logical ID. They
read the durable target from the JobSpec.

## Worker success

After `JobRun SUCCEEDED` and `WorkerResult.worker_success == true`:

```text
execution completed
staged candidate artifact exists
worker measurements exist
```

This does **not** mean structurally valid, QC PASS, production acceptable, or
published.

`ingest_worker_success_candidate` creates a Draft `DerivedVariantVersion` on
the JobSpec target `DerivedVariant` and promotes staged bytes to a durable
`PersistenceArtifact`. After staged-byte promotion succeeds, the candidate
Product graph is persisted in **one SQLite transaction**:

```text
reload / validate exact worker + JobSpec target
prove WorkerResult has not already produced a candidate
validate logical DerivedVariant
validate DerivedVariantVersion
validate PersistenceArtifact
atomically store version + artifact + location + logical.draft_version_id
```

Failure rolls back all database state. Promoted filesystem bytes may remain
orphaned/non-authoritative.

Public generic `put_validated(DerivedVariant)` cannot set, change, or clear
`draft_version_id`. Creation of a new logical with both pointers `None`
remains allowed. The draft pointer is owned by the candidate transaction.

Public generic `put_validated(DerivedVariantVersion)` cannot create a new
candidate version. New Draft `DerivedVariantVersion` records must enter
through the candidate transaction. Identical idempotent re-put of an
already stored payload may remain; authoritative field changes and new
candidate creation are rejected. QC / persistence-verification bindings
use crate-private trusted Catalog transactions.

It does not publish.

One successful `WorkerResult` creates at most one `DerivedVariantVersion`.
The cardinality guard is rechecked inside the candidate persistence
transaction. A second ingest for the same WorkerResult is rejected. A rerun
requires a new JobSpec, JobRun, and WorkerResult.

A previously Published version is never mutated.

Logical `DerivedVariant` identity is explicit. Unrelated transfers are not
merged by filename.
