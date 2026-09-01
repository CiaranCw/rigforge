# IA-1 — Independent W0 Architecture / Research Audit

**Audit status:** `AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING`

**Verdict:** `PASS`

**Sentinel:** `IA1_AUDIT_PASS_CANDIDATE`

**Audit date:** 2026-09-01

## 1 Audit mandate

IA-1 independently challenged the accepted W0 baseline for Product
consistency, V1 scope, historical reconciliation, thin-domain sufficiency,
authority, Blender and Preview boundaries, Mapping/Compatibility/QC,
evidence calibration, generality, provenance, deferrals, implementation
gates, and license/packaging treatment.

This was an adversarial repository-evidence audit. It was not security
testing, implementation, design continuation, a new PoC, technology
selection, or legal advice. No PoC was rerun. Historical evidence was not
repaired.

## 2 Auditor independence

The auditor is a **NEW independent Agent** serving only IA-1. It had no W0
synthesis, historical research, PoC implementation, or prior review role.
Repository evidence, not chat history, was treated as authority.

## 3 Repository baseline

Mandatory preflight passed before any mutation:

```text
branch: main
HEAD: 7a13605952fe85021243928c5d3f0097c86d86e3
refs/remotes/origin/main: 7a13605952fe85021243928c5d3f0097c86d86e3
HEAD subject: docs: complete W0 revised synthesis
working tree: clean
```

The audit then added only the three authorized files under
`docs/research/audits/`. No existing repository file was modified, staged,
committed, or otherwise rewritten.

## 4 Audit method

1. Read W0-RS synthesis and traceability in full as entry points.
2. Read the specified current Product, Architecture, ADR, roadmap, and
   research-baseline authority in full.
3. Read W0.1–W0.4, all scope-revision proposals, the W0-P replan, and all four
   accepted PoC reports in full.
4. Inspect decision-critical E2E and Preview source: frozen Mapping and
   Policy, Job construction, worker execution, reopen/QC/publication order,
   Preview generation, Product fixture, manifest binding, payload integrity,
   browser-control evidence, deletion/regeneration, and request audit.
5. Test each required challenge against repository evidence and separate
   observation, architectural decision, inference, and open work.
6. Search lifecycle and technology-selection wording for contextual
   overclaim and cross-document drift.
7. Record deduplicated defects only; accepted limitations and legitimate
   deferrals are observations.

No external factual verification was required. All conclusions are
`REPOSITORY EVIDENCE` plus explicitly identified `AUDITOR INFERENCE`.

## 5 Source coverage

Every user-mandated source was reviewed in full. Decision-critical committed
source under both requested experiment trees was inspected without execution.
Additional current indexes and the W0.4 license/dependency matrix were
reviewed to test lifecycle and packaging claims.

The comprehensive reviewed-source ledger is in
`IA_1_COVERAGE_MATRIX.md` under “Reviewed sources”.

## 6 Product consistency

**Result: PASS.**

Current authority consistently defines RigForge as a **Character Animation
Asset Workbench** with this transaction:

```text
Character + Motion
  -> Mapping / Compatibility
  -> Transfer
  -> Derived Variant
  -> Preview / Version
```

Character is already rigged/skinned. Motion carries Source Skeleton context.
Derived Variant is first-class and is not a transport-file alias. Mapping
detail is progressively disclosed. No current contract reverts to a general
format platform, Auto-Rig system, DCC replacement, direct engine integration
system, or heavy runtime.

## 7 Scope discipline

**Result: PASS.**

Auto-Rig, Skeleton generation, skinning, raw-mesh transfer, heavy Canonical
runtime, custom evaluator, custom generic retarget runtime, multiple DCC
backends, direct engine integrations, mandatory native FBX/glTF/USD, MCP,
and mandatory AI authority remain excluded, dropped, or deferred. Historical
candidates and PoC harness technologies are not promoted into current V1
scope.

One practical persistence or Preview representation remains allowed where the
workflow needs it. That does not reintroduce mandatory native stacks or
multiple Export Profiles.

## 8 Historical reconciliation

**Result: PASS.**

- W0.1 survives as semantic guards: rest/bind/IBM/default distinctions,
  source-local identity limits, axes/units/time/interpolation provenance,
  FBX recipe/layer loss, and sampled-data derivation. Heavy Canonical-first
  runtime is superseded for V1.
- W0.2 survives as Mapping, layered Compatibility, policy, trajectory, and QC
  knowledge. No solver or custom runtime is selected.
- W0.3 survives as adapter/authority/loss/determinism/process/Preview boundary
  knowledge. “DCC independence” is correctly narrowed to Product semantics
  and replaceability, not zero operational DCC dependency.
- W0.4 survives as option, dependency, license, and risk research. Its C++
  preference, native stack, Preview rankings, original Mandatory W0-P plan,
  and W1–W10 roadmap are not current implementation authority.
- Scope-revision proposal content is reflected in current contracts; proposal
  lifecycle labels remain historical.

Original W0-P dispositions are accounted for: GLTF-01 `MERGE`, RETARGET-01
`MERGE`, PREVIEW-01 `REDEFINE`, ENGINE-01 `DOWNGRADE / DROP_FROM_V1 critical
path`, USD-01 `DEFER`, FBX-02 `DEFER`, DCC-01 `MERGE`, and PREVIEW-02
`DEFER`. No unexecuted original question remains both decision-critical now
and unjustifiably uncovered.

## 9 Workflow Domain sufficiency

**Result: PASS WITH OPEN IMPLEMENTATION VALIDATION.**

The thin domain contains the required conceptual coverage:

```text
Asset / Asset Version
Character and Motion versions
Source Skeleton Reference
Skeleton Summary
Bone Mapping
Compatibility Result
Retarget Policy
Job / Worker Job Spec
Derived Variant
Validation / QC
Preview Artifact
optional Export Artifact
```

The Product transaction does not presently demonstrate a missing concept
that requires a MAJOR finding. Exact schemas, enum names, APIs, persistence,
and migration rules are intentionally open. The principal implementation
risk is maintaining thin, backend-neutral evidence without either hiding
Blender assumptions or rebuilding a DCC clone.

## 10 Authority boundaries

**Result: PASS.**

- Product identity/version, Source Skeleton binding, accepted Mapping,
  Retarget Policy, job intent/state, QC meaning/policy, and Derived Variant
  identity/lineage are authoritative RigForge truth.
- Source-authored semantics remain Source authority.
- Skeleton Summary, Compatibility Result, QC report, Persistence Artifact,
  Preview Artifact, and Preview payload are derived and context-bound.
- Worker scene/evaluated state is ephemeral.
- Export Artifact is an optional derivative.

The current documents sometimes use “authoritative” for the durable record of
a derived judgment, but their explicit matrices preserve the required split:
Product owns meaning and identity; recomputable reports and representations
do not become source or Product authority.

## 11 Blender boundary

**Result: PASS WITH GUARDS.**

Repository source supports the scoped claim: one real humanoid
different-Skeleton pair used an explicit backend-neutral Mapping, Policy, and
Job shape; isolated pinned Blender imported, evaluated, transferred, baked,
saved, reopened in a fresh process, and supplied measurements to an external
QC/publication gate; three semantically consistent runs and one ordinary
missing-input failure were recorded.

Decision-critical source confirms:

- exact Blender build and isolation controls are pinned;
- required Mapping entries fail closed;
- quaternion modes are recognized, supported, executed, and audited;
- each target frame resets to rest before solving;
- non-root `ROTATION_ONLY` keys rotation without location channels;
- worker `SUCCESS` does not publish;
- reopen precedes QC, and QC precedes publication;
- QC records `mutates_subject: false`;
- `bpy`, PoseBone, operators, and `.blend` are confined to adapter mechanics
  and concrete research artifact bindings, not Product identity.

No substantial separate RigForge retarget runtime was required for this
slice. The result does not establish broad retarget quality, all assets,
non-humanoid E2E, worker reliability, packaging, distribution, or backend
replacement.

## 12 Preview boundary

**Result: PASS WITH GUARDS.**

Product refs, Preview Artifact refs, and payload refs are distinct. The viewer
checks selected Product kind/id/version and complete expected lineage against
the manifest, then checks payload size and SHA-256 before display. Controlled
stale binding, wrong version, and payload mismatch fail visibly. Deleting and
regenerating Preview outputs does not mutate Product fixture truth.

Character, Motion, and Derived Variant click-to-preview worked in one Edge
version. Motion was independently previewed using a hierarchy/transform proxy
without selecting the target Character. Blender was absent at view time;
Unreal/Unity were not required; runtime requests remained local/blob. The
generator and payload are replaceable at the architecture boundary.

GLB and `@google/model-viewer` remain unselected harness choices. Real
non-humanoid Preview, production schema, material/texture fidelity, browser
breadth, and platform qualification remain open.

## 13 Mapping / Compatibility / QC

**Result: PASS.**

Current contracts retain all required separations:

```text
names as evidence != identity
Mapping completeness
  != structural/semantic compatibility
  != method/policy eligibility
  != Motion suitability
  != result acceptability

worker success
  != structural validity
  != QC quality
  != production/artistic acceptability
```

Root, pelvis, and trajectory remain distinct. Chain mismatch, helpers, twists,
controls, optional joints, ambiguity, and non-humanoid profiles remain
representable. Ready UX does not claim automatic Mapping is always correct.
QC reports and gates without silent repair; contact, foot-slide, and artistic
acceptance are later evidence problems.

## 14 Evidence calibration

**PROVEN within recorded scope:**

- W0.1–W0.4 semantic and boundary findings are reviewed research evidence.
- POC-CORE-01 executed equivalent C++ and Rust paths; result is
  `INCONCLUSIVE`.
- ufbx inspected the tested fixtures and two Level-3 FBX files with guards.
- one pinned-Blender humanoid transfer path executed, reopened, passed
  structural QC, repeated 3/3 semantically, and failed closed ordinarily.
- one Edge/browser Preview path displayed the tested Character, Motion, and
  Derived Variant with binding/integrity and control evidence.

**NOT ESTABLISHED:**

- a Core language;
- mandatory native ufbx, glTF, or USD infrastructure;
- general retarget quality or all Character/Motion compatibility;
- real non-humanoid Transfer or Preview;
- browser/platform breadth;
- GLB or a viewer as production selection;
- a second backend or demonstrated backend replacement;
- production packaging/distribution/legal clearance;
- exact E2E-to-Preview persistence byte correspondence.

**DEFERRED with owners:** production schemas, GUI/database/viewer/payload,
Auto-Mapping, advanced QC, worker reliability/pooling, native-format
placement, materials, outputs, direct engines, and MCP as detailed in the
coverage matrix.

The final E2E Rev2 semantic run hash is
`f2d650ec8a79f47a30bf485af58321e17258daac2756a30b4f12c83dd3aeaf82`.
Older hashes are revision-scoped. The final inspect path is repository
`run_e2e.py` -> repository `blender_worker.py` -> `inspect(job)` -> raw inspect
JSON.

## 15 Generality

**Result: PASS AT CONTRACT LEVEL / REAL NON-HUMANOID E2E OPEN.**

Durable concepts do not make an engine, format, Blender, FBX, GLB, viewer, or
humanoid role table authoritative. Wolf provides real non-humanoid structural
inspection evidence, not Transfer/Preview proof. One humanoid pair and one
browser are not generalized.

The required non-humanoid checkpoint is correctly retained. Its late failure
could expose lock-in, so it must begin at Mapping/Compatibility validation and
remain a release gate, not be postponed to post-release.

## 16 Provenance / versioning

**Result: PASS WITH PRODUCTION RULES OPEN.**

The baseline identifies Character, Motion, Source Skeleton, Mapping version,
Policy, Job, QC, backend/build, Preview, Persistence Artifact, and Derived
Variant lineage. Production must separately version:

```text
DerivedVariantVersion
PersistenceArtifactVersion + bytes
PreviewArtifactVersion + bytes
```

`DerivedVariantVersion != persistence bytes`. Regeneration may change
representation bytes without changing Product identity unless future Product
policy says otherwise. Exact E2E-to-Preview persistence byte continuity is
correctly retained as `NOT ESTABLISHED`.

## 17 Deferred / open work

**Result: LEGITIMATELY DEFERRED; NONE MISREPRESENTED AS SOLVED.**

The full matrix covers Core language, GUI, database, production schemas,
viewer/payload, ufbx/native glTF/USD placement, Auto-Mapping, non-humanoid,
advanced QC, crash/timeout/retry/cancel, concurrency, Blender packaging/legal,
materials/textures, outputs, direct engines, and MCP.

Each has an earliest implementation owner and release impact. Deferral does
not excuse a release when a declared blocker remains open. No correctly
deferred topic was failed by this audit.

## 18 Implementation gates

**Suitable for implementation if external acceptance/closure: YES.**

This answer is conditional. Candidate PASS authorizes only external review and
possible IA-1 closeout. Implementation remains unauthorized until external
review closes IA-1. V1-1 through V1-8 preserve the required order:
domain/provenance, catalog/orchestration, pinned worker, Mapping/
Compatibility, Transfer/QC/variant, engine-independent Preview, optional
output on concrete need, then real-asset/release hardening.

Future checkpoints are decision-capable for compatible Mapping pairs,
ambiguous/incompatible pairs, real non-humanoid E2E, pair/quality breadth,
Preview shapes, and Blender/package upgrades. Negative evidence may constrain
or reverse the accepted direction.

## 19 License / packaging treatment

**Result: PASS AS RISK TREATMENT / CLEARANCE OPEN.**

Repository treatment is calibrated: Blender is GPL; process isolation is an
engineering boundary, not a legal safe harbor; distributed worker scripts,
notices, packaging architecture, and dependency obligations require legal
review. Packaging size, installation, supported platforms, updates, rollback,
and upgrade qualification remain release blockers.

No new legal conclusion is made here.

## 20 Guard completeness

**Material lost guards: NONE.**

W0-RS captures all decision-critical Blender and Preview guards. Current
canonical Product/Architecture documents capture the guards necessary at
their abstraction level, and the roadmap assigns remaining implementation
and release owners. Detailed guard-by-guard evidence is in
`IA_1_COVERAGE_MATRIX.md`.

The frozen-pair and one-browser limitations remain visible. Core language,
viewer/payload, non-humanoid E2E, packaging/legal, and production schema are
not silently closed.

## 21 Cross-document consistency

**Material contradictions: 0.**

One nonblocking current-index lifecycle contradiction is recorded as
IA1-MINOR-001: `docs/research/poc/README.md` still directs readers to W0-RS
and labels it `READY / NOT STARTED`, although canonical lifecycle sources and
W0-RS itself record completion. Historical PoC/proposal lifecycle snapshots
are intentionally immutable and are not defects.

Required contradiction rows and their classifications are in
`IA_1_COVERAGE_MATRIX.md`.

## 22 Finding summary

```text
OPEN MAJOR: 0
OPEN MINOR: 1
OBSERVATIONS: 6
```

The authoritative inventory is `IA_1_FINDINGS.md`. No MAJOR finding blocks
PASS. IA1-MINOR-001 is a real traceability defect and is nonblocking.

## 23 Audit verdict

```text
VERDICT: PASS
SENTINEL: IA1_AUDIT_PASS_CANDIDATE

W0: ACTIVE
W0-RS: COMPLETE / PASS / BASELINED
IA-1: AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING
product implementation: NOT STARTED
```

Candidate PASS means external closeout may proceed. This report does not
baseline IA-1, close W0, or authorize implementation.
