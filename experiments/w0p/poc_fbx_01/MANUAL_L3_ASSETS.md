# POC-FBX-01 Level-3 assets — operator placement only

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

This experiment does **not** fetch third-party cloud drives, file-transfer
hosts, or pack archives. Level-3 real assets are **operator-placed**.

## Official pack pages

Record access date when you download.

| ID | Pack | Pack page | License on pack page (re-check) |
| --- | --- | --- | --- |
| F-L3-QCHAR | Ultimate Animated Character Pack | https://quaternius.com/packs/ultimatedanimatedcharacter.html | CC0 1.0 (link to creativecommons.org/publicdomain/zero/1.0/) |
| F-L3-QANIMAL | Ultimate Animated Animal Pack | https://quaternius.com/packs/ultimateanimatedanimals.html | CC0 1.0 (same) |

Policy: `DOWNLOAD_DURING_TEST_ONLY`. Do not commit asset bytes.

## Drop directory (outside the git repository)

```text
F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\
```

Suggested layout:

```text
qchar_License.txt
qanimal_License.txt
qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx
qanimal_extract\Ultimate Animated Animals - July 2021\FBX\Wolf.fbx
```

Frozen Rev1 files (SHA-256 must match `l3_run_config.json`):

```text
Knight_Male.fbx
  fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f

Wolf.fbx
  91d5c31678fa291571f82abb029fde5c14fdd5932a1569c3afaa5e9602bebf94
```

If either file is missing, restore it from the operator-placed pack extract.
Do not fetch a replacement through a third-party file-transfer / cloud-drive /
网盘 host.

If you keep the vendor zip, place it beside the extracted tree and keep the
original filename. The runner will hash archives and selected FBX files.

## Minimum selection after extract

- One **humanoid** rigged + skinned + animated FBX from QCHAR
- One **non-humanoid** rigged + skinned + animated FBX from QANIMAL

Do not pick a file only because it is small or easy to parse.

Optional: the matching `.blend` from the same pack for the same character
(hierarchy / names / animation-presence cross-check). Blender is **not**
Canonical authority.

## What the agent will do after you place files

1. Hash archives and selected FBX (and LICENSE).
2. Produce asset records (source inspection facts, not Canonical).
3. Run the pinned ufbx C99 harness (3-run determinism).
4. Independent name/hierarchy scan (not ufbx).
5. Write evidence + `docs/research/poc/POC_FBX_01.md`.

The harness and runner never call cloud-drive APIs.
