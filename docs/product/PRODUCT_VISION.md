# Product Vision

Working product vision. Object names below are expected and subject to W1 freeze. No fields are defined here.

## Problem

Character / Skeleton / Motion assets arrive from different sources:

- FBX
- glTF / GLB
- USD
- Blender
- Maya
- Unreal
- Unity
- other tools

Those sources do not share one semantic contract for:

- Character
- Skeleton
- Motion
- joint semantics
- coordinate conventions
- retarget compatibility
- validation
- publish provenance

## Intended Architecture

High-level only. Process and package boundaries are not frozen.

```text
Workbench UI / CLI / Automation
             ↓
         Domain API
             ↓
 Mapping / Compatibility / Retarget / QC
             ↓
       Canonical Domain
             ↑
          Adapters
```

Adapters, interchange formats, and DCC/engine representations are derived or boundary layers. They are not the Canonical Domain.

Unreal, Unity, Blender, Maya, MotionBuilder, and other tools are **reference systems**, **adapter targets**, and **optional integrations**. They are not Canonical authority and not a required runtime, preview engine, ingest host, or Workbench host.

## Long-Term Canonical Objects

Expected / subject to W1 freeze:

- Character
- Skeleton
- Motion
- Manifest
- Semantic Mapping
- Compatibility Result
- Retarget Request / Result
- Validation / QC Result
- Publish Manifest
- Asset Reference / Provenance

V1 working boundary: [V1_SCOPE.md](V1_SCOPE.md).

## Engine-Independent Preview

RigForge must ultimately provide a standalone, engine-independent way to
inspect and preview Character, Skeleton, and Motion assets.

A browser-like viewer is a preferred product direction, but the rendering
technology is not yet selected.

The preview path must not require Unreal Engine, Unity, Blender, Maya, or
another specific DCC/game engine to render Canonical RigForge assets.
