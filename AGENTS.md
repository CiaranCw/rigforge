# Agent Entry

## Project Identity

RigForge — Character Animation Asset Workbench

## Current Phase

```text
W0 COMPLETE / PASS / BASELINED
W0-RS COMPLETE / PASS / BASELINED
IA-1 COMPLETE / PASS / CLOSED
V1-1 COMPLETE / PASS / BASELINED
Gate A PASS / CLOSED
V1-2 COMPLETE / PASS / BASELINED
V1-3 COMPLETE / PASS / BASELINED
Gate B PASS / CLOSED
V1-4 COMPLETE / PASS / BASELINED
V1-5 COMPLETE / PASS / BASELINED
Gate C PASS / CLOSED
V1-6 COMPLETE / PASS / BASELINED
V1-7 NOT STARTED / OPTIONAL
V1-8 NOT STARTED
Gate D NOT STARTED
```

W0 research gates are closed. V1-1 Gate A is closed. V1-2 is baselined.
V1-3 is **COMPLETE / PASS / BASELINED**. Gate B is **PASS / CLOSED**.
V1-4 is **COMPLETE / PASS / BASELINED**. V1-5 is
**COMPLETE / PASS / BASELINED**. Gate C is **PASS / CLOSED**. V1-6 is
**COMPLETE / PASS / BASELINED**.

Do not begin arbitrary Product implementation.

Execute V1 work only under the currently authorized dedicated implementation
stage. Do not start V1-7 unless a concrete Product consumer requires an
output artifact/export contract. A GLB Preview payload is not that
requirement. Do not start V1-8 as part of V1-6 closeout. Do not open Gate C2.

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
Core language: [docs/architecture/decisions/ADR-0002-v1-core-language.md](docs/architecture/decisions/ADR-0002-v1-core-language.md) (`Accepted` at V1-1 Gate A). RigForge V1 Core is **Rust**.

W0-SR proposal files remain historical rationale/review evidence. Canonical
product/architecture/roadmap files are current project truth.

## Hard Rules

1. Do not treat a third-party format as product authority.
2. Do not treat a DCC as product authority. V1 may use a pinned hidden Blender worker (`ACCEPT_BLENDER_BACKEND_WITH_GUARDS`, ADR-0001, validated by POC-BLENDER-E2E-01). Durable product contracts must stay backend-neutral. Do not add further DCC backends in V1.
3. Core language is **Rust** (ADR-0002 **Accepted** at V1-1 Gate A). Do not reopen Rust vs C++ vs Python. POC-CORE-01 remains `INCONCLUSIVE` historical evidence, not silent C++ selection.
4. GUI framework is **Accepted: egui/eframe** ([ADR-0004](docs/architecture/decisions/ADR-0004-v1-workbench-gui.md), Accepted at V1-2 focused review). This selection does not choose a Preview viewer or payload. Preview viewer/payload/surface is **Accepted** independently in [ADR-0006](docs/architecture/decisions/ADR-0006-v1-engine-independent-preview.md) at V1-6 focused review / focused closure. Rust Core does not imply a Rust GUI; the GUI decision is independent.
5. The Domain crate lives at `domain/`. Catalog, persistence, and orchestration live at `app/`. The Workbench shell lives at `workbench/`. The pinned Blender adapter lives at `blender-worker/`. Do not add further DCC backends in V1.
6. Do not write a research hypothesis as a confirmed fact. Blender-backed execution is `ACCEPT_BLENDER_BACKEND_WITH_GUARDS` (POC-BLENDER-E2E-01). Derived Preview is `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS` (POC-PREVIEW-01R) and ADR-0006 **Accepted**. Neither Blender nor Preview is Product authority. GLB is a Preview payload only, not a Product format and not a V1-7 export.
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
Decide whether V1-7 has a concrete consumer requirement.

If none:
skip optional V1-7 and prepare V1-8 real-asset hardening / release qualification.

Do not start Gate D until V1-8 completes.
```

Do not start V1-7 unless a concrete Product consumer requires an output
artifact/export contract. A GLB Preview payload is not that requirement.
Do not open Gate C2. Do not open a post-Gate-C or V1-5 release audit.
Do not start Gate D until V1-8 completes.

GUI is **Accepted: egui/eframe** (ADR-0004). Catalog storage is **Accepted:
SQLite / rusqlite bundled** (ADR-0003). Blender worker process packaging is
**Accepted** (ADR-0005, V1-3 / Gate B Runtime Foundation Audit). Preview
viewer/payload/surface is **Accepted** (ADR-0006, V1-6 focused review /
focused closure). Do not infer Preview viewer/payload from the GUI crate.

Gate C independent audit (immutable historical evidence):
[docs/development/audits/GATE_C_PRODUCT_CORE_AUDIT.md](docs/development/audits/GATE_C_PRODUCT_CORE_AUDIT.md),
[docs/development/audits/GATE_C_FINDINGS.md](docs/development/audits/GATE_C_FINDINGS.md),
[docs/development/audits/GATE_C_COVERAGE_MATRIX.md](docs/development/audits/GATE_C_COVERAGE_MATRIX.md).
Independent result remains `GATE_C_PASS_CANDIDATE`. Project acceptance is
**Gate C PASS / CLOSED**. `GATE-C-OBS-001` is deferred / non-blocking
(Workbench integration / release hardening).

V1-6 contracts:
[docs/development/V1_6_IMPLEMENTATION_PLAN.md](docs/development/V1_6_IMPLEMENTATION_PLAN.md),
[docs/development/V1_6_PREVIEW_ARTIFACT_CONTRACT.md](docs/development/V1_6_PREVIEW_ARTIFACT_CONTRACT.md),
[docs/development/V1_6_VIEWER_CONTRACT.md](docs/development/V1_6_VIEWER_CONTRACT.md),
[docs/development/V1_6_PREVIEW_GENERATION.md](docs/development/V1_6_PREVIEW_GENERATION.md).
ADR-0006 is **Accepted**.

V1-5 contracts:
[docs/development/V1_5_IMPLEMENTATION_PLAN.md](docs/development/V1_5_IMPLEMENTATION_PLAN.md),
[docs/development/V1_5_TRANSFER_LIFECYCLE.md](docs/development/V1_5_TRANSFER_LIFECYCLE.md),
[docs/development/V1_5_QC_CONTRACT.md](docs/development/V1_5_QC_CONTRACT.md),
[docs/development/V1_5_DERIVED_VARIANT_PUBLICATION.md](docs/development/V1_5_DERIVED_VARIANT_PUBLICATION.md).

V1-4 contracts:
[docs/development/V1_4_IMPLEMENTATION_PLAN.md](docs/development/V1_4_IMPLEMENTATION_PLAN.md),
[docs/development/V1_4_MAPPING_WORKFLOW.md](docs/development/V1_4_MAPPING_WORKFLOW.md),
[docs/development/V1_4_COMPATIBILITY_CONTRACT.md](docs/development/V1_4_COMPATIBILITY_CONTRACT.md).

V1-3 contracts:
[docs/development/V1_3_IMPLEMENTATION_PLAN.md](docs/development/V1_3_IMPLEMENTATION_PLAN.md),
[docs/development/V1_3_WORKER_CONTRACT.md](docs/development/V1_3_WORKER_CONTRACT.md),
[docs/development/V1_3_PROCESS_LIFECYCLE.md](docs/development/V1_3_PROCESS_LIFECYCLE.md).

V1-2 contracts:
[docs/development/V1_2_IMPLEMENTATION_PLAN.md](docs/development/V1_2_IMPLEMENTATION_PLAN.md),
[docs/development/V1_2_CATALOG_CONTRACT.md](docs/development/V1_2_CATALOG_CONTRACT.md),
[docs/development/V1_2_ORCHESTRATION_CONTRACT.md](docs/development/V1_2_ORCHESTRATION_CONTRACT.md).

V1-1 contracts:
[docs/development/V1_1_IMPLEMENTATION_PLAN.md](docs/development/V1_1_IMPLEMENTATION_PLAN.md),
[docs/development/V1_1_DOMAIN_CONTRACT.md](docs/development/V1_1_DOMAIN_CONTRACT.md),
[docs/development/V1_1_VERSION_PROVENANCE.md](docs/development/V1_1_VERSION_PROVENANCE.md).

## Agent models

```text
Main Agent default:
Cursor + Grok 4.6 Extra High

Use GPT-5.6 Sol High as Main Agent only when task complexity genuinely
requires the stronger model.

Independent Gate Agent:
GPT-5.6 Sol High
```

Do not use GPT-5.6 Sol 1M High unless the user explicitly re-authorizes it.

Gate B is **PASS / CLOSED**. Gate C is **PASS / CLOSED**. Do not open Gate
B2, Gate C2, or another independent review of V1-3, V1-4, or V1-5. V1-6
did not get a default independent Gate. Next fixed independent gate is
Gate D after V1-8 (Final Release Readiness Audit).

IA-1 closeout:
[docs/research/audits/IA_1_INDEPENDENT_AUDIT.md](docs/research/audits/IA_1_INDEPENDENT_AUDIT.md),
[docs/research/audits/IA_1_EXTERNAL_REVIEW.md](docs/research/audits/IA_1_EXTERNAL_REVIEW.md).

Synthesis:
[docs/research/decisions/W0_REVISED_SYNTHESIS.md](docs/research/decisions/W0_REVISED_SYNTHESIS.md).
Traceability:
[docs/research/decisions/W0_RS_TRACEABILITY.md](docs/research/decisions/W0_RS_TRACEABILITY.md).

Roadmap: [docs/development/ROADMAP.md](docs/development/ROADMAP.md).
