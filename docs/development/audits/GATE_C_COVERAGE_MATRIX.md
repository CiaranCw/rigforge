# Gate C — Coverage Matrix

Audit date: 2026-09-02

Result: `GATE_C_PASS_CANDIDATE`

Legend:

```text
SOURCE = repository implementation inspected
TEST = independently rerun in the 406-test workspace suite
REAL = frozen-pair pinned Blender execution
```

## A. Mapping / Compatibility Authority — COVERED

Evidence: `domain/src/mapping.rs`, `app/src/mapping_workflow.rs`,
`app/src/preflight.rs`, `app/src/catalog.rs`; V1-4 Domain and Application
negative tests. `SOURCE + TEST`.

## B. Exact Transfer Authorization — COVERED

Evidence: `app/src/transfer.rs::authorize_transfer`,
`start_transfer`, `validate_transfer_graph`; Ready, warning acknowledgement,
MappingConfirmationRequired, and Unsupported tests. `SOURCE + TEST + REAL`.

## C. JobSpec / DerivedVariant Target Binding — COVERED

Evidence: JobSpec exact authorization/target fields, enqueue graph validation,
candidate ingestion from durable JobSpec target, wrong-target tests.
`SOURCE + TEST + REAL`.

## D. Worker Runtime Preservation — COVERED

Evidence: `app/src/dispatch.rs`, orchestration completion correlation,
`blender-worker/src/adapter.rs`, pin/isolation/policy/projection code; runtime
tests and frozen-pair execution. `SOURCE + TEST + REAL`.

## E. Candidate Creation Authority — COVERED

Evidence: `ingest_worker_success_candidate`,
`persist_candidate_graph`, `persist_candidate_on`; failed run, wrong target,
wrong artifact, and generic creation denials. `SOURCE + TEST + REAL`.

## F. WorkerResult Cardinality — COVERED

Evidence: pre-transaction and in-transaction worker-result reuse checks;
same-result second ingest/generic-put tests and distinct-result rerun test.
`SOURCE + TEST + REAL`.

## G. Durable Artifact Promotion — COVERED

Evidence: `app/src/artifact.rs`; staged and durable digest checks, missing and
mismatch tests, transactional graph persistence. `SOURCE + TEST + REAL`.

## H. QC Authority — COVERED

Evidence: exact durable artifact lookup, sealed inspect, Product evaluator,
trusted report binding. Worker measurements never construct a Product verdict.
`SOURCE + TEST + REAL`.

## I. QC Process Seal — COVERED

Evidence: private fixed production executable path, pin enforcement, isolated
environment, safety flags, no production inspector parameter, environment
override/spoof tests. `SOURCE + TEST + REAL`.

## J. QC Non-Mutation — COVERED

Evidence: worker `inspect_qc` does not save; before/after digest enforcement;
invalid-open and real durable-artifact non-mutation tests. `SOURCE + TEST + REAL`.

## K. QC Verdict Integrity — COVERED

Evidence: seven typed required checks, duplicate/missing fail-closed behavior,
derived verdict validation, false-PASS JSON tests. `SOURCE + TEST + REAL`.

## L. QC Exact Subject Binding — COVERED

Evidence: candidate, Policy, WorkerResult, artifact ID, instance, digest, and
rule-set validation; cross-artifact and generic-forgery tests.
`SOURCE + TEST + REAL`.

## M. Persistence Verification — COVERED

Evidence: exact durable path/digest, new pinned reopen process, target/action
structural checks, exact Product binding. `SOURCE + TEST + REAL`.

## N. Persistence Verification Authority — COVERED

Evidence: no production reopener parameter; generic verification persistence
denied; crate-private trusted binding; environment override/spoof tests.
`SOURCE + TEST + REAL`.

## O. Derived Variant Identity — COVERED

Evidence: logical/version/artifact/instance/digest types remain distinct;
explicit existing logical reuse; path and digest identity tests.
`SOURCE + TEST + REAL`.

## P. Publication Graph — COVERED

Evidence: `rebuild_publication_evidence`,
`validate_publication_lineage`, exact JobRun and all Product/evidence reloads;
cross-subject and mismatch tests. `SOURCE + TEST + REAL`.

## Q. Publication Preconditions — COVERED

Evidence: eligible Compatibility, acknowledgement, succeeded run, successful
worker, QC PASS, fresh reopen PASS, structural PASS, exact Draft candidate.
Failure lifecycle tests cover each denial. `SOURCE + TEST + REAL`.

## R. Atomic Publication — COVERED

Evidence: private SQLite transaction freezes exact version and binds logical
pointer after exact draft/target checks; rollback and half-state tests.
`SOURCE + TEST + REAL`.

## S. Public Catalog Authority Surface — COVERED

Evidence: private Connection/transaction, test-only unvalidated helpers,
generic-put authority rejection, guarded public publication operation, source
visibility tests. `SOURCE + TEST`.

## T. Test-Support Isolation — COVERED

Evidence: app default features empty, test-support non-default, production
Workbench/worker dependencies do not enable it, resolver 2; production-only
feature trees resolve app default only. `SOURCE + TEST`.

## U. Failure Lifecycles — COVERED

Evidence: transfer denial before enqueue; Worker, digest, QC, verification,
and publication graph failure tests preserve prior evidence and deny false
publication. `SOURCE + TEST`.

## V. Write-Once Evidence — COVERED

Evidence: Domain write-once bindings, trusted transactional binding, published
immutability, conflicting second-binding tests, new-result rerun lineage.
`SOURCE + TEST + REAL`.

## W. Workbench Product Flow — COVERED WITH OBSERVATION

Evidence: Application-backed authorization/start methods, Product
`TransferOutcome` display, no local persistence authority, 19 Workbench tests,
and no Preview viewer/payload selection. Native event wiring remains
`GATE-C-OBS-001`. `SOURCE + TEST`.

## Product / generality / independence — COVERED

Product-owned Mapping, Compatibility, QC, persistence meaning, and Derived
Variant lineage were checked. No engine dependency or second DCC backend was
introduced. FBX and `.blend` remain representations. The frozen pair supplies
real evidence without claiming universal humanoid or non-humanoid coverage.

## Scope discipline — COVERED

No V1-6 viewer/payload, export profile implementation, engine integration,
second DCC backend, Auto-Rig, new Auto-Mapping system, artistic scoring,
worker pool, or installer was introduced.
