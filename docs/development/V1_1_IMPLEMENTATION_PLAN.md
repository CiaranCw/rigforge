# V1-1 Implementation Plan

Thin Workflow Domain / version / provenance. First V1 implementation stage
after W0 / IA-1 closeout.

Status: `COMPLETE / PASS / BASELINED`

Gate A: `PASS / CLOSED`

Core language: Rust (`ADR-0002` **Accepted** at V1-1 Gate A).

## Requirements derived from W0 / IA-1

1. Keep the accepted user transaction; do not rebuild a Canonical runtime.
2. Product owns identity, Mapping, Compatibility, Policy, Job, QC meaning,
   Derived Variant, Preview orchestration, and versioning.
3. Durable contracts stay backend-neutral (IA-1 Observation 2).
4. `DerivedVariantVersion != PersistenceArtifact` (W0-RS retained limitation).
5. Preview is derived / rebuildable / non-authoritative.
6. Exact-version references in durable lineage.
7. Core language must be decided here; GUI and database must not.
8. Tests must run without Blender, assets, browser, or network.

## Core-language decision matrix

| Candidate | Strengths | Weaknesses | Evidence | V1-specific implications | Decision |
| --- | --- | --- | --- | --- | --- |
| **Rust** | Newtypes for ID/digest/path; serde JSON; Cargo.lock; memory safety; language-level worker boundary | Extra toolchain; slower schema iteration than Python; GUI not native by default | RFC 9562; uuid/serde crates.io licenses; Cargo Book lockfile; POC-CORE inconclusive on deferred FFI | Matches thin Domain + isolated Blender worker; does not select GUI or DB | **SELECTED (ACCEPTED)** |
| Python | Fast iteration; stdlib JSON; trivial subprocess; worker language familiarity | Weaker static ID kinds; Core/worker collapse risk; this machine is CPython 3.13 (uuid7 is 3.14) | Python 3.14 uuid7 docs; Blender worker is Python (not Core evidence) | Would ease worker debugging and raise `bpy` leakage risk | Rejected as Core |
| C++ | Native library consumption; historical W0.4 preference | Higher safety/build cost; POC-CORE-01 `INCONCLUSIVE`; JSON Domain does not need ufbx now | POC_CORE_01.md; ADR-0001 defers native importer | FFI tax without current native-parser mandate | Rejected as Core |

No spike was required: the candidates are distinguishable on V1-1’s actual
job (typed identity/provenance), not on deferred native parsing.

Details: [ADR-0002](../architecture/decisions/ADR-0002-v1-core-language.md).

## Toolchain

```text
language: Rust
crate: domain/ (package rigforge_domain)
edition: 2021
rustc: 1.98.0 (official rustup, x86_64-pc-windows-msvc)
cargo: 1.98.0
lockfile: domain/Cargo.lock committed with the crate
test: cargo test   (cwd domain/)
fmt: cargo fmt     (not a V1-1 gate)
lint: cargo clippy (not a V1-1 gate)
build: cargo test / cargo build --offline after deps cached
```

`rust-toolchain.toml` pins `1.98.0`. No CI expansion in this stage.

## Dependencies

| Crate | Pin | License | Purpose | Why not std | Owner |
| --- | --- | --- | --- | --- | --- |
| serde + serde_derive | 1.0.229 (lockfile) | Apache-2.0 OR MIT | typed JSON | no std JSON derives | V1-1 |
| serde_json | 1.0.151 (lockfile) | Apache-2.0 OR MIT | JSON encode/decode | no std JSON | V1-1 |
| uuid (features v7, serde) | 1.26.0 | Apache-2.0 OR MIT | RFC 9562 UUIDv7 | no std UUID generator | V1-1 |

Transitive crates (`syn`, `getrandom`, …) are recorded in `Cargo.lock`. No
database, GUI, HTTP, or Blender crates.

## Source tree

```text
domain/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  README.md
  src/          production Domain
  tests/        pure contract tests
docs/development/V1_1_*.md
docs/architecture/decisions/ADR-0002-v1-core-language.md
```

No empty V1-2..V1-8 placeholders. Research `experiments/` is untouched.
Product/Architecture W0 contracts are untouched.

## Implemented slice

Allowed: IDs, version types, domain records, JSON, validation, constructors,
publication graph, persistence *interface* trait, tests.

Not implemented: database, catalog filesystem, Blender, FBX, Preview
generation, GUI, HTTP, RPC, MCP, Mapping algorithm, retargeter.

## Rev1 corrections

External independent review of the first V1-1 worktree returned
`FAIL / REV1 REQUIRED`. This worktree was corrected in place (not restarted).
The Core-language survey was not reopened. V1-1 Gate A accepted Rust
(`ADR-0002` Accepted).

| Finding | Correction |
| --- | --- |
| V1-1-MAJOR-001 | Private fields on version/provenance records; validate-then-freeze; Source Skeleton option A |
| V1-1-MAJOR-002 | Graph-level exact provenance + PersistenceVerification publication gate; Published lineage retains `PersistenceVerificationId` (Rev2) |
| V1-1-MAJOR-003 | `from_json_validated` + `DomainRecord`; `Validated<T>` store/application boundary (Rev2) |
| V1-1-MAJOR-004 | Structured RetargetPolicy enums; unknown tags fail closed |
| V1-1-MINOR-001 | Parse / from_uuid / deserialize reject non-v7 Product IDs |

## V1-2 handoff

V1-2 receives:

| Item | Where |
| --- | --- |
| Selected Core language | ADR-0002 (**Accepted** at V1-1 Gate A) |
| Source-tree baseline | `domain/` crate |
| Product ID / version semantics | V1_1_VERSION_PROVENANCE.md + `identity.rs` |
| Serialization contract | JSON schema_version=1, deny unknown fields |
| Provenance rules | exact version ids on Job/Derived |
| Domain validation API | `DomainRecord::validate` / `from_json_validated` / `Validated<T>` / graph validators / `publish_derived_variant` |
| Artifact identity rules | Persistence id + instance id + digest; Published `PersistenceVerificationId`; Preview flags |
| Minimum persistence operations | `ProductVersionStore` accepts `Validated<T>` in `store.rs` |
| Migration expectations | fail closed on unknown schema; no engine yet |

V1-2 should not need to reinterpret W0 research to understand Product
identity. GUI remains OPEN. V1-2 is `READY / NOT STARTED`.

## Review

Independent GPT-5.6 Sol High Gate A: `PASS / CLOSED`. Findings 5/5 CLOSED.
No Rev3. V1-1: `COMPLETE / PASS / BASELINED`.
