# POC-BLENDER-E2E-01 research harness

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

Question: can Blender serve as a hidden execution backend for Character +
Motion → Derived Variant while durable RigForge semantics stay
backend-neutral?

This directory is not a production worker, not Auto-Mapping, not Preview
product code, and not a Core language selection.

```text
Python outside Blender + Blender Python inside the worker
!=
RigForge Core language = Python
```

## Layout

```text
experiments/w0p/poc_blender_e2e_01/
├── README.md
├── config/mapping_frozen.json
├── config/retarget_policy.json
├── schemas/
└── scripts/
    ├── inspect_asset.py
    ├── blender_worker.py
    └── run_e2e.py
```

Pinned Blender: **5.2.1 LTS** / hash `9e2066aef7ef` extracted outside git.

Real assets stay outside git:

```text
F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\...Knight_Male.fbx
F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\...UAL2_Standard.fbx
```

Evidence (external):

```text
F:\NewResearch\rigforge_w0p_evidence\poc_blender_e2e_01\
```

Report: [docs/research/poc/POC_BLENDER_E2E_01.md](../../../docs/research/poc/POC_BLENDER_E2E_01.md).

```text
POC-BLENDER-E2E-01: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_BLENDER_BACKEND_WITH_GUARDS
```

Inspect (Rev1): `run_e2e.py` always regenerates inspect JSON via
`blender_worker.py` mode=`inspect` from the source FBX + pinned Blender.
There is no hidden external inspect script.

Frozen quaternion Policy (Rev2): backend-neutral
`NORMALIZE_BEFORE_KEY` + `CONSECUTIVE_HEMISPHERE`. The Adapter validates
and executes those modes before key insertion. Interpolation remains
backend detail. Policy text does not name bpy / PoseBone / FCurve.

```text
python experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py
```
