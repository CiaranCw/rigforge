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
