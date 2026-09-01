# Gate B — Independent Runtime Foundation Audit

Audit date: 2026-09-01  
Auditor role: Independent Runtime Foundation Auditor  
Model: GPT-5.6 Sol High

## 1. PRE-STATE

```text
repository:   F:\NewResearch\rigforge
branch:       main
HEAD:         18c76c521c86c9d31382ece5c03d431c38b4df08
origin/main:  18c76c521c86c9d31382ece5c03d431c38b4df08
message:      feat: complete V1-2 local application foundation
```

The worktree contained the uncommitted V1-3 candidate. Changes outside the
review ZIP manifest (`.gitignore`, `Cargo.lock`, architecture/roadmap status
files) were inspected and are related to adding the V1-3 crate and lifecycle;
no unrelated mutation was found.

Repository status files still said
`V1-3 IMPLEMENTATION_IN_WORKTREE / FOCUSED_REVIEW_PENDING` and
`Gate B NOT STARTED`. The explicit Gate instruction supplied for this audit
authorized the independent Gate without modifying those candidate files.

## 2. GATE B SCOPE

Audited the combined Runtime Foundation:

```text
V1-1 accepted Product Domain
        ↓
V1-2 SQLite Catalog + orchestration + egui shell boundary
        ↓
V1-3 exact dispatch projection + pinned isolated Blender worker
```

The audit did not reopen Rust, SQLite, egui/eframe, or the accepted
Blender-with-guards decision. It did not design V1-4, modify production
source, run security/adversarial campaigns, or redownload assets.

## 3. PACKAGE / SOURCE INTEGRITY

Input review package:

```text
path:            F:\NewResearch\rigforge_v1_3_rev1_review.zip
ZIP SHA-256:     a3e4a6fed75c3efed8ee698422298e8b7869736327b4a2cbc80db0c0e0169f4c
manifest SHA-256:9a4a04d88a5e454d650087f0cfa909cd862233dedc8aa45f39554f14b78023e0
entry count:     52
manifest-bound:  51
```

All 51 manifest entries matched their ZIP hashes. All 43 manifest-bound
entries that also exist in the repository matched the worktree byte-for-byte;
the other eight are package-only review evidence files. The package identity
matches the expected identity.

The package is a baseline-relative review surface, not a standalone full
repository: `Cargo.lock` and four related lifecycle/ignore files are outside
its manifest. Those files were inspected directly from the worktree.

## 4. TESTS RERUN

The literal command initially could not run because `cargo` was not on the
PowerShell PATH. The same command was then run with the repository's pinned
Rust 1.98.0 toolchain explicitly placed in the environment:

```text
cargo test --offline --color=never
```

Independent result:

```text
173 PASS
0 FAIL
0 ignored
```

This reran:

- all Domain, Catalog, orchestration, Workbench, fake-adapter, and doc tests;
- the real pinned Blender 5.2.1/build `9e2066aef7ef` check;
- archive SHA-256 verification for the already-local Blender archive;
- two clean production-runtime Knight_Male + UAL2 Walk_Carry_Loop runs;
- a fresh second Blender process for reopen on each successful run;
- the ordinary missing-source fail-closed path.

The frozen source hashes were independently asserted by the real test:

```text
Knight_Male:
fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f

UAL2_Standard:
d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d
```

## 5. RUNTIME FOUNDATION VERDICT

The authority boundary, exact Catalog graph resolution, process isolation,
pin enforcement, successful-envelope checks, fresh reopen, and frozen
real-asset path are materially implemented.

Gate B is not a PASS candidate because four open MAJOR defects remain:

1. one WorkerResult can authorize multiple distinct attempts;
2. a rejected failed terminal outcome can leave a run `RUNNING`;
3. rational frame provenance can be silently changed at dispatch;
4. root scale can violate `KeepTargetRestScale` while worker success remains
   possible.

## 6. FINDINGS

Full finding records, reproductions, violated contracts, impacts, and
required corrections are in
`docs/development/audits/GATE_B_FINDINGS.md`.

```text
OPEN MAJOR:       4
OPEN MINOR:       2
OPEN OBSERVATION: 3
```

Blocking:

- `GATE-B-MAJOR-001` — WorkerResult lacks JobRun-attempt binding.
- `GATE-B-MAJOR-002` — rejected failed outcomes can remain RUNNING.
- `GATE-B-MAJOR-003` — rational frame points lose their denominator.
- `GATE-B-MAJOR-004` — root scale can violate exact Policy.

Non-blocking:

- `GATE-B-MINOR-001` — completed attempts remain in adapter memory.
- `GATE-B-MINOR-002` — Unicode stderr truncation can panic.
- `GATE-B-OBS-001` — acknowledged spawn-to-RUNNING crash window.
- `GATE-B-OBS-002` — worker script is bound to the development source tree.
- `GATE-B-OBS-003` — packaged app-test subtotal is off by two.

## 7. PRODUCT AUTHORITY CHECK

PASS for the authority split:

- Product IDs are typed UUIDv7 values, not SQLite row IDs, paths, digests,
  Blender datablocks, or `.blend` files.
- SQLite row IDs and location overlays remain implementation evidence.
- `WorkerDispatchRequest` is an in-memory projection, not Product identity.
- staged `.blend` bytes remain non-authoritative and unpublished.
- egui does not own Product invariants or call Blender.

No SQLite/GUI/Blender/filesystem authority inversion was found.

## 8. CATALOG / PERSISTENCE CHECK

PASS with no Catalog blocker found:

- empty-to-v1 migration is transactional;
- current schema verifies required tables, indexes, and migration row;
- future DB and Domain schema versions fail closed;
- loads re-enter Domain validation;
- published/frozen versions are immutable and artifact instances append;
- exact version loading and logical-object/version separation are preserved;
- source location overlays do not change Product identity;
- JobSpec+JobRun and WorkerResult+success transitions are transactional;
- real-file close/reopen tests pass.

Direct corruption beyond deterministic missing/wrong schema objects was not
treated as an enterprise database requirement.

## 9. ORCHESTRATION CHECK

The normal adapter-driven path preserves immutable JobSpec intent, exact
graph validation, orchestrator-owned attempt IDs, guarded transitions,
dispatch failure, and successful/failed WorkerResult transactions.

However, `complete_success` remains independently callable and validates only
JobSpec/success, so result evidence is not attempt-specific
(`GATE-B-MAJOR-001`). The failed-outcome application branch can also reject
its WorkerResult without durably failing the run
(`GATE-B-MAJOR-002`).

No `latest` or `current` resolution was found at dispatch.

## 10. WORKER / PROCESS CHECK

Verified in source and tests:

```text
Blender 5.2.1 LTS
build 9e2066aef7ef
--background
--factory-startup
--disable-autoexec
--python-exit-code 1
job-specific TEMP/TMP/TMPDIR
isolated BLENDER_USER_CONFIG/SCRIPTS/DATAFILES
fresh execute process
fresh reopen process
```

The production constructor seals pin verification, source verification,
fresh reopen, and script selection. Test-only guard bypasses are unavailable
from `BlenderWorker::production()`.

Completed attempt bookkeeping is not removed
(`GATE-B-MINOR-001`). Unicode stderr truncation can panic
(`GATE-B-MINOR-002`).

## 11. POLICY / MAPPING CHECK

The adapter projects exact Mapping entries and does not Auto-Map. The
structured proven Policy is recognized before launch, including
rotation-only non-root channels, rest-relative transfer, quaternion
normalization/continuity, every-source-frame bake, target-rest scale, and
ExplicitNoIK. Unknown Policy variants fail at Domain ingress and unsupported
projections fail before Blender mutation.

Two execution gaps are blocking:

- rational frame point denominators are lost
  (`GATE-B-MAJOR-003`);
- root matrix assignment can import source scale despite
  `KeepTargetRestScale` (`GATE-B-MAJOR-004`).

No hidden corrective IK or Auto-Mapping was found.

## 12. WORKERRESULT / REOPEN CHECK

The production adapter checks process/envelope agreement, JobSpec ID,
attempt ID, backend kind/version/build, adapter version, staged-file
existence, source digests before launch and in the script, and staged digest
after reopen. File existence alone cannot produce success. Failure evidence
does not publish a Derived Variant.

Fresh reopen is a real second process and cannot be skipped on the production
API. Reopen launch, envelope, semantic, or staged-artifact failure becomes a
failed outcome.

Durable WorkerResult-to-attempt integrity is incomplete
(`GATE-B-MAJOR-001`).

## 13. REAL-ASSET CHECK

Independent rerun PASS for the narrow claimed evidence:

```text
production V1-3 runtime path: PASS
Knight_Male source hash:      PASS
UAL2 Walk_Carry_Loop hash:    PASS
pinned Blender/build:         PASS
two clean runs:               PASS
fresh reopen:                 PASS
rotation-only audit:          PASS
quaternion audit:             PASS
loop closure:                 PASS
missing-source failure:       PASS
```

This evidence is limited to the frozen humanoid pair and proven Policy. It is
not generalized to all humanoids or non-humanoids.

## 14. GENERALITY / ENGINE / FORMAT / DCC CHECK

The Product/Domain/Catalog contracts remain engine-, format-, and
DCC-neutral. FBX paths, Blender action/datablock names, `.blend` output, and
Blender Python stay behind the execution adapter. A future worker can
implement the same Product intent without adopting Blender Product types.

Operational support is currently one Blender/FBX-backed path. That is the
accepted V1 direction, not evidence of universal format or DCC coverage.

## 15. DEFERRED HARDENING

Non-blocking future work:

- supervisor/reconciliation for the spawn-to-RUNNING crash window;
- installer/package worker-script binding and integrity;
- worker pool, timeout, cancellation, retry, and restart reattachment;
- V1-8 legal/distribution qualification;
- broader humanoid/non-humanoid and source-format coverage;
- advanced QC, Preview, export, and release qualification.

The two MINOR findings should be assigned to the surgical correction or V1-8
with explicit ownership. None authorizes starting V1-4.

## 16. COVERAGE MATRIX

See `docs/development/audits/GATE_B_COVERAGE_MATRIX.md`.

All requested dimensions A–O were covered. Dimensions D, E/I, I, L, and M
produced findings or deferred observations.

## 17. AUDIT PACKAGE

Created at:

```text
F:\NewResearch\rigforge_gate_b_review.zip
```

The final chat report records the computed ZIP SHA-256, manifest SHA-256, and
entry count. The ZIP includes these audit reports, a baseline-relative
candidate source snapshot, test/integrity evidence, and a SHA-256 manifest.
It excludes Blender binaries and real FBX assets.

## 18. GIT STATE

```text
candidate production source modified: NO
audit artifacts added:               YES
commit:                              NO
push:                                NO
```

Tests wrote only ignored build output. No candidate source was repaired.

## 19. RESULT

```text
GATE_B_FINDINGS
```

## 20. NEXT STEP

```text
Main Agent surgical Gate B correction.
```

Do not start V1-4.
