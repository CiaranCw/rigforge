# V1-2 Implementation Plan

Local-first Asset Catalog, durable persistence, job orchestration, worker
dispatch *boundary*, and Workbench GUI direction.

Status: `COMPLETE / PASS / BASELINED`

This stage does **not** execute Blender. V1-3 owns the real worker.

Related:

- [V1_2_CATALOG_CONTRACT.md](V1_2_CATALOG_CONTRACT.md)
- [V1_2_ORCHESTRATION_CONTRACT.md](V1_2_ORCHESTRATION_CONTRACT.md)
- [ADR-0003](../architecture/decisions/ADR-0003-v1-local-catalog-storage.md) (`Accepted` at V1-2 focused review)
- [ADR-0004](../architecture/decisions/ADR-0004-v1-workbench-gui.md) (`Accepted` at V1-2 focused review)
- V1-1: [V1_1_DOMAIN_CONTRACT.md](V1_1_DOMAIN_CONTRACT.md)

## Baseline

```text
branch: main
HEAD:   9aacbef29666e7e0d41e99be622294c0043ca42b
```

V1-1 Domain crate remains the Product Domain foundation. V1-2 does not
rewrite Domain semantics, Product contracts, ADR-0001, ADR-0002, or W0/IA-1
evidence.

## Goals

1. Persist `Validated<T>` Domain records in a local catalog.
2. Query Characters, Motions, Derived Variants, exact versions, mappings,
   policies, JobSpecs, and artifact metadata.
3. Orchestrate job lifecycle separately from immutable `JobSpec`.
4. Expose a backend-neutral `WorkerPort` with a fake implementation.
5. Decide storage (SQLite) and GUI (egui/eframe) with review-pending ADRs.
6. Prove a Workbench shell initializes without network or Blender.

## Non-goals (later stages)

Blender subprocess, Blender Python worker, retarget execution, Auto-Mapping,
Mapping quality, Compatibility algorithm, QC algorithms, Preview generation
or viewer, export pipeline, engine integrations, MCP.

## Source layout

Cargo workspace at repository root:

| Path | Crate | Role |
| --- | --- | --- |
| `domain/` | `rigforge_domain` | V1-1 Workflow Domain (unchanged semantics) |
| `app/` | `rigforge_app` | Catalog, SQLite, migrations, orchestration, application queries, WorkerPort |
| `workbench/` | `rigforge_workbench` | Minimal egui/eframe shell |

The split exists because persistence/orchestration is not Domain, and a GUI
crate should not own catalog transactions. No empty V1-3…V1-8 trees.

## Application boundary

```text
Workbench GUI
        ↓
rigforge_app::Application
        ↓
SqliteCatalog  +  JobOrchestrator
        ↓
WorkerPort (fake in V1-2)
        ↓
(V1-3 Blender worker — not implemented)
```

Product invariants live in Domain + catalog/orchestrator, not only in GUI
widgets.

## Persistence

See ADR-0003. SQLite file, bundled amalgamation, WAL, `db_schema_version = 1`.

Normal operation requires no network.

## GUI

See ADR-0004. egui/eframe Workbench shell. Preview viewer/payload remain
OPEN for V1-6. The shell has a `PreviewEmbeddingSlot` that is not a viewer.

## Tests

- empty DB init and migration
- store/load validated Character and Motion versions
- logical object → versions
- published overwrite forbidden
- Product IDs ≠ SQLite rowid
- path overlay does not change Product identity
- artifact and PersistenceVerification metadata round-trip
- unsupported Domain schema fails closed on load
- transaction rollback
- job lifecycle, negative transitions, fake worker, JobSpec immutability
- restart/reopen on a real file
- Asset Browser queries return Character / Motion / Derived separately
- filesystem: missing directory, unwritable target, invalid schema
- optional real-lineage metadata from local Knight_Male / UAL2 files (read-only)

No fuzzing, no security testing, no Blender execution.

## Review

Ordinary focused review after this worktree. Not an independent Gate.
Next independent Gate remains Gate B after V1-2 + V1-3.
