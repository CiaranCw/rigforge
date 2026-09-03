# V1-5 QC Contract

Product-owned structural QC. Not worker success, not persistence verification,
and not publication.

Status: `COMPLETE / PASS / BASELINED`

## Boundary

```text
PersistenceArtifact
        ↓
read-only pinned inspect (production path)
        ↓
structured inspection evidence
        ↓
Product QcEvaluator
        ↓
QcReport
```

QC used for publication evaluates the exact durable PersistenceArtifact
instance that will be published. Worker measurements may be supporting
evidence. They are not QC authority.

If Blender is used to inspect a persisted `.blend`, it is hidden behind the
configured production inspect path (`inspect_qc`). The production
Application QC API does not accept a caller-implementable inspector port.
QC must not:

```text
repair the artifact
re-key animation
fix transforms
silently save the scene
```

```text
digest before QC inspection
==
digest after QC inspection
```

When the inspector is the pinned Blender worker, Product QC evidence is
accepted only if **both** are true:

```text
Blender process exit SUCCESS
inspect_qc envelope status SUCCESS (case-insensitive)
```

Otherwise the inspector fails closed. A non-zero process exit or a FAIL /
unknown envelope status cannot produce QC PASS, even if boolean check fields
look like a pass. Typed Product checks remain evaluated only after that gate.

## Authority

Public Catalog `put_validated` / `put_validated_pair` cannot persist a
publication-critical `QcReport`. Application QC (`evaluate_and_bind_qc`)
runs the sealed pinned inspect path, then persists the report and binds
`qc_report_id` through an internal trusted Catalog transaction. Ordinary
downstream code cannot supply a fake inspector, and cannot replace the
production inspect executable through `RIGFORGE_BLENDER_EXECUTABLE`, a
public setter, constructor, or caller parameter. Raw Catalog SQL mutation
is not a public Application API.

Worker execute/collect also does **not** honor `RIGFORGE_BLENDER_EXECUTABLE`
as an arbitrary unpinned production fallback (V1-8 runtime layout). The
historical V1-3 note that execute/collect might still honor that variable
is superseded. Publication-critical inspect/reopen must not honor it.
Release installer/package integrity remains later distribution / POST-V1
work; V1-8 implemented relocatable runtime worker-package integrity.

In-memory test inspectors exist only behind `#[cfg(test)]` /
`test-support`. They are not a production Application API and cannot
authorize publication in ordinary builds.

## Creation cycle

```text
create DerivedVariantVersion Draft without QC
        ↓
run QC
        ↓
persist trusted QcReport + bind qc_report_id on the exact candidate
        (one Catalog transaction)
        ↓
publication requires QC present
```

`qc_report_id` is optional on Draft. Placeholder UUIDs are forbidden.
Published `DerivedVariantVersion` requires a QC report. QC binding is
write-once. A different QC attempt creates new evidence / a new candidate
version.

## Artifact identity

Publication-critical QC records:

```text
evaluated_persistence_artifact_id
evaluated_persistence_artifact_instance_id
evaluated_payload_digest
```

Publication requires those to equal the PersistenceArtifact and
PersistenceVerification instance/digest.

## Rule set

```text
rigforge-v1-structural-qc/1
```

Backend/DCC implementation names do not define Product QC semantics.

## Typed checks

Required checks:

```text
finite_transforms
required_mapped_joints_present
expected_baked_animation_present
time_range_duration_sane
gross_scale_transform_sane
root_trajectory_sane
persistence_digest_stable
```

Do not add foot-sliding artistic thresholds, contact quality scores, or
animation beauty scores.

## Verdict derivation

`QcReport.verdict` is not caller authority.

```text
all required checks PASS → QcVerdict::Pass
any required check FAIL/MISSING → QcVerdict::Fail
missing required check → fail closed (does not validate)
```

`QcReport::validate()` requires the stored verdict to equal
`derive_qc_verdict(checks)`. Untrusted JSON cannot store `verdict = pass`
with incomplete or failing checks.

## Subject binding

A QcReport retains exact:

```text
DerivedVariantVersion
RetargetPolicyVersion
WorkerResult
PersistenceArtifact instance/digest
QC rule-set version
```

A report from another WorkerResult, DerivedVariantVersion, Policy, or
PersistenceArtifact is rejected.
