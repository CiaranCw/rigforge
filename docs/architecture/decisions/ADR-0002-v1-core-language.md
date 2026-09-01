# ADR-0002: V1 Core language

Status: **Accepted**

Date: 2026-09-01

Accepted at: **V1-1 Gate A** (`PASS / CLOSED`).

This decision is an implementation choice, not Product identity.

RigForge V1 Core: **Rust**.

## Context

W0 / IA-1 closed with:

```text
Core language: OPEN / NOT SELECTED
POC-CORE-01: COMPLETE / PASS / BASELINED
Decision Impact: INCONCLUSIVE
```

ADR-0001 accepted a thin Workflow Domain and a pinned isolated Blender worker
behind a backend-neutral Job Spec. Native FBX/glTF stacks, a heavy Canonical
runtime, and a custom retarget solver are not V1 critical path.

V1-1 is the earliest owner of the Core-language decision. A production
source-tree architecture must not be invented before that decision is
sufficiently justified. GUI remains OPEN (V1-2 planning). Database / catalog
storage remains OPEN (V1-2).

The V1 Core does not run inside Blender. Preferred boundary:

```text
RigForge Core
        ↓
backend-neutral Job
        ↓
process boundary
        ↓
Blender Python worker
```

`Blender uses Python` is not an argument that the Product Core must be
Python. Preview-PoC JavaScript/TypeScript is not Core evidence. Historical
W0.4 C++ preference is not current authority. POC-CORE Rust/C++ evidence
remains informative but inconclusive, and it measured native ufbx FFI cost
for a deferred native importer — not the current thin Domain.

## V1-1 requirements

The first production slice must:

- model logical Product objects vs immutable versions
- generate portable Product IDs that are not paths, filenames, or content hashes
- bind durable provenance to exact versions
- keep JobSpec / Mapping / Policy backend-neutral
- distinguish DerivedVariantVersion from PersistenceArtifact and PreviewArtifact
- serialize a human-inspectable backend-neutral representation
- fail closed on unsupported schema versions and missing required provenance
- test purely, without Blender, FBX assets, browser, or network

It must not implement catalog storage, GUI, worker launch, Auto-Mapping,
retarget execution, QC execution, or Preview generation.

## Candidate set

Prior project evidence supports exactly these Core candidates:

```text
Rust
C++
Python
```

No additional language was added. No general language survey was performed.

## Decision criteria

Evaluated against V1-1 and near-term V1 stages:

- Domain-model type safety (Product ID vs digest vs path)
- immutable / versioned data modeling
- serialization ecosystem
- schema evolution / migration ergonomics
- deterministic validation
- testing without Blender
- cross-platform desktop distribution
- dependency management and build reproducibility
- subprocess / worker orchestration (V1-3)
- Blender worker process boundary / GPL isolation engineering
- future GUI integration without coupling (GUI remains unselected)
- local database interoperability (V1-2)
- CLI / diagnostic tooling
- FFI burden where actually needed (native importers remain deferred)
- security / memory-safety maintenance burden
- long-term maintainability
- developer iteration cost
- release packaging

Not used as sole selectors: popularity, shortest code, AI familiarity, PoC
harness language, or a future GUI guess.

## Evidence

| Fact | Classification | Source |
| --- | --- | --- |
| UUID version 7 is an IETF standard time-ordered UUID | `OFFICIAL_DOC` | [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562.html) (May 2024) |
| RFC 9562 says implementations SHOULD prefer UUIDv7 over v1/v6 when possible | `OFFICIAL_DOC` | RFC 9562 §5.7 |
| `uuid` 1.26.0 provides `Uuid::now_v7` behind the `v7` feature | `OFFICIAL_DOC` | [docs.rs/uuid/1.26.0](https://docs.rs/uuid/latest/uuid/) |
| `uuid` 1.26.0 license is Apache-2.0 OR MIT | `SOURCE_CONFIRMED` | crate `Cargo.toml` / LICENSE files from crates.io |
| `serde` 1.0.229 and `serde_json` 1.0.151 are Apache-2.0 OR MIT | `SOURCE_CONFIRMED` | crate LICENSE files from crates.io |
| Cargo.lock records exact dependency revisions for reproducibility | `OFFICIAL_DOC` | [Cargo Book: Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html) |
| Rust is installed via official rustup | `OFFICIAL_DOC` | [The Rust Book, Installation](https://doc.rust-lang.org/book/ch01-01-installation.html) |
| Python 3.14 added `uuid.uuid7()` | `OFFICIAL_DOC` | [What’s new in Python 3.14](https://docs.python.org/3/whatsnew/3.14.html) |
| This machine’s available CPython is 3.13.12 | `PROJECT_CLAIM` | local `python --version` |
| POC-CORE-01 C++ vs Rust ufbx FFI was `INCONCLUSIVE` | `PROJECT_CLAIM` | `docs/research/poc/POC_CORE_01.md` |
| Native ufbx importer is not a V1 mandate | `PROJECT_CLAIM` | ADR-0001, W0-RS |
| Blender in-process Core linking is a GPL legal-review issue; process isolation is an engineering candidate only | `PROJECT_CLAIM` | W0.4 license matrix |
| A second execution backend was not demonstrated | `PROJECT_CLAIM` | IA-1 Observation 1 |
| Durable production schema must stay backend-neutral even if a Job instance names capabilities | `PROJECT_CLAIM` | IA-1 Observation 2 |
| Linguistic separation of Core from Blender Python reduces accidental `bpy` leakage | `RIGFORGE_INFERENCE` | this ADR |
| Rust newtypes make Product ID / digest / path confusion a type error | `RIGFORGE_INFERENCE` | this ADR |
| Shipping Blender does not require the Core to be Python | `RIGFORGE_INFERENCE` | ADR-0001 process boundary |

## Tradeoffs

Rust vs Python: Python iterates faster and already exists beside the worker.
It also makes it easier to collapse Core records into worker dictionaries and
to treat paths or hashes as IDs. V1-1’s job is exactly those invariants.
Python as Core would not be selected merely because the worker is Python.

Rust vs C++: C++ was historically preferred for native library consumption.
That FFI question is deferred with the native importer. For a JSON Domain,
C++ adds memory-safety and build-graph cost without a current native-parser
requirement. POC-CORE-01 did not produce a decisive C++ win.

Rust packaging: a Domain library plus later CLI can ship as a native artifact
beside an isolated Blender worker. That is two operational runtimes (Core +
worker), which ADR-0001 already accepted. It does not select a GUI.

## Decision

**Select Rust as the V1 Core language.**

Accepted at V1-1 Gate A (`PASS / CLOSED`). The Core-language survey is not
reopened.

Production crate: `domain/` (`rigforge_domain`).

ID generation: RFC 9562 UUIDv7, canonical hyphenated lowercase text. Content
SHA-256 remains a separate `ContentDigest` type. Filesystem paths remain
`LocationEvidence`.

Serialization: JSON via `serde` / `serde_json`. Public ingress is
`from_json_validated` (deserialize then semantic validate). Validation is
semantic, not byte-canonical JSON.

## V1-1 Rev1 enforcement

The first V1-1 worktree review found that several stated Rust advantages were
not actually enforced (public mutable version fields, serde-only JSON
ingress, string Policy clauses, non-v7 UUID import). Rev1 keeps Rust as the
proposal and uses privacy, validated constructors, typed graph validation,
and fail-closed serde enums so those advantages are the public API boundary.
V1-1 Gate A accepted this ADR.

## Rejected alternatives

- **Python Core.** Rejected as V1 Core: weaker static separation of identity
  kinds; higher risk of Core/worker language collapse; Preview/Blender Python
  harnesses are not Core authority. Python remains the worker implementation
  language inside Blender, which is a different process.
- **C++ Core.** Rejected as V1 Core: POC-CORE-01 inconclusive; native FFI is
  not a V1-1 requirement; higher memory-safety maintenance for a record/JSON
  Domain.
- **TypeScript/JavaScript Core.** Not a supported candidate. Preview PoC
  viewer language is not Core evidence.
- **Leave OPEN.** Forbidden for V1-1 completion.

## Consequences

Benefits:

- Typed Product IDs, versions, and artifact digests.
- OS plus language process boundary from the Blender worker.
- Cargo.lock reproducibility for the Domain crate.
- GUI and database remain independently selectable later.

Risks / costs:

- V1-2 catalog/SQLite and V1-3 worker orchestration must be shown to fit a
  Rust Core.
- Desktop GUI, if later chosen as a non-Rust toolkit, must talk to Core
  through a process or FFI boundary rather than by rewriting Domain types.
- Team iteration is slower than a Python Domain for schema experiments.
- rustup/MSVC is an additional developer toolchain beside Blender.

## Revisit triggers

Reopen this implementation decision only with material evidence that:

- worker integration disproves process-boundary assumptions
- desktop packaging becomes materially infeasible
- V1-2 catalog/database needs conflict with the choice
- GUI integration reveals unacceptable architectural coupling
- cross-platform support fails
- build/release maintenance becomes disproportionate

A GUI framework choice must not silently reverse this ADR. Native importer
adoption (ufbx or otherwise) may add FFI cost; it does not automatically
restore C++ as Core.
