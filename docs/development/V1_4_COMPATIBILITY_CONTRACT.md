# V1-4 Compatibility Contract

Multi-dimensional Compatibility preflight. Not Transfer, not QC, not
publication.

Status: `COMPLETE / PASS / BASELINED`

## Binding

A `CompatibilityResult` binds exact versions only:

```text
CharacterAssetVersion
MotionAssetVersion
BoneMappingVersion
RetargetPolicyVersion
```

Never `latest`, `current`, logical-only IDs, or filesystem paths. If any bound
version changes, the previous result remains historical and a new preflight
must run.

## Dimensions

| Dimension | Role | V1-4 typical |
| --- | --- | --- |
| `mapping_completeness` | required correspondence sufficient for the requested workflow | PASS / PASS_WITH_WARNINGS / FAIL / UNKNOWN |
| `structural_compatibility` | summaries exist, mapped joints refer to real joints, roots/hierarchy coherent | PASS / PASS_WITH_WARNINGS / FAIL |
| `method_eligibility` | selected Policy + current worker capability can execute this combination | PASS / FAIL |
| `motion_suitability` | source skeleton resolved, time provenance supported | PASS / FAIL |
| `result_acceptability` | post-transfer artistic/QC acceptability | **UNKNOWN** (pre-execution) |

```text
mapping_completeness
!= structural_compatibility
!= method_eligibility
!= motion_suitability
!= result_acceptability
```

Do not collapse Compatibility into `compatible: true/false`.

### Preflight-decision-critical

```text
mapping_completeness
structural_compatibility
method_eligibility
motion_suitability
```

`FAIL` on any of these → summary `Unsupported`.

### Post-execution-only

```text
result_acceptability
```

Pre-transfer this dimension is `UNKNOWN`. That must not make every workflow
unusable and is **not** treated as QC PASS or QC FAIL.

`result_acceptability FAIL` is still decision-critical if it ever appears.
V1-4 preflight must not invent it.

## Completeness

```text
all required joints mapped          → PASS
optional/helper joints unmapped     → PASS or PASS_WITH_WARNINGS
required joint missing              → FAIL
mapping unconfirmed / ambiguous     → UNKNOWN
```

100% bones mapped is not completeness.

Unmapped joints carry typed `UnmappedDisposition` (`blocking` / `optional` /
`helper`). Completeness uses that typed field, not `reason` text and not a
second name-based classification of an accepted Mapping.

`unmapped_source` entries must be `SkeletonSide::Source`. `unmapped_target`
entries must be `SkeletonSide::Target`. Wrong-side records fail Domain
validation and never reach Compatibility.

A Mapping that is not `Lifecycle::Published` cannot contribute
`Ready` / `ReadyWithWarnings`. Draft preflight is allowed for diagnostics:
`mapping_completeness` is `UNKNOWN`, which yields
`MappingConfirmationRequired` unless another decision-critical dimension is
already `FAIL` (`Unsupported`). `reviewed=true` is not acceptance authority;
explicit Product publish is.

## Method eligibility

Uses the accepted V1-3 capability boundary, backend-neutrally:

```text
proven rest-relative rotation-only Policy
explicit no IK
keep target rest scale
integral frame points for Blender execution
```

Do not record `bpy` operators in Product Compatibility. Do not launch retarget
execution to discover a static preflight fact.

Unsupported Policy token, IK requirement, or fractional-frame execution →
`FAIL` / summary `Unsupported`.

## Motion suitability

Facts from Motion metadata only. Foot sliding and artistic quality are not
V1-4 claims. Wrong Source Skeleton → `FAIL`.

## Summary derivation

One deterministic function. UI must not choose the summary.

```text
any decision-critical FAIL
        → Unsupported

mapping_completeness UNKNOWN
or unresolved confirmation required
        → MappingConfirmationRequired
        (unless already Unsupported)

no FAIL, but a preflight dimension is PASS_WITH_WARNINGS
or a non-completeness preflight dimension is UNKNOWN
        → ReadyWithWarnings

all preflight-decision-critical dimensions PASS
(result_acceptability may be UNKNOWN)
        → Ready
```

`Ready` / `ReadyWithWarnings` still cannot coexist with a decision-critical
FAIL. `CompatibilityResult::validate` requires `summary == derive_summary(...)`.
Untrusted JSON cannot choose an inconsistent summary. `from_preflight`
derives the summary; `new` still cannot produce an inconsistent validated
record.

## Persistence

Store `Validated<CompatibilityResult>` in the Catalog. Load by exact ID.
Retrieve the most recent result for an exact version quadruple only when all
four IDs match. Never substitute another version set.

Generic Catalog persistence (`put_validated`) graph-validates a
CompatibilityResult against the exact referenced Character, Motion, Mapping,
and Policy versions before insert. It does not resolve `latest`, `current`,
or logical-only IDs.

```text
Ready / ReadyWithWarnings
→ referenced BoneMappingVersion.lifecycle must be Published
→ Mapping.source_skeleton_ref_id must equal Motion.source_skeleton_ref_id

MappingConfirmationRequired
→ may reference a Draft Mapping for diagnostics

Unsupported
→ may persist a motion/mapping Source Skeleton mismatch as a diagnostic
```

A Domain-valid Ready record bound to a Draft Mapping is rejected at the
Catalog boundary. `reviewed=true` is not acceptance authority.

## Transfer boundary

Compatibility preflight must not:

```text
launch retarget execution
create WorkerResult
mark JobRun SUCCEEDED
create DerivedVariant
run Product QC
```

Skeleton inspection is allowed for evidence acquisition.
