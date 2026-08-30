# R1 — Research Baseline

Research plan only. No findings are recorded yet.

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
| R1-N | UI Strategy | Workbench UI options after Domain API, without locking a stack. |
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

Evidence tags: [README.md](README.md). Citation records: [sources/README.md](sources/README.md).
