# Gate D Finding-Closure Matrix

Date: 2026-09-03  
Auditor: SAME independent Gate D agent  
Review type: closure review of the original five open findings; not Gate D2

Initial independent result: `GATE_D_FINDINGS`

## Finding disposition

| Finding | Closure status | Independent basis |
| --- | --- | --- |
| `GATE-D-MAJOR-001` | **CLOSED** | Production Workbench registration reaches Application-owned Domain construction and atomic Catalog persistence. Fresh-catalog real Knight + UAL2 registration-to-Published-Derived-Preview E2E reproduced without pre-seeded Character/Motion. |
| `GATE-D-MAJOR-002` | **CLOSED** | Character and Motion selection both invalidate pair-bound state; a new Compatibility result resets consent; `request_transfer` independently compares the retained Compatibility graph with current exact selections before creating a JobRun. Four targeted tests pass. |
| `GATE-D-MINOR-001` | **OPEN** | Architecture-selection contradictions are corrected, but current `README.md:64` and `README.md:121` still say `Gate D: READY / NOT STARTED`, contradicting `README.md:91-92`, `AGENTS.md`, and `ROADMAP.md` findings/closure-pending state. |
| `GATE-D-MINOR-002` | **CLOSED** | Normative scope now preserves generic Product semantics and honest non-humanoid assessment while explicitly moving successful real non-humanoid quality to POST-V1 qualification. |
| `GATE-D-MINOR-003` | **CLOSED** | Published Transfer outcome binds the exact outcome Derived Variant version into the Preview selection; denied publication does not bind. Unit/integration and real E2E evidence agree. |

## MAJOR-001 closure coverage

| Criterion | Result | Evidence |
| --- | --- | --- |
| Workbench → Application → Domain → Catalog | PASS | `workbench/src/lib.rs:649-701`; `app/src/registration.rs`; `app/src/catalog.rs:386-438` |
| File exists / regular file / FBX only | PASS | `source_evidence_from_local_path` |
| Application-computed size and SHA-256 | PASS | `std::fs::metadata` + `crate::qc::sha256_file`; no caller digest parameter |
| Location/media evidence | PASS | `SourceArtifactEvidence::new` |
| Already-rigged Character boundary | PASS | production `BlenderSkeletonInspector`; zero joints and failed inspection reject; no Auto-Rig path |
| Character logical + Published version | PASS | Domain Draft → validate → publish → logical pointer |
| Motion + SourceSkeletonReference + Published version | PASS | explicit Source Skeleton, clip, integral frames, FPS numerator/denominator |
| Target Character not required for Motion | PASS | registration signature and production path |
| Identity independent of path/digest | PASS | generated UUIDv7 identities; duplicate-byte registration test |
| Character pair atomicity | PASS | one SQLite transaction; forced failure after first write rolls back |
| Motion triple atomicity | PASS | one SQLite transaction; forced failure after first write rolls back |
| Fresh catalog entry | PASS | app registration integration tests |
| Real frozen-pair E2E | REPRODUCED | `gate_d_registration_e2e` PASS; IDs in closure registration evidence |

## MAJOR-002 closure coverage

| Criterion | Result | Evidence |
| --- | --- | --- |
| Character-change invalidation | PASS | `select_character_version` → `invalidate_selection_bound_workflow_state`; targeted test |
| Motion-change invalidation | PASS | `select_motion_version` → same helper; targeted test |
| Mapping / Compatibility / consent / authorization cleared | PASS | helper clears all enumerated pair-bound fields and Preview/transfer presentation |
| New Compatibility resets consent | PASS | `apply_compatibility_result` sets `warnings_acknowledged=false`, `transfer_auth=None` |
| Exact request-boundary defense | PASS | Catalog Compatibility result loaded and exact Character/Motion IDs compared before `start_transfer` |
| Mismatch creates no JobRun | PASS | defense-in-depth targeted test |
| Durable Catalog history preserved | PASS | invalidation changes Workbench-local state only |

Historical clarification: the initial report’s extra warning-consent subclaim
was overbroad. `apply_compatibility_result` already reset consent. The actual
blocking defect was old Pair A Compatibility/authorization/consent surviving
a Character or Motion selection change to Pair B. That exact defect is closed.

## MINOR closure coverage

| Criterion | Result |
| --- | --- |
| Accepted ADR-0002..0006 dispositions current | PASS |
| Adoption-time OPEN/DEFER text labeled historical | PASS |
| Current Gate D status internally consistent across source-of-truth docs | **FAIL — README.md has two stale current statements** |
| Arbitrary Blender env fallback claimed | NO |
| V1-8 recovery/package integrity still claimed entirely deferred | NO |
| Viewer/legal clearance falsely claimed complete | NO |
| Generic/non-humanoid Product semantics preserved | PASS |
| Successful real non-humanoid quality moved POST-V1 | PASS |
| Exact Published Derived rebound for Preview | PASS |

## Product / architecture regression

```text
Product identity weakened:       NO
Validated<T> ingress weakened:   NO
Mapping authority weakened:      NO
Compatibility authority weakened:NO
RetargetPolicy selection weakened:NO
Worker correlation weakened:     NO
QC authority weakened:           NO
Publication authority weakened:  NO
Preview authority weakened:      NO
V1-7 started:                    NO
```

## Test and package evidence

```text
offline serial suite: 503 PASS / 0 FAIL / 0 IGNORED
correction package:   48 / 48 manifest-bound files match SHA-256 and size
initial audit docs:   3 / 3 exact SHA-256 match
```

## Open inventory after this closure review

```text
OPEN MAJOR: 0
OPEN MINOR: 1

remaining:
GATE-D-MINOR-001
```

Final independent result: `GATE_D_FINDINGS`
