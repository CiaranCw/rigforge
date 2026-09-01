# W0-RS Traceability

**Status:** `COMPLETE / PASS / BASELINED`

**Source baseline:** `a1dce331fe0d145405a8b05497e2ffe749d94232`

Compact evidence, disposition, and audit index for
[W0_REVISED_SYNTHESIS.md](W0_REVISED_SYNTHESIS.md). This index does not
replace source documents and does not make new Product or technology
decisions.

## 1. Classification key

| Class | Meaning |
| --- | --- |
| `CURRENT_CANONICAL` | Current accepted Product, Architecture, ADR, or roadmap truth |
| `HISTORICAL_RESEARCH` | Accepted research whose facts/risks survive subject to current scope |
| `POC_EVIDENCE` | Scoped observed experiment evidence and its accepted decision impact |
| `SCOPE_REVISION` | Historical rationale and replan evidence adopted into current contracts |
| `SUPERSEDED_FOR_V1` | Not the active V1 plan; not a claim that the research was wrong |
| `DEFER / OPEN` | No selection; needs concrete later evidence or requirement |

Evidence-strength labels used by W0-RS:

```text
RESEARCH_REVIEWED
REAL_ASSET_POC
BROWSER_POC
ARCHITECTURAL_DECISION
INFERENCE
OPEN
```

## 2. Current canonical truth

| Source | Original/current purpose | Accepted status | Current V1 disposition and surviving facts | Superseded/deferred aspects | Reflected in |
| --- | --- | --- | --- | --- | --- |
| [README.md](../../../README.md) | Project entry and lifecycle | Current | RigForge is a Character Animation Asset Workbench; Preview and Blender PoCs accepted with guards; W0-RS accepted/baselined; IA-1 next | No implementation authorization until IA-1 PASS / CLOSED | W0-RS §§1–4, 15 |
| [AGENTS.md](../../../AGENTS.md) | Agent constraints and next step | Current | Formats/DCCs/Preview remain non-authoritative; Core/GUI unfrozen; IA-1 next via a newly opened independent Agent | Product implementation, IA-1 execution | W0-RS §§2, 15 |
| [PRODUCT_VISION.md](../../product/PRODUCT_VISION.md) | Product definition | Current | Browse/select/Transfer/Derived Variant/Preview workflow; already-rigged Character; Motion has Source Skeleton context | Auto-Rig and engine/DCC authority excluded | W0-RS §3 |
| [V1_SCOPE.md](../../product/V1_SCOPE.md) | V1 boundary | Current | Thin Product capabilities; Blender and Preview validated with guards; explicit exclusions/deferrals | Exact technologies and schemas remain open | W0-RS §§3, 11 |
| [architecture/README.md](../../architecture/README.md) | Stable architecture principles | Current | Thin Workflow Domain, backend-neutral semantics, authoritative/derived split, engine independence | Canonical-first active framing superseded | W0-RS §§4, 7 |
| [V1_BLENDER_BACKED_ARCHITECTURE.md](../../architecture/V1_BLENDER_BACKED_ARCHITECTURE.md) | Worker placement and ownership | Current | Hidden isolated Blender worker; RigForge owns policy/QC/publication; Preview viewer is independent | Other DCCs and permanent Preview technology not selected | W0-RS §§4, 7.1 |
| [V1_WORKFLOW_DOMAIN.md](../../architecture/V1_WORKFLOW_DOMAIN.md) | Thin conceptual model | Current | Asset/version, Skeleton Summary, Mapping, Compatibility, Policy, Job, Derived Variant, QC, Preview, optional Export | Production schema/API/storage open | W0-RS §§4, 4.1 |
| [ADR-0001](../../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md) | Accepted V1 direction | Accepted | Thin Workflow Domain; `ACCEPT_BLENDER_BACKEND_WITH_GUARDS`; `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS` | Viewer/payload/non-humanoid/material/browser questions remain open | W0-RS §§4, 6 |
| [ROADMAP.md](../../development/ROADMAP.md) | Current phase sequence | Current | W0-RS accepted/baselined; IA-1 next; V1-1–V1-8 are planning labels | Historical W1–W10 superseded as active plan | W0-RS §§8, 15 |
| [R1_RESEARCH_BASELINE.md](../R1_RESEARCH_BASELINE.md) | Historical research index plus current-state bridge | Current index | W0.1–W0.4 and PoCs remain accepted evidence; current Product truth is separate | Old recommendations are not implementation authority | W0-RS §§2, 5 |
| [poc/README.md](../poc/README.md) | PoC lifecycle index | Current index | Four accepted PoC outcomes and W0-P replan lifecycle | Unexecuted replanned PoCs are not failures | W0-RS §§6, 8 |

## 3. Historical W0 research disposition

| Source stage/document | Original purpose | Accepted status | Key surviving facts | Superseded/deferred aspects | Current canonical reflection |
| --- | --- | --- | --- | --- | --- |
| [W0.1 Canonical Foundations](../foundations/W0_1_CANONICAL_FOUNDATIONS.md) | Cross-format Character/Skeleton/Motion semantics | `COMPLETE / BASELINED` historical research | Rest/bind/IBM distinctions; names ≠ identity; units/axes/time provenance; authored vs evaluated transforms; humanoid roles are profiles; semantic loss explicit | Full Canonical Character/Skeleton/Motion runtime as V1 foundation is `SUPERSEDED_FOR_V1`; complete DCC semantic duplication not required | Architecture authoritative/derived split; thin Skeleton Summary; worker provenance/loss guards |
| [W0.2 Mapping / Compatibility / Retarget](../retargeting/W0_2_MAPPING_COMPAT_RETARGET.md) | Mapping, compatibility, solver candidates, QC | `COMPLETE / BASELINED` historical research | Mapping ≠ Compatibility ≠ result quality; names are evidence; chains/extras/ambiguity; root ≠ pelvis ≠ trajectory; Product-owned Policy; QC does not silently repair | Exact solver, IK placement, thresholds, and custom runtime not selected | Product Vision/V1 Scope Mapping workflow; Workflow Domain Mapping/Compatibility/Policy/QC |
| [W0.3 Adapter / Infrastructure](../infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md) | Adapter, loss, isolation, Preview boundaries | `COMPLETE / BASELINED` historical research | Adapter ≠ authority; describe capability/loss before mutate; declared determinism context; derived cannot reverse-authorize; Preview engine-independent | “DCC optional” no longer means zero operational DCC dependency; exact topology/renderer/storage open | ADR-0001 and Blender worker boundary; derived Preview contract |
| [W0.4 Product / Technology Decisions](W0_4_PRODUCT_TECH_DECISIONS.md) | Ranked options, license/dependency risks, original W0-P plan | `COMPLETE / BASELINED` historical research | Option/risk knowledge; pinning and redistribution review; I/O asymmetry; generic engine/DCC boundaries; AI cannot become silent authority | Core preference not selected; native format critical path and Preview rankings superseded; original six Mandatory PoCs replanned | V1 Scope exclusions/deferrals; Roadmap; PoC replan |

## 4. Scope revision and replan provenance

| Source | Original purpose | Accepted status | Current V1 disposition | Key surviving facts | Where reflected |
| --- | --- | --- | --- | --- | --- |
| [W0_SCOPE_REVISION_PROPOSAL.md](W0_SCOPE_REVISION_PROPOSAL.md) | Reconcile clarified Product transaction with W0 | Historical proposal, adopted through W0-SR | `SCOPE_REVISION`; not current contract by itself | Heavy Canonical-first plan did not match the clarified transaction | Product/V1 Architecture/ADR-0001; W0-RS §6.1 |
| [V1_PRODUCT_SCOPE_REVISION_PROPOSAL.md](../../product/V1_PRODUCT_SCOPE_REVISION_PROPOSAL.md) | Proposed revised V1 boundary | Historical proposal, adopted | Product direction provenance | Already-rigged Character, Motion with source context, first-class Derived Variant, exclusions | Current PRODUCT_VISION/V1_SCOPE; W0-RS §3 |
| [V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md](../../architecture/V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md) | Proposed hidden Blender execution path | Historical proposal, adopted with PoC gate | Architecture rationale | Product semantics outside Blender; replaceable worker boundary | Current Blender Architecture/ADR; W0-RS §§4, 7.1 |
| [V1_WORKFLOW_DOMAIN_PROPOSAL.md](../../architecture/V1_WORKFLOW_DOMAIN_PROPOSAL.md) | Proposed thin Domain | Historical proposal, adopted with PoC gate | Domain rationale | Own workflow truth without duplicating a DCC/runtime | Current Workflow Domain; W0-RS §4 |
| [ROADMAP_SCOPE_REVISION_PROPOSAL.md](../../development/ROADMAP_SCOPE_REVISION_PROPOSAL.md) | Proposed revised sequencing | Historical proposal, adopted | Roadmap provenance | Validate architecture/Preview before revised synthesis and IA-1 | Current Roadmap; W0-RS §15 |
| [W0P_REPLAN_PROPOSAL.md](../poc/W0P_REPLAN_PROPOSAL.md) | Replan original Mandatory/Conditional/Optional PoCs | Historical proposal, adopted | `SCOPE_REVISION` and final old-PoC disposition | Merge/redefine/downgrade/defer rather than mark missing experiments failed | W0-RS §8 |

## 5. PoC decisions and limits

| Source | Original purpose | Accepted status / decision | Established | Not established / surviving guard | Current canonical reflection |
| --- | --- | --- | --- | --- | --- |
| [POC_CORE_01.md](../poc/POC_CORE_01.md) | Compare equivalent C++ raw-C and supported Rust ufbx paths | `COMPLETE / PASS / BASELINED`; `INCONCLUSIVE` | Both paths worked; official Rust binding removed much of the alleged handwritten-FFI tax | No Core language selected; later Python/JS harnesses are not language evidence | AGENTS, V1 Scope, Roadmap; W0-RS §6.2 |
| [POC_FBX_01.md](../poc/POC_FBX_01.md) | Test ufbx inspection on curated and real FBX | `COMPLETE / PASS / BASELINED`; `KEEP_UFBX_WITH_GUARDS` | Technically credible on tested Knight/Wolf and lower-level fixtures; stable inspection evidence | No mandatory importer; source axes/units, transforms, stacks/layers, bone/deform, bind, names, weights, multilayer, and exporter-family guards remain | V1 Scope native FBX `DEFER`; W0-RS §6.3 |
| [POC_BLENDER_E2E_01.md](../poc/POC_BLENDER_E2E_01.md) | Test minimal real Character+Motion transfer architecture | `COMPLETE / PASS / BASELINED`; `ACCEPT_BLENDER_BACKEND_WITH_GUARDS` | One pinned humanoid cross-Skeleton transfer; backend-neutral durable contracts; isolated bake/persist/reopen; QC-gated publication; ordinary failure closed | No broad pair/non-humanoid/quality/reliability/packaging proof | ADR-0001, Blender Architecture; W0-RS §§6.4, 7.1 |
| [POC_PREVIEW_01R.md](../poc/POC_PREVIEW_01R.md) | Test Product-bound engine-independent click-to-preview | `COMPLETE / PASS / BASELINED`; `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS` | Tested Character, independent Motion, and Derived Variant; real controls; Product binding and payload integrity; fail-closed stale cases; delete/regenerate; DCC-free view time | No model-viewer/GLB selection, broad browser/platform, fidelity, non-humanoid, or production-schema proof | ADR-0001, Preview sections, Workflow Domain; W0-RS §§6.5, 7.2 |

## 6. Original W0-P disposition ledger

| Original experiment | Disposition | Evidence replacement / later route |
| --- | --- | --- |
| POC-GLTF-01 | `MERGE` | Candidate Preview artifact/reopen evidence absorbed; parser/payload selection deferred |
| POC-RETARGET-01 | `MERGE` | Minimal execution absorbed into Blender E2E; breadth/quality later |
| POC-PREVIEW-01 | `REDEFINE` | Executed as POC-PREVIEW-01R |
| POC-ENGINE-01 | `DOWNGRADE / DROP_FROM_V1 critical path` | No direct engine work required |
| POC-USD-01 | `DEFER` | Concrete workflow required to reopen |
| POC-FBX-02 | `DEFER` | Concrete output requirement required |
| POC-DCC-01 | `MERGE` | Initial process observations absorbed; operational qualification later |
| POC-PREVIEW-02 | `DEFER` | Conditional only if accepted Preview path later proves insufficient |

The ledger records replanning, not experiment failure.

## 7. Decision-to-evidence audit map

| Current decision / invariant | Primary evidence | Evidence strength | Limitation to challenge in IA-1 |
| --- | --- | --- | --- |
| Character Animation Asset Workbench Product definition | Product Vision, V1 Scope, W0-SR proposals | `ARCHITECTURAL_DECISION` | Scope consistency and exclusion discipline |
| Thin Workflow Domain | W0-SR proposals, ADR-0001, Blender E2E | `ARCHITECTURAL_DECISION` + `REAL_ASSET_POC` | Too thin versus accidental DCC clone |
| Names are evidence, not identity | W0.1, W0.2, POC-FBX | `RESEARCH_REVIEWED` + `REAL_ASSET_POC` | Durable ids and migration still require design |
| Mapping/Compatibility/quality separation | W0.2, Blender E2E | `RESEARCH_REVIEWED` + `REAL_ASSET_POC` | Auto-Mapping and incompatible pairs untested |
| Blender worker accepted with guards | Blender E2E, ADR-0001 | `REAL_ASSET_POC` + `ARCHITECTURAL_DECISION` | One pair/build; reliability and legal risks open |
| Derived Preview path accepted with guards | Preview PoC, ADR-0001 | `BROWSER_POC` + `ARCHITECTURAL_DECISION` | One browser/set/payload; schema and fidelity open |
| Core language remains open | Core PoC | `REAL_ASSET_POC` result `INCONCLUSIVE` | Must not infer selection from harness language |
| ufbx credible but not mandatory | FBX PoC + V1 Scope | `REAL_ASSET_POC` + `ARCHITECTURAL_DECISION` | Corpus breadth and Product need open |
| Non-humanoid-capable contracts | W0.1–W0.3, Workflow Domain | `RESEARCH_REVIEWED` + `ARCHITECTURAL_DECISION` | Real transfer/Preview E2E absent |

## 8. IA-1 navigation

Recommended independent-audit order:

1. Read [W0_REVISED_SYNTHESIS.md](W0_REVISED_SYNTHESIS.md).
2. Challenge current truth against the canonical documents in §2.
3. Use §§3–5 to trace disputed claims to historical evidence or PoCs.
4. Verify old-PoC disposition in §6 against
   [W0P_REPLAN_PROPOSAL.md](../poc/W0P_REPLAN_PROPOSAL.md).
5. Audit scope and evidence inflation using §7 and the open/deferred matrix in
   W0-RS.

No chat transcript is an audit dependency.

## 9. Resolved W0-RS evidence review record

These findings are traceability defects inside immutable accepted evidence,
not silent amendments or reversals of the accepted decisions. External W0-RS
review resolved their interpretation. See W0-RS §17 for the full record.

| Finding | Conflict reviewed | External-review resolution |
| --- | --- | --- |
| `W0-RS-FINDING-001` | E2E §15 persistence blend SHA-256 differs from the Preview §4 persistence SHA-256 attributed to the accepted lineage | `RESOLVED / NON_DECISION_CRITICAL_PROVENANCE_LIMITATION`: exact byte correspondence is `NOT ESTABLISHED`; persistence SHA is not Product identity; architecture decisions are unaffected |
| `W0-RS-FINDING-002` | E2E Rev2 closeout, Rev1 closeout, and unqualified §17 body contain three semantic-run hashes | `CLOSED / DOCUMENT_VERSIONING`: Rev2 final = `f2d650ec8a79f47a30bf485af58321e17258daac2756a30b4f12c83dd3aeaf82`; Rev1 = `1c9d191f6a11f129323c3443e9a734d667f36e2b489e1e74b6fa9bc934136c91`; legacy §17 `76d68b45...` is not current Rev2 authority |
| `W0-RS-FINDING-003` | E2E Rev1 closeout says repository-controlled inspect with no hidden external script; §22 says an external script produced the JSON | `CLOSED / STALE_HISTORICAL_WORDING`: accepted path is `run_e2e.py` → repository `blender_worker.py` → `inspect(job)` → raw JSON; hidden external prerequisite = `NO` |

Finding-001 retains one bounded evidence limitation:

```text
exact E2E ↔ Preview persistence representation byte correspondence:
NOT ESTABLISHED
```

`DerivedVariantVersion` remains authoritative Product truth.
Persistence/Preview artifact versions and byte identities are representations
that production provenance must distinguish. Exact production rules remain
open.
