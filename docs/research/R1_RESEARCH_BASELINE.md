# R1 — Research Baseline

Plan document and historical W0 research index. Findings live in the linked
stage documents. This file is not a second copy of those findings.

## Current V1 product truth (W0-SR)

Distinguish **historical research truth** (the rest of this file) from
**current V1 product truth**.

```text
W0.1–W0.4     historical accepted research (findings intact)
POC-CORE-01   COMPLETE / PASS / BASELINED
              Decision Impact: INCONCLUSIVE
              Core language: NOT SELECTED
POC-FBX-01    COMPLETE / PASS / BASELINED
              Decision Impact: KEEP_UFBX_WITH_GUARDS
W0-SR         COMPLETE / ADOPTED / BASELINED
              accepted product / architecture scope revision
POC-BLENDER-E2E-01  COMPLETE / PASS / BASELINED
              Decision Impact: ACCEPT_BLENDER_BACKEND_WITH_GUARDS
              Core language: NOT SELECTED
POC-PREVIEW-01R     COMPLETE / PASS / BASELINED
              Decision Impact: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
              viewer library: NOT SELECTED
              payload candidate: GLB (not a permanent format selection)
```

Reports: [poc/POC_CORE_01.md](poc/POC_CORE_01.md), [poc/POC_FBX_01.md](poc/POC_FBX_01.md),
[poc/POC_BLENDER_E2E_01.md](poc/POC_BLENDER_E2E_01.md),
[poc/POC_PREVIEW_01R.md](poc/POC_PREVIEW_01R.md).

V1 is a Character Animation Asset Workbench with a thin Workflow Domain and
a Blender execution backend (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`). Historical
conclusions may remain
valid as risk/semantic knowledge without remaining V1 implementation
requirements. Native FBX/glTF/USD stacks, a heavy Canonical runtime, and a
custom retarget runtime are not V1 critical-path mandates.

Current contracts: [../product/V1_SCOPE.md](../product/V1_SCOPE.md),
[ADR-0001](../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Active PoC path: [poc/README.md](poc/README.md).

---

## Historical research record

W0.1 executed a **foundations slice** of R1-C, R1-D, R1-E, R1-F, and R1-O (corpus prep only). Findings live in [foundations/](foundations/W0_1_CANONICAL_FOUNDATIONS.md).

W0.1 Canonical Foundations completed external focused review.

Rev1 closed F1–F7.
Rev2 closed F8–F10.
Final closeout corrected bookkeeping / wording only.

W0.1 is now the accepted research baseline feeding later W0 stages. Status: **COMPLETE**.

W0.2 executed a **mapping / compatibility / deterministic retarget slice** of R1-G, R1-H, R1-I, and R1-O (retarget pair prep only). Findings live in [retargeting/](retargeting/W0_2_MAPPING_COMPAT_RETARGET.md).

W0.2 completed external focused review.

Initial review produced:

```text
R1-MAJOR-001
R1-MAJOR-002
R1-MAJOR-003
R1-REQ-004
```

Rev1 closed those findings.

Final closeout separated:

- root/pelvis pair compatibility from Motion trajectory suitability
- target asset capability from method/execution capability

W0.2 is now the accepted Mapping / Compatibility / Deterministic Retarget research baseline. Status: **COMPLETE**. No retarget algorithm and no IK placement are selected.

W0.3 executed an **adapter / infrastructure / preview-boundary slice** of R1-J, R1-K, R1-L, R1-M, R1-N (preview data/authority only), and R1-P (dependency facts only). Findings live in [infrastructure/](infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md).

W0.3 completed external focused review.

Initial focused review produced:

```text
R1-MAJOR-001
R1-MAJOR-002
R1-MAJOR-003
R1-MODERATE-004
R1-MODERATE-005
```

Rev1 closed all findings.

Accepted W0.3 baseline establishes:

- generic Adapter authority / capability / loss boundaries
- explicit determinism-context requirements
- Domain Semantic Mapping vs host-binding separation
- optional DCC / Engine boundaries
- storage / runtime-cook separation
- engine-independent Preview authority / data boundary
- real W0-P Adapter / Preview evidence plan

No format library, language, process topology, Preview path, renderer, or engine is selected.

W0.3 status: **COMPLETE**.

W0.4 executed a **product / tech-stack / frontier / engineering-decision slice** of R1-A, R1-J (cook placement only), R1-K (storage placement only), R1-L / R1-M (priority, not authority), R1-N (host / renderer / Preview path ranking), and R1-P (dependency governance). Recommendations live in [decisions/](decisions/W0_4_PRODUCT_TECH_DECISIONS.md).

W0.4 completed external focused review.

Initial review produced decision critical-path / experiment-quality / freshness corrections.

Rev1 minimized W0-P and deconfounded Core/Preview decisions.

Rev2 made Mandatory PoCs execution-ready, asset-ready, version-controlled, and generic-engine-validating.

Rev3 corrected named real-asset pack license authority.

Accepted W0.4 baseline now provides:

- product necessity verdict
- ranked technology recommendations
- explicit `DECIDE_NOW` vs `PROVISIONAL_W0P_GATED` decisions
- V1 scope recommendation
- dependency/license risk matrix
- frontier/AI classification
- six Mandatory W0-P experiment contracts
- one Conditional and three Optional PoCs
- named license-cleared Level-3 asset pools

Preferred candidates remain `PROVISIONAL_W0P_GATED` unless the row is an architectural role marked `DECIDE_NOW`. No Core language, format library, Preview renderer, Preview data path, or GUI stack is selected as implementation authority.

No product implementation has started. W0.4 recommendations under `docs/research/decisions/` are unchanged.

W0.4 status: **COMPLETE**. W0 remains ACTIVE. Gate A / IA-1 are not claimed.

W0-P original remaining sequence is **PAUSED / REPLANNED**. POC-BLENDER-E2E-01 is `COMPLETE / PASS / BASELINED` (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`). POC-PREVIEW-01R is `COMPLETE / PASS / BASELINED` (`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`). Next is W0-RS (`READY / NOT STARTED`), then IA-1. See [poc/README.md](poc/README.md). POC-CORE-01 and POC-FBX-01 results above are unchanged. Core language is still not selected. POC-FBX-01 does not freeze FBX as product authority and does not make a native ufbx importer a V1 mandate. Viewer library and Preview payload format remain not selected.

From W0.2 Rev1 onward, every later stage must explicitly check: **product requirements**, **generality**, **engine independence**, **format independence**, **DCC independence**, and **real-asset evidence**. Unreal, Unity, Blender, Maya, MotionBuilder, and peers are not product authority. V1 may use a hidden Blender worker (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`, validated by POC-BLENDER-E2E-01) without making Blender Canonical.

Goal: independently review the product space, standards, formats, skeleton/motion semantics, retarget, validation, publish, adapters, language/GUI options, and license risk before any implementation freeze.

Do not start implementation from this document. Decision gates come after evidence is collected.

## Tracks

| ID | Track | Question |
| --- | --- | --- |
| R1-A | Product Space | What existing tools occupy Character / Skeleton / Motion workbench space, and where is the gap? |
| R1-B | Canonical Model References | Which existing models are useful references, not authorities? |
| R1-C | FBX | What does FBX actually specify for character, skeleton, and motion? |
| R1-D | glTF / GLB | Same for glTF 2.0 and common extensions. |
| R1-E | USD / UsdSkel | Same for OpenUSD and UsdSkel. |
| R1-F | Skeleton Semantics | Joint roles, naming, hierarchy, rest pose, coordinate conventions. |
| R1-G | Compatibility | What “compatible enough to retarget” means across sources. |
| R1-H | Deterministic Retarget | Non-AI retarget methods, required inputs, failure modes. |
| R1-I | Validation / QC | What can be checked structurally vs semantically. |
| R1-J | Runtime / Cook | How runtime/cooked representations differ from authoring/canonical. |
| R1-K | Asset Identity / Publish | Identity, provenance, derived vs authoritative publish. |
| R1-L | DCC Integration | Blender / Maya (and peers) as optional adapters, not ingest prerequisites. |
| R1-M | Engine Integration | Unreal / Unity (and peers) as publish/adapter targets. |
| R1-N | UI Strategy | Workbench UI options after Domain API, without locking a stack; includes engine-independent preview (see below). |
| R1-O | Golden Fixtures | What minimal, legal fixtures R1 should reserve for later tests. |
| R1-P | License / Dependency Risk | License, maintenance, and lock-in of every candidate. |

## Candidate inventory

Research candidates. None of these is selected.

| Candidate | Initial note |
| --- | --- |
| ufbx | FBX read candidate |
| ufbx-write | FBX write candidate |
| Autodesk FBX SDK | Official FBX SDK |
| glTF 2.0 | Format / spec |
| glTF Transform | glTF processing candidate |
| cgltf | glTF parser candidate |
| tinygltf | glTF parser candidate |
| fastgltf | glTF parser candidate |
| Khronos Validator / Sample Assets | Validation and fixture reference |
| OpenUSD | Format / scene composition |
| UsdSkel | USD skeleton/motion schema |
| VRM 1.0 | Humanoid profile; not a universal schema |
| VRMC_vrm_animation | VRM animation extension |
| ozz-animation | Runtime animation candidate; not Canonical |
| Animation Compression Library | Compression / runtime candidate |
| Maya HumanIK | DCC characterization / retarget reference |
| MotionBuilder Characterization | DCC characterization reference |
| Unreal IK Rig | Engine rig adapter reference |
| Unreal IK Retargeter | Engine retarget reference |
| Unreal Retarget Chains | Engine mapping reference |
| Unreal Auto Retarget | Engine automation reference |
| Unreal Retarget Operation Stack | Engine retarget pipeline reference |
| Unity Humanoid Avatar | Engine humanoid mapping reference |
| Unity Generic Rig | Engine generic rig reference |
| Unity Mecanim / Root Motion | Engine playback / root-motion reference |
| OpenAssetIO | Asset identity / manager interface |
| AYON | Pipeline / production tracker |
| SkinTokens | Research / optional backend |
| UniRig | Research / optional backend |
| AnyTop | Research / optional backend |
| SATA | Research / optional backend |
| Motius | Research / optional backend |
| RigAnything | Research / optional backend |
| UniMate | Research / optional backend |
| NVIDIA SOMA Retargeter | Research / optional backend |
| GMR | Research / optional backend |
| Blender | DCC adapter reference |
| Maya | DCC adapter reference |
| Unreal | Engine adapter reference |
| Unity | Engine adapter reference |
| Godot / O3DE | Architecture-boundary references, not default targets |
| ISO/IEC 19774 HAnim | **NEW DISCOVERY (W0.1)** — semantic humanoid reference; not a V1 ingest target |
| COLLADA 1.4/1.5 skin | **NEW DISCOVERY (W0.1)** — historical interchange / geom-bind analogue |
| BVH | **NEW DISCOVERY (W0.1)** — motion-only historical reference |
| ASF / AMC | **NEW DISCOVERY (W0.1)** — historical mocap split |

## Required answers per item

For each candidate and each track finding, answer:

- What does it solve?
- What does it explicitly not solve?
- Which RigForge layer does it belong to?
- Canonical candidate? Adapter? Optional backend? Reference only? Reject?
- Supported asset types?
- Transform semantics?
- Skeleton semantics?
- Motion semantics?
- Retarget semantics?
- Deterministic?
- Headless?
- Batchable?
- Language / API?
- Platform limitations?
- License?
- Maintenance activity?
- Official claim vs source-confirmed fact?
- V1 relevance?
- Lock-in / semantic-loss / maintenance risks?

## Preview / UI research routing (W0.2 Rev1)

Historical W0.2 routing (findings intact). Current V1 Preview is a derived,
rebuildable, non-authoritative artifact plus an engine-independent viewer;
see [../product/V1_SCOPE.md](../product/V1_SCOPE.md).

Long-term product requirement: Canonical Character / Skeleton / Motion must be inspectable without Unreal, Unity, or another specific engine. A browser-like viewer is a **preferred direction**, not a technology decision.

R1-N and later **W0.3 / W0.4** must include, without assuming a particular engine:

- engine-independent preview strategy
- browser-like preview feasibility
- standalone renderer boundary
- Canonical → Preview representation / data flow
- preview adapter / renderer boundary
- headless / domain separation
- standalone rendering constraints

W0.3 recorded Preview **data and authority** requirements (derived, engine-free, path A/B/C/D not selected). W0.4 compared host / renderer / data-path / bridge candidates and recorded preferred directions. No renderer or Preview data path is selected as implementation authority.

Whether a full viewer ships in V1 remains a later scope gate. Minimal Workbench must not create a mandatory Unreal/Unity dependency.

Evidence tags: [README.md](README.md). Citation records: [sources/README.md](sources/README.md).
