# IA-1 — Coverage Matrix

**Status:** `AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING`

**Baseline:** `7a13605952fe85021243928c5d3f0097c86d86e3`

**Verdict:** `PASS` — `IA1_AUDIT_PASS_CANDIDATE`

This matrix is the comprehensive IA-1 source, challenge, claim/evidence,
guard, contradiction, and deferred-work record. `PASS` here is a candidate
for external review, not implementation authorization.

## Reviewed sources

| Source | Read scope | Audit role | Result |
| --- | --- | --- | --- |
| `docs/research/decisions/W0_REVISED_SYNTHESIS.md` | Full | W0_SYNTHESIS / audit entry | CHALLENGED; materially supported |
| `docs/research/decisions/W0_RS_TRACEABILITY.md` | Full | W0_SYNTHESIS / traceability | CHALLENGED; one linked current-index defect |
| `README.md` | Full | CURRENT_CANONICAL / lifecycle | CONSISTENT |
| `AGENTS.md` | Full | CURRENT_CANONICAL / constraints | CONSISTENT |
| `docs/product/PRODUCT_VISION.md` | Full | CURRENT_CANONICAL / Product | CONSISTENT |
| `docs/product/V1_SCOPE.md` | Full | CURRENT_CANONICAL / V1 boundary | CONSISTENT |
| `docs/architecture/README.md` | Full | CURRENT_CANONICAL / principles | CONSISTENT |
| `docs/architecture/V1_BLENDER_BACKED_ARCHITECTURE.md` | Full | CURRENT_CANONICAL / worker boundary | CONSISTENT |
| `docs/architecture/V1_WORKFLOW_DOMAIN.md` | Full | CURRENT_CANONICAL / domain | CONSISTENT |
| `docs/architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md` | Full | CURRENT_CANONICAL / accepted decision | CONSISTENT |
| `docs/development/ROADMAP.md` | Full | CURRENT_CANONICAL / gates | CONSISTENT |
| `docs/research/R1_RESEARCH_BASELINE.md` | Full | CURRENT_CANONICAL / research bridge | CONSISTENT |
| `docs/research/README.md` | Full | CURRENT INDEX / evidence policy | CONSISTENT |
| `docs/research/poc/README.md` | Full | CURRENT INDEX / PoC lifecycle | MINOR lifecycle contradiction, IA1-MINOR-001 |
| `docs/research/foundations/W0_1_CANONICAL_FOUNDATIONS.md` | Full | HISTORICAL_RESEARCH | RECONCILED |
| `docs/research/retargeting/W0_2_MAPPING_COMPAT_RETARGET.md` | Full | HISTORICAL_RESEARCH | RECONCILED |
| `docs/research/infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md` | Full | HISTORICAL_RESEARCH | RECONCILED |
| `docs/research/decisions/W0_4_PRODUCT_TECH_DECISIONS.md` | Full | HISTORICAL_RESEARCH | RECONCILED |
| `docs/research/decisions/LICENSE_DEPENDENCY_MATRIX.md` | Full | HISTORICAL_RESEARCH / packaging risk | RECONCILED |
| `docs/research/decisions/W0_SCOPE_REVISION_PROPOSAL.md` | Full | SCOPE_REVISION | HISTORICAL; adopted content traced |
| `docs/product/V1_PRODUCT_SCOPE_REVISION_PROPOSAL.md` | Full | SCOPE_REVISION | HISTORICAL; adopted content traced |
| `docs/architecture/V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md` | Full | SCOPE_REVISION | HISTORICAL; adopted content traced |
| `docs/architecture/V1_WORKFLOW_DOMAIN_PROPOSAL.md` | Full | SCOPE_REVISION | HISTORICAL; adopted content traced |
| `docs/development/ROADMAP_SCOPE_REVISION_PROPOSAL.md` | Full | SCOPE_REVISION | HISTORICAL; adopted content traced |
| `docs/research/poc/W0P_REPLAN_PROPOSAL.md` | Full | SCOPE_REVISION / old-PoC disposition | CONSISTENT |
| `docs/research/poc/POC_CORE_01.md` | Full | POC_REPORT | `INCONCLUSIVE` correctly retained |
| `docs/research/poc/POC_FBX_01.md` | Full | POC_REPORT | `KEEP_UFBX_WITH_GUARDS` correctly scoped |
| `docs/research/poc/POC_BLENDER_E2E_01.md` | Full | POC_REPORT | accepted claim supported with historical discrepancies retained |
| `docs/research/poc/POC_PREVIEW_01R.md` | Full | POC_REPORT | accepted claim supported with limitations retained |
| `experiments/w0p/poc_blender_e2e_01/scripts/blender_worker.py` | Full | POC_SOURCE / execute, inspect, reopen | decision-critical claims supported |
| `experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py` | Full | POC_SOURCE / orchestration, QC, publish | gate order and failures supported |
| `experiments/w0p/poc_blender_e2e_01/scripts/reopen_verify.py` | Full | POC_SOURCE / pointer to worker reopen | no independent logic claimed |
| `experiments/w0p/poc_blender_e2e_01/config/mapping_frozen.json` | Full | POC_SOURCE / Mapping | explicit, pair-specific, outside Blender |
| `experiments/w0p/poc_blender_e2e_01/config/retarget_policy.json` | Full | POC_SOURCE / Policy | backend-neutral intent supported |
| `experiments/w0p/poc_blender_e2e_01/schemas/job_spec.md` | Full | POC_SOURCE / boundary | Blender Product semantics forbidden |
| `experiments/w0p/poc_blender_e2e_01/schemas/worker_result.md` | Full | POC_SOURCE / result | backend-neutral meaning |
| `experiments/w0p/poc_blender_e2e_01/schemas/derived_variant.md` | Full | POC_SOURCE / publication | reopen/QC lineage |
| `experiments/w0p/poc_blender_e2e_01/schemas/skeleton_summary.md` | Full | POC_SOURCE / derived evidence | thin and non-authoritative |
| `experiments/w0p/poc_preview_01r/viewer/app.js` | Full | POC_SOURCE / binding and integrity | fail-closed path supported |
| `experiments/w0p/poc_preview_01r/generator/blender_preview_generator.py` | Full | POC_SOURCE / generator | generic Motion proxy; losses declared |
| `experiments/w0p/poc_preview_01r/scripts/generate_previews.py` | Full | POC_SOURCE / Product fixture, rebuild | identity/deletion claims supported |
| `experiments/w0p/poc_preview_01r/scripts/preview_selftest.py` | Full | POC_SOURCE / browser evidence | controls and local request audit supported |
| `experiments/w0p/poc_preview_01r/config/product_fixture.json` | Full | POC_SOURCE / Product truth | lineage fixture independent of manifest |
| `experiments/w0p/poc_preview_01r/config/preview_generation.json` | Full | POC_SOURCE / pins | harness choices unselected |
| `experiments/w0p/poc_preview_01r/schemas/preview_artifact.md` | Full | POC_SOURCE / Preview contract | authority split supported |

No binary asset, generated Preview payload, `.blend`, browser binary, Blender
binary, external evidence directory, or third-party relay was accessed.

## Audit dimensions

| Dimension | Challenge | Primary evidence | Result |
| --- | --- | --- | --- |
| A | Product transaction and exclusions | Product Vision, V1 Scope, ADR, synthesis | PASS |
| B | V1 scope discipline | V1 Scope, Architecture, roadmap | PASS |
| C | Historical/current reconciliation | W0.1–W0.4, proposals, synthesis | PASS |
| D | Thin Workflow Domain sufficiency | Workflow Domain, E2E contracts | PASS; schemas open |
| E | Authority boundaries | Workflow Domain, synthesis authority matrix, Preview source | PASS |
| F | Blender boundary and guards | E2E report and source | PASS WITH GUARDS |
| G | Preview boundary and guards | Preview report and source | PASS WITH GUARDS |
| H | Evidence calibration | all reports, synthesis evidence matrix | PASS |
| I | Generality | domain contracts, Wolf inspection, E2E limits | PASS contract / E2E open |
| J | Mapping/Compatibility | W0.2, Workflow Domain, Mapping fixture | PASS |
| K | QC separation and non-mutation | V1 Scope, worker/harness source | PASS |
| L | Provenance/versioning | Derived fixture, synthesis W0-RS findings | PASS; production rules open |
| M | Original W0-P disposition | W0P replan and synthesis | PASS |
| N | Roadmap sequence/checkpoints | Roadmap and synthesis §§11–13 | PASS |
| O | Deferred/open matrix | synthesis §11 and canonical open questions | PASS |

## Claim / evidence matrix

| Claim | Evidence class (ARCHITECTURAL_DECISION/RESEARCH_REVIEWED/REAL_ASSET_POC/BROWSER_POC/INFERENCE/OPEN) | Evidence source | Evidence scope | Overclaim risk | Audit result |
| --- | --- | --- | --- | --- | --- |
| RigForge is a Character Animation Asset Workbench | ARCHITECTURAL_DECISION | Product Vision, V1 Scope, ADR-0001 | Current Product | Low | SUPPORTED |
| Character input is already rigged/skinned | ARCHITECTURAL_DECISION | Product Vision, V1 Scope | V1 contract | Low | SUPPORTED |
| Motion requires Source Skeleton context | RESEARCH_REVIEWED + ARCHITECTURAL_DECISION | W0.1/W0.2, Workflow Domain | interpretation and Product binding | Low | SUPPORTED |
| Derived Variant is first-class Product identity | ARCHITECTURAL_DECISION | Product Vision, Workflow Domain | lineage, not bytes | Medium | SUPPORTED |
| Heavy Canonical runtime is unnecessary for V1 critical path | ARCHITECTURAL_DECISION + REAL_ASSET_POC | ADR-0001, E2E | one slice informs scope decision | Medium | SUPPORTED AS DECISION, not universal proof |
| Names are evidence, not identity | RESEARCH_REVIEWED + REAL_ASSET_POC | W0.1/W0.2, POC-FBX duplicate `Body` | cross-format + tested FBX | Low | SUPPORTED |
| Mapping/Compatibility/quality are separate | RESEARCH_REVIEWED + ARCHITECTURAL_DECISION | W0.2, Workflow Domain | conceptual layers | Low | SUPPORTED |
| Root, pelvis, trajectory are distinct | RESEARCH_REVIEWED | W0.2 | policy/suitability distinction | Low | SUPPORTED |
| Core language is selected | OPEN | POC-CORE-01 | equivalent tested slice inconclusive | High if inferred from harnesses | NOT ESTABLISHED |
| ufbx can inspect tested FBX material | REAL_ASSET_POC | POC-FBX-01 | fixtures + Knight/Wolf; one L3 exporter family | Medium | SUPPORTED WITH GUARDS |
| A native ufbx importer is required | OPEN | V1 Scope, ADR | placement deferred | High | NOT ESTABLISHED |
| Blender can execute one real cross-Skeleton transfer | REAL_ASSET_POC | E2E report/source | one humanoid pair/build/policy | Medium | SUPPORTED |
| Blender is Product authority | OPEN (rejected claim) | current architecture | no supporting evidence | Critical | REJECTED |
| Durable E2E schema is backend-neutral | REAL_ASSET_POC + ARCHITECTURAL_DECISION | E2E config/schema/source | tested shape; second backend absent | Medium | SUPPORTED FOR SLICE; replacement inferred |
| No large custom retarget runtime was needed | REAL_ASSET_POC | worker source | compact pair-specific adapter | Medium | SUPPORTED FOR SLICE |
| Blender works for all pairs/non-humanoids | OPEN | explicit PoC limits | not tested | Critical | NOT ESTABLISHED |
| Preview architecture works for tested three asset kinds | BROWSER_POC | Preview report/source | one frozen set, one Edge | Medium | SUPPORTED |
| Motion can Preview without target Character | BROWSER_POC | generator/selftest | one humanoid Motion via generic hierarchy proxy | Medium | SUPPORTED FOR TEST |
| Product binding and payload integrity fail closed | BROWSER_POC | app.js/selftest fixtures | wrong hash/version/payload SHA | Low | SUPPORTED |
| Preview is rebuildable without Product mutation | BROWSER_POC | generate_previews.py | literal delete/regenerate against fixture | Medium | SUPPORTED FOR TEST |
| Viewer requires no DCC/game engine at view time | BROWSER_POC | selftest/request audit | local Edge runtime | Low | SUPPORTED FOR TEST |
| GLB is the V1 Preview payload | OPEN | Preview PoC explicitly declines selection | candidate only | High | NOT ESTABLISHED |
| `model-viewer` is the V1 viewer | OPEN | Preview PoC explicitly declines selection | harness only | High | NOT ESTABLISHED |
| Real non-humanoid Transfer/Preview works | OPEN | no real E2E | Wolf is inspection only | Critical | NOT ESTABLISHED |
| Exact E2E/Preview persistence bytes correspond | OPEN | conflicting report hashes | repository-only continuity unavailable | Medium | NOT ESTABLISHED |
| Final Rev2 semantic hash is `f2d650ec...` | REAL_ASSET_POC | E2E Rev2 closeout | revision-scoped semantic summary | Low | SUPPORTED |
| Final inspect path is repository-controlled | REAL_ASSET_POC | run_e2e.py + blender_worker.py | current source | Low | SUPPORTED |
| Blender packaging/legal clearance exists | OPEN | architecture risks/license matrix | technical PoC only | Critical | NOT ESTABLISHED |
| AI may silently accept Mapping/QC | OPEN (rejected claim) | Architecture/Workflow Domain | policy boundary | High | REJECTED |

## Guard coverage matrix

| Guard | Source evidence | W0-RS captured? | Current canonical captured where necessary? | Implementation-stage owner | Audit result |
| --- | --- | --- | --- | --- | --- |
| Pin exact Blender build and provenance | E2E §§3,9; source constants | YES §7.1.1 | YES Blender Architecture | V1-3/V1-8 | COVERED |
| Isolated/headless/factory startup/disabled autoexec/temp/diagnostics/nonzero failure | E2E §§3,18; run_e2e.py | YES §7.1.2 | YES process controls | V1-3 | COVERED |
| Backend-neutral durable contracts | E2E §§7–9,20; schemas | YES §7.1.3 | YES Architecture/Domain | V1-1/V1-3 | COVERED; second backend open |
| Validate Mapping and fail closed | worker required-entry checks | YES §7.1.4 | YES Domain invariants | V1-4 | COVERED |
| Policy recognized/supported/executed/audited | quaternion resolver/audit; PoC Rev2 | YES §7.1.5 | YES worker contract | V1-3/V1-5 | COVERED |
| No accidental previous-frame pose dependency | reset_pose_to_rest per frame | YES §7.1.6 | Necessary detail delegated to implementation | V1-3/V1-5 | COVERED |
| `ROTATION_ONLY` no ordinary translation leak | worker key logic and pose audit | YES §7.1.7 | Policy families captured | V1-5 | COVERED |
| Root/trajectory/scale/twist/helper/missing behavior explicit | Policy fixture + W0.2 | YES §7.1.7 | YES Workflow Domain | V1-4/V1-5 | COVERED |
| Worker success is not publication | run_e2e publication branch | YES §7.1.8 | YES all architecture docs | V1-2/V1-5 | COVERED |
| Fresh-process reopen plus QC before publication | run_e2e order | YES §7.1.8 | YES Blender Architecture | V1-5 | COVERED |
| QC non-mutating/no silent repair | qc_report + policy | YES §7.1.9 | YES V1 Scope/Domain | V1-5 | COVERED |
| Preview/export bytes not Product truth | E2E manifest, Preview contract | YES §7.1.10 | YES canonical matrices | V1-5/V1-6 | COVERED |
| One humanoid pair only; broader and non-humanoid hardening | E2E §§21–22 | YES §7.1.11 | YES V1 Scope/Roadmap | V1-4/V1-8 | COVERED / OPEN EVIDENCE |
| Packaging/redistribution/upgrade/legal open | E2E gap; license matrix | YES §7.1.12 | YES architecture risks | V1-3/V1-8/before distribution | COVERED / OPEN BLOCKER |
| Core language remains unselected | POC-CORE | YES §7.1.12 | YES ADR/V1 Scope | V1-1 | COVERED |
| Preview derived/rebuildable/non-authoritative | Preview contract/rebuild test | YES §7.2 | YES Product/Domain | V1-6 | COVERED |
| Product truth validates Preview, never reverse | app.js + fixture | YES §7.2 | YES Architecture/Domain | V1-6 | COVERED |
| ProductRef != PreviewArtifactRef != payload path | catalog/manifest/payload_ref | YES §7.2 | YES Workflow Domain | V1-1/V1-6 | COVERED |
| Independent kind/id/version/lineage binding | product_fixture + app.js | YES §7.2.1–2 | YES Domain | V1-1/V1-6 | COVERED |
| Payload identity/integrity before display | app.js size/SHA | YES §7.2.3 | Required outcome captured; mechanism open | V1-6 | COVERED |
| Stale/mismatch/missing/corrupt visibly fail | selftest fixtures + app.js | YES §7.2.4 | YES synthesis/Scope boundary | V1-6 | COVERED |
| Delete/regenerate without Product mutation | generate_previews.py | YES §7.2.5 | YES Workflow Domain | V1-6 | COVERED |
| Motion Preview independent of target Character | generator + browser test | YES §7.2.6 | YES Product scope | V1-6 | COVERED |
| Generic hierarchy proxy not humanoid/Canonical authority | build_hierarchy_proxy | YES §7.2.7 | YES Scope/Domain | V1-6 | COVERED |
| Viewer DCC/game-engine independent; generator replaceable | selftest + architecture | YES §7.2.8 | YES canonical architecture | V1-6 | COVERED |
| GLB/viewer unselected and non-authoritative | Preview report/config | YES §7.2.9 | YES all current contracts | V1-6 | COVERED |
| Preserve losses; non-humanoid/fidelity/platform/schema open | Preview report §14/19 | YES §7.2.10 | YES V1 Scope/Roadmap | V1-6/V1-8 | COVERED / OPEN EVIDENCE |
| Exact persistence correspondence not claimed | conflicting report hashes | YES §17 finding 001 | YES synthesis/open provenance | V1-1/V1-5 | COVERED AS NOT ESTABLISHED |
| Old semantic hashes revision-scoped | E2E report closeouts | YES §17 finding 002 | Synthesis is audit authority | audit/provenance | COVERED |
| Inspect path repository-controlled | run_e2e.py/worker | YES §17 finding 003 | Synthesis is audit authority | V1-3 | COVERED |

## Contradiction matrix

Status values are exactly `CONSISTENT`, `AMBIGUOUS`, or `CONTRADICTORY`.
“Material” means architecture/gate-changing; the one contradiction below is
real but non-material and has a MINOR finding.

| Topic | Current canonical | Historical/proposal/PoC pressure | Status | Finding / disposition |
| --- | --- | --- | --- | --- |
| Product definition | Character Animation Asset Workbench | W0.4 broad infrastructure framing | CONSISTENT | historical framing superseded |
| Character | already rigged/skinned | older broad Character construction implications | CONSISTENT | explicit V1 contract |
| Motion | animation + Source Skeleton context | W0.3 unbound metadata inspection | CONSISTENT | metadata inspect does not permit context-free playback/retarget |
| Derived Variant | first-class Product result | artifact-centric PoC representations | CONSISTENT | bytes remain representations |
| Auto-Rig | out of V1 | frontier AI/older option research | CONSISTENT | historical only |
| Canonical | heavy runtime not V1 critical path; semantic guards survive | W0.1 Canonical-first wording | CONSISTENT | explicitly reconciled |
| Core language | unselected; POC-CORE INCONCLUSIVE | W0.4 provisional C++ preference | CONSISTENT | historical preference not authority |
| Blender | pinned hidden worker with guards, not Product authority | W0.3 zero-operational-DCC rule | CONSISTENT | deliberate ADR reconciliation |
| Preview | derived/non-authoritative/engine-independent | W0.4 path rankings | CONSISTENT | role accepted, technology open |
| GLB | candidate only | W0.4 preferred derived publish candidate | CONSISTENT | unselected |
| viewer | unselected | model-viewer research harness | CONSISTENT | harness only |
| FBX/glTF/USD | transports/options; mandatory native stacks deferred | historical native critical path | CONSISTENT | current scope governs |
| engine integrations | out of V1 | W0.4 future Unreal/Unity ranking | CONSISTENT | post-V1 option |
| non-humanoid | contracts permit; real E2E open | Wolf inspection and humanoid E2E | CONSISTENT | evidence scopes distinguished |
| W0 status | ACTIVE | historical reports snapshot earlier ACTIVE substates | CONSISTENT | no source claims W0 complete |
| IA-1 status | baseline canonical `READY / NOT STARTED`; this report `AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING` | current PoC index still routes to W0-RS before IA-1 | CONTRADICTORY | IA1-MINOR-001; non-material |
| implementation authorization | NOT STARTED / unauthorized until IA-1 closed | no accepted source authorizes start | CONSISTENT | candidate PASS does not authorize |

Material contradictory rows: **0**. Non-material contradictory rows: **1**.

## Deferred / open matrix

| Topic | Current state | Legitimacy | Earliest stage | Release impact | Hidden assumption / audit result |
| --- | --- | --- | --- | --- | --- |
| Core language | OPEN / POC `INCONCLUSIVE` | Legitimate until concrete integration | V1-1 planning | Must resolve to build | Harness Python/JS/C++/Rust select nothing; ACCEPTED DEFER |
| GUI framework | OPEN | Needs Workbench/accessibility/packaging evidence | V1-2 planning | Must resolve for shipped UI | Preview harness is not Product UI; ACCEPTED DEFER |
| Database/catalog storage | OPEN | Scale, migration, recovery, concurrency unknown | V1-1/V1-2 | Must resolve | “local-first” does not select storage; ACCEPTED DEFER |
| Production schemas | OPEN | PoC fixtures are research-only | V1-1 | Must resolve before durable implementation | Must preserve migration/backend replacement; ACCEPTED DEFER |
| Viewer library | OPEN | One Edge/model-viewer test insufficient | V1-6 | Blocks Preview release | No harness selection; ACCEPTED DEFER |
| Preview payload | OPEN; GLB candidate | Required shapes/loss/performance unknown | V1-6 | Blocks Preview release | GLB is not source/round-trip authority; ACCEPTED DEFER |
| ufbx placement | DEFER / optional | PoC proves credibility, not necessity | V1-3/V1-4 | Conditional | Hidden assumption would be Blender metadata suffices; measure; ACCEPTED DEFER |
| Native glTF | DEFER | No current mandatory parser/writer need | V1-6/V1-7 | Conditional on payload/output | GLB candidate does not force native stack; ACCEPTED DEFER |
| OpenUSD | DEFER | No concrete V1 workflow | V1-7 exception or post-V1 | No for current release scope | Industry importance is not requirement; ACCEPTED DEFER |
| Auto-Mapping algorithm | OPEN | E2E held Mapping constant | V1-4 | Release blocker | Ready UX must not imply correctness; REQUIRED GATE |
| Non-humanoid | real E2E OPEN | Architecture PoC could use humanoid; contract stays neutral | V1-4, complete by V1-8 | Blocks claimed support/release scope | Wolf inspection is insufficient; REQUIRED GATE |
| Advanced QC/contact | later hardening | Universal thresholds unavailable | V1-5/V1-8 | Depends on quality claims | Structural PASS is not artistic acceptance; ACCEPTED DEFER |
| Crash/timeout/retry/cancel | OPEN | Initial PoC required ordinary fail-closed only | V1-3/V1-8 | Release blocker | Missing-input failure is not reliability campaign; REQUIRED GATE |
| Concurrency/worker pool | OPEN | Throughput/resource constraints untested | V1-3/V1-8 | Blocks supported scale | One-process evidence only; REQUIRED GATE |
| Blender packaging | OPEN | Portable research zip is not shipping package | V1-3/V1-8 | Release blocker | Install/update/rollback/platform assumptions open; REQUIRED GATE |
| Blender licensing/distribution legal | OPEN / legal review | Technical audit cannot decide | Before distribution | Release blocker | Subprocess is not legal safe harbor; REQUIRED EXTERNAL REVIEW |
| Materials/textures | incomplete | Preview architecture did not target fidelity | V1-6/V1-8 | Depends on acceptance criteria | Simplified albedo declared; ACCEPTED DEFER |
| Export formats/profiles | OPEN / optional | No concrete consumer | V1-7 | No unless release consumer needs output | Persistence is not automatic Export Profile; ACCEPTED DEFER |
| Direct engines | DROP_FROM_V1 | Not part of accepted transaction | Post-V1 on changed requirement | No | Optional future adapters do not shape V1; ACCEPTED EXCLUSION |
| MCP | DEFER | Stable Domain/job API must precede it | post-Domain/post-V1 decision | No | Automation protocol does not define Domain; ACCEPTED DEFER |
| AI mapping/retarget | optional/deferred, non-authoritative | No V1 necessity | later evidence/post-V1 | No | Suggestions cannot silently accept truth; ACCEPTED DEFER |
| Native FBX write | DEFER | No symmetry requirement or consumer | V1-7 if needed | No unless consumer requires | ufbx-read PASS does not prove write; ACCEPTED DEFER |
| Blender upgrade qualification | OPEN | One build tested | V1-8 and each upgrade | Release/maintenance blocker | Version pin is not upgrade proof; REQUIRED GATE |
| Browser/platform breadth | OPEN | One Edge version tested | V1-6/V1-8 | Preview support blocker | One browser is not qualification; REQUIRED GATE |
| Preview schema/version rules | OPEN | Research contract only | V1-1/V1-6 | Release blocker | Artifact regeneration/identity rules must be explicit; REQUIRED GATE |

## Roadmap checkpoint coverage

| Checkpoint | Earliest owner | Decision capability | Audit result |
| --- | --- | --- | --- |
| Real compatible Mapping pairs | V1-4 | continue/adjust Mapping and Compatibility | PRESENT |
| Ambiguous and incompatible pairs | V1-4 | fail closed, redesign UX/rules | PRESENT |
| Real non-humanoid full E2E | V1-4/V1-8 | constrain scope or redesign contracts/worker | PRESENT |
| Pair and motion breadth | V1-5/V1-8 | revise policy/QC/backend claims | PRESENT |
| Preview asset/payload/material/platform shapes | V1-6/V1-8 | replace viewer/payload/generator | PRESENT |
| Blender/package upgrades and rollback | V1-8/every upgrade | accept/reject new build or packaging | PRESENT |
| Reliability/failure campaigns | V1-3/V1-8 | revise process topology/publication | PRESENT |
| Concrete output consumer | V1-7 | add one output or keep absent | PRESENT |

## Matrix conclusion

```text
OPEN MAJOR: 0
OPEN MINOR: 1
MATERIAL CONTRADICTIONS: 0
MATERIAL LOST GUARDS: 0
VERDICT: PASS
SENTINEL: IA1_AUDIT_PASS_CANDIDATE
```
