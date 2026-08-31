# Frontier / AI Matrix — W0.4

Access date: **2026-08-31**. Decision date: **2026-08-31**.

Permanent boundary (architecture + this stage):

```text
AI suggestion
        !=
Canonical truth
```

Any learned mapping / rig / repair must be a **proposal** with diagnostics / confidence / evidence, then explicit Domain acceptance, before authoritative mutation.

Deterministic Core remains the V1 default unless W0-P overturns it. This pass does **not** overturn it.

---

## Classification tokens

`V1 CORE` / `V1 OPTIONAL` / `POST-V1` / `RESEARCH ONLY` / `REJECT`

---

## Matrix

| Candidate | Problem solved | Maturity | Code | Weights | License notes | Data restrictions | Hardware | Integration | V1 fit | Authority risk | Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **SkinTokens / TokenRig** | Unified autoregressive skeleton + skinning from mesh | Paper + inference repo 2026 [PROJECT_CLAIM] arXiv:2602.04805 | GitHub VAST-AI-Research/SkinTokens MIT [SOURCE_CONFIRMED] | Hugging Face VAST-AI/SkinTokens; two ckpts required [OFFICIAL_DOC] | Code MIT. **Weight / training-data commercial use UNCONFIRMED** this pass | Trained on ArticulationXL 2.0 + VRoid Hub + ModelsResource [PROJECT_CLAIM] — dataset ToS **REQUIRES LEGAL REVIEW** | GPU inference; Qwen3-0.6B backbone [PROJECT_CLAIM] | Optional auto-rig **assistant**; not Domain | Low | High if accepted silently | `POST-V1` / `RESEARCH ONLY` |
| **UniRig** | Staged skeleton then skinning (SkinTokens predecessor, SIGGRAPH 2025) | Public inference | MIT [SOURCE_CONFIRMED] | Hugging Face VAST-AI/UniRig [OFFICIAL_DOC] | Code MIT; weights/data same caution | Articulation-XL2.0 / Rig-XL / VRoid [PROJECT_CLAIM] | GPU | Superseded for new work by SkinTokens | Low | Same | `RESEARCH ONLY` |
| **RigAnything** | Template-free autoregressive rig; GLB/OBJ in, rigged GLB out (TOG 2025) | Inference scripts + HF ckpt | Code availability **PARTIAL** (HF card) [PROJECT_CLAIM] | Isabellaliu/RigAnything [OFFICIAL_DOC] | Code/weight license **UNKNOWN** this pass — **REQUIRES LEGAL REVIEW** | Not independently audited | GPU | Mesh→rig proposal | Low | High | `RESEARCH ONLY` |
| **AnyTop** | Topology / correspondence research (R1 inventory) | Paper-first | Not re-verified as production lib | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | Research mapping hint | None | High | `RESEARCH ONLY` |
| **SATA** | Research backend (R1 inventory) | Paper-first | Not selected | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | None | None | High | `RESEARCH ONLY` |
| **UniMate** | Research backend (R1 inventory) | Paper-first | Not selected | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | None | None | High | `RESEARCH ONLY` |
| **Learned semantic mapping** | Joint role inference from names/mesh | Scattered papers / vendor features | No single Core candidate | N/A | N/A | Often web-scraped names | GPU/CPU | Optional suggestion layer on Mapping | Suggestion only | High if auto-commit | `POST-V1` |
| **Learned retarget** | Motion transfer without explicit map | Academic + engine “auto retarget” | Host-specific (W0.2) | Vendor | Vendor EULA | Clip libraries | GPU | Must not replace deterministic Core | None in V1 | High | `POST-V1` |
| **NVIDIA SOMA Retargeter** | Human mocap → robot (W0.2) | Official docs evolving | Research / Isaac | NVIDIA | Vendor | SOMA/G1 assumed | GPU | Robot, not game Canonical | None | Medium | `RESEARCH ONLY` |
| **GMR** | Human→humanoid robot IK | Paper + configs | Research | N/A | Check repo | Robot-specific | CPU/GPU | Not game-character Domain | None | Medium | `RESEARCH ONLY` |
| **Motius** | R1 inventory research backend (motion / transfer family) | Paper-first; **not independently re-verified** this pass | UNKNOWN | UNKNOWN | UNKNOWN — **REQUIRES LEGAL REVIEW** if revisited | UNKNOWN | UNKNOWN | None | None | High | `RESEARCH ONLY` |
| **Mixamo auto-rig** | Named humanoid auto-rig | Production service | Closed | Adobe | Adobe ToS; **service dependency** | Adobe content rules | Cloud | Violates DCC-independence if mandatory | `REJECT` as Core | High | `REJECT` as dependency |
| **Cascadeur AI pose** | Physics-assisted posing | Production app | Closed | Vendor | Subscription | Vendor | Local GPU | Not Domain | `REJECT` as host | Medium | `REJECT` as Workbench host |

---

## Placement decision

| Capability | Class | Why |
| --- | --- | --- |
| Deterministic Mapping / Compat / Retarget / QC | `V1 CORE` | Architecture; W0.2 |
| AI semantic **suggestion** (non-authoritative) | `POST-V1` | Useful; needs UX + license + evidence |
| AI retarget backend | `POST-V1` | Must not define Domain or Preview |
| AI auto-rig (SkinTokens / RigAnything class) | `POST-V1` | Mesh→rig is out of V1 Character ingest core; license/data risk |
| Mandatory learned model in Core | `REJECT` | Hardware, license, non-determinism, dataset ToS |
| Mixamo / cloud auto-rig as ingest authority | `REJECT` | Service + humanoid-only + no provenance |

---

## What a paper is not

```text
paper exists
        !=
production dependency is usable
```

Weights on Hugging Face ≠ cleared commercial dataset rights. VRoid / ModelsResource / ArticulationXL terms were **not** independently cleared this pass.

---

## Safety / correctness (DECIDE_NOW)

This is not a library pick. It is a product invariant:

```text
proposal
        →
diagnostics / confidence / evidence
        →
human or Domain acceptance
        →
Canonical mutation
```

Never: model output → Canonical write.

Re-check triggers: a permissive-weight, offline, documented-dataset model that produces **mapping proposals** with calibrated confidence; still would be optional, not Core.
