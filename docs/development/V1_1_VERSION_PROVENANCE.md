# V1-1 Version and Provenance

Authoritative V1-1 rules for identity, versioning, artifacts, and lineage.
Implementation: `domain/` (`rigforge_domain`).

Status: `COMPLETE / PASS / BASELINED`. Gate A: `PASS / CLOSED`.
Core language: Rust (`ADR-0002` **Accepted**).

## Schema version vs entity version

```text
schema_version = 1
```

is the Domain contract version. It is not a Character/Motion/Derived version
id and not an artifact instance id.

Unsupported schema versions fail closed. No migration engine ships in V1-1.
A later schema must be introduced as a new integer with an explicit reader
policy.

## ID strategy

**Decision:** RFC 9562 UUIDv7, locally generated, canonical hyphenated
lowercase serialization.

Rationale:

- locally generatable, no registry
- collision-safe for V1 catalog scale
- portable across processes and future storage
- not a filesystem path
- not a DB autoincrement
- time-ordered, which helps later catalog locality without selecting a database
- distinct from SHA-256 text (32-nibble UUID vs 64 hex digest)

Parsed IDs reject nil UUID, non-v7 UUID versions (including UUIDv4), paths,
and 64-hex digests. Generation uses `Uuid::now_v7()` only.

Typed IDs (`CharacterAssetVersionId`, `JobSpecId`, …) are not interchangeable
at the type level.

## Lifecycle

Lifecycle is not encoded in the ID.

```text
DRAFT        in-place replace allowed
READY        frozen candidate; no in-place replace
PUBLISHED    immutable; no in-place replace
INVALIDATED  immutable; must not be used as new Job input
```

V1-1 implements Draft → Published (and Ready as a representable frozen
state). No larger workflow engine.

Published Product versions are immutable. Edits create a new version. Logical
objects may update `published_version_id` to the new version (supersession).
Old published versions remain.

Draft in-place replace keeps the same version id. Published in-place replace
is forbidden. Public fields are not a mutation path: Published records expose
getters only; Draft-only mutators fail with `ImmutableVersion`. `publish()`
validates the complete local record, then applies the allowed lifecycle
transition. Invalid Draft cannot become Published. Published cannot return to
Draft or Ready.

## Source Skeleton stability (Rev1 option A)

`MotionAssetVersion` binds `SourceSkeletonReferenceId`. The referenced
`SourceSkeletonReference` is immutable once constructed. V1-1 does not
introduce a Source Skeleton version runtime. A persisted Motion version
cannot silently change historical Source Skeleton meaning because another
caller mutated the referenced record in place.

## Regeneration

| Event | Product version | Artifact |
| --- | --- | --- |
| Edit accepted Mapping after publish | new `BoneMappingVersion` | none |
| Re-run transfer with same exact inputs and new worker observation | new `DerivedVariantVersion` if publication lineage is new | new artifacts as produced |
| Rebuild Persistence bytes for an existing DerivedVariantVersion | **no** new Product version | new `PersistenceArtifactInstanceId` and digest |
| Rebuild Preview payload | **no** new Product version | new Preview artifact instance/digest; Product ids unchanged |
| Delete Preview | Product identity unchanged | Preview record may disappear |

```text
DerivedVariantVersion != PersistenceArtifact
ProductRef != PreviewArtifactRef != payload path
```

W0-RS retained limitation (E2E persistence bytes vs Preview fixture bytes
NOT ESTABLISHED) is represented structurally: Product version identity is
independent of representation bytes.

## Exact-version provenance

Durable Job / Derived lineage MUST NOT store `latest` / `current`.

UI may later display “latest”. Resolution to exact version ids happens before
execution and is what gets persisted.

Every `DerivedVariantVersion` binds:

- CharacterAssetVersion
- MotionAssetVersion
- SourceSkeletonReference
- BoneMappingVersion
- RetargetPolicyVersion
- JobSpec
- `backend_id` (one BackendExecutionContext identity, shared with WorkerResult)
- WorkerResult
- QCReport
- PersistenceArtifact (required at publication)
- PersistenceVerificationId (bound at publication; exact authorizing verification)
- PersistenceVerification (immutable evidence: instance + digest + fresh reopen + structural)
- zero or more PreviewArtifacts

The graph is representable without querying Blender state. Graph validators:

- `validate_job_inputs`
- `validate_execution_lineage`
- `validate_publication_lineage`

No `latest` / `current`. JobSpec exact-version fields must equal the Derived
lineage. WorkerResult.job_spec_id must equal JobSpec.id.
WorkerResult.execution.id must equal DerivedVariantVersion.backend_id.

## Source artifact evidence

Imported sources record:

- location evidence (path/URI; not identity)
- SHA-256
- size
- observed media type
- optional ingest time
- optional adapter evidence

Complete source-format semantics are not copied into the Domain.

## Artifact identity / integrity

PersistenceArtifact has:

- artifact identity (`PersistenceArtifactId`)
- instance identity (`PersistenceArtifactInstanceId`) — new on regeneration
- payload digest and size
- media type
- producer/backend/build
- exact `DerivedVariantVersion` binding

PreviewArtifact has its own id, exact Product version binding, digest/size,
producer, and the flags derived=true, rebuildable=true, authoritative=false.

A digest must never equal a Product id. Payload path is location evidence.

## Publication gate

```text
validate complete local records
        ↓
validate allowed lifecycle / exact JobSpec ↔ Derived graph
        ↓
worker success
        AND
staged required Persistence digest
        AND
Persistence Artifact instance bound
        AND
fresh-process reopen PASS
        AND
structural verification PASS
        AND
QC PASS on this DerivedVariantVersion
        AND
QC WorkerResult + Policy exact bindings
        ↓
DerivedVariantVersion may become PUBLISHED
```

`publish_derived_variant` validates the publication graph, binds
`DerivedVariantVersion.persistence_verification_id` to the exact authorizing
`PersistenceVerification`, then freezes Published. After publication the
version record reconstructs which Persistence Artifact, instance, digest,
fresh-reopen result, and structural-verification result authorized that
publication. Regenerating Persistence bytes (new instance id + digest) cannot
retroactively change that reference.

V1-1 represents this evidence; it does not execute worker, persist, reopen,
or QC algorithms. WorkerResult alone cannot publish. QC cannot silently
mutate the subject. An old PersistenceVerification cannot apply to
regenerated bytes (new instance id and digest).

## Mapping / Policy versions

Same immutability rule as other Product versions. Mapping versions bind the
exact Character version and Source Skeleton reference they correspond to.
Policy versions contain structured backend-neutral enums, not worker
operators and not free-form strings.

## V1-2 expectations

V1-2 stores immutable version records and artifact metadata, and resolves
logical object → versions. It must not reinterpret “latest” inside stored
Jobs. It must not treat SHA-256 or paths as Product IDs.
