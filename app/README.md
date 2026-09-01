# rigforge_app

V1-2/V1-3 application crate: local SQLite Asset Catalog, schema migration,
orchestration, WorkerPort launch + collect, and Workbench queries.

This crate is not a Blender worker, Preview generator, Mapping algorithm, or
GUI framework. The Workbench shell lives in `../workbench`. The pinned Blender
adapter lives in `../blender-worker`.

```text
GUI → Application → Catalog / Orchestrator → WorkerPort → blender-worker
```

Domain records enter persistence only as `Validated<T>`.
