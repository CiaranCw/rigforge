# R1 — Research Baseline

Plan document. W0.1 executed a **foundations slice** of R1-C, R1-D, R1-E, R1-F, and R1-O (corpus prep only). Findings live in [foundations/](foundations/W0_1_CANONICAL_FOUNDATIONS.md). This file is not a second copy of those findings.

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

W0.2 is now the accepted Mapping / Compatibility / Deterministic Retarget research baseline. Status: **COMPLETE**. Other R1 tracks remain not started. W0 is not complete. Gate A / IA-1 / W1 are not claimed. No retarget algorithm and no IK placement are selected.

From W0.2 Rev1 onward, every later stage must explicitly check: **product requirements**, **generality**, **engine independence**, and **real-asset evidence**. Unreal, Unity, Blender, Maya, MotionBuilder, and peers remain references / adapters / optional integrations — not Canonical authority.

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

Long-term product requirement: Canonical Character / Skeleton / Motion must be inspectable without Unreal, Unity, or another specific engine. A browser-like viewer is a **preferred direction**, not a technology decision.

R1-N and later **W0.3 / W0.4** must include, without assuming a particular engine:

- engine-independent preview strategy
- browser-like preview feasibility
- standalone renderer boundary
- Canonical → Preview representation / data flow
- preview adapter / renderer boundary
- headless / domain separation
- standalone rendering constraints

W0.3 is the natural home for adapter / preview-data-flow, renderer **boundary**, headless/domain separation, and standalone rendering constraints. W0.4 later compares technologies. No renderer is selected in W0.2.

Whether a full viewer ships in V1 remains a later scope gate. Minimal Workbench must not create a mandatory Unreal/Unity dependency.

Evidence tags: [README.md](README.md). Citation records: [sources/README.md](sources/README.md).
