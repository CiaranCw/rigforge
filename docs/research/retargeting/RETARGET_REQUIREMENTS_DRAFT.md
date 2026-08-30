# Deterministic Retarget Requirements Draft — W0.2

**This is not W1. This is not an API.**

Classification: `MUST` / `SHOULD` / `OPEN`.

Access date: 2026-08-30.

Mathematical symbols follow [W0_2_MAPPING_COMPAT_RETARGET.md](W0_2_MAPPING_COMPAT_RETARGET.md) §6.

Rev1 (W0.2-R1-MAJOR-002): no Retarget method is frozen as V1 Core from documentation alone. Local rest-relative FK is a W0-P **baseline candidate**. Deterministic IK placement is OPEN pending real-asset evidence. Automatic Foot IK remains non-core under current [V1_SCOPE.md](../../product/V1_SCOPE.md).

---

## RT-SPACE

### RT-SPACE-001 — MUST

Every transferred rotation or translation must state **space** (local, global, chain, body, root/trajectory) and **reference** (rest, declared base pose, parent, previous frame).

Evidence: Unreal chain + retarget pose [OFFICIAL_DOC]. Unity Body vs Root [OFFICIAL_DOC]. W0.1 local vs world vs authored recipe.

Reason: “Copy rotation” is undefined.

### RT-SPACE-002 — MUST

Retarget must consume **evaluated** rest and animated transforms unless a documented cook from an authored recipe is marked derived (W0.1 CR-TRANSFORM-002, CR-PROVENANCE-001).

### RT-SPACE-003 — OPEN

Canonical storage convention (axis, units) remains OPEN (W0.1 CR-TRANSFORM-004). Adapters convert; retarget must not assume glTF Y-up meters silently.

---

## RT-REST

### RT-REST-001 — MUST

Retarget must apply an explicit **rest / base-pose alignment**. Identity alignment is allowed only when documented that source and target rest frames already match.

Evidence: Unreal T-pose vs A-pose Retarget Pose [OFFICIAL_DOC]. MotionBuilder stance [OFFICIAL_DOC]. Unity Avatar T-Pose [OFFICIAL_DOC].

### RT-REST-002 — MUST

Direct assignment `R^ℓ_out(t) := R^ℓ_anim(s)` is forbidden as the undocumented default.

Reason: Different rest local frames / joint orient / pre-rotation.

### RT-REST-003 — OPEN

The exact operator `A` (per-joint conjugation, chain-frame, global) is not selected.

---

## RT-ROT

### RT-ROT-001 — MUST

Rotation transfer MUST declare space, rest/base-pose alignment, and how correspondence (joint or chain) is applied. “Copy rotation” without those declarations is forbidden.

W0-P **MUST evaluate** at least one deterministic, rest-aligned FK / local-rest-relative **baseline candidate** of the form

```text
Δ^ℓ(s) = inv(R^ℓ_rest(s)) · R^ℓ_anim(s)
R^ℓ_out(t) = R^ℓ_rest(t) · A · Δ^ℓ(s) · inv(A)
```

or an evidence-equivalent rest-aligned local transfer.

This formula is a **candidate**, **not normative**, and **requires W0-P**. It is not frozen V1 Core.

Evidence: production systems introduce a shared retarget/stance pose before FK-like transfer [OFFICIAL_DOC]. That is evidence a baseline is worth testing, not proof it is the V1 algorithm.

### RT-ROT-002 — SHOULD

W0-P should also evaluate chain-space transfer when mapping uses chain↔chain with unequal member counts.

Evidence: Unreal interpolates along named chains [OFFICIAL_DOC]. The requirement is the capability candidate, not Unreal’s object model.

### RT-ROT-003 — OPEN

Global-orientation transfer and end-effector-constrained transfer remain method options. No rotation method is a silent default. V1 algorithm selection waits for W0-P.

---

## RT-TRANS

### RT-TRANS-001 — MUST

Translation transfer must be **policy-selected** per class: root, pelvis/hips, ordinary joints, special translational joints. “Copy all translations” and “copy root only” are not universal defaults.

Evidence: Unity `hasTranslationDoF` default false [OFFICIAL_DOC]. HumanIK Hips vs HipsTranslation vs Reference [OFFICIAL_DOC]. Unreal pelvis vs root ops [OFFICIAL_DOC].

### RT-TRANS-002 — SHOULD

Ordinary ball-and-socket joints should default to rotation-only unless the policy or source DOF says otherwise.

### RT-TRANS-003 — OPEN

Exact `TranslationPolicy` API is not designed.

---

## RT-SCALE

### RT-SCALE-001 — MUST

Coordinate unit/axis conversion must be separable from **character proportion** scaling.

Evidence: W0.1 CR-TRANSFORM-003. Gleicher [PAPER]. MotionBuilder Action Space Compensation [OFFICIAL_DOC].

### RT-SCALE-002 — SHOULD

Proportion scaling must name its anchor (height, pelvis height, chain length, pairwise bone distance). No single global anchor is mandated.

### RT-SCALE-003 — OPEN

Selected V1 anchor and non-uniform scale refusal policy remain OPEN.

---

## RT-ROOT

### RT-ROOT-001 — MUST

Retarget must distinguish skeleton root, pelvis/hips, and locomotion trajectory. Format silence on root-motion (W0.1) means trajectory behavior is a **declared RigForge policy**, not an inferred glTF/USD fact.

Evidence: Unity Root Motion [OFFICIAL_DOC]. Unreal pelvis + Root Motion op [OFFICIAL_DOC]. HumanIK Reference [OFFICIAL_DOC].

### RT-ROOT-002 — SHOULD

In-place vs accumulating root-motion must be an explicit clip/policy flag, analogous to Unity Bake Into Pose vs apply root motion [OFFICIAL_DOC].

### RT-ROOT-003 — OPEN

Final trajectory extraction formula is OPEN.

---

## RT-QUAT

### RT-QUAT-001 — MUST

Output quaternions must be normalizable. `q` and `-q` are the same orientation.

Evidence: glTF Appendix C normalize interpolated quaternions (W0.1) [SPEC].

### RT-QUAT-002 — SHOULD

Temporal sign continuity (hemisphere tracking) should be applied at retarget output and/or Canonical Motion normalization, and checked in QC. It may exist at more than one layer.

Reason: Does not contradict W0.1; W0.1 did not freeze continuity ownership.

### RT-QUAT-003 — OPEN

Whether continuity is Canonical-only, retarget-only, or both is OPEN for W1.

---

## RT-TWIST

### RT-TWIST-001 — SHOULD

When twist/roll counts differ, retarget must apply an explicit helper policy: preserve target rest, distribute roll, derive from neighbors, or ignore optional helpers.

Evidence: Unity twist parameters [OFFICIAL_DOC]. HumanIK roll extraction [OFFICIAL_DOC].

### RT-TWIST-002 — MUST

Twist/helper/IK/control/end joints must not be treated as equal to deform joints by default.

Evidence: W0.1: no interchange taxonomy. Production systems special-case roll and IK goals [OFFICIAL_DOC].

### RT-TWIST-003 — OPEN

Final distribution weights and algorithms are OPEN.

---

## RT-MISS

### RT-MISS-001 — MUST

Missing **required** mapping for the requested profile+method is a **hard failure**. Retarget must not invent motion for those joints.

### RT-MISS-002 — SHOULD

Unmapped optional source deform joints produce a loss/warning. Unmapped target helpers follow policy (rest / reconstruct / omit).

### RT-MISS-003 — MUST

Missing animation channels on mapped joints follow **source** fallback semantics (W0.1 CR-MOTION-003), then retarget policy — never silent identity unless the source or policy requires identity.

---

## RT-IK

Deterministic IK / constraint correction is a **candidate method family**. It is not AI.

**Automatic Foot IK** remains explicitly non-core under current [V1_SCOPE.md](../../product/V1_SCOPE.md). That feature is not the same as “all deterministic IK / constraints.”

Whether V1 Core requires no IK, limited deterministic IK, or optional IK correction is **OPEN** pending W0-P real-asset evidence. W0.2 must not preclude a later finding that a limited constraint stage is necessary for acceptable V1 behavior.

### RT-IK-001 — MUST

If IK / constraints are used, the stage MUST be:

- explicit
- versioned
- policy-declared
- diagnosable
- and MUST NOT silently mutate authoritative Canonical Motion

It is not an AI backend.

Evidence: Unreal can run IK after FK as a declared op [OFFICIAL_DOC]. Architecture: deterministic-first. This requirement does **not** say the stage must be architecturally optional.

### RT-IK-002 — OPEN

Whether V1 Core requires no IK, limited deterministic IK, or optional IK correction is decided only after W0-P evidence.

Automatic Foot IK remains non-core under current V1_SCOPE. That statement is a product-scope fact, not a proof that every constraint stage is optional.

### RT-IK-003 — OPEN

Full-body HumanIK-class or spacetime Gleicher solvers are not selected. GMR/SOMA are references / optional backend candidates, not Core dependencies. AI retarget remains out of V1 Core (V1_SCOPE).

---

## RT-DET

### RT-DET-001 — MUST

Same Canonical inputs + mapping + compatibility policy + retarget policy + implementation version must yield **repeatable** output within a documented numeric tolerance (tolerance value OPEN).

### RT-DET-002 — SHOULD

Sources of non-repeatability to record: solver iteration count/order, IK convergence, parallelism, platform float mode.

### RT-DET-003 — OPEN

Numeric epsilon and platform policy are OPEN. W0-P must not claim bit-identical GPU IK.

---

## RT-FAIL

### RT-FAIL-001 — MUST

Fail closed (no silent invented motion) when: required mapping missing; mapping ambiguous without confirmation; incompatible root semantics for the requested policy; invalid rest transforms; policy cannot perform the requested operation.

### RT-FAIL-002 — SHOULD

Distinguish hard failure, warning/risk, manual confirmation, and optional omission. Do not freeze error codes here.

### RT-FAIL-003 — MUST

Derived retarget output must be marked derived (W0.1 CR-PROVENANCE-001). Policy identifiers and versions are provenance.

---

## RT-TIME

### RT-TIME-001 — MUST

Retarget must preserve or explicitly convert the Motion time domain with provenance (W0.1 CR-MOTION-007). It must not assume FBX frame/fps or USD TimeCodes-as-seconds.

---

## Algorithm freeze

No single rotation, scale, root, or IK algorithm is selected as the RigForge law. The requirements above constrain what any candidate must *declare* and *not* do.

**Real-asset evidence principle:** no Retarget method is promoted to frozen V1 Core solely from documentation or reference-system behavior. W0-P real-asset results are decision evidence.

For each major candidate (rest-aligned FK baseline, chain transfer, twist handling, translation, proportion scale, root motion, IK/constraint correction), later tests must record:

```text
EXPECTED
OBSERVED
GAP
ROOT CAUSE
DECISION IMPACT
```

and a recommendation:

```text
CONTINUE
ADJUST
RESEARCH MORE
REDESIGN
DROP / DEFER
```
