# Architecture

Only principles that are already stable. Implementation language, GUI stack, serialization format, and package layout are not frozen.

## Stable principles

- **Canonical-first.** The Canonical Domain is independent of any engine or interchange format.
- **Adapter boundary.** FBX / glTF / USD / DCC / engine types are adapters, interchange, or derived representations.
- **Canonical != Runtime.** Authoring/canonical objects are not the same as cooked or runtime representations.
- **Authoritative != Derived.** Derived assets must not become the source of truth.
- **Domain API before UI.** CLI, GUI, automation, and later MCP call the same Domain API.
- **Deterministic-first.** V1 retarget and QC must be deterministic. AI backends are optional.

## Decisions

Record changes of mind as ADRs: [decisions/README.md](decisions/README.md).

Do not treat research candidates as selected dependencies.
