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
W0: ACTIVE
W0-SR: COMPLETE / ADOPTED / BASELINED
POC-CORE-01: COMPLETE / PASS / BASELINED
POC-FBX-01: COMPLETE / PASS / BASELINED
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
POC-PREVIEW-01R: READY / NOT STARTED
W0-RS: NOT STARTED
IA-1: NOT STARTED
product implementation: NOT STARTED
```

W0-SR is **COMPLETE / ADOPTED / BASELINED**. POC-BLENDER-E2E-01 is
**COMPLETE / PASS / BASELINED** (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`).
No product implementation has started. The next experiment is
**POC-PREVIEW-01R**. Execute it only under its dedicated experiment prompt /
contract. Do not execute it from this README.

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
POC-BLENDER-E2E-01 with guards. It is not product authority. Product
implementation remains unauthorized until POC-PREVIEW-01R, W0-RS, and IA-1
complete the remaining required gates.

## Roadmap

[docs/development/ROADMAP.md](docs/development/ROADMAP.md)
