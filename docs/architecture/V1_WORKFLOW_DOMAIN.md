# V1 Workflow Domain

Current V1 conceptual model. Not a production schema or API.

The domain is **thin** and **`VALIDATED_WITH_GUARDS` by POC-BLENDER-E2E-01**:
the tested vertical slice ran the product path without leaking Blender
semantics into durable Job / Mapping / Policy / QC / Derived Variant state.
Derived Preview is **`VALIDATED_WITH_GUARDS` by POC-PREVIEW-01R**.

Exact production schema remains **OPEN**. Future worker replaceability is
architecturally preserved but not multi-backend-demonstrated. Non-humanoid
coverage is not yet real-E2E validated.

Decision: [ADR-0001](decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Worker: [V1_BLENDER_BACKED_ARCHITECTURE.md](V1_BLENDER_BACKED_ARCHITECTURE.md).
Rationale (historical): [V1_WORKFLOW_DOMAIN_PROPOSAL.md](V1_WORKFLOW_DOMAIN_PROPOSAL.md).

## Authority split

```text
Source representations                 RigForge authority
----------------------                 ------------------
FBX / GLB / .blend / later formats --> asset / version identity
                                       source Skeleton reference
                                       Skeleton Summary (derived evidence)
                                       Mapping + evidence
                                       Compatibility
                                       Retarget Policy
                                       job + execution record
                                       QC / validation meaning
                                       Derived Variant provenance

Blender scene / evaluated poses  -->   ephemeral execution state
Preview / export files           -->   rebuildable derivatives
```

Exact names, fields, and storage remain open.

## Concepts

### Asset / Asset Version

Catalog identity and version history. Authoritative for RigForge identity,
not for source-format semantics. Does not own decoded mesh arrays, all
source curves, a DCC scene, or an engine object.

### CharacterAsset

Selectable already-rigged target: mesh + skeleton/armature + skin weights.
Not a raw static mesh and not an Auto-Rig request. Does not require a
duplicate of all mesh and skin data merely to enter the catalog.

### MotionAsset

Selectable clip plus Source Skeleton context, time-domain provenance, and
fallback/missing-channel semantics as needed. Several Motions may share one
Source Skeleton. Must not silently equate FBX `FbxTime`, USD TimeCode, and
glTF seconds, or replace missing channels with identity transforms.

### SourceSkeletonReference

Binding/reference relation so Motions need not duplicate skeleton bytes.
Does not own Blender armature objects or humanoid-only slots.

### SkeletonSummary

**Derived evidence**, rebuildable from a source Asset Version under a
recorded inspector/backend. Candidate contents: source-local joint keys,
display names, hierarchy, roots, rest/base-pose evidence, orientation
evidence, deforming/helper classification, chain candidates, skin-binding
presence, units/axes, extraction diagnostics. Rest, bind, inverse-bind,
geometry-bind, and default transforms stay conceptually distinct.

Does not own all mesh data, every skin weight, FBX pivot recipes, every
curve, full constraint networks, or animation evaluation.

### BoneMapping

Authoritative accepted correspondence. More than `Map<string,string>`:

- joint-to-joint where valid
- chain-to-chain with unequal counts
- source-only, target-only, optional, helper, twist, IK/control, end, unmapped
- roles/profile namespace without requiring humanoid roles
- evidence: name/alias, hierarchy, orientation/rest, deform status, profile,
  competing candidates, rule/model version
- manual confirmation and overrides
- future AI suggestions as non-authoritative candidates

Names are evidence, not identity. Mapping is distinct from how motion is
distributed across different chain counts.

### CompatibilityResult

Derived product judgment for exact Character, Motion, Mapping, policy, and
worker capability. Layers include mapping completeness, structural/semantic
compatibility, method/policy eligibility, Motion suitability, risks, and
confirmation needs.

```text
MAPPING_COMPLETE
  != RETARGET_COMPATIBLE
  != RESULT_ACCEPTABLE
```

Illustrative UX (not frozen): Ready, Ready with warnings, Mapping
confirmation required, Unsupported.

### RetargetProfile / RetargetPolicy

Authoritative product intent, backend-neutral. Families include rest
alignment, root/trajectory, translation, scale/proportion, twist/helper
treatment, chain treatment, IK capability, bake/sample policy, time-domain
conversion, missing-channel fallback, quaternion normalization/continuity.

Does not own Blender operator names, constraint node types, or one universal
formula. Solver math is not specified here.

### RetargetJob / Worker Job Spec

Durable orchestration. The Job Spec is a versioned command derived from the
job. It is not an interactive Blender session.

### DerivedVariant

First-class result. A version records target Character/version, source
Motion/version, Source Skeleton, Mapping/version, Retarget Policy/version,
Validation result, Preview Artifact, generation/version metadata, and
backend/build. A full exported character file may be one representation; it
is not the product object.

### ValidationReport

Recorded checks and policy context. Worker measurements are not the policy.

```text
execution success
  != structural validity
  != QC quality
  != production acceptability
```

QC **MUST NOT** silently mutate or repair its subject.

V1 structural examples: NaN/Inf, required mapped bones, duration/time-domain
sanity, expected baked animation, gross invalid scale/transform, basic
root-trajectory sanity. Foot contact, sliding, artistic acceptance, and
complete discontinuity policy are later hardening.

### PreviewArtifact

Thin derived concept. **DERIVED / REBUILDABLE / NON-AUTHORITATIVE.** Bound to
exact Product / Version lineage. Payload identity / integrity is recorded
before valid display. Regeneration does not alter Product identity.
Deleting a Preview Artifact does not change Character, Motion, Derived
Variant, Mapping, Retarget Policy, or QC identity.

Product truth validates Preview. Preview never validates Product truth.

```text
ProductRef
!=
PreviewArtifactRef
!=
payload path
```

POC-PREVIEW-01R validated these properties for one research Character /
Motion / Derived Variant set. Exact production schema remains **OPEN**. This
is not a glTF / GLB schema and not a viewer-library selection. Does not own
identity, Mapping, QC thresholds, or source truth.

### ExportArtifact (optional)

Optional derived concept. Multiple versioned Export Profiles are `DEFER` /
implementation-gated. Direct engine integrations are `DROP_FROM_V1`.

## What V1 does not need to copy

Unless later evidence proves otherwise, RigForge does not own complete copies
of mesh geometry arrays, all skin-weight arrays, all FBX authored semantics,
all animation curves, a complete animation evaluator, a custom IK solver, or
a custom retarget runtime.

POC-FBX-01 shows many of those facts are inspectable with ufbx. That does
not make them a V1 Canonical persistence requirement.

Reversal: if E2E work shows mapping, compatibility, QC, or backend
replacement cannot work without a richer product-owned fact, add **that
fact**. Do not default back to a universal DCC clone.

## Invariants

1. Blender identifiers never become stable RigForge identities.
2. Worker output cannot create or alter accepted Mapping without an explicit
   RigForge command.
3. Every worker run records source hashes, backend/build, Job Spec, policy,
   relevant environment, and diagnostics.
4. Capabilities and expected losses are checked before work starts.
5. Source and derived artifacts are visibly distinct.
6. Motion playback/retarget requires resolved source Skeleton context.
7. Humanoid profiles are optional Mapping profiles, not the Skeleton model.
8. A future worker must implement the Job Spec without pretending to be Blender.
9. AI cannot silently accept Mapping, mutate Derived Variant lineage, or
   declare a result acceptable.
10. Exact storage technology and production schema remain deferred.
