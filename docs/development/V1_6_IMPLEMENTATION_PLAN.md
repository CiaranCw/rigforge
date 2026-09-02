# V1-6 Implementation Plan

Engine-independent Preview.

Status: `COMPLETE / PASS / BASELINED`

V1-5 remains `COMPLETE / PASS / BASELINED`. Gate C remains `PASS / CLOSED`.
V1-7 is `NOT STARTED / OPTIONAL`. V1-8 / Gate D are `NOT STARTED`.

Related:

- [V1_6_PREVIEW_ARTIFACT_CONTRACT.md](V1_6_PREVIEW_ARTIFACT_CONTRACT.md)
- [V1_6_VIEWER_CONTRACT.md](V1_6_VIEWER_CONTRACT.md)
- [V1_6_PREVIEW_GENERATION.md](V1_6_PREVIEW_GENERATION.md)
- ADR: [ADR-0006](../architecture/decisions/ADR-0006-v1-engine-independent-preview.md) (`Accepted` at V1-6 focused review / focused closure)
- V1-5 Product context: [V1_5_TRANSFER_LIFECYCLE.md](V1_5_TRANSFER_LIFECYCLE.md), [V1_5_QC_CONTRACT.md](V1_5_QC_CONTRACT.md), [V1_5_DERIVED_VARIANT_PUBLICATION.md](V1_5_DERIVED_VARIANT_PUBLICATION.md)

## Baseline

```text
branch: main
HEAD:   0141f0b32588a04b7073b3c451a6d7f6678a481a
```

This stage does not reverse accepted V1-1 through V1-5 semantics, Product
contracts, ADR-0001–0005, or Gate C. Independent Gate C reports remain
`GATE_C_PASS_CANDIDATE`. Project acceptance remains **PASS / CLOSED**.
`GATE-C-OBS-001` is not rewritten.

## Product transaction

```text
Product selection
        ↓
exact Product / Version resolution
        ↓
Preview request
        ↓
generation (if missing)
        ↓
PreviewArtifact + payload
        ↓
exact Product binding
        ↓
digest / size validation
        ↓
engine-independent viewer
        ↓
camera / playback
```

Kinds:

```text
CharacterAssetVersion
MotionAssetVersion
DerivedVariantVersion
```

Invariant:

```text
Preview = DERIVED / REBUILDABLE / NON-AUTHORITATIVE
Product truth validates Preview.
Preview NEVER validates Product truth.
```

Preferred relationship:

```text
PreviewArtifact → exact Product Version
```

No authoritative inverse pointer is written onto Published Product records.

## Non-goals

- mutate Published `DerivedVariantVersion` (including `preview_artifact_ids`)
- route Preview through Transfer JobRun / WorkerResult / QC / publication
- rerun Mapping / retarget / QC to generate Derived Preview
- require Unreal / Unity / Blender UI / Maya at view time
- use a runtime CDN
- advertise GLB as a V1-7 export
- close `GATE-C-OBS-001` without wiring Transfer native controls
- installer, license clearance, WebView bootstrap, GPU/browser matrix (V1-8)

## Source layout

| Path | Role |
| --- | --- |
| `domain/` | `PreviewArtifact` exact-one Product kind constructors and flags |
| `app/` | Catalog persist/resolve, Application generate/display, sealed Blender generator |
| `blender-worker/python/preview_gen.py` | Generation-time GLB producer |
| `workbench/preview-viewer/` | Offline local viewer surface |
| `workbench/` | Click-to-preview; sidecar host; Application retained for Preview |

## Lifecycle after this stage

```text
V1-5: COMPLETE / PASS / BASELINED
Gate C: PASS / CLOSED
V1-6: COMPLETE / PASS / BASELINED
V1-7: NOT STARTED / OPTIONAL
V1-8: NOT STARTED
Gate D: NOT STARTED
```

V1-7 starts only if a concrete Product consumer requires an output
artifact/export contract. A GLB Preview payload is not that requirement.
