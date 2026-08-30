# Retarget QC and Real-Asset Plan — W0.2

Preparation for later **W0-P** and **RV-2 / W5**. No assets are downloaded into this repository.

Access date: 2026-08-30.

License tokens reuse W0.1: `REPO_FIXTURE_CANDIDATE`, `DOWNLOAD_DURING_TEST_ONLY`, `REFERENCE_ONLY`, `LICENSE_UNCLEAR`, `SERVICE_RESTRICTED`.

**Real-asset evidence principle:** no Retarget method is promoted to frozen V1 Core solely from documentation or reference-system behavior. W0-P results are decision evidence.

---

## 1. Success layers (names not frozen)

| Layer | Question |
| --- | --- |
| Executed | Did the algorithm produce finite output? |
| Structurally valid | Mapping+policy satisfied; required joints present; no NaN |
| QC signals | Metrics computed in a stated space |
| Production acceptable | Human/pipeline sign-off; **not** a universal numeric PASS |

“Algorithm ran” is never sufficient.

---

## 2. QC metric candidates

Do **not** freeze thresholds (e.g. “slide < 2 cm”).

### 2.1 Structural / mapping

| Metric | Detects | Cannot detect | Universal threshold? |
| --- | --- | --- | --- |
| Required semantic coverage | Missing required slots/chains | Motion quality | Profile-specific, not universal |
| Optional coverage | Quality risk | Blockers | No |
| Ambiguous mapping count | Name/hierarchy collisions | Correctness of the winner | No |
| Unmapped deform joints | Loss of skin motion | Auto-rig helpers | Asset-specific |
| Chain coverage | Missing limbs | Intra-chain interpolation | Profile-specific |

Units: counts / fractions. Space: mapping object.

### 2.2 Numerical

| Metric | Units | Detects | Cannot detect | Universal? |
| --- | --- | --- | --- | --- |
| NaN / Inf | flag | Corrupt solve | Subtle sliding | Yes as a hard fail |
| Quaternion norm | 1 | Non-unit output | q vs −q flips | Near-1; epsilon OPEN |
| Quat discontinuity | rad / frame | Hemisphere pops | Artistic snaps | No |
| Transform jump | m or rad / frame | Teleports | Fast valid motion | Asset/fps specific |
| Unexpected scale | 1 | Non-uniform blow-up | Proportion retarget | Policy-specific |

### 2.3 Geometric

| Metric | Space | Detects | Cannot detect | Universal? |
| --- | --- | --- | --- | --- |
| EE trajectory error | world or root-relative m | Hands/feet mismatch vs source after scale | Style | No; needs scale policy |
| Pose deviation | joint local or global | Drift from rest-Δ prediction | Contacts | Method-specific |
| Bone-length violation | rest-length ratio | Stretch/squash | Soft tissue | Policy (Unity stretch exists) |
| Root trajectory deviation | world XZ/Y | Lost locomotion | In-place intent | Policy-specific |

### 2.4 Contact

| Metric | Detects | Cannot detect | Universal? |
| --- | --- | --- | --- |
| Foot planar slide | Horizontal EE drift during contact | Soft shoes | No |
| Penetration / float | Height vs plane | Uneven terrain | Scene-specific |
| Hand contact drift | Same for hands | Prop interaction | No |

Motius documents foot slide, floating, penetration under a **stated skeleton protocol** [OFFICIAL_DOC]. Use as family inspiration, not numbers.

**Detect without auto-fix:** compute the metric; do not run IK inside the QC reporter.

### 2.5 Constraints

Joint-limit violation, DOF violation, extreme rotation: require a limit source (Unity muscles, robot URDF, authored). No universal degree threshold.

### 2.6 Motion completeness

Missing transferred tracks; time-domain provenance (W0.1); root-motion policy honored.

### 2.7 Determinism

Same inputs+policy+version → output distance below a **later** tolerance. GPU IK (SOMA) is a poor V1 determinism oracle [PROJECT_CLAIM] active API.

---

## 3. Why each benchmark class exists

| ID | Class | Why it exists |
| --- | --- | --- |
| A | Same topology, different proportions | Gleicher core case; FK vs contact |
| B | Same humanoid semantics, different names | Names are evidence only |
| C | T-pose source → A-pose target | Rest alignment; Unreal Retarget Pose |
| D | Different spine segment counts | Chain↔chain with unequal members; distribution policy is not frozen N→1 identity |
| E | Source no twist → target twist | Helper reconstruction |
| F | Source twist → target none | Collapse / ignore |
| G | Missing optional fingers | Optional ≠ required |
| H | Root-motion locomotion | Trajectory vs pelvis |
| I | In-place locomotion | Bake-into-pose analogue |
| J | Large pelvis translation | Translation policy |
| K | Walk / run / turn / jump | Contact + root Y |
| L | Upper-body-only | Partial track coverage |
| M | Humanoid, large proportion gap | Scale anchor stress |
| N | Non-humanoid mapping | Profile/chains/structural |
| O | Deliberately incompatible pair | Fail-closed |

Not every class enters the V1 fixture corpus.

---

## 4. Candidate assets (no download)

### Already in W0.1 plan

| Asset | License class | Retarget role |
| --- | --- | --- |
| Khronos SimpleSkin | `REPO_FIXTURE_CANDIDATE` CC0 model | Too simple for humanoid retarget; FK identity / two-joint math |
| Khronos InterpolationTest | `REPO_FIXTURE_CANDIDATE` | Interpolation cook, not mapping |
| Khronos Fox | `DOWNLOAD_DURING_TEST_ONLY` CC0+CC-BY | Non-humanoid / simple creature motion; class N candidate |
| CesiumMan | `DOWNLOAD_DURING_TEST_ONLY` + trademark | Humanoid-like; naming vs indices |
| VRM samples | `DOWNLOAD_DURING_TEST_ONLY` until license attached | Slot mapping B; VRM→VRM identity |
| ufbx testdata | `LICENSE_UNCLEAR` per file | FBX recipe + names |
| Mixamo FBX | `SERVICE_RESTRICTED` | Local B/C only; never repo |

### Additional families to investigate later

| Asset / family | URL | License (as documented) | Class | Why / planned test |
| --- | --- | --- | --- | --- |
| Khronos Sample Assets other humanoids | github.com/KhronosGroup/glTF-Sample-Assets | per-model LICENSE.md | check per model | B, K if licensed |
| VRM Consortium samples | github.com/vrm-c | per model | check | B, G, VRM profile |
| LAFAN1 BVH | academic mocap often used by GMR [PAPER] | **verify per redistribution** | `LICENSE_UNCLEAR` until read | H, K; not Canonical ingest |
| CMU mocap | mocap.cs.cmu.edu | academic terms vary | `LICENSE_UNCLEAR` | H, K |
| SEED / Bones Studio (SOMA) | huggingface.co/datasets/bones-studio/seed | **verify** | `LICENSE_UNCLEAR` | Robot path only; not V1 Core |
| Autodesk FBX SDK samples | SDK installer | proprietary | `REFERENCE_ONLY` | Characterization stance |
| Unity / Unreal sample characters | engine samples | engine license | `REFERENCE_ONLY` / `DOWNLOAD_DURING_TEST_ONLY` | Production-tool comparison |
| Synthetic USDA / glTF authored in W0-P | new numbers, not copied docs | project-owned | future `REPO_FIXTURE_CANDIDATE` | A, C, D, E, O oracles |

Do not assume redistribution rights.

---

## 5. Pair design (rows, not single files)

A benchmark row:

```text
Source Character + Source Motion
Target Character
Semantic Mapping expectation
Expected compatibility layer result
Expected retarget challenge
Expected QC signals
```

### Seed pair ideas (not executed)

| Pair | Source | Target | Mapping expectation | Compat | Challenge | QC |
| --- | --- | --- | --- | --- | --- | --- |
| P1 identity | Synthetic identical skeleton | Same | 1:1 all deform | Complete + FK compatible | None | Near-zero local Δ error (oracle) |
| P2 T→A | Humanoid T-pose rest | Same topo A-pose rest | 1:1 slots | Mapping OK; FK needs alignment | C | Pose error if `A` omitted |
| P3 names | VRM or Unity-style names | Mixamo-style names (local) | Same roles, different boneName | Complete if aliases/manual | B | Ambiguity count |
| P4 spine | 3-spine source clip | 5-spine target | Spine chain↔chain (unequal members) | Structurally mappable; fill policy OPEN | D | Chain EE vs mid-spine |
| P5 twist | No roll bones | Target with roll/twist helpers | Correspondence + **distribution** policy (not frozen 1→N identity) | Optional helpers | E | Skin twist / rest leftover |
| P6 loco | Root-motion walk | Different height | Humanoid slots + pelvis | FK OK; contact risky | H, M | Slide metric; no auto IK |
| P7 fox | Khronos Fox | Second simple creature or generic | Structural/custom | May be complete generic | N | Must not require VRM |
| P8 fail | Humanoid | Two-joint SimpleSkin | Required slots missing | Incompatible humanoid profile | O | Hard fail, no invented spine |

---

## 6. Oracle strategy

| Oracle | Strength | Limitation |
| --- | --- | --- |
| Exact synthetic (known `Δ`, known `A`) | Strong numerical | Not production messiness |
| Same-skeleton identity retarget | Strong “do no harm” | Misses proportion |
| Known equivalent authored pair | Medium | Rare; license |
| Production-tool comparison (UE / Mobu / Unity) | Useful **reference baseline** | Disagreement ≠ RigForge wrong |
| EE / contact invariant | Strong for loco | Needs contact labels |
| Metric-only | Repeatable | Threshold politics |
| Manual review | Catches obvious junk | Not sole V1 evidence |

---

## 7. Production-tool comparison (plan only)

Where licensing/environment permits, compare **the same pair** through:

- Unreal 5.8 IK Retargeter (FK-only vs default ops)
- MotionBuilder / Maya HumanIK (Match Source on/off)
- Unity Humanoid import+preview

Record: tool version, poses, chain/slot maps, IK on/off. Treat outputs as references, not oracles.

---

## 8. Future W0-P / RV-2 tests (not executed)

1. P1 identity numeric oracle.
2. P2 without alignment must fail QC pose metric; with alignment should pass structure.
3. P4 spine chain vs per-joint map.
4. P5/P6 twist and loco metrics computed, IK **off**.
5. P8 humanoid→SimpleSkin fail-closed.
6. Time-domain: retarget must not drop W0.1 FbxTime / TimeCode provenance.
7. Later: enable a declared IK/constraint stage, record metric change, and keep V1 placement OPEN.

---

## 9. Experiments in this W0.2 pass

None. No local numeric script, no parser, no dependency, no corpus.

---

## 10. W0-P decision record (required later)

For every major candidate (rest-aligned FK baseline, chain transfer, twist handling, translation, proportion scale, root motion, IK/constraint correction), each executed pair must record:

```text
EXPECTED
OBSERVED
GAP
ROOT CAUSE
DECISION IMPACT
```

Recommendation:

```text
CONTINUE
ADJUST
RESEARCH MORE
REDESIGN
DROP / DEFER
```

Those records may change method selection, IK placement, translation policy, scale/proportion policy, twist handling, and root policy. They may not be skipped because a reference engine already “does it that way.”
