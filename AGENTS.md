# Agent Entry

## Project Identity

RigForge — Character Animation Asset Workbench

## Current Phase

```text
W0 COMPLETE / PASS / BASELINED
W0-RS COMPLETE / PASS / BASELINED
IA-1 COMPLETE / PASS / CLOSED
V1 implementation AUTHORIZED / NOT STARTED
V1-1 READY / NOT STARTED
```

W0 research gates are closed.

Do not begin arbitrary Product implementation.

Execute V1 work only under the currently authorized dedicated implementation
stage. Current authorized next stage is V1-1.

## Source of Truth

| Path | Use |
| --- | --- |
| [README.md](README.md) | Project entry and overall status |
| [docs/product/](docs/product/) | Product definition and V1 boundary |
| [docs/research/](docs/research/) | Research facts, sources, and research matrix |
| [docs/architecture/](docs/architecture/) | Current V1 architecture principles |
| [docs/architecture/decisions/](docs/architecture/decisions/) | Architecture Decision Records |
| [docs/development/ROADMAP.md](docs/development/ROADMAP.md) | Phase status and next plans |

Read the relevant file. Do not treat chat history as the contract.

Current V1 decision: [docs/architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md](docs/architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).

W0-SR proposal files remain historical rationale/review evidence. Canonical
product/architecture/roadmap files are current project truth.

## Hard Rules

1. Do not treat a third-party format as product authority.
2. Do not treat a DCC as product authority. V1 may use a pinned hidden Blender worker (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`, ADR-0001, validated by POC-BLENDER-E2E-01). Durable product contracts must stay backend-neutral. Do not add further DCC backends in V1.
3. Do not lock an implementation language before its owner stage. Core language remains OPEN / NOT SELECTED (POC-CORE-01 `INCONCLUSIVE`). Earliest owner: V1-1 planning.
4. Do not lock a GUI framework before its owner stage. GUI framework remains OPEN / NOT SELECTED. Earliest owner: V1-2 planning.
5. Do not create a production source-tree architecture that depends on Core-language or GUI choices before those owner stages decide them.
6. Do not write a research hypothesis as a confirmed fact. Blender-backed execution is `ACCEPT_BLENDER_BACKEND_WITH_GUARDS` (POC-BLENDER-E2E-01). Derived Preview is `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS` (POC-PREVIEW-01R). Neither is product authority, permanently final, a viewer/payload selection, or coverage of all Characters/Motions.
7. Classify every third-party conclusion as one of: official specification/documentation, source-confirmed, project claim, or RigForge inference. See [docs/research/README.md](docs/research/README.md).
8. Record architecture decisions as ADRs. Do not leave them only in agent chat.
9. A derived representation must not become product authority.
10. Do not commit or push unless the user explicitly authorizes it.

## Git Identity and Commit Attribution

The only allowed commit identity in this repository is:

```text
CiaranCw <1399538830@qq.com>
```

Set it with repository-local Git config only:

- `user.name = CiaranCw`
- `user.email = 1399538830@qq.com`

Author and Committer must both be that identity. Do not use `git commit --author` or `GIT_AUTHOR_*` / `GIT_COMMITTER_*` to substitute another identity.

AI / Agent attribution is forbidden in commit metadata and commit messages. `cursoragent` is forbidden. Do not add `Co-authored-by`, `Generated-by`, `Assisted-by`, or equivalent trailers for Cursor, Claude, Codex, ChatGPT, Copilot, or any other agent.

### Preflight

Before every commit, verify:

```bash
git config --local --get user.name
git config --local --get user.email
```

These must be exactly `CiaranCw` and `1399538830@qq.com`. Also check the staged message and planned trailers for AI attribution.

If identity does not match: **DO NOT COMMIT**. Restore the repository-local identity first.

### Postflight

After every commit, confirm with `git show -s --format=fuller HEAD` (or `git log -1 --format=fuller`):

```text
Author:     CiaranCw <1399538830@qq.com>
Commit:     CiaranCw <1399538830@qq.com>
```

and that the message contains none of: `Co-authored-by`, `cursoragent`, `Cursor Agent`, `Claude`, `Codex`, `ChatGPT`, `Copilot`, `Generated-by`, `Assisted-by`.

If identity or attribution is wrong: **do not push**. Report and fix the local commit first.

## Current Next Step

```text
V1-1 —
Thin Workflow Domain / version / provenance contract and implementation
planning.

Execute only under a dedicated V1-1 prompt.
```

Core language remains NOT SELECTED. GUI remains OPEN. Do not infer either
from research harness technology.

Independent audit / review model:
`GPT-5.6 Sol High`

Do not use GPT-5.6 Sol 1M High for routine project audit/review work unless the
user explicitly re-authorizes it.

Do not substitute another audit model unless the user explicitly changes this
rule.

IA-1 closeout:
[docs/research/audits/IA_1_INDEPENDENT_AUDIT.md](docs/research/audits/IA_1_INDEPENDENT_AUDIT.md),
[docs/research/audits/IA_1_EXTERNAL_REVIEW.md](docs/research/audits/IA_1_EXTERNAL_REVIEW.md).

Synthesis:
[docs/research/decisions/W0_REVISED_SYNTHESIS.md](docs/research/decisions/W0_REVISED_SYNTHESIS.md).
Traceability:
[docs/research/decisions/W0_RS_TRACEABILITY.md](docs/research/decisions/W0_RS_TRACEABILITY.md).

Roadmap: [docs/development/ROADMAP.md](docs/development/ROADMAP.md).
