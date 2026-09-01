# rigforge_app

V1-2 application crate: local SQLite Asset Catalog, schema migration,
orchestration, WorkerPort, and Workbench queries.

This crate is not a Blender worker, Preview generator, Mapping algorithm, or
GUI framework. The Workbench shell lives in `../workbench`.

```text
GUI → Application → Catalog / Orchestrator → WorkerPort → (V1-3 worker)
```

Domain records enter persistence only as `Validated<T>`.
