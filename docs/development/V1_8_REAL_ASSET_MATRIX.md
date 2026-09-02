# V1-8 Real-Asset Matrix

Representative campaign. Not a universal support claim.

Status: `COMPLETE / PASS / BASELINED`

## Frozen regression (retained)

```text
Character:
Knight_Male.fbx
SHA-256:
fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f
source:
Quaternius Ultimate Animated Character Pack - Nov 2019
local POC-FBX-01 extract
pack page License = CC0 [SOURCE_CONFIRMED]

Motion:
UAL2_Standard.fbx
SHA-256:
d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d
clip:
Armature|Armature|Walk_Carry_Loop
source:
Quaternius Universal Animation Library 2 [Standard]
local POC-BLENDER-E2E-01 extract
pack page License = CC0 [SOURCE_CONFIRMED]
```

Transfer / QC / publication / Preview for this pair remain the V1-5 / V1-6
production checkpoints. V1-8 does not replace them.

## V1-8 diversity rows

All Motions below use the frozen UAL2 clip unless noted. Assets are local
accepted research extracts. No third-party file relay was used.

| Pair | Observed Mapping/Compatibility | Transfer |
| --- | --- | --- |
| Knight_Male + UAL2 | Workbench handlers completed; Compatibility `ReadyWithWarnings` | NO in campaign; YES in retained V1-5 checkpoint |
| Goblin_Male + UAL2 | 21 mapped / 44 unmapped source / 11 unmapped target; `ReadyWithWarnings` | NO |
| Mannequin_F + UAL2 | 65/65 mapped; `Ready` | NO |
| Horse + UAL2 | 6 mapped / 59 unmapped each side; confirmation required; `Unsupported` | NO (honest reject) |

Horse source: Quaternius Ultimate Animated Animals - July 2021; local
POC-FBX-01 extract; pack page License = CC0 [SOURCE_CONFIRMED].

## Non-humanoid objective

```text
synthetic non-humanoid Mapping/Preview architecture: SUPPORTED
real non-humanoid retarget quality: NOT YET PROVEN
```

V1-8 asks whether architecture stays generic and whether failure/ambiguity
is honest. Valid outcomes: successful retarget, Compatibility reject,
Mapping requires review, explicit unsupported.

A truthful controlled rejection is better than a false PASS.

Do not claim general Auto-Mapping quality from this corpus.

## Evidence

Campaign implementation: `blender-worker/tests/v1_8_real_assets.rs`.

JSON rows (observed/inferred distinguished in fields):
`review_evidence/v1_8_asset_matrix.json` after the campaign test runs.

## Supported / unsupported envelope

Supported for current V1 release candidate intent:

- already-rigged Character + Motion with Source Skeleton context
- pinned Blender 5.2.1 LTS isolated worker
- deterministic Mapping proposal + explicit accept
- Compatibility Ready / ReadyWithWarnings (acknowledgement) / honest reject
- Transfer on an authorized exact graph only
- QC Product-owned / non-mutating
- Preview derived GLB / model-viewer, non-authoritative

Unsupported / not claimed:

- Auto-Rig
- all FBX files
- all humanoids
- all non-humanoids
- all browsers / GPUs / Windows machines
- production animation aesthetics
- direct engine integration
- V1-7 export
