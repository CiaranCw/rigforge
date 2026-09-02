# V1-5 Derived Variant Publication

Publication of an exact DerivedVariantVersion. Not worker success and not QC.

Status: `COMPLETE / PASS / BASELINED`

## Identity

```text
DerivedVariant                = logical result object
DerivedVariantVersion         = exact transfer result candidate/version
PersistenceArtifact           ≠ DerivedVariantVersion
PersistenceArtifactInstance   ≠ payload digest ≠ filesystem path
```

A regenerated artifact instance does not change Product identity. Publication
binds the exact verified instance through `PersistenceVerification`.

`.blend` and FBX paths are not Product identity.

## Artifact promotion

```text
successful WorkerResult
        ↓
locate exact staged candidate bytes
        ↓
verify staged digest
        ↓
copy/promote to durable local artifact storage
        ↓
rehash durable bytes
        ↓
PersistenceArtifact
```

Required:

```text
actual staged SHA == WorkerResult staged digest
durable persisted SHA == actual staged SHA
```

Mismatch: FAIL, no publication. File existence is not success.

## Persistence verification

An exact `PersistenceVerification` records:

```text
exact artifact ID
exact artifact instance ID
exact payload digest
exact DerivedVariantVersion
exact producer/backend
fresh_reopen outcome
structural_verification outcome
```

Fresh reopen uses a new process against the durable persisted bytes. A
successful staged reopen does not automatically prove a later copied
persistence instance.

```text
PersistenceVerification.structural_verification
  → is the persisted artifact structurally present/readable and consistent
    with the expected product candidate?

QcReport
  → does the candidate satisfy the V1 Product QC rules?
```

They may consume shared inspection evidence. They remain separate Product
meanings.

## Publication graph

`validate_publication_lineage` / `publish_derived_variant` remain Domain
authority. Publication additionally proves:

```text
JobSpec compatibility authorization → exact CompatibilityResult
CompatibilityResult summary → Ready or ReadyWithWarnings
CompatibilityResult exact Character/Motion/Mapping/Policy
  → same lineage as DerivedVariantVersion
ReadyWithWarnings → acknowledgement recorded
QcReport evaluated artifact/instance/digest
  → PersistenceArtifact and PersistenceVerification
```

A DerivedVariant cannot be published from `Unsupported`,
`MappingConfirmationRequired`, or a wrong/stale CompatibilityResult.

Application/Catalog also prove:

```text
JobRun.state == SUCCEEDED
JobRun.job_spec_id == DerivedVariantVersion.job_spec_id
JobRun.worker_result_id == DerivedVariantVersion.worker_result_id
```

Generic Catalog persistence must not permit an externally supplied Published
DerivedVariantVersion, forged QC PASS, or mismatched PersistenceVerification
to bypass this graph.

Public `put_validated` / `put_validated_pair` reject publication-critical
`QcReport` and `PersistenceVerification`. They also reject insert or update
of `DerivedVariant.published_version_id` and `DerivedVariant.draft_version_id`,
and they reject creation of a new `DerivedVariantVersion` candidate.
Only `persist_candidate_graph` may author a new candidate version, after
graph validation and WorkerResult cardinality checks. Only
`publish_derived_variant_transaction` may set the published pointer,
and only after the version is Published, `version.variant_id` equals the
logical id, `logical.draft_version_id` equals that version, and
`JobSpec.target_derived_variant_id` equals `DerivedVariantVersion.variant_id`.

Raw SQLite mutation is not a public Catalog API. `SqliteCatalog` does not
expose `in_transaction` or a `rusqlite::Transaction` to ordinary production
callers. Unvalidated payload insertion exists only behind `#[cfg(test)]`.
Public Catalog operations remain the validated Product/application path.

Publication also fail-closes if another `DerivedVariantVersion` in the
Catalog is already bound to the same `WorkerResult`. A legitimate rerun
with a new WorkerResult may create a new version of the same logical.

Catalog persistence of `PersistenceVerification` additionally requires:

```text
PersistenceArtifact.bound_derived_variant_version_id
  == PersistenceVerification.subject_derived_variant_version_id
PersistenceVerification.producer_id == PersistenceArtifact.producer_id
PersistenceArtifact.producer_id == DerivedVariantVersion.backend_id
```

## Final transaction

External work (worker, promotion, QC inspect, reopen) cannot be one SQLite
transaction. After every external evidence artifact exists and validates, one
database transaction:

```text
reload exact evidence
revalidate publication graph
freeze exact DerivedVariantVersion as Published
update/bind logical DerivedVariant published_version_id
```

Logical Published pointer and version publication are atomic.

## Failure lifecycles

```text
Worker FAIL
  → no Derived Variant publication
  → no false QC PASS

QC FAIL
  → Worker remains SUCCEEDED
  → candidate remains Draft / failed-QC
  → publication DENIED

Verification FAIL
  → Worker/QC history unchanged
  → publication DENIED

Publication graph mismatch
  → publication DENIED
```

Candidate/evidence records may remain for diagnosis. Orphaned filesystem
bytes are non-authoritative.

Evidence bindings (`QCReport`, `PersistenceArtifact`,
`PersistenceVerification`) are write-once on a Draft candidate. Published
remains immutable. A rerun creates a new JobSpec, JobRun, WorkerResult,
DerivedVariantVersion, and evidence.
