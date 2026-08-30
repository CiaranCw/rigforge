# Reference System Matrix — W0.2

Compare production and research retarget systems. Cells are **not** RigForge product capabilities.

Access date: 2026-08-30.

## Cell legend

| Token | Meaning |
| --- | --- |
| `NATIVE` | First-class authored concept |
| `SUPPORTED` | Documented capability without a dedicated named object |
| `PARTIAL` | Some cases, bake, or optional stage |
| `NOT DEFINED` | Official text does not define the concept |
| `NOT APPLICABLE` | Outside the system’s stated domain |
| `UNKNOWN` | Official text insufficient |

---

## Matrix

| Topic | VRM 1.0 | HAnim 2019 | HumanIK / MotionBuilder 2026 | Unreal 5.8 IK Retargeter | Unity 6 Humanoid | GMR | SOMA Retargeter | Motius |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Mapping unit | node→role slot | named Joint objects | HumanIK node slots | **chains** (start/end) + pelvis | Mecanim bone slots | key body ↔ robot link | SOMA body ↔ robot | representation route + optional bone map |
| Joint slots | `NATIVE` humanBones | `NATIVE` Joint names | `NATIVE` 15 required + optional | `PARTIAL` via chain bones | `NATIVE` HumanBodyBones | `PARTIAL` IK match table | `PARTIAL` config map | `PARTIAL` SMPL-22 / named bridges |
| Chain semantics | `NOT DEFINED` | `NOT DEFINED` as retarget chains | `PARTIAL` limb groups / effectors | `NATIVE` | `NOT DEFINED` (muscles per bone) | `NOT DEFINED` | `NOT DEFINED` | `NOT DEFINED` |
| Required joints | 15 humanoid [SPEC] | LOA-dependent [SPEC] | 15 nodes or cannot lock [OFFICIAL_DOC] | chains you choose to define | ≥15 required bones [OFFICIAL_DOC] | configured key bodies | SOMA + G1 assumed | route-dependent |
| Optional joints | schema optional roles | higher LOA | spine/neck/fingers/roll/shoulders | extra chains / IK goals | optional Avatar bones | extra table rows | config | extras in bridges |
| Non-humanoid | `NOT APPLICABLE` profile | `NOT APPLICABLE` humanoid | `PARTIAL` quadruped characterization | `PARTIAL` tail/tentacle chain names; any skeleton with chains | Generic rig path (not Humanoid) | `NOT APPLICABLE` humanoid robots | `NOT APPLICABLE` SOMA→G1 | SMPL-centric; FBX map optional |
| Manual mapping | `NATIVE` | authored names | `NATIVE` Definition tab | `NATIVE` chain map | `NATIVE` Configure Avatar | `NATIVE` JSON | `NATIVE` JSON | explicit bone map / auto Mixamo names |
| Auto mapping | `NOT DEFINED` in spec | `NOT DEFINED` | name-list characterize [OFFICIAL_DOC] | Auto Create Chains; fuzzy names | hierarchy + names [OFFICIAL_DOC] | `NOT DEFINED` (manual table) | `NOT DEFINED` | Mixamo name detect [OFFICIAL_DOC] |
| Base / rest pose | companion T-pose guidance (non-normative, W0.1) | default Joint centers | biped T-stance / quadruped stance [OFFICIAL_DOC] | source+target Retarget Pose [OFFICIAL_DOC] | Avatar T-Pose / Body [OFFICIAL_DOC] | human rest implicit in motion | SOMA rest | SMPL rest / bind via body model |
| Proportion handling | `NOT DEFINED` (same-clip any VRM) | nominal dimensions annex | Action Space / Match Source | proportional pelvis/root; chain FK | stretch IK params; not a general scaler | scale in IK config | proportional human→robot [OFFICIAL_DOC] | root-motion scale in FBX path |
| Root handling | inherits glTF (no portable RM, W0.1) | humanoid_root | Reference + HipsTranslation | Root Motion op + pelvis | Body vs Root Transform [OFFICIAL_DOC] | robot base T/R | exported root + yaw/scale knobs | root in motion135 / G1 qpos |
| Translation handling | node TRS if authored | Joint translation | hips translation node; Match Source | pelvis/root ops | `hasTranslationDoF`; Body displacement | base translation + IK | IK + export scale | root translation + IK fit |
| Twist handling | `NOT DEFINED` | `NOT DEFINED` as roll bones | `NATIVE` roll extraction [OFFICIAL_DOC] | `PARTIAL` chain/IK | `NATIVE` twist 0–1 params [OFFICIAL_DOC] | absorbed in IK | absorbed in IK | HML263 twist not unique [OFFICIAL_DOC] |
| Joint limits | `NOT DEFINED` | `NOT DEFINED` in LOA text reviewed | solver / stiffness / pull | `PARTIAL` IK / op | `NATIVE` muscles / HumanBone.limit | velocity/joint limits [PROJECT_CLAIM] | per-DOF clamp [OFFICIAL_DOC] | G1 clamp; SMPL limits via model |
| IK | `NOT DEFINED` | `NOT DEFINED` | `NATIVE` full-body HumanIK | `PARTIAL` optional goals + Run IK | Humanoid IK stretch/goals; not the Avatar map | `NATIVE` mink IK | `NATIVE` Newton multi-obj IK | position IK / GMR vendor |
| Contact handling | `NOT DEFINED` | `NOT DEFINED` | floor contact / Reach (product) | Speed Planting / stride warp optional | Root Y Feet option; no auto slide metric | weights / foot tasks (paper) | feet stabilization [OFFICIAL_DOC] | physical metrics (slide/float/penetrate) |
| QC / metrics | `NOT DEFINED` | `NOT DEFINED` | visual / validation status in UI | Retarget Output Log | Avatar match success | tracking/RL metrics [PAPER] | viewer + planned robots | `fit_mpjpe_mm`; physical metrics |
| Determinism | clip apply is deterministic | `UNKNOWN` | solver settings dependent | op stack + poses; float solver if IK | Avatar+clip deterministic if no IK | iterative IK; CPU | GPU IK; API evolving | refine_iters / IK fit |
| Domain limitation | humanoid avatar | humanoid figures | biped/quad DCC characters | UE skeletons | Humanoid or Generic, not both at once | human→humanoid robot | SOMA BVH → G1 (planned more) | research reps; motion135 ≠ Canonical |

---

## How to use this matrix

- `NATIVE` in one column is a **coverage demand or profile**, not a RigForge schema.
- Unreal chain ≠ Unity slot ≠ HumanIK node. Extract **general** requirements; do not union them into one enum and do not copy object names into Canonical.
- These systems are **references**. They are not Canonical authority and not required runtimes.
- **Mapping vs distribution (Rev1):** unequal-member **chain↔chain** is a transferable correspondence idea. Unity twist split, HumanIK roll extraction, and GMR many-to-few DOF are primarily **motion distribution / solver** patterns, not proof that Semantic Mapping must store direct joint 1→N / N→1 identities.
- Robot IK systems (GMR, SOMA) prove constraint/limit/QC ideas. They do not define game-character Canonical Motion.
