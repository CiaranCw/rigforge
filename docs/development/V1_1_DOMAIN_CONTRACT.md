# V1-1 Domain Contract

Production Workflow Domain contract for V1-1. This is not a Blender schema,
not a catalog database, and not a GUI model.

Core language: Rust (`ADR-0002` **Accepted** at V1-1 Gate A). Crate: `domain/`.
Schema version: `1`.

Status: `COMPLETE / PASS / BASELINED`. Gate A: `PASS / CLOSED`.

Related:

- [V1_1_VERSION_PROVENANCE.md](V1_1_VERSION_PROVENANCE.md)
- [V1_1_IMPLEMENTATION_PLAN.md](V1_1_IMPLEMENTATION_PLAN.md)
- [ADR-0002](../architecture/decisions/ADR-0002-v1-core-language.md)

Accepted Product/Architecture contracts are unchanged. This file instantiates
them; it does not rewrite them.

## Transaction preserved

```text
Character + Motion
        ↓
Mapping / Compatibility
        ↓
Transfer
        ↓
Derived Variant
        ↓
Preview / Version
```

Character is already rigged/skinned. Motion carries animation, Source
Skeleton context, and time-domain provenance. Derived Variant is a first-class
versioned Product result.

## Logical object vs version

| Logical object | Version record | Stable across versions | New version when |
| --- | --- | --- | --- |
| `CharacterAsset` | `CharacterAssetVersion` | Asset id, catalog identity | New ingest/edit intended as history |
| `MotionAsset` | `MotionAssetVersion` | Asset id | New clip interpretation or source bytes intended as history |
| `BoneMapping` | `BoneMappingVersion` | Mapping id | Correspondence or review change intended as history |
| `RetargetPolicy` | `RetargetPolicyVersion` | Policy id | Intent change intended as history |
| `DerivedVariant` | `DerivedVariantVersion` | Variant id | New transfer/publication lineage |

Logical objects may point at `published_version_id` and `draft_version_id`.
Those pointers are not durable Job/provenance references. Jobs and Derived
lineage bind exact version IDs only.

## Identity

```text
Product identity != filesystem path
Product identity != source filename
Product identity != FBX identity
Product identity != .blend identity
Product identity != GLB identity
Product identity != SHA-256
```

IDs are RFC 9562 UUIDv7 values, generated locally, serialized as canonical
hyphenated lowercase text. They are not database row IDs.

`ContentDigest.sha256` is integrity/evidence only.

`LocationEvidence` is location/source evidence only.

## Concepts

Exact Rust names match this table.

| Concept | Kind | Notes |
| --- | --- | --- |
| Asset / AssetVersion | logical + version | Catalog identity, not source-format semantics |
| CharacterAsset / CharacterAssetVersion | logical + version | Already-rigged target |
| MotionAsset / MotionAssetVersion | logical + version | Clip + required `SourceSkeletonReferenceId` + time provenance |
| SourceSkeletonReference | logical binding | Shared across Motions; not Blender armature identity |
| SkeletonSummary | derived evidence | Hierarchy/name/rest/deform observations; not Skeleton authority |
| BoneMapping / BoneMappingVersion | logical + version | JointRef pairs, participation, evidence, review, unmapped lists |
| CompatibilityResult | derived judgment | Five separable dimensions + summary; not one boolean |
| RetargetPolicy / RetargetPolicyVersion | logical + version | Structured W0/E2E-proven enums; not free-form kind/detail strings |
| JobSpec | execution intent | Exact version bindings; product capabilities, not `bpy` |
| WorkerResult | execution observation | Success ≠ QC ≠ publication; diagnostics may mention backend facts |
| QcReport | derived evidence | PASS/FAIL, exact subject one-of, WorkerResult + Policy binding |
| DerivedVariant / DerivedVariantVersion | logical + version | Full exact-version lineage; `backend_id` not a duplicated context |
| PersistenceArtifact | derived representation | Own id + instance id + digest; ≠ DerivedVariantVersion |
| PersistenceVerification | derived evidence | Fresh reopen + structural verification bound to exact instance/digest |
| PreviewArtifact | derived representation | Derived/rebuildable/non-authoritative; exact Product version binding |
| ExportArtifact | optional derived | Never Product identity |
| BackendExecutionContext | execution provenance | Backend kind/version/build/adapter/policy; not Product identity |

## Mapping

A Mapping is not `dict<string,string>`. Each entry has:

- source `JointRef` (side + source-local `JointKey`)
- target `JointRef`
- required / optional participation
- optional role/profile namespace (not a mandatory humanoid skeleton)
- evidence
- mapping-level review + ambiguity provenance
- unmapped source/target lists

Auto-Mapping is not implemented.

## Compatibility

```text
mapping_completeness
!= structural_compatibility
!= method_eligibility
!= motion_suitability
!= result_acceptability
```

`CompatibilitySummary` may be Ready / ReadyWithWarnings /
MappingConfirmationRequired / Unsupported. That summary does not erase the
dimensions. `Ready` / `ReadyWithWarnings` cannot coexist with a
decision-critical FAIL dimension. Ingress and constructors reject
contradictory summaries.

## Retarget Policy

Product-owned intent. Decision-critical semantics are structured enums, not
free-form `kind`/`detail` strings. V1-1 represents the W0 / Blender E2E
proven set:

- `RootPolicy::CopyWorldTranslationDelta`
- `ChannelPolicy::RotationOnlyMappedNonRoot` (ROTATION_ONLY)
- `QuaternionNormalizationPolicy::NormalizeBeforeKey`
- `QuaternionContinuityPolicy::ConsecutiveHemisphere`
- `QuaternionInterpolationPolicy::BackendInterpolationAfterNormalizedKeys`
- `TimeBakePolicy::EverySourceFrame`
- `RestAlignmentPolicy::RestRelativeWorldDelta`
- missing-channel: unmapped target remain at rest; missing source joint fail closed / do not invent
- `ScalePolicy::KeepTargetRestScale`
- `IkPolicy::ExplicitNoIk`

Unknown serialized Policy tags fail closed. `PolicyContractStatus` keeps
recognized / supported / executed / audited distinct. V1-1 marks the proven
set recognized, supported, and audited, and not executed. Worker capability
negotiation and retarget execution are V1-3. Optional human explanation may
remain text. Structured schema is the primary backend-neutral boundary;
lexical denylist is defense-in-depth for Product fields (JobSpec, mapping
evidence), not for WorkerResult/QC diagnostics.

## JobSpec

Binds exact Character, Motion, Source Skeleton, Mapping, and Policy versions.
May request `persistence_artifact`, `preview_payload`, or `export_artifact`
capabilities. Must not require Blender scene names, Object datablock IDs,
PoseBone references, or `.blend` identity.

Backend translation belongs to V1-3.

## WorkerResult / QC

```text
worker success != QC success != persistence verification != publication
```

`publish_derived_variant` receives the publication graph: exact Character /
Motion / Source Skeleton / Mapping / Policy versions, JobSpec, WorkerResult,
QC PASS, Persistence Artifact, PersistenceVerification, and one
BackendExecutionContext identity. It cannot publish from only Derived +
Worker + QC PASS. Worker success alone is rejected.

QC must not mutate its subject. QC subject unions are exact-one-of.
Worker-backed QC requires `worker_result_id` and matching Policy version.
Advanced foot-contact QC is out of V1-1. Worker diagnostics may describe
backend-specific observations without becoming Product authority.

## Skeleton Summary

Derived, rebuildable evidence produced under a recorded backend context.
Not a universal Skeleton runtime.

## Time provenance

`TimePoint` is explicitly `seconds` or `frames` with rational values. Frame
points require fps numerator/denominator. Start and end must share a kind
and, for frames, the same fps rational. Start must be <= end. Missing-channel
semantics are required. Frames are not seconds.

## Serialization

Public Domain ingress is `from_json_validated` / `ingest_validated`: parse
untrusted JSON → deserialize (`deny_unknown_fields`) → schema/record-type
check → semantic `DomainRecord::validate` → `Validated<T>`. Raw serde of
Domain structs may still deserialize; those values cannot enter
`ProductVersionStore` without `Validated::certify`. `Validated<T>` is the
normal application / storage boundary and does not implement `Deserialize`.

JSON, `schema_version`, `record_type`, `#[serde(deny_unknown_fields)]`.

| Case | Behavior |
| --- | --- |
| Unknown field | fail closed |
| Missing required field | fail closed |
| `schema_version = 1` | accepted |
| unsupported `schema_version` | fail closed |
| `schema_version = 0` | fail closed |

Byte-identical JSON is not required. Semantic equality after
serialize → deserialize → validate is required.

Migration engine is not implemented. Future schemas must be explicit; V1-1
readers must not guess.

## Persistence interface (not implemented)

V1-2 must provide, technology-independently, a **typed/validated** store:

- store immutable `Validated<T: DomainRecord>`
- load exact `Validated<T: DomainRecord>` by version id
- resolve logical object → version ids
- store artifact metadata as `Validated<T: DomainRecord>`

A raw deserialized `T` is not a legal store argument. JSON strings are not a
legal store/load contract. No database is selected or implemented in V1-1.

## Immutability / constructors (Rev1)

Decision-critical Product/version/provenance records keep fields private.
Public API is read-only getters, validated constructors, Draft-only mutators
where needed, and lifecycle transitions that validate before freeze. Invalid
Drafts cannot become Published. Source Skeleton uses option A: the
`SourceSkeletonReference` record is immutable once created; Motion binds its
id. There is no Source Skeleton runtime.

Product IDs reject non-v7 UUIDs on generate, parse, `from_uuid`, and
deserialize. Content digest remains a separate type.

## Out of crate

Database, filesystem catalog, Blender launch, FBX parsing, Preview
generation/viewing, GUI, HTTP, RPC, MCP, Mapping algorithm, retarget solver,
scene graph, animation evaluator.
