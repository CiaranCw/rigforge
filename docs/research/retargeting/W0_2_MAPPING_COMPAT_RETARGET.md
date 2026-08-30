# W0.2 — Semantic Mapping / Compatibility / Deterministic Retarget

Research report. Not a schema. Not an implementation.

Access date for cited sources: **2026-08-30**.

Related: [REFERENCE_SYSTEM_MATRIX.md](REFERENCE_SYSTEM_MATRIX.md), [SEMANTIC_MAPPING_REQUIREMENTS_DRAFT.md](SEMANTIC_MAPPING_REQUIREMENTS_DRAFT.md), [COMPATIBILITY_REQUIREMENTS_DRAFT.md](COMPATIBILITY_REQUIREMENTS_DRAFT.md), [RETARGET_REQUIREMENTS_DRAFT.md](RETARGET_REQUIREMENTS_DRAFT.md), [RETARGET_QC_AND_REAL_ASSET_PLAN.md](RETARGET_QC_AND_REAL_ASSET_PLAN.md).

W0.1 input remains authoritative. Conflicts with W0.1, if any, are recorded here and are **not** applied back to foundations.

Status: **COMPLETE** (accepted Mapping / Compatibility / Deterministic Retarget research baseline).

Permanent stage checks (from Rev1 onward):

```text
PRODUCT REQUIREMENTS
GENERALITY
ENGINE INDEPENDENCE
REAL-ASSET EVIDENCE
```

Unreal, Unity, Blender, Maya, MotionBuilder, and peers are **reference systems / adapter targets / optional integrations**. They are not Canonical authority and not a required runtime, preview, ingest, or Workbench host.

---

## 1. Scope

Establish an evidence-grounded requirements baseline for:

```text
Semantic Skeleton Mapping
        →
Skeleton / Retarget Compatibility
        →
Deterministic Retarget
        →
Retarget Validation / QC
```

Out of scope: W0.3 adapter/library selection, W0.4 AI-first retarget, W0-P execution, W1 field freeze, any product code.

---

## 2. Method / evidence policy

1. Read repository authorities (product, architecture, W0.1).
2. Prefer official current product docs (Unreal 5.8, Unity 6, Maya 2026, MotionBuilder 2026), then official APIs, then official source/READMEs, then formal papers.
3. Classify every material claim. Confidence: `HIGH` = explicit spec/doc sentence; `MEDIUM` = official + consistent companion; `LOW` = incomplete docs or project claim.
4. Do not treat a production UI as Canonical architecture. A production concept may justify a **general** requirement; its object model must not become a RigForge Core object.
5. No downloaded corpus. No product implementation. No numeric experiment in this pass.
6. No Retarget method is promoted to frozen V1 Core from documentation or reference-system behavior alone. W0-P real-asset results are decision evidence.

---

## 3. Semantic mapping findings

### CONFIRMED — HIGH

**Three mapping paradigms exist in production and are not interchangeable.**

| Paradigm | What is mapped | Primary evidence |
| --- | --- | --- |
| Joint-slot / role | Source joint → named semantic slot | VRM `humanBones` node→role map [SPEC]. Unity `HumanBone.humanName` ↔ `boneName` [OFFICIAL_DOC] Unity 6 `HumanBone`. Maya/MotionBuilder HumanIK 15 required nodes [OFFICIAL_DOC] Maya 2026 HumanIK character structure. HAnim named Joint objects + LOA [SPEC] ISO/IEC 19774-1:2019. |
| Chain | Start/end bone span, name-matched across skeletons | Unreal 5.8: “you define it by joint chains, rather than by individual bones” to handle different bone counts [OFFICIAL_DOC] IK Rig Animation Retargeting. |
| Structural / context | Hierarchy + designated root, no humanoid slots | Unity Generic: “everything else” (teakettle to dragon); only a designated root-motion bone is required [OFFICIAL_DOC] Unity 6 Creating models for animation. FBX/glTF/UsdSkel generic graphs (W0.1). |

**Joint name is evidence, not identity.** W0.1 CR-SKELETON-002 survives. Production systems still use names as *matching evidence*: Unity recommends semantic names to improve auto-match [OFFICIAL_DOC]; MotionBuilder can auto-characterize when names follow the Mapping List [OFFICIAL_DOC]; Unreal chain names fuzzy-match (`ArmLeft` ≈ `left_arm`) [OFFICIAL_DOC]. That does not make a display name a Canonical ID. [RIGFORGE_INFERENCE]

**Required vs optional vs extra is a real production distinction.** HumanIK cannot save/lock characterization until 15 required nodes are mapped [OFFICIAL_DOC]. Unity Avatar match succeeds when required bones match; optional bones improve quality [OFFICIAL_DOC]. VRM schema `required` is 15 humanoid roles; extras exist as untyped glTF nodes (W0.1). Missing *required* slots block mapping completeness. Missing *optional* slots reduce coverage / quality, not the same failure. [RIGFORGE_INFERENCE]

**Per-joint mapping alone is insufficient; chain correspondence is a required capability candidate.** Unreal chains absorb different bone counts in one named chain [OFFICIAL_DOC]. That evidence supports **chain↔chain with unequal members**, which is generally useful independent of Unreal. HumanIK/MotionBuilder document differing spine segmentation [OFFICIAL_DOC].

**Direct joint 1→N / N→1 is not a proven Mapping primitive.** Unity twist parameters **distribute** roll [OFFICIAL_DOC]. GMR/Motius map many human bodies onto fewer robot DOF via IK [PROJECT_CLAIM] / [OFFICIAL_DOC]. Those are retarget **distribution / solver** patterns. Semantic Mapping answers “what corresponds to what”; policy/solver answers “how motion is applied.” Whether 1→N / N→1 belong to Mapping, Retarget Distribution, or Solver is OPEN for W1 + W0-P. A future `Map<SourceJoint,TargetJoint>` remains insufficient because chains and extras exist. [RIGFORGE_INFERENCE]

**Humanoid slot sets are profiles, not Skeleton.** Aligns with W0.1 CR-SEMANTICS-001. VRM, Unity Humanoid, HumanIK, and HAnim are human-shaped. Unreal chain names include `Tail`/`tentacle` [OFFICIAL_DOC]. MotionBuilder distinguishes Biped vs Quadruped characterization and stance [OFFICIAL_DOC]. Generic / robot / mechanical graphs need another profile or structural mapping. [RIGFORGE_INFERENCE]

### OPEN / MEDIUM

- Exact future enum names for mapping diagnostics (`MATCHED` / `AMBIGUOUS` / …) remain undesigned.
- Whether V1 stores one “active profile” or multiple simultaneous maps (humanoid + custom) is OPEN for W1.
- Auto-map rule tables (Unreal bone-name search lists, HumanIK Mapping List) are implementation evidence, not a frozen RigForge alias catalog.

### REQUIRES_POC

- Ambiguous name+hierarchy collisions on real Mixamo / DCC exports.
- Twist **distribution** and spine **chain fill** quality on licensed pairs (policy/solver, not frozen joint 1→N / N→1 identity).
- Non-humanoid chain naming without a humanoid slot set.

---

## 4. Non-humanoid implications

A future mapping model must remain valid for humanoid, quadruped, creature, robot, mechanical, and generic skeletons.

Architectural *candidates* (not frozen):

- **Semantic profile / namespace:** humanoid, quadruped, custom.
- **Chain descriptors:** start/end + laterality + region (Unreal is evidence that this works without humanoid slots).
- **Structural mapping:** parent/child + root designation (Unity Generic is evidence, not the required model).

Do not force every asset into `HumanBodyBones`, `VRM HumanBoneName`, or HumanIK slots. [RIGFORGE_INFERENCE] Confidence HIGH for the prohibition (W0.1 + generic graphs + creature/tail chain evidence). Final architecture OPEN for W1. Core Mapping/Compatibility contracts must remain valid for quadruped, creature, robot, mechanical, and arbitrary generic skeletons.

---

## 5. Compatibility findings

### CONFIRMED — HIGH

**These are not synonyms:**

```text
MAPPING_COMPLETE
  ≠ RETARGET_COMPATIBLE
    ≠ RETARGET_RESULT_ACCEPTABLE
```

Required order:

```text
Semantic Mapping
        ↓
Structural / Semantic Compatibility
        ↓
Requested Method / Policy Capability
        ↓
Motion-specific Suitability
        ↓
Retarget Execution
        ↓
QC
```

Evidence:

- Unity: a successful Avatar match means required bones matched; quality still needs optional bones and a proper pose [OFFICIAL_DOC].
- Unreal: chains can match while T-pose vs A-pose still requires a rest/base pose edit [OFFICIAL_DOC]. A declared FK-only method can leave hands in the wrong world location relative to a contact/reach quality request [OFFICIAL_DOC].
- Gleicher 1998: identical structure + different segment lengths already violates contact constraints if only joint angles are copied [PAPER].
- MotionBuilder Match Source vs default stride scaling: same characterization, different *policy*, different spatial result [OFFICIAL_DOC].

**Eligibility is method- and policy-aware.** `Skeleton A + Skeleton B + Mapping` is not a universal boolean. The skeleton pair may remain **structurally / semantically compatible** while a particular FK-only method is insufficient for a specific contact-heavy Motion under the requested quality/policy constraints. That is **method capability / motion suitability**, not universal pair incompatibility. Production method names are examples, not a frozen RigForge taxonomy. [RIGFORGE_INFERENCE] Confidence HIGH.

**Motion-specific suitability is a later layer.** Source Motion is not required for mapping completeness. Structural mapping can succeed while a clip exposes extreme hip translation, missing tracks, or contacts that a chosen method cannot preserve. Do not fold that into one Compatibility bit. [RIGFORGE_INFERENCE]

**`bone_count_same` is not compatibility.** Unreal exists specifically to retarget across different bone counts [OFFICIAL_DOC].

---

## 6. Deterministic retarget mathematical baseline

Notation (working; not a schema). Transforms are rigid unless noted. Quaternions assumed unit after normalization.

| Symbol | Meaning |
| --- | --- |
| `R^ℓ_rest(s,j)` | Source joint `j` rest **local** rotation |
| `R^ℓ_anim(s,j)` | Source joint `j` animated **local** rotation |
| `G_rest(s,j)`, `G_anim(s,j)` | Source rest / animated **global** |
| Same with `t` | Target |
| `Δ^ℓ(s,j) = inv(R^ℓ_rest(s,j)) · R^ℓ_anim(s,j)` | Local rest-relative rotation |
| `A(s,j→t,k)` | Alignment taking source rest local frame toward target rest local frame (policy-defined) |
| `C_s`, `C_t` | Chain frames (e.g. hip→foot) |
| `root`, `pelvis`, `traj` | Distinct objects (see §6.6) |

Coordinate-unit conversion (cm↔m, axis bake) is **not** proportion retarget. W0.1 CR-TRANSFORM-003.

### 6.1 Why direct local-quaternion copy fails

If source and target rest local frames differ (T-pose vs A-pose, Maya pre-rotation / joint orient, Unreal retarget pose offsets), then

```text
R^ℓ_out(t,k) := R^ℓ_anim(s,j)
```

applies source *absolute local orientation* in the target’s different rest basis. The visible limb then inherits the source rest offset. Unreal documents editing a Retarget Pose when reference poses differ [OFFICIAL_DOC]. HumanIK/MotionBuilder require a characterization stance (biped T-stance / quadruped extended stance) [OFFICIAL_DOC]. Unity Humanoid stores muscle curves relative to Avatar T-Pose / Body Transform [OFFICIAL_DOC] Root Motion.

W0.1 already forbids treating FBX evaluated world as `ParentWorld × simple(LclTRS)`. Retarget must consume **evaluated** rest/anim locals (or a documented cook), not raw pivot recipes, unless V1 stores the recipe. [RIGFORGE_INFERENCE]

### 6.2 Rest / base pose alignment

Required metadata: source rest (or retarget pose), target rest (or retarget pose), and the correspondence used to build `A`. Unreal stores source **and** target retarget poses (5.6+ API; older single-pose storage deprecated) [OFFICIAL_DOC] `IKRetargeter` / `IKRetargeterController`. Copying one mesh’s bind pose into the other without aligning chain frames is insufficient when proportions and hip placement differ [OFFICIAL_DOC] Epic retarget tutorial caution; treat as MEDIUM.

Final formula for `A` is **OPEN** (per-joint conjugation vs chain-frame vs global). Requirement: some explicit alignment must exist; identity `A` is valid only when rest frames already match.

### 6.3 Rotation transfer candidates

| Method | Assumption | Benefit | Failure | Metadata |
| --- | --- | --- | --- | --- |
| Local Δ transfer `R_out = R_t,rest · A · Δ^ℓ · inv(A)` | Similar hierarchy, corresponding joints | Preserves authored local acting | Extra/missing intermediates; different rest without `A` | Rest + map + `A` |
| Global orientation copy | End-effector / look-at style | Matches world facing | Breaks parent-relative animation; scale mismatch | Globals + parent |
| Chain-space transfer | Named chains with start/end | Different bone counts (Unreal) | Ambiguous intermediates; no chain definition | Chains + retarget pose |
| EE-constrained (IK) | Contacts / reach matter | Restores constraints Gleicher/Choi | Solver, limits, determinism cost | Goals + limits + policy |

Local Δ is a **W0-P baseline candidate**, not frozen V1 Core. Global/chain/IK are additional **candidate** modes, not silent substitutes. Final algorithm remains OPEN until real-asset evidence. The formula above is **candidate / not normative / requires W0-P**.

### 6.4 Translation policy

Do not copy all translations or only root as universal truth.

Production evidence:

- Unity `hasTranslationDoF` default false; Body Transform is the stable displacement model; Root is a Y-plane projection of Body [OFFICIAL_DOC].
- HumanIK `HipsTranslation` vs `Hips` vs optional `Reference` [OFFICIAL_DOC].
- Unreal: pelvis defined separately so root motion transfers proportionally; default op stack includes Pelvis Motion and Root Motion [OFFICIAL_DOC] `add_default_ops`.
- MotionBuilder Match Source adapts stride/world position; Action Space Compensation scales feet/legs by proportion [OFFICIAL_DOC].

Candidate transfer classes: root / pelvis-hips / ordinary joint (usually rotation-only) / special translational joints (jaws, sliders). Policy object remains undesigned.

### 6.5 Scale / proportion

Unit conversion ≠ character proportion scaling.

Anchors observed, none selected: chain length (Unreal proportional pelvis/root), action-space compensation (MotionBuilder), HumanIK “follow source scale exactly” via Reference, SOMA/GMR proportional human→robot scale [OFFICIAL_DOC] / [PROJECT_CLAIM]. Height-only scale fails when limb ratios differ (Gleicher segment-length case) [PAPER].

### 6.6 Root / pelvis / trajectory

Must stay distinct:

| Concept | Working sense | Evidence |
| --- | --- | --- |
| Skeleton root | Hierarchy root(s) | W0.1 multi-root |
| Pelvis / hips | Semantic mass/hip joint | Unreal Set Pelvis; HumanIK Hips |
| Motion root / trajectory | World displacement used for locomotion | Unity Root Transform; HumanIK Reference / HipsTranslation |
| In-place vs accumulating clip | Motion suitability + policy on whether Δroot moves the character | Unity Bake Into Pose vs apply root motion [OFFICIAL_DOC] |

Format sources (glTF/UsdSkel) do **not** define portable root-motion (W0.1). Trajectory intent and in-place vs accumulating behavior are **Motion suitability / retarget policy**, not skeleton-pair Compatibility. Pair-level Compatibility describes root/pelvis **structure and asset capability** only. [RIGFORGE_INFERENCE]

### 6.7 Twist / helper

Unity distributes roll via `upperArmTwist` / `lowerArmTwist` / leg analogues [OFFICIAL_DOC]. HumanIK roll extraction moves a percentage of parent roll onto child roll nodes [OFFICIAL_DOC]. Unreal IK goals are optional extras on chains [OFFICIAL_DOC].

Cases: source no twist / target twist; inverse; different counts. Candidate ops: keep target rest; distribute roll; derive from neighbor; ignore optional helpers. Algorithm OPEN. Helpers/IK/control bones should not map as equal deform joints. [RIGFORGE_INFERENCE]

### 6.8 Missing / extra joints

| Kind | Typical effect |
| --- | --- |
| Missing required semantic | Mapping incomplete → hard fail if that profile is required |
| Missing optional semantic | Coverage warning; policy may omit |
| Extra source deform unmapped | Loss / warning; not always a blocker |
| Extra target helper | Reconstruct, rest, or leave; policy |
| Unmapped IK/control | Usually omit from transfer |

Blockers vs policy: required-slot absence is a completeness/compatibility blocker for that profile. Extra fingers are not (HumanIK peg-leg / missing-finger example) [OFFICIAL_DOC].

### 6.9 IK boundary

IK is **not** AI. Deterministic IK / constraint correction can satisfy deterministic-first.

Evidence that rest-aligned FK **can be insufficient** for some quality contracts: Gleicher (contacts after proportion change); Choi/Ko (online EE tracking); Unreal can run IK after FK for planting / blend-to-source [OFFICIAL_DOC]; HumanIK full-body solver; GMR/SOMA multi-objective IK + limits + feet.

Candidate layering (a **research hypothesis**, not an accepted architecture):

```text
Deterministic rest-aligned transfer (FK / chain)
        →
declared correction (twist, scale, root policy)
        →
declared IK / contact / constraint stage (if used)
```

Unreal’s default op order (Pelvis, FK Chains, IK Chains, IK Solve, Root Motion) is one production instance of staged ops [OFFICIAL_DOC] `add_default_ops`. It is not a RigForge Core object model.

**V1 placement (Rev1 — not frozen):**

- Rest-aligned FK / local-rest-relative: **W0-P baseline candidate**, not frozen V1 Core
- Deterministic IK / constraints: **candidate method family**; placement **OPEN** pending W0-P
- Automatic Foot IK **feature**: still **non-core** under current V1_SCOPE (that feature ≠ all deterministic IK)
- Full-body HumanIK-class / spacetime solvers: not selected
- GMR/SOMA: **reference / optional robot backend**, not Core dependencies

If IK / constraints are used, the stage must be explicit, versioned, policy-declared, diagnosable, and must not silently mutate authoritative Canonical Motion. W0.2 does **not** require that stage to be architecturally optional.

---

## 7. Reference systems

See [REFERENCE_SYSTEM_MATRIX.md](REFERENCE_SYSTEM_MATRIX.md). Short form:

**HumanIK / MotionBuilder 2026.** Slot characterization (15 required), optional roll/spine/fingers, biped/quadruped stance, Match Source, Reach, Action Space Compensation, live solver retarget. Concepts needed because production retarget needs hips/trajectory split and required coverage — not because of one Autodesk panel.

**Unreal 5.8 IK Retargeter.** Chain + pelvis + retarget poses + modular op stack. Different bone counts are first-class. IK is optional. Auto chain / auto align exist. Do not copy Unreal assets into Canonical.

**Unity 6 Humanoid.** Slot Avatar, 15 required bones, muscles/limits, twist distribution, stretch, translation DoF, Body vs Root motion. Generic path is the non-humanoid escape. Not a universal schema.

**HAnim / VRM.** Named humanoid taxonomy + required/optional. HAnim LOA is a coverage ladder. Useful for aliases/profiles. Neither is Canonical Skeleton.

**GMR.** Human global body poses → robot base + DOF via configured IK match table, limits, CPU real-time. Domain: humanoid robots / RL tracking. Does not automatically generalize to game DCC skeletons or Canonical Motion.

**SOMA Retargeter.** BVH SOMA → G1 CSV; scale + multi-objective IK + feet stabilize + per-DOF clamp. Active development; API may change [PROJECT_CLAIM] README. Not a Core dependency.

**Motius.** Representation bridges through SMPL-22 `motion135` plus optional GMR/SOMA/FBX retarget; reports fit MPJPE. **`motion135` is not RigForge Canonical Motion.** Use as reference organization / optional backend / metric ideas (foot slide, penetration).

---

## 8. Classical research

Retained papers (concrete requirements only):

1. **Gleicher, SIGGRAPH 1998, Retargetting Motion to New Characters** [PAPER]. Same topology, different lengths; constraints (foot plants); spacetime optimization; per-frame copy is insufficient; minimize change to original while restoring constraints.
2. **Choi & Ko, 1999/2000, On-line Motion Retargetting** [PAPER]. Jacobian / inverse-rate IK; track EEs while minimizing joint-angle difference; preserves high-frequency detail; online.

Other hierarchical B-spline / style-preserving IK literature is acknowledged as `USEFUL_REFERENCE` and not required to freeze V1.

---

## 9. QC findings

“Algorithm ran” ≠ “result acceptable.” Separate executed / structurally valid / QC signals / production-acceptable. Do not freeze names or numeric thresholds.

Deterministic families: mapping coverage, NaN/Inf, quaternion norm, discontinuity, bone-length violation vs rest lengths, missing tracks, time-domain consistency (W0.1), repeatability under same policy.

Geometric/contact families need a reference space and usually asset-specific thresholds. Motius physical metrics (foot slide, float, penetration) are useful *families* on a stated skeleton protocol [OFFICIAL_DOC] Motius evaluation docs — not universal pass numbers.

---

## 10. Real-asset implications

A retarget test is a **pair** (source character+motion, target character, expected mapping, expected challenge, expected QC). Single-asset walk clips are insufficient. License classes reuse W0.1 tokens. Oracle types: synthetic identity, same-skeleton identity, production-tool *reference* (not ground truth), EE/contact invariants, manual review as last resort. See [RETARGET_QC_AND_REAL_ASSET_PLAN.md](RETARGET_QC_AND_REAL_ASSET_PLAN.md).

---

## 11. Confirmed foundations

| Finding | Disposition |
| --- | --- |
| Mapping ≠ compatibility ≠ result QC | `CONFIRMED FOUNDATION` |
| Joint name ≠ semantic identity; names are evidence | `CONFIRMED FOUNDATION` (extends W0.1) |
| Humanoid slots are a profile | `CONFIRMED FOUNDATION` (W0.1) |
| Chain mapping is a first-class **capability candidate** (unequal members allowed) | `MAPPING REQUIREMENT CANDIDATE` |
| Direct joint 1→N / N→1 as Mapping primitives | `OPEN` (may be distribution / solver) |
| Mapping correspondence ≠ retarget distribution | `CONFIRMED FOUNDATION` (Rev1) |
| Rest/base pose alignment required | `RETARGET REQUIREMENT CANDIDATE` |
| Local quat copy generally insufficient | `RETARGET REQUIREMENT CANDIDATE` |
| Root ≠ pelvis ≠ trajectory | `RETARGET REQUIREMENT CANDIDATE` |
| Unit conversion ≠ proportion scale | `RETARGET REQUIREMENT CANDIDATE` |
| Eligibility is method/policy-aware; pair compatibility ≠ method sufficiency | `COMPATIBILITY REQUIREMENT CANDIDATE` |
| Deterministic IK allowed; IK ≠ AI | `CONFIRMED`; V1 placement `OPEN` pending W0-P |
| Automatic Foot IK feature | still non-core (current V1_SCOPE) |
| `motion135` ≠ Canonical Motion | `CONFIRMED` (prior project constraint + Motius docs) |
| No method frozen as V1 Core from docs alone | `CONFIRMED FOUNDATION` (Rev1) |

---

## 12. Challenged assumptions

| Working idea | Status |
| --- | --- |
| Per-joint map is enough | **Challenged** — chains + extras |
| Direct joint 1→N / N→1 is foundational Mapping | **Challenged (Rev1)** — may be distribution / solver |
| One universal compatible? boolean | **Challenged** |
| Compatibility decidable from skeletons alone | **Partial** — structure yes; motion suitability no |
| Copy local rotation | **False** without rest alignment |
| Copy all / no translations | **False** as universals |
| FK/rest-Δ is already V1 Core | **Premature (Rev1)** — baseline candidate only |
| IK mandatory for V1 | **Not decided**; automatic Foot IK still non-core |
| Unreal/HumanIK output is ground truth | **False** — reference baseline only |

No W0.1 rewrite required. No contradiction of rest≠bind, IBM caveat, glTF no-shear, FbxTime vs EMode, or time-domain provenance.

---

## 13. Open questions

- W1: profile/namespace design; mapping object fields; identity of a Semantic Mapping asset; whether 1→N / N→1 live in Mapping vs policy vs solver.
- W0.3: whether engine retarget assets are adapter-only; Canonical → Preview representation / data flow; standalone renderer boundary.
- W0.4: learned retarget / mapping; **engine-independent preview** technology comparison (browser-like vs other standalone surfaces). Technology not selected here.
- W0-P: real pairs for T/A pose, twist mismatch, spine count, locomotion contact, incompatible pair; FK baseline vs chain vs constraint necessity.

---

## 14. W0.2 decision implications (not decisions)

| Item | Class |
| --- | --- |
| Chain + slot + structural mapping | `MAPPING REQUIREMENT CANDIDATE` / `OPEN FOR W1` shape |
| Layered compatibility | `COMPATIBILITY REQUIREMENT CANDIDATE` |
| Rest-aligned FK / local-rest-relative | `RETARGET REQUIREMENT CANDIDATE` / W0-P **baseline** / not frozen V1 Core |
| Deterministic IK / constraints | candidate family; V1 placement `OPEN` / `REQUIRES W0-P` |
| Automatic Foot IK feature | still non-core (V1_SCOPE) |
| QC families | `QC REQUIREMENT CANDIDATE` / thresholds `REQUIRES W0-P` |
| GMR / SOMA / Motius | references / optional backends; not Core authority |
| Language / GUI / preview renderer | still unfrozen; preview is a long-term product requirement |
| Engine-independent preview | `DEFER W0.3` / `DEFER W0.4` research; not a W0.2 tech pick |

---

## 15. W0.2 gate assessment

| Gate | Ready? | Reason |
| --- | --- | --- |
| DG-MAP | **YES** | Slot, chain, structural, extras, ambiguity, and mapping-vs-distribution understood enough for W1 questions |
| DG-COMPAT | **YES** | Layers separated; method examples not a taxonomy; enums not frozen |
| DG-RETARGET | **YES** | Means: enough to define W1 questions and W0-P candidates (spaces, rest, rotation alternatives, translation/scale/root/twist/missing/IK). Does **not** mean a V1 algorithm was selected |
| DG-QC | **YES** | Families + limitations; no fake thresholds |
| DG-REAL | **PARTIAL** | Pair matrix and oracles planned; licenses not all cleared; no pair executed |

---

## 16. Required questions (short answers)

1. Per-joint sufficient? **No** as the only model.
2. Chain first-class? **Yes** as a capability candidate (unequal members allowed). Unreal is evidence, not the object model.
3. 1 source → N target as Mapping primitive? **OPEN** — observed as **distribution** (twist). Chain fill is chain↔chain with unequal members, not proven joint 1→N identity.
4. N source → 1 as Mapping primitive? **OPEN** — representable as chain↔chain. Robot many-to-few DOF is solver, not mapping identity.
5. Helper/twist/IK/end ≠ deform? **Yes** — classify; do not transfer equally.
6. Non-humanoid? **Profile + chains + structural**; do not force humanoid slots. Core contracts must not require humanoid semantics.
7. Mapping complete? **Required slots/chains of the active profile are resolved without conflict.** Optional may remain open.
8. Extra compatibility facts? Rest/base pose, proportions, DOF/limits, requested method capability, root/pelvis **pair model**. Track availability and trajectory intent are Motion suitability.
9. Binary compatible? **No** — structural/semantic pair compatibility is separate from method/policy **eligibility**.
10. Source motion in Compatibility? **Suitability / QC later**, not the structural layer. Motion is not required for mapping completeness.
11. Direct local quat fails? **Different rest frames / joint orient / T vs A.**
12. Local transfer when? **Corresponding joints, aligned rest, no contact contract.** W0-P baseline, not frozen Core.
13. Global/chain when? **Different counts, EE facing, named-chain correspondence.**
14. Rest alignment info? **Both rests or declared base poses + correspondence + `A`.**
15. Translations? **Root/pelvis/special; not ordinary joints by default.** Policy OPEN.
16. Proportions vs units? **Different problems.**
17. Root vs pelvis vs trajectory? **§6.6.**
18. Twist problems? **Count mismatch; roll not in FK joints; skin collapse.** Distribution policy OPEN.
19. Missing blockers? **Required profile slots.** Optional/extra are policy.
20. When is rest-aligned FK insufficient? **Contacts, reach, large proportion change, limits** — as quality/policy contracts, not automatic pair incompatibility.
21. Deterministic IK vs V1 principle? **Compatible** (IK ≠ AI).
22. IK Core/Optional/Post-V1? **OPEN pending W0-P.** Automatic Foot IK remains non-core under current V1_SCOPE.
23. Detect slide without fix? **Measure EE/foot planar drift vs source or contact flags; do not auto-solve in QC.**
24. Deterministic QC? **Coverage, NaN, quat norm, discontinuities, missing tracks, repeatability.**
25. Universal thresholds? **Generally no.**
26. Real assets? **T/A, names, spine, twist, loco, non-humanoid, incompatible.** Pair tests decide method promotion.
27. W0.1 survivors? **Name≠id, rest≠bind, VRM≠universal, time provenance, FBX recipe, no format Canonical.**
28. W0-P? **Pairs, oracles, FK baseline vs chain vs constraint necessity, auto-map ambiguity, proportion anchors.**

---

## Discovery pass (missed systems)

| Candidate | Class | Note |
| --- | --- | --- |
| Mixamo Humanoid | `USEFUL_REFERENCE` / `SERVICE_RESTRICTED` | Name conventions; ToS-bound assets |
| SMPL / SMPL-X | `USEFUL_REFERENCE` | Research IR; not Canonical |
| Blender Rigify | `DEFER_W0_4` | DCC generator |
| Unreal Control Rig | `DEFER` W0.3/W8 | Runtime rig, not mapping contract |
| Unreal Animation Warping | `USEFUL_REFERENCE` | Correction, not core map |
| Unity Animation Rigging | `DEFER_W0_4` | Constraints package |
| ozz-animation | `USEFUL_REFERENCE` | Runtime; not Canonical (W0.1) |
| iClone / Cascadeur / Rokoko | `DEFER_W0_4` | Vendor tools |
| OpenXR body tracking | `OUT_OF_SCOPE` W0.2 | Runtime capture |
| Lee & Shin hierarchical editing | `USEFUL_REFERENCE` [PAPER] | Not required to freeze V1 |

AI-first retarget (UniMate, AnyTop, …) remains W0.4.

---

## Sources record (compact)

| Source | Authority | Version / context | URL | Class |
| --- | --- | --- | --- | --- |
| IK Rig Animation Retargeting | Epic | Unreal Engine 5.8 | https://dev.epicgames.com/documentation/unreal-engine/ik-rig-animation-retargeting-in-unreal-engine | [OFFICIAL_DOC] |
| Retargeting Bipeds with IK Rig | Epic | UE 5.8 | https://dev.epicgames.com/documentation/unreal-engine/retargeting-bipeds-with-ik-rig-in-unreal-engine | [OFFICIAL_DOC] |
| Auto Retargeting | Epic | UE 5.8 | https://dev.epicgames.com/documentation/unreal-engine/auto-retargeting-in-unreal-engine | [OFFICIAL_DOC] |
| `IKRetargeterController.add_default_ops` | Epic | UE 5.8 Python API | https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/IKRetargeterController | [OFFICIAL_DOC] |
| HumanIK character structure | Autodesk | Maya 2026 | https://help.autodesk.com/cloudhelp/2026/ENU/Maya-CharacterAnimation/files/GUID-5DEFC6E5-033C-45D5-9A0E-224E7A35131B.htm | [OFFICIAL_DOC] |
| Character Settings Reference | Autodesk | MotionBuilder 2026 | https://help.autodesk.com/cloudhelp/2026/ENU/MotionBuilder-Reference/files/GUID-CBF838CE-2F65-4A26-B390-88BB891CEDC5.html | [OFFICIAL_DOC] |
| Character Retargeting properties | Autodesk | MotionBuilder (historical official) | https://download.autodesk.com/global/docs/motionbuilder2014-tutorial/files/Character_settings_Character_Retargeting_properties.htm | [OFFICIAL_DOC] |
| Bipeds and quadrupeds | Autodesk | MotionBuilder official | https://download.autodesk.com/global/docs/motionbuilder2015/en-us/files/Models_Bipeds_and_quadrupeds.htm | [OFFICIAL_DOC] |
| HumanDescription / HumanBone / HumanTrait / HumanBodyBones | Unity | 6.0 / 6000.x | docs.unity3d.com ScriptReference | [OFFICIAL_DOC] |
| How Root Motion works | Unity | 6.0 | https://docs.unity3d.com/6000.0/Documentation/Manual/RootMotion.html | [OFFICIAL_DOC] |
| Creating models for animation | Unity | 6 | https://docs.unity3d.com/6/Documentation/Manual/UsingHumanoidChars.html | [OFFICIAL_DOC] |
| VRMC_vrm 1.0 humanoid | VRM Consortium | 1.0 | W0.1 + spec repo | [SPEC] |
| ISO/IEC 19774-1:2019 HAnim concepts | ISO / Web3D | 2019 | https://www.web3d.org/documents/specifications/19774/V2.0/Architecture/concepts.html | [SPEC] |
| GMR README | YanjieZe/GMR | current | https://github.com/YanjieZe/GMR | [PROJECT_CLAIM] / [SOURCE_CONFIRMED] README |
| Retargeting Matters / GMR | arXiv 2510.02252 | 2025 | https://arxiv.org/html/2510.02252v1 | [PAPER] |
| SOMA Retargeter README | NVIDIA | current; active development | https://github.com/NVIDIA/soma-retargeter | [PROJECT_CLAIM] / [OFFICIAL_DOC] README |
| Motius retargeting.md | ZeyuLing/Motius | main | https://github.com/ZeyuLing/Motius/blob/main/docs/motion/retargeting.md | [OFFICIAL_DOC] |
| Gleicher SIGGRAPH 1998 | ACM / author preprint | 1998 | https://graphics.cs.wisc.edu/Papers/1998/Gle98/ | [PAPER] |
| Choi & Ko On-line Motion Retargetting | 1999/2000 | preprint | https://www.cmlab.csie.ntu.edu.tw/~ming/courses/icg/Reference/online_motion_retargeting_pj11.pdf | [PAPER] |

---

## 17. Rev1 corrections (W0.2-R1)

External review accepted the research direction and required focused correction. This section does not restart W0.2.

| Finding | Correction |
| --- | --- |
| R1-MAJOR-001 | Mapping correspondence ≠ retarget distribution. Unequal-member chain↔chain remains MUST. Direct joint 1→N / N→1 moved to OPEN. |
| R1-MAJOR-002 | Rest-aligned FK is a W0-P baseline candidate, not frozen V1 Core. Deterministic IK placement OPEN. Automatic Foot IK still non-core. If IK runs, it must be explicit / versioned / diagnosable. |
| R1-MAJOR-003 | Layer order fixed. Method examples are not a taxonomy. Contact-heavy clips affect method capability / suitability, not universal pair incompatibility. |
| R1-REQ-004 | Engine-independent preview persisted in product vision. Tech OPEN. Routed to W0.3 / W0.4. |

### Engine-independence audit of MUST requirements

Every remaining `SM-*` / `CP-*` / `RT-*` MUST was checked against: would it still make sense if Unreal, Unity, or HumanIK did not exist?

Result: remaining MUST statements are general (correspondence, layers, spaces, rest alignment, policies, determinism, fail-closed, provenance). Engine terms stay in **Evidence / Examples / Reference matrix**. They are not required Canonical object names (`IK Rig`, `Avatar`, `HumanBodyBones`, `Match Source`, `Operation Stack`, `Retarget Pose` as RigForge types).

### Generality

Core Mapping/Compatibility contracts do not require humanoid semantics. Humanoid taxonomies remain profiles. Quadruped, creature, robot, mechanical, and generic skeletons remain in scope.

### Real-asset decision loop

W0-P / RV-2 results may **CONTINUE / ADJUST / RESEARCH MORE / REDESIGN / DROP** any candidate method, including FK baseline, chain transfer, twist, translation, proportion, root, and IK/constraints. Documentation alone cannot freeze V1 Core retarget.

### Final closeout (C1 / C2)

R1-MAJOR-001 / 002 / 003 and R1-REQ-004 remain CLOSED.

- **C1:** Pair-level Compatibility describes root/pelvis structure and asset capability. Clip trajectory intent, in-place vs accumulating motion, and actual root displacement belong to Motion suitability / policy (`CP-MOTION`).
- **C2:** Method eligibility evaluates three distinct inputs: target-asset capability, requested method/policy, and execution/backend capability. “IK solver enabled” is not a Skeleton fact. No engine runtime is required.
