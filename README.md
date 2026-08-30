# RigForge

RigForge — Character / Motion Asset Workbench

## What RigForge Is

A planned workbench for Character / Skeleton / Motion assets across file formats, DCCs, and game engines.

Intended work:

- Ingest
- Normalize
- Canonical representation
- Inspect
- Validation / QC
- Semantic Skeleton Mapping
- Compatibility
- Deterministic Retarget
- Publish
- DCC / Engine integration
- Automation

Product definition: [docs/product/PRODUCT_VISION.md](docs/product/PRODUCT_VISION.md).

## What RigForge Is Not

- Not a full DCC
- Not a replacement for Blender or Maya
- Not a replacement for Unreal or Unity
- Not a system that treats FBX, glTF, or USD as Canonical Authority
- Not a product whose V1 depends on AI Retarget

## Current Status

```text
Status: Repository Initialization / W0 Research Baseline
```

The repository is initialized. Research Baseline R1 has not started. No ingest, inspect, mapping, retarget, or publish capability exists. Implementation language and GUI stack are not chosen.

## Core Principles

- Canonical-first
- Authoritative vs Derived separation
- Canonical vs Runtime separation
- Deterministic-first
- DCC-independent ingest
- Domain API before UI
- Adapters around the Canonical Core

Frozen-so-far architecture notes: [docs/architecture/README.md](docs/architecture/README.md).

## Roadmap

[docs/development/ROADMAP.md](docs/development/ROADMAP.md)
