# V1-4 Mapping Workflow

Product Mapping workflow. Names are evidence, not identity. Blender bone
names are not RigForge Product identity.

Status: `COMPLETE / PASS / BASELINED`

## Authority

Existing Domain records remain authority:

```text
BoneMapping / BoneMappingVersion / BoneMappingEntry
JointRef / JointKey / JointParticipation
UnmappedJoint
MappingReviewProvenance
SkeletonSummary / JointObservation
```

A Mapping is not `HashMap<String,String>`. Automatic output is a candidate
until an explicit Product accept/publish operation.

## SkeletonSummary

Derived, rebuildable evidence. Not a canonical skeleton runtime.

Subject binding:

```text
Character summary → exact CharacterAssetVersion
Source summary    → exact SourceSkeletonReference
```

A rebuilt summary receives a new `SkeletonSummaryId`. Historical Mapping
provenance may record the exact summary IDs used at acceptance. Old summaries
must not be treated as evidence for a different source digest/version.

Inspect path (when Blender is used):

```text
isolated Blender inspect
        ↓
backend-neutral observation envelope
        ↓
Validated<SkeletonSummary>
```

The application consumes Domain summaries, not `bpy` structures. The W0
research harness is not production runtime.

## Candidates

Deterministic first-pass signals (inspectable, not a magic score):

```text
EXACT_NAME
NORMALIZED_NAME
ALIAS
PARENT_CONTEXT
ROLE_HINT
ROOT_POSITION
DEFORM_EVIDENCE
REST_EVIDENCE
```

Normalized names are matching evidence only. They are not `JointKey` and not
Product identity. Original display names are preserved.

Optional humanoid aliases may assist the frozen V1 pair. They are an optional
profile, not a mandatory Skeleton schema. Non-humanoid skeletons must map
without hips/spine/arm/leg slots.

## One-to-one (V1 simple path)

```text
one source joint must not silently map to multiple targets
one target joint must not silently accept conflicting required sources
```

Ties become confirmation required, not first-match selection. This does not
claim a universal chain-to-chain solver.

## Participation

```text
required mapped joint
optional joint
helper / control / non-deforming evidence
unmapped but acceptable
unmapped and blocking
```

Not every bone must map. Helpers, fingers, and optional toe/ball joints may
remain unmapped without blocking completeness when participation is optional.

## Acceptance

```text
create mapping candidate
        ↓
if unambiguous: ready for explicit acceptance
if ambiguous: confirmation required
        ↓
user accepts / edits / overrides
        ↓
published BoneMappingVersion
```

Heuristic/worker/AI cannot silently publish Mapping. `BoneMappingVersion::publish`
requires `reviewed=true` and rejects `automatic_candidate`. A Published
record must carry an accepted `review_kind` that matches durable workflow
history (`generated_from_candidates` / `user_modified`):

```text
generated, not edited, explicit accept  → automatic_confirmed
generated, then override/unmap, accept  → user_override
not generated from candidates           → manual
```

Callers cannot choose the accepted kind. Untrusted JSON that publishes an
automatic candidate, or that disagrees with history, fails Domain validation.

V1 joint-to-joint Mapping is one-to-one: duplicate source or target JointKey
entries fail closed.

Published versions are immutable. A later correction is a new draft/version.

`UnmappedJoint.disposition` (`blocking` / `optional` / `helper`) is decision
authority. `reason` is explanation only and cannot change completeness.

`unmapped_source` may contain only `SkeletonSide::Source` records.
`unmapped_target` may contain only `SkeletonSide::Target` records. Wrong-side
JSON fails Domain validation.

Ready / ReadyWithWarnings require a `Lifecycle::Published` MappingVersion.
A reviewed Draft is still `MappingConfirmationRequired`.

`MappingReviewKind`:

| Kind | Meaning |
| --- | --- |
| `automatic_candidate` | generated, not accepted |
| `automatic_confirmed` | generated then explicitly accepted without entry edits |
| `user_override` | generated then edited, then accepted |
| `manual` | constructed by explicit joint choices |

Do not claim manual review when none occurred.

## Mapping vs Motion

A Mapping primarily binds `SourceSkeletonReference` → target
`CharacterAssetVersion`. Optional `source_motion_version_id` is Motion-specific
evidence and must not be required for reuse across Motions that share the
same Source Skeleton.

## Workbench

GUI → Application mapping workflow → Domain/Catalog.

Closing the application must preserve accepted Mapping records. Workbench
acceptance and override call Application operations and display the published
MappingVersion ID. Transfer remains unavailable in V1-4.

Reopened Mapping state is bound to the current exact selection:

```text
Mapping.target_character_version_id == selected CharacterAssetVersion
Mapping.source_skeleton_ref_id == selected Motion.source_skeleton_ref_id
```

No silent fallback to an unrelated first Catalog row. If several published
versions match, the greatest `BoneMappingVersionId` (UUIDv7 creation order)
wins.

`store_mapping_draft` binds summaries only when the source summary subject is
the selected `SourceSkeletonReference` and the target summary subject is the
selected `CharacterAssetVersion`. Reversed sides fail before persistence.
