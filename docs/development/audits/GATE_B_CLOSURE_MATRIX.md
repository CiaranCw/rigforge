# Gate B Focused Finding Closure Matrix

Review target: the six findings from the existing Gate B Runtime Foundation
audit. This is not a new independent Gate or a repeated A–O audit.

| Finding | Closure | Focused evidence | Conclusion |
| --- | --- | --- | --- |
| GATE-B-MAJOR-001 | CLOSED | Required serialized `ExecutionCorrelation`; exact JobSpec/attempt/execution-reference/success binding; WorkerResult reuse guard; focused app and real-Blender tests | Stale or reused result cannot complete a different attempt through ordinary APIs |
| GATE-B-MAJOR-002 | CLOSED | Rejected failed evidence transitions atomically to `FAILED` without persisting the result; rejection diagnostic retained | Malformed terminal failure evidence no longer leaves a durable `RUNNING` attempt |
| GATE-B-MAJOR-003 | CLOSED | Rational numerator/denominator retained; exact cross-multiply ordering; fractional dispatch rejected before WorkerPort | Persisted rational frame provenance is not truncated or rounded |
| GATE-B-MAJOR-004 | OPEN | Root transfer, mapped-root audit, no scale FCurves, execute measurement guards, procedural violation fixture, real reopen all pass; adapter accepts reopen `root_scale_audit: None` | Fresh reopen does not fail closed when required root-scale evidence is absent |
| GATE-B-MINOR-001 | CLOSED | `LaunchedAttempt` removed from adapter map during terminal collection; success and failure cleanup test passes | Completed attempts no longer accumulate in memory |
| GATE-B-MINOR-002 | CLOSED | UTF-8 boundary-safe tail helper; Unicode helper/collection and ASCII tests pass | Unicode diagnostics cannot trigger the original invalid-boundary panic |

## Automated rerun

| Suite | Result |
| --- | ---: |
| Domain | 88 PASS |
| App | 69 PASS |
| Blender worker | 43 PASS |
| Workbench | 3 PASS |
| Total | 203 PASS / 0 FAIL |

## Narrow regression matrix

| Check | Result |
| --- | --- |
| V1-2 persistence/orchestration authority | PRESERVED |
| Product IDs distinct from DB/path/digest/Blender identity | PRESERVED |
| JobSpec immutable | PRESERVED |
| Exact Catalog graph resolution | PRESERVED |
| Blender 5.2.1 LTS / `9e2066aef7ef` pin | PRESERVED |
| One isolated process per attempt | PRESERVED |
| Fresh second-process reopen | PRESENT, but root-scale evidence presence is not mandatory |
| Exact Mapping / no Auto-Mapping | PRESERVED |
| Structured Policy | PRESERVED |
| Worker SUCCESS distinct from QC PASS/publication | PRESERVED |
| DerivedVariant publication | NOT IMPLEMENTED |
| Preview | NOT IMPLEMENTED |
| V1-4 | NOT STARTED |

## Closure count

```text
MAJOR CLOSED: 3 / 4
MAJOR OPEN:   1 / 4
MINOR CLOSED: 2 / 2
MINOR OPEN:   0 / 2
```

## Verdict

```text
Gate B:
FINDINGS / OPEN

RESULT:
GATE_B_CLOSURE_FINDINGS
```
