# RigForge

RigForge is a **Character Animation Asset Workbench**.

Users browse rigged Character and Motion assets, select one of each, run
animation transfer, and receive a previewable, versioned Derived Variant.

## What RigForge Is

A workbench for Character animation assets:

- browse and preview Character, Motion, and Derived Variant assets
- select one already-rigged Character and one Motion
- run automatic Mapping / Compatibility preflight
- Transfer on the Ready path without opening a Mapping editor
- receive a Derived Variant with QC, Preview, Version, and provenance

Character input is already rigged:

```text
Character Asset = mesh + skeleton / armature + skin weights
```

Motion carries animation data plus source Skeleton context. Multiple Motions
may share one Source Skeleton. The result is a first-class Derived Variant,
not a copied FBX character for every clip.

Product: [docs/product/PRODUCT_VISION.md](docs/product/PRODUCT_VISION.md).
Scope: [docs/product/V1_SCOPE.md](docs/product/V1_SCOPE.md).
Architecture: [docs/architecture/README.md](docs/architecture/README.md).
Decision: [ADR-0001](docs/architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).

## What RigForge Is Not

- Not a full DCC, and not a replacement for Blender or Maya
- Not an Auto-Rig product; raw unrigged mesh is not a V1 transfer target
- Not a replacement for Unreal or Unity; no V1 engine integration
- Not a system that treats FBX, glTF, USD, or Blender as product authority
- Not a product whose V1 depends on AI Retarget

## Current Status

```text
W0: COMPLETE / PASS / BASELINED
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-CORE-01: COMPLETE / PASS / BASELINED
POC-FBX-01: COMPLETE / PASS / BASELINED
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
W0-RS: COMPLETE / PASS / BASELINED
IA-1: COMPLETE / PASS / CLOSED
V1-1: COMPLETE / PASS / BASELINED
Gate A: PASS / CLOSED
V1-2: COMPLETE / PASS / BASELINED
V1-3: COMPLETE / PASS / BASELINED
Gate B: PASS / CLOSED
V1-4: COMPLETE / PASS / BASELINED
V1-5: READY / NOT STARTED
Gate C: NOT STARTED
```

W0-SR is **COMPLETE / ADOPTED / BASELINED**. POC-BLENDER-E2E-01 is
**COMPLETE / PASS / BASELINED** (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`).
POC-PREVIEW-01R is **COMPLETE / PASS / BASELINED**
(`ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`). W0-RS is **COMPLETE / PASS /
BASELINED**: [synthesis](docs/research/decisions/W0_REVISED_SYNTHESIS.md) and
[traceability](docs/research/decisions/W0_RS_TRACEABILITY.md). IA-1 is
**COMPLETE / PASS / CLOSED**:
[independent audit](docs/research/audits/IA_1_INDEPENDENT_AUDIT.md) and
[external review](docs/research/audits/IA_1_EXTERNAL_REVIEW.md). W0 is
**COMPLETE / PASS / BASELINED**. V1-1 is **COMPLETE / PASS / BASELINED**.
Gate A is **PASS / CLOSED**. Core language is **Rust**
([ADR-0002](docs/architecture/decisions/ADR-0002-v1-core-language.md)
**Accepted**). V1-2 is **COMPLETE / PASS / BASELINED**
(local catalog SQLite **Accepted** in [ADR-0003](docs/architecture/decisions/ADR-0003-v1-local-catalog-storage.md);
Workbench GUI egui/eframe **Accepted** in [ADR-0004](docs/architecture/decisions/ADR-0004-v1-workbench-gui.md)).
V1-3 is **COMPLETE / PASS / BASELINED**. Gate B is **PASS / CLOSED**.
Pinned Blender worker process packaging is **Accepted** in
[ADR-0005](docs/architecture/decisions/ADR-0005-v1-blender-worker-process.md).
V1-4 is **COMPLETE / PASS / BASELINED**. V1-5 is `READY / NOT STARTED`.
Do not open Gate C until after V1-5.

Historical W0.1–W0.4, POC-CORE-01, and POC-FBX-01 remain accepted research
evidence. They are not a mandate to ship a heavy Canonical runtime or a
native FBX/glTF stack in V1.

## V1 execution shape

```text
RigForge Workbench
        ↓
Application / Workflow Domain
        ↓
backend-neutral Job Spec
        ↓
Worker Boundary
        ↓
Blender Worker   (ACCEPT_BLENDER_BACKEND_WITH_GUARDS)
```

RigForge owns product semantics: assets, Mapping, Compatibility, Retarget
Policy, jobs, Derived Variant, QC meaning, Preview orchestration, and
versioning. Blender is a **hidden execution backend**, validated by
POC-BLENDER-E2E-01 with guards. It is not product authority. V1 work
proceeds only under the currently authorized dedicated implementation
stage. Current authorized stage is V1-5 (`READY / NOT STARTED`).
Gate C is NOT STARTED.

## Roadmap

[docs/development/ROADMAP.md](docs/development/ROADMAP.md)
