# Semantic Mapping Requirements Draft — W0.2

**This is not W1. This is not a schema.**

Classification: `MUST` / `SHOULD` / `OPEN`.

Evidence tags follow [docs/research/README.md](../README.md). Access date: 2026-08-30.

Rev1 (W0.2-R1-MAJOR-001): Semantic Mapping answers correspondence. Retarget policy / solver answers distribution or reconstruction. Direct joint 1→N / N→1 is not a frozen Mapping MUST.

---

## SM-ASSET

### SM-ASSET-001 — MUST

A Semantic Mapping is a **provenance-bearing object** distinct from Skeleton, Motion, and any one third-party characterization or retarget asset.

Evidence: production systems store mapping separately from the skeleton graph — Unreal IK Retargeter, Unity Avatar, HumanIK characterization [OFFICIAL_DOC]. Those names are **reference examples**, not Canonical object types.

Reason: Adapters wrap those files. Canonical Mapping must not *be* an engine or DCC asset. The requirement remains if Unreal, Unity, or HumanIK did not exist.

Open implications: serialization and identity algorithm remain OPEN for W1.

### SM-ASSET-002 — SHOULD

A mapping should record the **semantic profile / namespace** it uses (humanoid, quadruped, custom, structural-only).

Evidence: MotionBuilder biped vs quadruped [OFFICIAL_DOC]. Unity Humanoid vs Generic [OFFICIAL_DOC]. VRM is a humanoid profile (W0.1).

Open implications: whether multiple profiles may bind one skeleton is OPEN.

---

## SM-JOINT

### SM-JOINT-001 — MUST

Joint **display name** and **alias matches** are mapping *evidence*, not Canonical joint identity and not semantic identity.

Evidence: W0.1 CR-SKELETON-002. Unity `HumanBone` stores `humanName` and `boneName` separately [OFFICIAL_DOC]. Unreal fuzzy-matches chain names [OFFICIAL_DOC].

Reason: Name collision and retarget reuse.

### SM-JOINT-002 — MUST

Mapping must be able to attach a **semantic role** (or “unknown”) to a source-local joint without requiring that role to exist on every skeleton.

Evidence: VRM optional bones; Unity optional Avatar bones; generic graphs have no roles (W0.1).

### SM-JOINT-003 — SHOULD

Useful cross-system semantic *concepts* (not a frozen enum): laterality (L/R/center), body region, chain membership, chain position (root/mid/end), deform vs helper vs twist vs IK/control vs end vs extra vs unknown.

Evidence: HumanIK required vs roll vs Reference [OFFICIAL_DOC]. Unreal chain start/end + pelvis [OFFICIAL_DOC]. Unity twist bones vs required set [OFFICIAL_DOC].

Humanoid-only concepts (eyes, jaw, fingers, HumanIK ExtraFinger) must be profile-scoped.

---

## SM-CHAIN

### SM-CHAIN-001 — MUST

Chain correspondence (named span from start joint to end joint) must be representable as a **first-class mapping construct**, not only a pair of joint IDs. A source chain and a target chain **may have unequal member counts**. That does not require a direct joint 1→N or N→1 identity table.

Evidence: Unreal 5.8 retarget chains exist so different bone counts still retarget [OFFICIAL_DOC]. The requirement is engine-independent: many real skeletons differ in spine and limb subdivision.

Reason: Per-joint mapping alone cannot state “this limb corresponds to that limb” when segmentation differs.

### SM-CHAIN-002 — SHOULD

A chain mapping should record laterality and an optional IK/end-effector goal without requiring the goal for FK-only transfer.

Evidence: Unreal IK Goal is optional on a chain [OFFICIAL_DOC].

### SM-CHAIN-003 — OPEN

Whether V1 Canonical always stores chains, or derives them from slot lists, is OPEN for W1.

---

## SM-ROLE

### SM-ROLE-001 — MUST

Humanoid role taxonomies (VRM, HumanIK, Unity HumanBodyBones, HAnim Joint names) are **profiles**, not the Skeleton, and not a universal required set.

Evidence: W0.1 CR-SEMANTICS-001/002. Unity Generic exists [OFFICIAL_DOC]. HAnim is humanoid-only [SPEC].

### SM-ROLE-002 — SHOULD

Required vs optional roles **inside an active profile** must be distinguishable.

Evidence: HumanIK cannot lock without 15 required nodes [OFFICIAL_DOC]. VRM schema `required` [SPEC]. Unity `HumanTrait.RequiredBone` [OFFICIAL_DOC].

### SM-ROLE-003 — OPEN

Concrete role strings, aliases, and LOA-like coverage ladders are not designed here. HAnim LOA is a reference, not adopted naming.

---

## SM-EVIDENCE

### SM-EVIDENCE-001 — MUST

A deterministic mapper must be able to record **why** a correspondence was chosen: rule id, alias hit, hierarchy support, competing candidates, and whether manual confirmation is required.

Evidence: Unity auto-match plus Configure Avatar [OFFICIAL_DOC]. MotionBuilder auto map only when names follow the Mapping List; otherwise manual [OFFICIAL_DOC]. Unreal fuzzy chain names can mis-assign if a “more accurate” name exists [OFFICIAL_DOC].

Reason: Deterministic-first. Evidence strength ≠ learned probability.

### SM-EVIDENCE-002 — SHOULD

Candidate result *families* (not frozen names): matched, unmapped, optional-unmapped, ambiguous, conflict.

Do not treat these as ML confidence scores.

### SM-EVIDENCE-003 — OPEN

Numeric “evidence strength” encoding is OPEN. Must not be documented as a probability unless a statistical model is later adopted (W0.4).

---

## SM-CARDINALITY

Semantic Mapping answers: **what corresponds to what?**

Retarget policy / solver answers: **how is source motion distributed or reconstructed on the target?**

These layers must not be collapsed.

### SM-CARDINALITY-001 — MUST

The mapping model must be capable of representing, as correspondence (not as motion-distribution rules):

- joint ↔ joint correspondence where appropriate
- chain ↔ chain correspondence
- source and target chains with **unequal member counts**
- source-only semantic members
- target-only semantic members
- unmapped / extra members

Evidence: Unreal retarget chains absorb different bone counts without a per-bone identity table [OFFICIAL_DOC]. HumanIK/MotionBuilder document differing spine segmentation [OFFICIAL_DOC]. Extra / unmapped members appear in VRM, Unity optional bones, and generic graphs (W0.1).

Reason: A future `Map<SourceJoint, TargetJoint>` is insufficient as the only contract because chains and extras exist. Unequal-member chain correspondence is enough to state “source spine ↔ target spine” without defining N×M joint identity.

Open implications: storage shape remains OPEN for W1.

### SM-CARDINALITY-002 — MUST

Semantic correspondence must remain distinct from **retarget distribution / reconstruction**. Examples that are *not* automatically Semantic Mapping identities:

- source upper-arm roll → multiple target twist joints (may be a distribution rule)
- 3 source spine joints → 5 target spine joints (may be chain↔chain plus a retarget fill policy)
- many human bodies → fewer robot DOF (solver constraint, GMR/SOMA domain)

Evidence: Unity twist parameters distribute roll [OFFICIAL_DOC]. Unreal interpolates along a chain [OFFICIAL_DOC]. GMR/Motius many-to-few DOF is IK configuration [PAPER] / [PROJECT_CLAIM]. These describe **how motion is applied**, not necessarily what a joint “is.”

### SM-CARDINALITY-003 — OPEN

Whether direct joint **1→N** or **N→1** belongs to Semantic Mapping, Retarget Distribution Policy, or Solver Constraints is undecided until W1 + W0-P. Do not freeze those patterns as first-class Mapping primitives.

---

## SM-NONHUMANOID

### SM-NONHUMANOID-001 — MUST

Mapping contracts that only accept VRM / HumanIK / HumanBodyBones slots are **insufficient** as the only V1 mapping model.

Evidence: Product V1_SCOPE (“VRM Humanoid as universal schema” is non-core). Unity Generic [OFFICIAL_DOC]. Unreal creature/tail chains [OFFICIAL_DOC]. W0.1 generic skeletons.

### SM-NONHUMANOID-002 — SHOULD

Non-humanoid mapping should be expressible via custom roles, chain descriptors, and/or structural (parent/child + root) correspondence.

### SM-NONHUMANOID-003 — OPEN

Whether quadruped is a shipped V1 profile or only a custom namespace is OPEN (MotionBuilder has official quadruped characterization [OFFICIAL_DOC]).

---

## SM-DIAGNOSTIC

### SM-DIAGNOSTIC-001 — MUST

Mapping completeness for a profile means: every **required** slot or required chain in that profile is resolved without conflict. Optional gaps do not by themselves equal “incomplete required mapping.”

Evidence: Unity required vs optional [OFFICIAL_DOC]. HumanIK lock rule [OFFICIAL_DOC].

### SM-DIAGNOSTIC-002 — SHOULD

Diagnostics should list unmapped deform joints separately from unmapped helpers.

Evidence: None of FBX/glTF/UsdSkel mark helper vs deform (W0.1). Classification is a mapping-layer duty. [RIGFORGE_INFERENCE]

### SM-DIAGNOSTIC-003 — OPEN

Exact diagnostic state names and UI copy are OPEN.
