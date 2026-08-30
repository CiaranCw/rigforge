# Compatibility Requirements Draft — W0.2

**This is not W1. This is not a schema.**

Classification: `MUST` / `SHOULD` / `OPEN`.

Access date: 2026-08-30.

Rev1 (W0.2-R1-MAJOR-003): method capability and motion suitability stay separate from structural / semantic pair compatibility. Production method names are examples, not a RigForge enum.

Final closeout: root/pelvis pair facts stay in Compatibility; clip trajectory intent stays in Motion suitability / policy. Target-asset capability, requested method/policy, and execution/backend capability are distinct inputs.

---

## Layer rule (non-negotiable)

The following must never be treated as synonyms:

```text
mapping completeness
structural / semantic compatibility
method-specific compatibility / eligibility
motion-specific suitability
retarget result quality
```

Required conceptual order:

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

`bone_count_same` is not compatibility. `mapping_complete` is not compatibility.

A structurally compatible pair is not “incompatible” merely because one method cannot meet a particular Motion’s quality contract. That is method capability / motion suitability.

---

## CP-LAYER

### CP-LAYER-001 — MUST

Compatibility evaluation must be **layered**. A complete mapping is a necessary input to some layers, not a sufficient declaration that retarget will succeed or that output is acceptable.

Evidence: Unity Avatar match vs quality [OFFICIAL_DOC]. Unreal matched chains still need Retarget Pose [OFFICIAL_DOC]. Gleicher: same topology, different lengths, contacts fail [PAPER].

### CP-LAYER-002 — MUST

A future report must be able to say what passed, failed, is risky, is missing, needs confirmation, and **which retarget methods/policies remain available**.

Evidence: Unreal Retarget Output Log [OFFICIAL_DOC]. MotionBuilder characterization validation status [OFFICIAL_DOC]. HumanIK cannot run without required nodes [OFFICIAL_DOC].

### CP-LAYER-003 — OPEN

Exact state names (`COMPATIBLE`, `COMPATIBLE_WITH_RISK`, …) are not frozen.

---

## CP-STRUCT

### CP-STRUCT-001 — MUST

Structural compatibility must consider hierarchy relations that the active profile requires (e.g. hips parent of spine and legs in HumanIK) when that profile is in use.

Evidence: HumanIK “typical arrangements” (shoulder parent of elbow parent of wrist) [OFFICIAL_DOC]. Unity recommended hierarchies [OFFICIAL_DOC].

### CP-STRUCT-002 — MUST

Multiple roots, missing intermediates, and extra unmapped joints must be **reported**, not silently normalized into a single-root humanoid.

Evidence: W0.1 CR-SKELETON-004. UsdSkel multiple roots.

### CP-STRUCT-003 — SHOULD

Chain endpoint correspondence (shoulder→hand, hip→foot) should be checked when chains are the mapping unit.

Evidence: Unreal start/end bones [OFFICIAL_DOC].

---

## CP-SEMANTIC

### CP-SEMANTIC-001 — MUST

Semantic compatibility is relative to an **active profile**. A generic pair with a complete structural map may be semantically compatible under “custom/generic” and incompatible under “VRM humanoid.”

Evidence: VRM required 15 [SPEC]. Unity Humanoid vs Generic [OFFICIAL_DOC].

### CP-SEMANTIC-002 — MUST

Missing **required** profile joints/chains is a completeness/compatibility failure for that profile. Missing **optional** joints is a coverage/risk signal, not the same class.

Evidence: HumanIK 15-node lock [OFFICIAL_DOC]. Unity required vs optional [OFFICIAL_DOC]. HumanIK missing-finger / peg-leg example [OFFICIAL_DOC].

---

## CP-METHOD

### CP-METHOD-001 — MUST

Compatibility / eligibility evaluation MUST be capable of evaluating the **declared requirements and capabilities of the requested retarget method/policy**.

Specific method families observed in production systems (local rest-aligned FK, chain transfer, IK/constraint correction, match-source / world-lock stride policies) are **research examples**, not a frozen RigForge method enum.

Evidence: Unreal FK vs optional IK [OFFICIAL_DOC]. MotionBuilder Match Source vs default stride scale [OFFICIAL_DOC]. Gleicher vs per-frame copy [PAPER]. Those names must not become Canonical object names.

### CP-METHOD-002 — MUST

Method-specific eligibility MUST evaluate **target-asset capabilities**, **requested method/policy requirements**, and **required execution capabilities** as **distinct** inputs.

Target Character / Skeleton facts (examples, not an enum): available rotational DOF, translation DOF, joint limits, mapped semantic structures, twist/helper structure, root/pelvis capabilities when the active profile defines them, chain topology.

Requested method / policy (examples, not an enum): supports joint correspondence; supports unequal-member chain transfer; supports translational joints; supports proportion correction; supports root trajectory handling; supports deterministic constraints / IK if requested.

Execution environment / implementation (not Skeleton facts): required deterministic solver implementation available; required backend/version available; required optional stage configured. “IK solver enabled” is execution / method capability, not a target-asset capability. No engine runtime is an architectural requirement.

Evidence: Unity `hasTranslationDoF`, muscle limits [OFFICIAL_DOC]. GMR/SOMA joint/velocity limits [PROJECT_CLAIM] / [OFFICIAL_DOC]. HumanIK roll nodes optional [OFFICIAL_DOC]. Those are asset or solver facts in their own systems, not a RigForge “target = IK on” field.

### CP-METHOD-003 — SHOULD

A pair may be recorded as compatible for method A and not for method B without being globally “incompatible.”

---

## CP-REST

### CP-REST-001 — MUST

Rest / base-pose discrepancy (T vs A, different local frames) is a compatibility **risk or blocker for naive FK**, not proof of mapping failure.

Evidence: Unreal Retarget Pose [OFFICIAL_DOC]. MotionBuilder stance requirements [OFFICIAL_DOC]. Unity T-Pose [OFFICIAL_DOC].

### CP-REST-002 — SHOULD

Joint-orientation / pre-rotation differences should be listed as alignment work, consistent with W0.1 FBX recipe findings.

---

## CP-SCALE

### CP-SCALE-001 — MUST

Character proportion differences (height, limb ratios, chain-length ratios, non-uniform joint scale) are compatibility dimensions **separate from** coordinate unit conversion.

Evidence: Gleicher segment lengths [PAPER]. MotionBuilder Action Space Compensation [OFFICIAL_DOC]. W0.1 units/axes.

### CP-SCALE-002 — SHOULD

Extreme non-uniform scale should be flagged even if V1 retarget later refuses it (W0.1 CR-TRANSFORM-005).

---

## CP-ROOT

### CP-ROOT-001 — MUST

Structural / semantic compatibility must explicitly describe the source and target **root/pelvis model** and the **asset capabilities** relevant to the requested retarget method.

Pair-level facts include: skeleton root structure; multiple-root implications; pelvis/hips semantic correspondence **when the active profile defines such a role**; available root/pelvis translation/rotation DOF; whether the target root/pelvis capability satisfies the requested method.

Humanoid pelvis semantics are **not** required for generic, mechanical, robot, or creature profiles. The active profile/method determines whether a pelvis-like role exists.

Do **not** treat as pair-level Compatibility: in-place vs accumulating clips, trajectory intent, actual clip root displacement, or contact-heavy root behavior. Those belong to Motion-specific Suitability and/or retarget policy.

```text
Skeleton / Mapping
        ↓
root/pelvis structural compatibility

Motion
        ↓
trajectory intent
in-place / accumulating
actual root displacement

Policy
        ↓
preserve
extract
bake
scale
ignore / reconstruct
```

Evidence: production systems distinguish hierarchy root from a hips/pelvis role (HumanIK Hips, Unreal Set Pelvis) [OFFICIAL_DOC]. W0.1: no portable format root-motion — therefore clip trajectory is not a recovered pair fact.

---

## CP-MOTION

### CP-MOTION-001 — MUST

Source **Motion** must not be required to answer structural or semantic mapping completeness. Motion-specific issues belong to a **suitability** or QC layer.

Evidence: Characterization exists before a take is applied (HumanIK/MotionBuilder) [OFFICIAL_DOC]. IK Retargeter maps skeletons then previews clips [OFFICIAL_DOC].

### CP-MOTION-002 — SHOULD

Suitability checks (when a clip is known) should include: available tracks vs mapped joints, extreme translations, contact-heavy locomotion versus a rotation-only / no-constraint policy, duration/time-domain consistency (W0.1 CR-MOTION-007), **root / trajectory intent**, **in-place vs accumulating motion**, and **root displacement characteristics**. A method that cannot meet those constraints does not rewrite structural compatibility of the skeleton pair.

### CP-MOTION-003 — OPEN

Whether V1 computes suitability automatically on every retarget request is OPEN.

---

## CP-DIAGNOSTIC

### CP-DIAGNOSTIC-001 — MUST

Compatibility must fail closed when required mapping is missing or conflicting **and** the requested method needs that mapping. It must not invent correspondences.

### CP-DIAGNOSTIC-002 — SHOULD

Risk vs hard-fail vs needs-confirmation should be distinguishable. Confirmation is for ambiguous evidence (SM-EVIDENCE), not for substituting identity motion.

### CP-DIAGNOSTIC-003 — OPEN

Thresholds that turn “risk” into “fail” (e.g. limb ratio) are OPEN and must not be invented in W0.2.
