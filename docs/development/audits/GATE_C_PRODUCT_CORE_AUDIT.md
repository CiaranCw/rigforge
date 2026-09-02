# Gate C — Independent Product Core E2E Audit

Audit date: 2026-09-02

Auditor: new independent GPT-5.6 Sol High agent

Result: `GATE_C_PASS_CANDIDATE`

## 1. PRE-STATE

Verified before audit:

```text
repository: F:\NewResearch\rigforge
branch: main
HEAD: d633932fbc4a33a2c86b8f16e84992cb4ffb2ee5
origin/main: d633932fbc4a33a2c86b8f16e84992cb4ffb2ee5
message: feat: complete V1-4 mapping compatibility workflow
```

The worktree contained the uncommitted V1-5 candidate and its focused
corrections. The changed files were confined to V1-5 lifecycle/status
documentation, Domain transfer/QC/artifact bindings, Application catalog and
workflow code, the pinned worker, Workbench transfer state, and their tests.
No unrelated mutation requiring the audit to stop was found.

Lifecycle used for this audit:

```text
V1-4: COMPLETE / PASS / BASELINED
V1-5: GATE_C_READY_CANDIDATE
Gate C: INDEPENDENT_AUDIT_ACTIVE
V1-6: NOT STARTED
```

## 2. GATE C SCOPE

The audit reconstructed and challenged the combined V1-4 + V1-5 transaction:

```text
exact Character + Motion + Published Mapping + Policy + Compatibility
  -> explicit Transfer authorization
  -> exact JobSpec and JobRun
  -> pinned isolated Worker and exact WorkerResult
  -> Draft DerivedVariantVersion
  -> durable PersistenceArtifact
  -> non-mutating Product QC
  -> PersistenceVerification
  -> publication graph reload
  -> atomic Published version + logical pointer
```

Gate A and Gate B were not reopened. Gate B runtime semantics were checked
only where V1-5 consumes them. Preview, export, release qualification, engine
integration, and additional DCC backends were excluded.

## 3. PACKAGE / SOURCE INTEGRITY

The final V1-5 Rev4 focused package was independently hashed and every
manifest-bound entry was streamed and rehashed:

```text
path: F:\NewResearch\rigforge_v1_5_rev4_review.zip
ZIP SHA-256: 903517b3955201921e8ecf8914392bbbcce222bfecdee2752bc3b7f91dd79b96
manifest SHA-256: 14815456f983755471e23c0fddc7193c6180fc9050e2f57cd172f0546b6b4918
entry count: 26
manifest-bound: 25
result: 25 / 25 MATCH
```

All 15 repository-source entries in that narrow package matched the current
worktree byte-for-byte. The package was used as focused correction evidence;
the complete worktree remained the audit authority.

The frozen asset hashes independently matched:

```text
Knight_Male.fbx:
fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f

UAL2_Standard.fbx:
d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d
```

The installed execution backend independently reported:

```text
Blender 5.2.1 LTS
build hash: 9e2066aef7ef
```

## 4. TESTS RERUN

The clean shell did not initially expose Cargo on `PATH`. The accepted local
wrapper was therefore used; it configures rustc/cargo 1.98.0, the MSVC
environment, the isolated Cargo cache, and executes:

```text
cargo test --offline --color=never
```

The final unfiltered independent rerun produced:

```text
rigforge_app:             210 PASS
rigforge_blender_worker:   53 PASS
rigforge_domain:          124 PASS
rigforge_workbench:        19 PASS
TOTAL:                    406 PASS
FAIL:                       0
IGNORED:                    0
```

The run included all three real `v1_5_transfer` tests. The
`two_clean_frozen_pair_product_core_runs` test itself executed two complete
new worker attempts and publications.

Production-only Cargo feature trees were also resolved with dev edges
excluded. Workbench and blender-worker each resolved
`rigforge_app feature "default"`; neither resolved `test-support`.

## 5. PRODUCT CORE VERDICT

PASS candidate. No open MAJOR finding exists.

The implementation materially preserves the required separation:

```text
Compatibility readiness
!= Transfer authorization
!= Worker success
!= QC PASS
!= Persistence verification
!= publication
```

The exact persisted graph is revalidated at enqueue, candidate persistence,
QC/verification binding, and publication. Generic Catalog persistence cannot
author the publication-critical records or pointers. Final publication is one
SQLite transaction.

## 6. FINDINGS

```text
OPEN MAJOR: 0
OPEN MINOR: 0
OBSERVATION: 1
```

`GATE-C-OBS-001` records that Workbench state methods call Application
operations correctly, while the current native entry point and rendered
controls remain an unwired shell. This cannot assert Product authorization,
QC PASS, or publication and does not invalidate Product Core.

Detailed inventory:
[GATE_C_FINDINGS.md](GATE_C_FINDINGS.md).

## 7. MAPPING / COMPATIBILITY AUTHORITY

PASS.

- Automatic proposals remain non-authoritative.
- `BoneMappingVersion::publish` requires reviewed acceptance and rejects
  `automatic_candidate`.
- Accepted review kind is derived from durable workflow history.
- Published Mapping versions are immutable.
- Duplicate source or target joint keys fail one-to-one V1 validation.
- `UnmappedDisposition` is typed authority; reason text is explanatory.
- `CompatibilitySummary` is derived from typed dimensions.
- Ready and ReadyWithWarnings persistence requires the exact Published Mapping.
- V1-5 calls the existing graph validation and introduces no weaker
  Compatibility interpretation.

## 8. TRANSFER AUTHORIZATION

PASS.

`start_transfer` loads the exact persisted CompatibilityResult and exact
Character, Motion, Mapping, and Policy versions. It rejects
MappingConfirmationRequired and Unsupported before JobSpec/JobRun creation.
ReadyWithWarnings requires acknowledgement, and the durable JobSpec records
both `compatibility_result_id` and
`compatibility_warnings_acknowledged`.

No caller supplies substitute Character, Motion, Mapping, or Policy IDs to
`start_transfer`; they are loaded from the exact CompatibilityResult.

## 9. JOB / WORKER CORRELATION

PASS.

New V1-5 JobSpecs bind exact `compatibility_result_id` and
`target_derived_variant_id`. Candidate ingestion reloads the durable JobSpec
target and accepts no caller-supplied substitute logical ID.

V1-3 semantics remain intact: launch and collect are separate, attempt IDs are
orchestrator-owned, WorkerResult correlation checks JobSpec/attempt/execution
reference, one process is used per attempt, source digests are verified,
fractional frames fail before Blender mutation, KeepTargetRestScale remains
checked, and fresh execute-output reopen remains mandatory.

Historical JobSpecs may omit V1-5 fields for historical execution, but V1-5
candidate ingestion and publication fail closed when target or compatibility
authorization is missing.

## 10. CANDIDATE / ARTIFACT AUTHORITY

PASS.

Candidate ingestion requires JobRun SUCCEEDED, a matching successful
WorkerResult, the exact durable JobSpec target, staged bytes, and a staged
digest present in WorkerResult.

Promotion verifies:

```text
actual staged SHA == WorkerResult staged digest
durable copied SHA == actual staged SHA
```

The candidate version, PersistenceArtifact, exact artifact location evidence,
and logical draft pointer are stored in one trusted transaction. A failed
transaction leaves promoted bytes non-authoritative.

WorkerResult-to-version cardinality is checked before and inside the
transaction. Generic `put_validated` and `put_validated_pair` cannot create a
new DerivedVariantVersion or change logical draft/published pointers.

## 11. QC AUTHORITY

PASS.

Production QC loads the exact candidate-bound durable PersistenceArtifact,
uses the sealed pinned inspect path, hashes before and after inspection, and
rejects process failure, non-SUCCESS envelope status, a saved-scene claim, or
digest change.

The Product evaluator emits all seven typed checks for
`rigforge-v1-structural-qc/1`. Verdict is derived: any FAIL/MISSING becomes
Fail, and omission of a required check fails validation.

The QcReport binds exact candidate, Policy, WorkerResult, artifact ID,
artifact instance, and payload digest. Generic Catalog persistence rejects
QcReport. Trusted report persistence and candidate binding are one
crate-private transaction with write-once semantics.

Production finalization accepts no inspector, executable, or reopener
parameter and ignores `RIGFORGE_BLENDER_EXECUTABLE` for publication-critical
inspection.

## 12. PERSISTENCE VERIFICATION

PASS.

Verification is separate from QC and binds exact artifact ID, instance,
digest, candidate, and producer. The production path launches a new pinned
Blender process against the durable copied bytes. It checks process/envelope
success, target presence, baked action presence, and digest stability.

Generic Catalog persistence rejects PersistenceVerification. The trusted
binding is crate-private, graph-validated, and write-once.

## 13. PUBLICATION GRAPH / ATOMICITY

PASS.

Publication reloads Character, Motion, SourceSkeleton, Mapping, Policy,
JobSpec, CompatibilityResult, JobRun, WorkerResult, QcReport,
PersistenceArtifact, PersistenceVerification, DerivedVariantVersion, and the
logical DerivedVariant.

It requires exact ID agreement, JobRun SUCCEEDED, WorkerResult success,
eligible Compatibility, warning acknowledgement when required, QC PASS, fresh
reopen PASS, structural verification PASS, exact target logical, and exact
draft pointer.

The final transaction freezes the version as Published and binds the logical
published pointer. Any error rolls back both writes; neither half-state can
commit.

## 14. PUBLIC CATALOG SURFACE

PASS.

- `SqliteCatalog.conn` is private.
- `SqliteCatalog::in_transaction` is private.
- No public API returns `rusqlite::Connection` or `Transaction`.
- Unvalidated insertion and internal rowid helpers are `#[cfg(test)]
  pub(crate)`.
- Candidate, trusted QC, and trusted verification persistence are
  `pub(crate)`.
- Generic puts reject QcReport, PersistenceVerification, new or modified
  DerivedVariantVersion records, and logical pointer changes.
- The public publication transaction performs the complete persisted graph
  validation and atomic update; it is not a raw mutation callback.
- `Application::catalog_mut` exposes only the guarded Catalog API, not SQLite.

## 15. WORKBENCH CHECK

PASS for Product authority; see `GATE-C-OBS-001`.

Workbench transfer methods request authorization and start transfer through
Application, reset acknowledgement when a new CompatibilityResult is applied,
and display Product outcomes rather than inventing QC or publication state.
Preview remains an empty V1-6 boundary with no viewer or payload selection.

The current native executable is still a presentation shell rather than a
wired end-user transaction surface. It cannot bypass Product authority.

## 16. REAL PRODUCT-CORE E2E

PASS for the frozen pair only.

Independent full-suite execution confirmed:

```text
Compatibility: ReadyWithWarnings
without acknowledgement: denied
with acknowledgement: authorized and durably recorded
real production Worker: YES
JobRun: SUCCEEDED
WorkerResult: exact attempt-bound / success
durable promotion: PASS
Product QC: PASS (7 / 7 required checks)
fresh durable reopen: PASS
structural verification: PASS
publication: PASS
```

The two-clean-run checkpoint produced distinct JobSpecs, JobRuns,
WorkerResults, DerivedVariantVersions, PersistenceArtifacts, QcReports, and
PersistenceVerifications while explicitly reusing one logical DerivedVariant.
The first Published version remained immutable. QC inspection left its durable
artifact digest unchanged.

This is evidence for the Knight_Male + UAL2_Standard path, not universal
humanoid or general asset support.

## 17. PRODUCT / GENERALITY / INDEPENDENCE

PASS with calibrated scope.

- Product requirements: Mapping, Compatibility, Transfer authorization, QC,
  persistence evidence, and Derived Variant identity are Product-owned.
- Generality: Domain identity and graph records do not require humanoid slots;
  broader real non-humanoid execution evidence remains deferred.
- Engine independence: no game engine is required.
- Format independence: FBX and `.blend` are source/output representations, not
  Product identity.
- DCC independence: Blender is a pinned hidden operational backend; durable
  Product contracts remain backend-neutral.
- Real-asset evidence: the frozen pair ran through real pinned execution and
  publication.

## 18. DEFERRED HARDENING

Accepted future work, not Gate C blockers:

- broader Character/Motion and real non-humanoid coverage;
- release packaging and worker-script/distribution integrity;
- replacing the development-machine absolute Blender install path;
- crash recovery, reattachment, pooling, and scheduling;
- timeout/resource campaigns and upgrade rehearsal;
- end-user Workbench event wiring;
- Preview, which remains V1-6 and was not started.

## 19. COVERAGE MATRIX

The dimension-by-dimension evidence map is:
[GATE_C_COVERAGE_MATRIX.md](GATE_C_COVERAGE_MATRIX.md).

All dimensions A through W are covered. No dimension is covered solely by the
Rev4 package report.

## 20. AUDIT PACKAGE

The immutable review package is created after these reports and its final
hashes are recorded in the audit handoff. The package contains these audit
records, contracts, relevant candidate source/tests, independent execution
evidence, and a self-excluding SHA-256 manifest.

## 21. GIT STATE

```text
candidate production modified: NO
audit records added: YES
commit: NO
push: NO
tag: NO
PR: NO
```

## 22. RESULT

```text
GATE_C_PASS_CANDIDATE
```

## 23. NEXT STEP

```text
Gate C acceptance + V1-5 baseline closeout.
```

Do not start V1-6.
