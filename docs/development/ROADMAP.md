# Roadmap

Phase list and status only. Design details live in product, research, and architecture docs.

## Status

| Phase | Name | Status |
| --- | --- | --- |
| — | Repository Initialization | COMPLETE |
| W0 | Research Baseline | ACTIVE |
| W0.1 | Canonical Foundations | COMPLETE |
| W0.2 | Mapping / Compatibility / Retarget | COMPLETE |
| W0.3 | Adapter / Infrastructure | COMPLETE |
| W0.4 | Product / Tech-stack / Frontier | COMPLETE |
| W0-P | Research Validation PoCs | READY |
| W1 | Canonical Domain Contract | NOT STARTED |
| W2 | I/O Adapter Contract | NOT STARTED |
| W3 | Canonical I/O MVP | NOT STARTED |
| W4 | Validation + Semantic Mapping | NOT STARTED |
| W5 | Compatibility + Deterministic Retarget | NOT STARTED |
| W6 | Domain API + CLI | NOT STARTED |
| W7 | Minimal Workbench | NOT STARTED |
| W8 | DCC / Engine Adapters | NOT STARTED |
| W9 | Automation / MCP-ready API | NOT STARTED |
| W10 | CI / Structural Validation / Final Audit | NOT STARTED |

## Immediate next

W0.1 remains the accepted Canonical Foundations baseline.

W0.2 is the accepted Mapping / Compatibility / Deterministic Retarget research baseline: [docs/research/retargeting/W0_2_MAPPING_COMPAT_RETARGET.md](../research/retargeting/W0_2_MAPPING_COMPAT_RETARGET.md). Status: **COMPLETE**.

W0.3 is the accepted Adapter / Infrastructure / Preview Boundary research baseline: [docs/research/infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md](../research/infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md). Status: **COMPLETE**.

W0.4 is the accepted Product / Tech-stack / Frontier / Engineering **decision-research** baseline: [docs/research/decisions/W0_4_PRODUCT_TECH_DECISIONS.md](../research/decisions/W0_4_PRODUCT_TECH_DECISIONS.md). Status: **COMPLETE**.

It is not an implementation freeze. `DECIDE_NOW` is architectural/product rule. `PROVISIONAL_W0P_GATED` remains reversible by W0-P.

Next: W0-P — Research Validation PoCs. Status: **READY**. Do not treat W0 as complete.

W0 remains ACTIVE until W0-P executes, evidence is synthesized, W0 final synthesis occurs, and IA-1 executes. IA-1 is **NOT STARTED**. Gate A is **NOT PASSED**. W1 is **NOT STARTED**. Preferred candidates are not implementation selections unless a row is `DECIDE_NOW` (architectural role). No Core language, GUI, Preview renderer, process topology, Preview data path, or format library is selected as authority.

## Stage invariants (from W0.2 Rev1)

Every later stage must explicitly check:

```text
PRODUCT REQUIREMENTS
GENERALITY
ENGINE INDEPENDENCE
FORMAT INDEPENDENCE
DCC INDEPENDENCE
REAL-ASSET EVIDENCE
```

Reference formats, DCCs, engines, runtimes, and renderers are not Canonical authority and not required hosts.
