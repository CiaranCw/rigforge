# V1-2 Catalog Contract

Local-first Asset Catalog and persistence contract. This is Product
infrastructure, not an enterprise AMS, and not source-format authority.

Status: `COMPLETE / PASS / BASELINED`

Storage decision: [ADR-0003](../architecture/decisions/ADR-0003-v1-local-catalog-storage.md)
(`Accepted` at V1-2 focused review).

Domain records remain defined by [V1_1_DOMAIN_CONTRACT.md](V1_1_DOMAIN_CONTRACT.md).

## Version axes (do not collapse)

| Axis | Current value | Owner |
| --- | --- | --- |
| Domain contract / serialization schema | `schema_version = 1` | V1-1 `rigforge_domain` |
| Database schema / migration version | `db_schema_version = 1` | V1-2 `rigforge_app` |
| Product entity version | `CharacterAssetVersion`, `MotionAssetVersion`, … | V1-1 records |

Loading an unsupported **Domain** schema fails at the Domain boundary
(`UnsupportedSchemaVersion`). Loading an unsupported **database** schema
fails in the catalog (`UnsupportedDbSchema`). Neither is silently
reinterpreted.

## Ingress

```text
serialized / raw input
        ↓
Domain validation
        ↓
Validated<T>
        ↓
Catalog transaction
```

Illegal:

```text
store(String, String)
raw JSON as the persistence API
database row ID as Product ID
filesystem path as Product identity
```

The catalog implements `ProductVersionStore` by consuming `Validated<T>` and
returning `Validated<T>` after load-time Domain validation.

## Identity

Externally durable references are V1-1 typed UUIDv7 IDs.

SQLite `rowid` / `location_rowid` may exist internally. They are never:

- Product IDs
- Product Version IDs
- Artifact IDs

`JobRun.run_id` and `attempt_id` are orchestration identifiers (also UUIDv7
text). They are **not** Product IDs. Jobs still reference exact `JobSpecId`
and exact Character/Motion/Mapping/Policy version IDs.

## Logical object → versions

Supported:

```text
logical object ID  →  all known version IDs
logical object ID  →  current published version pointer (if bound)
```

Persisted Jobs and provenance continue to use **exact version IDs**. The
catalog will not store `Job.character = latest`.

## Immutable published versions

From the Product perspective:

| Operation | Result |
| --- | --- |
| store a new version | allowed |
| load an exact version | allowed |
| replace a Published / Ready / Invalidated version in place | forbidden |

Idempotent re-store of the *same* payload is allowed. Draft versions may be
replaced in place (`Lifecycle::Draft`). Logical objects may update published
pointers without mutating historical version rows.

PersistenceArtifact **instances** are append-only: a regenerated artifact
keeps `PersistenceArtifactId` and inserts a new `instance_id` row. Historical
instance metadata is not overwritten.

## Artifact metadata

Persisted separately from Product identity. Minimum fields (Domain record):

- Artifact ID
- Artifact instance ID
- Product-version binding (`bound_derived_variant_version_id`)
- digest
- size
- media type
- producer / backend context id
- optional location evidence on the Domain record

V1-2 additionally stores catalog-side **payload location overlays**. A moved
file may be recorded there without changing Product IDs and without mutating
a Published version payload.

Binary payloads are **not** stored as SQLite BLOBs.

## Location evidence

Paths remain location evidence. The catalog tolerates:

```text
same Product version
payload observed at another path
```

without changing Product identity. There is no relocation service in V1-2.

## Transactions

Operations that would leave an invalid graph use a SQLite transaction:

- logical object + new version
- JobSpec + JobRun enqueue (after `validate_job_inputs` on the exact Catalog graph)
- WorkerResult + JobRun `SUCCEEDED` transition
- Artifact metadata + location overlay (when written together)
- PersistenceVerification with its subject ids (as stored Domain records)

A failing callback rolls back.

## Queries (minimal, no DSL)

| Query | Returns |
| --- | --- |
| list Character assets | logical Character rows + basic metadata |
| list Motion assets | logical Motion rows |
| list Derived Variants | logical Derived rows |
| load logical Asset | `Validated` logical record |
| load exact AssetVersion | `Validated` version record |
| list versions for an Asset | version ID list |
| lookup SourceSkeletonReference | exact record |
| lookup exact MappingVersion | exact record |
| lookup exact RetargetPolicyVersion | exact record |
| load JobSpec | exact immutable intent |
| load artifact metadata | latest instance or listed instances |
| load PersistenceVerification | exact record |

Asset Browser uses the list queries. No search language is provided.

## Local-first

Open, migrate, store, query, and reopen work on a local file. No cloud
account, remote database, or network is required for those operations.

## Fail-closed

- unknown Domain `schema_version` → Domain error on load
- database `user_version` newer than this binary → catalog error
- `user_version == 1` with missing required objects (`schema_migrations`,
  `records`, `payload_locations`, `job_runs`, declared indexes, or the v1
  migration row) → catalog error at `SqliteCatalog::open`
- empty → v1 migration is a single SQLite transaction; failure rolls back
- corrupt / non-SQLite file → open error
- missing parent directory → open error (catalog does not create arbitrary
  missing trees)
