# ADR-0003: V1 local catalog storage

Status: **Accepted**

Date: 2026-09-01

Accepted at: **V1-2 focused review** (`PASS`).

Owner stage: V1-2

This decision is storage infrastructure, not Product identity. It does not
replace V1-1 Domain IDs, and it does not select a cloud catalog, AMS, or
source-format authority.

## Context

V1-1 delivered a typed Workflow Domain (`rigforge_domain`) with `Validated<T>`
ingress and a persistence *interface* (`ProductVersionStore`). No database
was implemented.

V1-2 must make that Domain usable as a real local application foundation:

```text
local-first Asset Catalog
durable persistence
schema migration baseline
Product / version lookup
artifact metadata persistence
```

Hard constraints from V1-1 and the V1-2 stage contract:

- Product ID ≠ path, filename, digest, or database row ID
- logical Product ≠ Product Version
- Published versions are append-only from the Product perspective
- durable Jobs bind exact version IDs only
- raw serialized data → Domain validation → `Validated<T>` → catalog write
- Preview / Persistence artifacts are not Product authority
- normal catalog operations must work without network, cloud account, or a
  remote database

Database technology was OPEN after V1-1. This ADR is the owner decision.

## Requirements

Evaluated requirements (single-user local desktop Rust application):

| Requirement | Why it matters |
| --- | --- |
| ACID transactions | Partial writes must not leave an invalid catalog graph |
| Single-user local desktop | V1 is a workbench, not a multi-tenant service |
| Schema migration | Empty DB → current schema, with room for N → N+1 |
| Referential / graph integrity | JobSpec must exist before JobRun; versions must remain loadable |
| Crash recovery | Ordinary process kill / reopen must not lose committed records |
| Backup / copy | A catalog file (or small file set) must be copyable as a unit |
| Rust ecosystem maturity | V1 Core is Rust; the catalog crate must be maintainable |
| Cross-platform packaging | Windows is a V1 target; no mandatory system DB service |
| Inspection / debuggability | Engineers must inspect records without a proprietary tool |
| Binary / tooling burden | Avoid shipping a database server next to the workbench |
| Concurrency model | Single writer / short transactions is enough for V1 |
| Long-term maintenance | Prefer boring, widely understood technology |

Out of scope for this decision: worker execution, Preview payload storage as
BLOBs, multi-user replication, and any cloud catalog.

## Candidate set

Smallest serious set for a local desktop Rust workbench. No broad database
survey.

| Candidate | Class | Why it is in the set |
| --- | --- | --- |
| SQLite via `rusqlite` (bundled) | relational embedded DB | Default local-ACID candidate for desktop apps |
| redb | embedded transactional KV | Pure-Rust embedded store; realistic alternative to SQLite |
| PostgreSQL (local server) | client/server RDBMS | Serious relational option if embedded SQLite were insufficient |

JSON-on-filesystem was considered only as a rejected foil: it cannot provide
cross-record ACID, referential checks, or a migration story without inventing
a database.

## Decision

**Selected: SQLite**, accessed from Rust through **`rusqlite` with the
`bundled` feature**.

Database schema/migration version is **1** (`v1_2_catalog`). This is distinct
from:

```text
Domain contract version     schema_version = 1   (V1-1)
Database schema version     db_schema_version = 1  (this ADR / V1-2)
Product entity version      CharacterAssetVersion, JobSpec, ...
```

Catalog implementation crate: `app/` (`rigforge_app`).

Payload policy:

- persist Domain JSON that has already passed `Validated<T>`
- persist artifact *metadata* (id, instance id, digest, size, media type,
  producer, location evidence)
- do **not** store binary FBX/blend/GLB payloads as database BLOBs
- paths are location evidence, not identity; a catalog-side location overlay
  may record a moved payload without changing Product IDs

## Why SQLite satisfies the requirements

- **ACID / crash recovery:** SQLite transactions plus WAL journal are the
  ordinary local-desktop durability mechanism. V1-2 tests reopen-after-close
  on a real file, not only in-memory maps.
- **Single-user desktop:** one process, one catalog file. No server daemon.
- **Schema migration:** `PRAGMA user_version` plus a `schema_migrations`
  table. V1-2 implements empty → 1 and leaves N → N+1 as an explicit path.
- **Integrity:** relational tables, application-enforced Domain validation on
  every load, and transactions around logical-object + version / JobSpec +
  JobRun writes.
- **Backup/copy:** copy `*.sqlite` (and `-wal`/`-shm` if the process is live).
- **Rust maturity:** `rusqlite` is the standard SQLite binding. `bundled`
  compiles the SQLite amalgamation into the crate so packaging does not
  depend on a system `sqlite3.dll`.
- **Inspectability:** `sqlite3` CLI / DB Browser can read records. Domain
  payloads remain JSON.
- **Tooling burden:** no PostgreSQL install, no cloud account.
- **Concurrency:** SQLite writer lock is acceptable for a local workbench.
- **Maintenance:** SQL schema is reviewable without a specialized KV mental
  model.

## Alternatives considered

### redb

Pure-Rust, transactional, embeddable, no C compile step.

Rejected for V1:

- no relational schema / SQL inspection
- referential integrity and ad hoc Asset Browser queries become application
  code on a KV map
- weaker everyday debuggability than a `.sqlite` file
- migration story is less conventional than SQL DDL + `user_version`

Revisit if bundled C SQLite becomes a packaging blocker and a KV model is
enough.

### PostgreSQL

Strong ACID, SQL, migrations, and tooling.

Rejected for V1:

- requires a server (or a hidden local server) — not local-first in the
  “open the workbench, no extra service” sense
- backup/copy and packaging are heavier
- overkill for single-user desktop

Revisit only if V1 later requires concurrent multi-process writers that
SQLite cannot honestly serve. That is not a V1-2 requirement.

### JSON files / ad hoc directory catalog

Rejected: no ACID across JobSpec + JobRun, easy partial graphs, and path
layout would be mistaken for Product identity.

## Tradeoffs

- Bundled SQLite compiles C (`sqlite3.c`) with the platform C toolchain.
  That is a real packaging/build dependency, accepted because it removes a
  runtime `sqlite3` DLL requirement.
- SQLite is not a multi-master sync engine. V1 does not need one.
- Internal SQLite `rowid` values exist. They are **not** Product IDs,
  version IDs, or Artifact IDs. External durable references remain V1-1
  UUIDv7 values.
- Domain JSON in a TEXT column is slightly denormalized. This is deliberate:
  the catalog is not a second schema language. Load always re-enters Domain
  validation. An unsupported Domain `schema_version` fails closed.

## Migration implications

V1-2 migration:

```text
empty file / user_version 0
        →
db_schema_version 1
(records, payload_locations, job_runs, schema_migrations)
```

Later stages add N → N+1 migrations in `app` without rewriting Product IDs.

A catalog with `user_version` *greater* than this binary's supported version
fails closed. V1-2 does not silently read a future database schema.

Domain `schema_version` is checked by `rigforge_domain`, not by SQL CHECK
constraints substituting for Domain validation.

## Revisit triggers

Reopen this ADR if:

- a second process must write the catalog concurrently as a product requirement
- bundled SQLite C compilation becomes an unacceptable packaging cost
- a required query cannot be expressed without becoming a generic DSL
  (do not add a query DSL merely because SQL exists)
- evidence shows WAL file handling is unworkable for V1 users

Do not revisit merely because another embedded database is popular.

## Evidence

Implementation: `app/` (`rigforge_app`).

Contracts:

- [V1_2_CATALOG_CONTRACT.md](../../development/V1_2_CATALOG_CONTRACT.md)
- [V1_2_IMPLEMENTATION_PLAN.md](../../development/V1_2_IMPLEMENTATION_PLAN.md)

Related: [ADR-0002](ADR-0002-v1-core-language.md) (Rust Core, Accepted).
GUI is a separate decision: [ADR-0004](ADR-0004-v1-workbench-gui.md).
