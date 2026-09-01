# Architecture Decision Records

Use one file per decision. Do not record decisions only in chat.

Suggested name: `ADR-XXXX-<short-title>.md`

## Records

| ID | Title | Status |
| --- | --- | --- |
| [ADR-0001](ADR-0001-v1-scope-and-blender-backed-execution.md) | V1 scope and Blender-backed execution | Accepted |
| [ADR-0002](ADR-0002-v1-core-language.md) | V1 Core language | Accepted |
| [ADR-0003](ADR-0003-v1-local-catalog-storage.md) | V1 local catalog storage | Accepted |
| [ADR-0004](ADR-0004-v1-workbench-gui.md) | V1 Workbench GUI | Accepted |

## Format

```text
ADR-XXXX: <Title>

Status:
Proposed | Accepted | Superseded | Rejected

Context

Decision

Alternatives Considered

Consequences

Evidence

Open Questions
```

Status changes stay in the same file. Superseding ADRs must name the ADR they replace.
