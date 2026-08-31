# POC-FBX-01 research harness

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

Question: is **ufbx** a sufficiently semantically faithful, inspectable,
diagnosable and general FBX reader for RigForge V1 ingest?

This directory is not a production adapter, not Canonical, and not a Core
language experiment.

```text
C is the test harness language only.
This is not evidence selecting the RigForge Core language.
```

POC-CORE-01 remains `INCONCLUSIVE` / `PROVISIONAL_W0P_GATED`. This PoC does
not freeze C++.

## Layout

```text
experiments/w0p/poc_fbx_01/
├── README.md
├── MANUAL_L3_ASSETS.md
├── l3_run_config.json
├── harness/inspect_fbx.c
├── fixtures/
├── schema/inspection_output.md
└── scripts/
    ├── run_poc_fbx.py
    ├── fbx_name_scan.py
    └── build_harness.cmd
```

Pinned ufbx extract, build trees, binaries, and Level-3 bytes stay outside git:

```text
F:\NewResearch\rigforge_w0p_work\poc_fbx_01\
F:\NewResearch\rigforge_w0p_evidence\poc_fbx_01\
F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\
```

ufbx pin: **v0.23.0** / commit `fcc5d6ba444cfd3eb80677dba5e37e493941abe5`
(same extract as POC-CORE-01). Do not switch to master/latest.

## Level-3 assets

The runner **does not** access cloud drives or file-transfer hosts.
Place packs yourself: [MANUAL_L3_ASSETS.md](MANUAL_L3_ASSETS.md).

L3 operator archives were moved to
`F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\` and must stay out of git.

Report: [docs/research/poc/POC_FBX_01.md](../../../docs/research/poc/POC_FBX_01.md)
(status `COMPLETE / PASS`).

## Frozen Level-3 experiment variables

Recorded in [l3_run_config.json](l3_run_config.json). The repository runner
passes them to `inspect_fbx`. They are not Canonical anatomy.

| ID | File SHA-256 | Stack | Index | Samples |
| --- | --- | --- | --- | --- |
| QCHAR | `fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f` | `CharacterArmature\|Walk` | 15 | Bone, Hips, UpperArm.L, Head_end |
| QANIMAL | `91d5c31678fa291571f82abb029fde5c14fdd5932a1569c3afaa5e9602bebf94` | `AnimalArmature\|Walk` | 11 | Body, Torso3, FrontUpperLeg.L, Head_end |

Effective values are written to
`F:\NewResearch\rigforge_w0p_evidence\poc_fbx_01\run_config.json`.

If either L3 file is missing or the SHA-256 does not match, the runner
**STOP**s. Restore the operator-placed local file. Do not fetch a replacement
through a third-party file-transfer / cloud-drive / 网盘 host.

## Run (after operator L3 drop, or L1 only)

From a shell that can call VS 2022 `vcvars64`, using the same Python that can
import the stdlib only (example: `F:\miniconda\python.exe`):

```text
python experiments/w0p/poc_fbx_01/scripts/run_poc_fbx.py --l1-only
```

Reproducible Level-3 command (defaults read `l3_run_config.json`; local Knight
and Wolf paths are resolved from the operator asset tree):

```text
python experiments/w0p/poc_fbx_01/scripts/run_poc_fbx.py
```

Equivalent explicit form:

```text
python experiments/w0p/poc_fbx_01/scripts/run_poc_fbx.py ^
  --qchar-fbx "F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx" ^
  --qanimal-fbx "F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qanimal_extract\Ultimate Animated Animals - July 2021\FBX\Wolf.fbx" ^
  --qchar-stack-index 15 ^
  --qchar-sample-names Bone,Hips,UpperArm.L,Head_end ^
  --qanimal-stack-index 11 ^
  --qanimal-sample-names Body,Torso3,FrontUpperLeg.L,Head_end
```

The runner rebuilds `inspect_fbx.exe`, inspects L1 fixtures and the pinned
Maya L2 testcase, then runs Knight and Wolf **three times each** through
this same command path. It also runs `fbx_name_scan.py` (independent
structural cross-check; not Canonical) and writes `run_config.json`,
`l3_run_summary.json`, and `crosscheck_comparison.json`.

UTF-8 byte search of FBX bytes is classified as
**TOKEN-PRESENCE SECONDARY CROSS-CHECK**. It does not prove hierarchy,
transform, skin, bind, or animation evaluation.
