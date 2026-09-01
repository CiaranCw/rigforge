# Gate B — Independent Focused Finding Closure

Closure date: 2026-09-01  
Auditor role: Independent Gate B Closure Auditor  
Model: GPT-5.6 Sol High

## 1. PRE-STATE

```text
repository:   F:\NewResearch\rigforge
branch:       main
HEAD:         18c76c521c86c9d31382ece5c03d431c38b4df08
origin/main:  18c76c521c86c9d31382ece5c03d431c38b4df08
message:      feat: complete V1-2 local application foundation
```

The worktree contains the uncommitted V1-3 candidate, Gate B correction, and
Gate B audit artifacts. No reset, stash, clean, commit, push, tag, or PR was
performed.

## 2. CLOSURE SCOPE

```text
Existing Gate B focused finding closure
NOT a new independent Gate
```

Only the original four MAJOR and two MINOR findings were reviewed, together
with narrow regression checks directly affected by their corrections. The
original A–O audit was not restarted. This was not a V1-4 or release review.

## 3. PACKAGE / AUDIT INTEGRITY

The three original audit reports are unchanged:

```text
GATE_B_RUNTIME_FOUNDATION_AUDIT.md
1ed852f87dd6444691b440426552e86ced371b8e3c9de86227cd5e62cec3fa84

GATE_B_FINDINGS.md
1164dca1dc584763b6a6df98aa50f67ff8dbd9460023ec58dce817c913fd078e

GATE_B_COVERAGE_MATRIX.md
786bd7641dd109d00a3d24860c3e8f62f41764ac429afe6cfccad99cc38a32aa
```

Correction package:

```text
path:            F:\NewResearch\rigforge_gate_b_correction_review.zip
ZIP SHA-256:     7960f6a37914ecec58622336b74410c79e2101e288fb140411e813fb26861d1c
manifest SHA-256:3bf2e0723bb0f2c07090878ac4c5718a37f5b161efa9e2d20440748cb466947c
entry count:     94
manifest-bound:  93
```

All 93 manifest bindings validate. Every packaged source file exists in and
matches the current worktree byte-for-byte.

## 4. TESTS RERUN

Independent command:

```text
cargo test --offline --color=never
```

Result:

```text
domain:          88 PASS
app:             69 PASS
blender-worker:  43 PASS
workbench:        3 PASS
TOTAL:          203 PASS
FAIL:             0
```

The count matches the correction report exactly. The run included the real
pinned Blender checks and four procedural root-scale tests.

## 5. FINDING CLOSURE

```text
GATE-B-MAJOR-001: CLOSED
GATE-B-MAJOR-002: CLOSED
GATE-B-MAJOR-003: CLOSED
GATE-B-MAJOR-004: OPEN
GATE-B-MINOR-001: CLOSED
GATE-B-MINOR-002: CLOSED
```

### GATE-B-MAJOR-004 remaining violation

**Reproduction:** deserialize or produce an otherwise-successful
`ReopenEnvelope` without `root_scale_audit`. The field is optional with a
Serde default. Collection checks it only inside `if let Some(scale)`, so
`None` bypasses the root-scale verification and the attempt can still return
successful `WorkerResult`.

**Affected files:**

- `blender-worker/src/envelope.rs`
- `blender-worker/src/adapter.rs`
- `blender-worker/tests/adapter_fake.rs`

**Remaining contract violation:** the execution worker now enforces mapped
root scale correctly, but fresh-process reopen does not *require*
`root_scale_audit == PASS`. Missing reopen evidence is accepted instead of
failing closed.

**Required correction:** make the reopen root-scale result required for a
successful envelope, or explicitly fail collection when it is absent or not
`PASS`. Add an adapter test in which reopen reports `SUCCESS` but omits
`root_scale_audit`; the JobRun must become `FAILED` with no successful result.

## 6. ATTEMPT / WORKERRESULT CHECK

`ExecutionCorrelation` now requires non-empty `attempt_id` and
`worker_execution_ref`, is serialized as a required private WorkerResult
field, and exposes read-only accessors. Successful Catalog completion checks
JobSpec, attempt, execution reference, and `worker_success == true` in one
transaction. WorkerResult ID reuse across different JobRuns is rejected.

The same-JobSpec/two-attempt, wrong-attempt, wrong-execution-reference,
matching-result, result-reuse, and real-Blender correlation tests pass.
The stale-result success from MAJOR-001 is no longer reproducible through
ordinary runtime APIs.

## 7. FAILURE-DURABILITY CHECK

Rejected failed evidence is not persisted. The transaction durably moves the
JobRun to `FAILED`, leaves `worker_result_id` empty, and records the rejection
diagnostic. JobSpec mismatch, success-valued failure evidence, attempt
mismatch, and execution-reference mismatch all follow this fail-closed path.
MAJOR-002 is closed.

## 8. RATIONAL-TIME CHECK

Domain rational provenance retains `value_num` and `value_den`.
`start <= end` uses exact `i128` cross multiplication, including
`3/2 < 2/1`. V1-3 rejects non-unit-denominator frame points while assembling
the resolved dispatch request, before either FakeWorker or Blender receives
the request. Integral-frame behavior is unchanged. No truncation, rounding,
floor, or ceil path remains. MAJOR-003 is closed.

## 9. ROOT-SCALE POLICY CHECK

The Python execution fix separates root translation/rotation from scale,
forces rest-equivalent target scale, authors no scale FCurves, audits the
mapped root, and refuses execution success when either required scale
measurement is not `PASS`.

The procedural fixture genuinely animates source root scale from `1.0` to
`3.0`; transfer, audit, violation, and reopen tests pass with Blender 5.2.1.
MAJOR-004 nevertheless remains open because adapter collection accepts a
missing fresh-reopen `root_scale_audit` field.

## 10. RESOURCE / UNICODE CHECK

Collection removes the launched attempt from adapter bookkeeping while
retaining the owned attempt data needed to read process and envelope
evidence. Both successful and failed terminal paths return with zero retained
attempts.

Diagnostic truncation advances to a valid UTF-8 character boundary before
slicing. Unicode helper/collection and ASCII-preservation tests pass.
Both MINOR findings are closed.

## 11. REGRESSION CHECK

Narrow source and test checks confirm:

- SQLite remains local Catalog implementation, not Product authority.
- Product IDs remain UUIDv7 and distinct from DB rows, paths, digests, and
  Blender identity.
- JobSpec remains immutable and exact Catalog graph resolution is preserved.
- Blender remains pinned to 5.2.1 LTS/build `9e2066aef7ef`.
- One isolated process per attempt and fresh second-process reopen remain.
- Mapping remains exact with no Auto-Mapping; Policy remains structured.
- Worker success still does not imply QC PASS or publication.
- DerivedVariant publication and Preview are not implemented.
- V1-4 has not started.

## 12. REAL BLENDER CHECK

Independent rerun verified:

```text
pinned Blender/build:                   PASS
frozen source hashes:                   PASS
two clean frozen-pair executions:       PASS
fresh-process reopen:                   PASS
attempt-bound WorkerResult:             PASS
missing-source fail closed:             PASS
animated-root-scale fixture:            PASS
mapped-root scale audit:                PASS
root-scale violation rejects success:   PASS
fixture reopen preserves rest scale:    PASS
```

Frozen hashes remain:

```text
Knight_Male:
fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f

UAL2_Standard:
d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d
```

Passing current production-script evidence does not remove the missing-field
acceptance defect in the adapter contract.

## 13. DEFERRED OBSERVATIONS

The acknowledged spawn-to-RUNNING crash window and release worker-script
binding/installer integrity remain deferred hardening concerns. The focused
correction introduced no evidence requiring either to be escalated.

## 14. CLOSURE PACKAGE

Created at:

```text
F:\NewResearch\rigforge_gate_b_closure_review.zip
```

The final closure response records the computed ZIP SHA-256, manifest
SHA-256, entry count, and manifest-bound count. The package includes focused
and original reports, correction evidence, candidate source, independent
rerun evidence, and a manifest. Blender binaries, FBX files, `.blend`
outputs, `target/`, dependency caches, and prior ZIP archives are excluded.

## 15. GIT STATE

```text
candidate production source modified: NO
focused audit artifacts added:        YES
commit:                               NO
push:                                 NO
tag:                                  NO
PR:                                   NO
```

## 16. VERDICT

```text
Gate B:
FINDINGS / OPEN

OPEN MAJOR:
1
```

Five of the six original findings close. Gate B cannot close until
fresh-process reopen requires explicit root-scale audit PASS evidence.

## 17. RESULT

```text
GATE_B_CLOSURE_FINDINGS
```

## 18. NEXT STEP

```text
Main Agent surgical Gate B correction.
```

Do not start V1-4.
