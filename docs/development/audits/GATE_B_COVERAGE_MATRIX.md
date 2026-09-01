# Gate B Coverage Matrix

Audit target: V1-2 baseline plus uncommitted V1-3 candidate.

| Dimension | Evidence inspected / rerun | Result |
| --- | --- | --- |
| A. Product Authority Boundary | Typed Domain IDs; Catalog schema; location overlays; dispatch projection; worker envelopes; Workbench shell | PASS — no authority inversion |
| B. V1-1 Domain Preservation | Domain contracts and graph validators; all 85 Domain tests | PASS |
| C. Catalog / Persistence Integrity | SQLite DDL/migration/load/store paths; corrupt-schema tests; real-file reopen | PASS |
| D. Orchestration State Machine | JobRun transitions; dispatch/collect; public completion APIs; 15 orchestration tests | FAIL — `GATE-B-MAJOR-001`, `GATE-B-MAJOR-002` |
| E. Dispatch Resolution | exact JobSpec graph loads; `WorkerDispatchRequest`; no latest/current search | FAIL — exact IDs preserved, but rational frame intent is altered (`GATE-B-MAJOR-003`) |
| F. Worker Boundary / Replaceability | WorkerPort/CompletionPort; projection JSON; adapter/Python split | PASS with D-state findings |
| G. Blender Pin / Process Isolation | source flags/env; production guard API; real pin/archive tests | PASS |
| H. Production Worker Script Boundary | compile-time source-tree binding; production override checks | PASS for current dev runtime; `GATE-B-OBS-002` |
| I. Policy / Mapping Execution | Rust policy projection; Python execution; frozen Mapping/Policy; real run | FAIL — root scale can violate Policy (`GATE-B-MAJOR-004`); rational frame loss also applies |
| J. WorkerResult Integrity | envelope parsing; process agreement; identity checks; source/staged digests; Catalog binding | FAIL — result lacks durable attempt binding (`GATE-B-MAJOR-001`) |
| K. Fresh Reopen | second command/process; skip/failure tests; real Blender rerun | PASS |
| L. Runtime Crash Window | launch.json; spawn/order review; terminal collection paths | Deferred spawn window `GATE-B-OBS-001`; Unicode collection panic `GATE-B-MINOR-002` |
| M. Long-lived Worker Resource State | `launched` HashMap insertion/collection lifecycle | MINOR — `GATE-B-MINOR-001` |
| N. Real-Asset Evidence | two current production runs; frozen hashes; fresh reopen; missing-source path | PASS for the claimed frozen pair only |
| O. Scope Discipline | crate/source scan; Cargo workspace; Workbench boundary | PASS — no premature V1-4+ capability |

## Test Coverage

| Suite | Current rerun |
| --- | ---: |
| Domain | 85 PASS |
| App/Catalog/Orchestration | 54 PASS |
| Blender worker | 31 PASS |
| Workbench | 3 PASS |
| Total | 173 PASS / 0 FAIL |

The input package's `test_results.txt` states 56 app tests, but its overall
173 total and the independent rerun establish the correct app subtotal as 54.

## Contract Coverage

| Contract | Audit conclusion |
| --- | --- |
| Product IDs remain UUIDv7 and distinct from rows/paths/digests | Covered / PASS |
| Published versions remain immutable | Covered / PASS |
| Exact Catalog graph resolves before Blender | Covered / partial FAIL on rational frame projection |
| Worker success remains distinct from QC/publication | Covered / PASS |
| One process and workspace per attempt | Covered / PASS |
| Successful result belongs to the completed attempt | Covered / FAIL |
| Terminal collection cannot leave a dead attempt RUNNING | Covered / FAIL |
| Pin and isolation guards cannot be disabled in production | Covered / PASS |
| Exact Mapping only; no Auto-Mapping | Covered / PASS |
| Proven Policy execution | Covered / FAIL on target-rest root scale |
| Fresh second-process reopen required | Covered / PASS |
| Frozen real pair evidence | Covered / PASS |
| V1-4/V1-5/V1-6 scope remains future work | Covered / PASS |
