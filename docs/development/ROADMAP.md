# Roadmap

Phase list and status only. Design details live in product, research, and architecture docs.

## Status

| Phase | Name | Status |
| --- | --- | --- |
| — | Repository Initialization | COMPLETE |
| W0 | Research Baseline | ACTIVE |
| W0.1 | Canonical Foundations | COMPLETE |
| W0.2 | Mapping / Compatibility / Retarget | COMPLETE |
| W0.3 | Adapter / Infrastructure | READY |
| W0.4 | Product / Tech-stack / Frontier | NOT STARTED |
| W0-P | Research Validation PoCs | NOT STARTED |
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

Next: W0.3 — Adapter / Infrastructure Research. W0.3 is **READY**, not started.

W0 remains ACTIVE. W0.4, W0-P, and W1–W10 stay NOT STARTED. Gate A / IA-1 are not claimed. No retarget algorithm and no IK placement are selected.

## Stage invariants (from W0.2 Rev1)

Every later stage must explicitly check:

```text
PRODUCT REQUIREMENTS
GENERALITY
ENGINE INDEPENDENCE
REAL-ASSET EVIDENCE
```

Reference DCCs and engines are not Canonical authority and not required hosts.
