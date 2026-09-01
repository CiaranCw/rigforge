# rigforge_blender_worker

V1-3 pinned isolated Blender worker adapter.

```text
Workbench
   ↓
Application / Catalog / Orchestrator
   ↓
WorkerDispatchRequest   (exact Catalog projection)
   ↓
WorkerPort::dispatch_resolved  (launch)
   ↓
Blender adapter
   ↓
python/worker.py in one isolated Blender process
   ↓
WorkerCompletionPort::collect  (wait + envelope + reopen)
   ↓
WorkerResult
```

This crate does not import `experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py`.
It does not call bpy from Rust, GUI, Domain, or SQLite schema.

GUI does not call Blender. Preview viewer/payload remain unselected.
No Derived Variant publication.
