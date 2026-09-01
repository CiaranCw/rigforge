# W0-RS — Revised W0 Synthesis

**Status:** `COMPLETE / PASS / BASELINED`

**Source baseline:** `a1dce331fe0d145405a8b05497e2ffe749d94232`

**Classification:** FINAL W0 SYNTHESIS FOR INDEPENDENT AUDIT / NOT PRODUCT
IMPLEMENTATION

This document reconciles accepted W0 research, the adopted W0 scope revision,
and the two real vertical PoCs into one decision-oriented baseline. It does
not replace the current Product or Architecture contracts, rewrite historical
evidence, execute IA-1, or authorize implementation.

## 1. Synthesis conclusion

RigForge V1 is a **Character Animation Asset Workbench**, not a general
character-format platform, DCC replacement, Auto-Rig product, game-engine
integration, or complete animation runtime.

The accepted V1 direction is:

```text
browse Character + Motion
        ↓
select one of each
        ↓
automatic Mapping / Compatibility preflight
        ↓
Ready
        ↓
Transfer
        ↓
Derived Variant
        ↓
Preview / Version
```

The product owns identity, versions, Mapping, Compatibility, Retarget Policy,
job intent, QC meaning, Derived Variant lineage, Preview orchestration, and
publication. A pinned isolated Blender worker may execute the first transfer
path behind backend-neutral contracts. Preview is a separate derived,
rebuildable, non-authoritative path whose binding and payload integrity are
validated against independently owned Product truth before display.

Two scoped vertical PoCs support this direction:

- POC-BLENDER-E2E-01 established one real humanoid cross-Skeleton transfer
  path with a hidden Blender worker and QC-gated publication.
- POC-PREVIEW-01R established browser click-to-preview for the tested
  Character, Motion, and Derived Variant without Blender or a game engine at
  view time.

Both decisions include material guards. Neither PoC establishes broad asset,
format, browser, non-humanoid, production-quality, packaging, or release
coverage. Core language, GUI framework, database, production schema, viewer
library, and Preview payload format remain open.

No material contradiction was found between the accepted evidence and the
current canonical Product / Architecture truth. W0 remains active. This
synthesis completed external focused review and is the accepted W0 revised
synthesis feeding IA-1. IA-1 has NOT started. Product implementation is NOT
authorized.

## 2. Authority and document hierarchy

When wording differs, use this order:

1. Current Product / Architecture contracts and accepted ADRs define current
   project truth.
2. Accepted research and PoCs provide evidence, limits, and guards.
3. This document reconciles that evidence; it does not silently amend either
   level.
4. Historical proposals explain why the current direction was adopted; they
   are not current contracts.

Current contracts are:

- [Product Vision](../../product/PRODUCT_VISION.md)
- [V1 Scope](../../product/V1_SCOPE.md)
- [Architecture](../../architecture/README.md)
- [V1 Blender-Backed Architecture](../../architecture/V1_BLENDER_BACKED_ARCHITECTURE.md)
- [V1 Workflow Domain](../../architecture/V1_WORKFLOW_DOMAIN.md)
- [ADR-0001](../../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md)
- [Roadmap](../../development/ROADMAP.md)

Audit traceability is in
[W0_RS_TRACEABILITY.md](W0_RS_TRACEABILITY.md).

## 3. Final accepted product definition

### 3.1 Inputs and result

**Character Asset**

```text
already rigged / skinned
mesh + Skeleton / Armature + skin weights
```

Raw unrigged mesh is not a V1 transfer target. RigForge does not generate the
Skeleton or skinning in V1.

**Motion Asset**

```text
animation
+
Source Skeleton reference / context
+
time-domain provenance
```

Several Motions may share one Source Skeleton. Missing source Skeleton context
is diagnosable and must not be silently invented.

**Derived Variant**

A first-class, versioned Product result with exact Character, Motion, Source
Skeleton, Mapping, Retarget Policy, Job, QC, artifact, backend/build, and
generation provenance. It is not defined as a copied FBX, `.blend`, GLB, or
other transport file.

### 3.2 In-scope capabilities

- Local-first browsing and management of Character, Motion, and Derived
  Variant assets.
- One Character plus one Motion selection and a Transfer Tray.
- Thin, portable Skeleton Summary evidence.
- Automatic Mapping / Compatibility preflight with progressive disclosure.
- Product-owned Mapping, Compatibility, Retarget Policy, Job, QC meaning,
  versions, provenance, and publication.
- Hidden Blender-backed transfer, accepted with guards and subject to
  implementation qualification.
- Derived, engine-independent Preview, validated with guards.
- Basic structural QC that reports rather than silently repairs.
- Optional derived output only when a concrete consumer requires one.

### 3.3 Explicit V1 exclusions and deferrals

| Topic | V1 disposition |
| --- | --- |
| Auto-Rig | `DROP_FROM_V1` |
| Automatic Skeleton generation | `DROP_FROM_V1` |
| Automatic skinning | `DROP_FROM_V1` |
| Raw unrigged mesh transfer | `DROP_FROM_V1` |
| Heavy Canonical asset runtime | Not V1 critical path |
| Custom animation evaluator | `DROP_FROM_V1` |
| Custom retarget runtime | `DROP_FROM_V1` |
| Multiple DCC backends | `DROP_FROM_V1` |
| Direct Unreal / Unity / Godot integrations | `DROP_FROM_V1` |
| Mandatory native FBX stack | `DEFER` |
| Mandatory native glTF stack | `DEFER` |
| Mandatory OpenUSD stack | `DEFER` |
| Multiple Export Profiles | `DEFER` / implementation-gated |
| MCP | `DEFER` |
| Mandatory AI authority | `DEFER`; silent authority is forbidden |

Historical research into these topics remains useful risk and option
knowledge. It does not restore them to the V1 critical path.

## 4. Current architecture

```text
Asset Browser / Workbench
        ↓
Thin Workflow Domain
        ↓
Character / Motion / Mapping / Compatibility / Policy
        ↓
backend-neutral Job Spec
        ↓
isolated Blender Worker
        ↓
Result / measurements
        ↓
RigForge QC / publication gate
        ↓
Derived Variant
        ↓
Preview Artifact
        ↓
binding / integrity validation
        ↓
engine-independent Viewer
```

The durable Workflow Domain remains thin. Its concepts include Asset, Asset
Version, Character Asset, Motion Asset, Source Skeleton Reference, Skeleton
Summary, Bone Mapping, Compatibility Result, Retarget Policy, Job / Worker
Job Spec, Derived Variant, Validation / QC, Preview Artifact, and optional
Export Artifact. Exact production schemas, fields, enum names, storage, and
APIs remain **OPEN**.

Blender owns disposable execution mechanics within a process boundary. It
does not own Product identity or policy. A future worker must be able to
consume the durable Job Spec without pretending to be Blender. This
replaceability is an architectural property; a second worker has not been
implemented or demonstrated.

### 4.1 Authority matrix

| Concept | Classification | Owning layer | Boundary |
| --- | --- | --- | --- |
| Product Asset identity / version | `AUTHORITATIVE` | RigForge Product Domain | Independent of filenames, DCC objects, and payload paths |
| Source file semantics | `SOURCE AUTHORITY` | Source representation plus its format semantics | RigForge records identity, provenance, interpretation, and loss; it does not rewrite source facts |
| Source Skeleton reference | `AUTHORITATIVE` | RigForge Product Domain | Binds Motion to its source context; not a humanoid-slot table |
| Skeleton Summary | `DERIVED` | RigForge-defined evidence, extracted by an inspector/worker | Rebuildable portable facts; not complete source or DCC state |
| Bone Mapping | `AUTHORITATIVE` | RigForge Product Domain | Accepted correspondence and evidence; worker receives a projection |
| Compatibility Result | `DERIVED` | RigForge Product Domain | Product judgment for exact versions, Mapping, Policy, and capability |
| Retarget Policy | `AUTHORITATIVE` | RigForge Product Domain | Backend-neutral intent; worker mechanics are not policy |
| Worker execution state | `EPHEMERAL` | Isolated worker | Disposable scene, evaluated poses, constraints, and temporary files |
| QC meaning / acceptance policy | `AUTHORITATIVE` | RigForge Product Domain | Worker may measure; success and measurements do not publish by themselves |
| Validation / QC report | `DERIVED` | RigForge validation layer | Bound to exact subject, policy, versions, and context; must not mutate its subject |
| Derived Variant identity / lineage | `AUTHORITATIVE` | RigForge Product Domain | First-class result; persistence/output bytes are representations |
| Persistence Artifact | `DERIVED` | RigForge artifact orchestration | Versioned representation and byte identity; not the sole Derived Variant identity |
| Preview Artifact | `DERIVED` | RigForge Preview orchestration | Bound, rebuildable, non-authoritative; cannot validate Product truth |
| Preview payload | `DERIVED` | Replaceable Preview generation/transport path | Identity/integrity checked before display; GLB is not selected |
| Export Artifact | `OPTIONAL DERIVATIVE` | RigForge output orchestration | Exists only for a concrete consumer; never Product identity |

## 5. Historical research disposition

Historical documents retain their accepted status. Their original language
must be read in the context of the later adopted scope revision.

### 5.1 W0.1 — Canonical Foundations

**Current disposition:** accepted semantic risk knowledge; the full
Canonical-first runtime is `SUPERSEDED_FOR_V1`.

Keep:

- Rest, bind, inverse bind, default transforms, and geometry bind are not one
  universal object.
- Source-local names are evidence, not stable identity.
- Joint order is a skin-binding contract, not a universal Skeleton model.
- Humanoid semantics are optional profile data, not the Skeleton definition.
- Axes, units, transform recipes, time domains, interpolation, missing-channel
  fallback, and authored-versus-evaluated distinctions require explicit
  interpretation and provenance.
- FBX pivot/inherit/geometric/layer semantics can be lost by simple TRS
  flattening; glTF and UsdSkel have different contracts.
- Sampled TRS may be a derived cook; it must not silently become source
  authority.
- Format-independent reasoning and explicit semantic-loss reporting survive.

Superseded or not required for V1:

- A complete Product-owned Canonical Character / Skeleton / Motion runtime as
  the first implementation foundation.
- Complete duplication of source/DCC semantics in durable Product objects.
- Native format adapters as the mandatory V1 critical path.

This research was not wrong. Its distinctions constrain the worker boundary,
Skeleton Summary, Mapping, provenance, QC, and loss reporting.

### 5.2 W0.2 — Mapping / Compatibility / Retarget

**Current disposition:** Product capability and policy knowledge retained;
solver selection and a custom RigForge retarget runtime are not selected.

Keep:

```text
MAPPING_COMPLETE
!=
RETARGET_COMPATIBLE
!=
RESULT_ACCEPTABLE
```

- Mapping is first-class and product-owned.
- Names are evidence, not identity.
- Mapping correspondence is distinct from retarget distribution.
- Chains, helpers, twists, controls, end joints, optional joints, ambiguity,
  and unequal chain counts must remain representable without making humanoid
  roles universal.
- Root, pelvis, and motion trajectory are distinct; pair compatibility is not
  motion-trajectory suitability.
- Rest/base-pose alignment, units, proportions, translation policy,
  time-domain handling, and missing-channel behavior are decision-relevant.
- Retarget Policy records Product intent independently of worker mechanics.
- QC separates execution, structural validity, quality signals, and
  production acceptability. Universal artistic thresholds were not found.

Not selected:

- One exact retarget solver as Product architecture.
- Deterministic IK placement as a general requirement. IK is compatible with
  deterministic-first principles but remains policy/capability-specific.
- Automatic Foot IK as a V1 core feature.

### 5.3 W0.3 — Adapter / Infrastructure / Preview Boundary

**Current disposition:** boundary and guard knowledge retained; original
zero-DCC operational interpretation reconciled with ADR-0001.

Keep:

- Adapter / parser / worker success does not create Product authority.
- Capabilities and expected semantic losses are declared before mutation.
- Provenance records adapter/worker identity, versions, configuration, source
  identity, and declared execution context.
- Determinism claims require a declared context and must distinguish
  semantic, structural, and byte determinism.
- Authoritative and derived representations remain visibly separated.
- Runtime cooks, compressed clips, exports, and Preview cannot
  reverse-authorize Product or source truth.
- Engine-independent Preview remains a boundary requirement and now has
  scoped browser-PoC evidence.
- Storage, resolver, composition, and cooked-runtime distinctions remain
  useful later risk knowledge.

Reconciliation of DCC independence:

```text
DCC independence now means:
  Product semantics are DCC-independent
  durable contracts are backend-neutral
  the execution boundary is replaceable

It does not mean:
  V1 has zero operational DCC dependency
```

The accepted V1 direction may operationally depend on one pinned hidden
Blender worker. Blender, Maya, game engines, and formats remain rejected as
Product authority.

### 5.4 W0.4 — Product / Technology Decisions

**Current disposition:** accepted historical option, dependency, license, and
risk research; its technology rankings and original W0-P plan are not current
implementation authority.

- The historical C++ preference was provisional and is not selected after
  POC-CORE-01 returned `INCONCLUSIVE`.
- Historical native FBX/glTF critical-path preferences are
  `SUPERSEDED_FOR_V1`; POC-FBX evidence survives without mandating an importer.
- Preview host, renderer, and payload rankings are not selected.
- The original six Mandatory W0-P sequence was `SUPERSEDED / REPLANNED` by
  W0-SR.
- I/O asymmetry survives: V1 need not write every format it reads.
- W0.4's “Domain AssetReference / Provenance, not an AMS” survives as a
  rejection of a mandatory external/enterprise asset-management dependency.
  It does not remove the current local Asset Browser, catalog, versions, or
  provenance from V1.
- Generic engine and DCC boundaries, derived-not-authoritative outputs,
  dependency pinning, explicit license classification, redistribution review,
  ABI/process isolation, and fail-closed unsupported-feature policy remain
  relevant.
- MCP, AI, runtime cook implementations, multiple DCC/engine products, and
  broad format infrastructure remain deferred or outside V1 as recorded in
  current scope.

## 6. Scope revision and accepted PoC decisions

### 6.1 W0-SR — adopted product-direction reconciliation

**Status:** `COMPLETE / ADOPTED / BASELINED`

W0-SR changed the active V1 direction without invalidating earlier research:

```text
heavy Canonical-first infrastructure
        ↓
thin Workflow Domain

native custom retarget runtime
        ↓
backend-neutral Policy + hidden worker execution

engine-specific Preview/export path
        ↓
derived engine-independent Preview
```

### 6.2 POC-CORE-01

```text
Status: COMPLETE / PASS / BASELINED
Decision Impact: INCONCLUSIVE
Core language: NOT SELECTED
```

Established:

- The C++ raw-C ABI path worked for the tested equivalent slice.
- The Rust path using the official ufbx crate worked.
- The supported Rust binding removed much of the alleged handwritten-FFI tax
  for the dependency actually tested.

Not established: C++, Rust, Python, or any other language as RigForge Core.
Python/bpy and browser JavaScript in later PoCs are research harness choices,
not language selections.

### 6.3 POC-FBX-01

```text
Status: COMPLETE / PASS / BASELINED
Decision Impact: KEEP_UFBX_WITH_GUARDS
```

Established: ufbx is technically credible for inspected FBX facts on the
tested synthetic/curated material and the two frozen Level-3 files.

Not established: a mandatory native ufbx V1 importer, broad exporter-family
coverage, non-empty multilayer composition, complex Maya animation, or
lossless FBX round-trip.

Surviving guards:

- inspect source axes and units before normalization;
- distinguish authored transforms from evaluated transforms;
- inventory animation stacks, layers, time ranges, and time modes before
  flattening;
- do not equate a bone attribute with a deforming cluster bone;
- preserve rest/bind/pose/cluster distinctions;
- treat names as evidence;
- do not assume skin weights are normalized;
- keep non-empty multilayer/additive Maya animation guarded and unvalidated;
- remember the Level-3 files share a Blender 2.79 exporter family;
- do not require Autodesk FBX SDK fallback based on the tested material;
- keep the FBX Adapter non-authoritative.

### 6.4 POC-BLENDER-E2E-01

```text
Status: COMPLETE / PASS / BASELINED
Decision: ACCEPT_BLENDER_BACKEND_WITH_GUARDS
```

Established for one frozen real humanoid cross-Skeleton pair:

- backend-neutral Mapping, Retarget Policy, Job Spec, Result Envelope, and
  Product lineage;
- hidden isolated Blender import, transfer, bake, and persistence;
- fresh-process reopen;
- structural/QC-gated publication;
- three repeated semantically consistent successful runs;
- an ordinary missing-input failure that failed closed;
- no large custom RigForge retarget runtime was required for this slice.

Not established:

- all Character/Motion pairs or Blender versions;
- real non-humanoid E2E;
- Auto-Mapping quality or incompatible/ambiguous pair handling;
- production retarget, contact, foot-slide, or artistic quality;
- crash/timeout/cancel/retry campaigns or worker-pool scheduling;
- Blender packaging, redistribution, licensing, or legal qualification.

### 6.5 POC-PREVIEW-01R

```text
Status: COMPLETE / PASS / BASELINED
Decision: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
```

Established for the frozen research set:

- Character click-to-preview;
- Motion click-to-preview without selecting a target Character;
- Derived Variant click-to-preview;
- a synthetic generic hierarchy/transform Motion proxy;
- browser play, pause, seek, and restart for animated previews;
- measured camera interaction on the tested views;
- view-time operation without Blender, a game engine, an external CDN, or a
  full RigForge animation runtime;
- selected Product id/version/lineage validated against independently owned
  Product fixture truth;
- payload size and SHA-256 checked before valid display;
- stale binding and payload mismatch failed closed;
- literal Preview deletion/regeneration without Product identity change;
- audited runtime requests with no external network resource.

Not established:

- `@google/model-viewer` as V1 viewer;
- GLB as permanent Preview payload or source round-trip;
- multiple browsers, mobile, AR, engine parity, or broad platform support;
- material-perfect or texture-complete fidelity;
- real non-humanoid Preview;
- a production Preview schema.

## 7. Consolidated normative guards

### 7.1 Blender execution guards

1. Pin the exact Blender version/build and record it in execution provenance.
2. Run isolated/headless with factory startup, controlled configuration,
   disabled automatic embedded-script execution, job-specific temporary
   storage, captured diagnostics, and an explicit non-zero failure path.
3. Keep durable Asset, Mapping, Compatibility, Policy, Job, Result, QC, and
   Derived Variant contracts backend-neutral.
4. Validate the reviewed Mapping and fail closed on missing required joints or
   unsupported correspondence.
5. Require each Retarget Policy capability to be explicitly recognized,
   supported, executed, and auditable. Unsupported policy fails before
   publication.
6. Per-frame execution must initialize/evaluate required state explicitly and
   must not depend on accidental residual pose state.
7. `ROTATION_ONLY` semantics must not leak ordinary joint translation; root,
   trajectory, scale, twist, helper, and missing-channel behavior remain
   explicit policy.
8. Worker `SUCCESS` is not publication. Publication requires staged outputs,
   persistence, fresh-process reopen, structural validation, and applicable
   RigForge QC.
9. QC reports and gates; it must not silently mutate or repair the result.
10. Treat Preview/export bytes as derivatives, not worker-created Product
    truth.
11. Do not generalize from the one humanoid pair. Real non-humanoid and
    broader pair hardening remain required.
12. Blender packaging, redistribution, upgrade, and legal obligations remain
    open. Core language remains not selected.

### 7.2 Preview guards

```text
Preview = DERIVED / REBUILDABLE / NON-AUTHORITATIVE

Product truth validates Preview.
Preview never validates Product truth.

ProductRef
!=
PreviewArtifactRef
!=
payload path
```

1. Product kind, id, version, and relevant source/version lineage are
   independently owned and validated before display.
2. Derived Variant binding preserves applicable Character, Motion, clip,
   Mapping, Retarget Policy, Job, QC, and persistence/source artifact lineage.
3. Payload identity/integrity is checked before valid display. The exact
   production mechanism remains open.
4. Stale, mismatched, missing, or corrupt Preview fails visibly; no silent
   stale display.
5. Deleting or regenerating Preview must not mutate Character, Motion, Derived
   Variant, Mapping, Retarget Policy, or QC identity.
6. Motion Preview remains independent of target Character selection.
7. The Motion proxy is generic hierarchy/transform-derived evidence, not
   Canonical Skeleton or humanoid-role authority.
8. Viewer runtime remains DCC- and game-engine-independent. Generation backend
   remains replaceable.
9. GLB is neither Product/source authority nor a permanent selection.
   Viewer library is not selected.
10. Preserve declared losses. Non-humanoid coverage, material/texture fidelity,
    broader platforms, and production schema remain open.

## 8. Original W0-P and roadmap disposition

Unexecuted superseded PoCs are not failures:

| Original PoC | Final disposition | Current meaning |
| --- | --- | --- |
| POC-GLTF-01 | `MERGE` | Candidate Preview artifact/reopen evidence absorbed; native parser and permanent payload selection deferred |
| POC-RETARGET-01 | `MERGE` | Minimal execution slice absorbed into POC-BLENDER-E2E-01; breadth and quality move to later V1 validation |
| Original POC-PREVIEW-01 | `REDEFINE` | Executed as POC-PREVIEW-01R |
| POC-ENGINE-01 | `DOWNGRADE / DROP_FROM_V1 critical path` | Direct engine work is outside V1 |
| POC-USD-01 | `DEFER` | Revisit only for a concrete workflow |
| POC-FBX-02 | `DEFER` | Native FBX write only for a concrete output requirement |
| POC-DCC-01 | `MERGE` | First process observations absorbed into Blender E2E; full qualification moves to worker hardening |
| POC-PREVIEW-02 | `DEFER` | Conditional only if later evidence shows the accepted Preview path is insufficient |

The historical W1–W10 implementation roadmap is
**SUPERSEDED AS THE ACTIVE PLAN**. Historical references remain evidence of
prior reasoning.

Current implementation planning labels, not frozen lifecycle IDs:

```text
V1-1 thin Workflow Domain / provenance
V1-2 local-first Asset Catalog / orchestration
V1-3 pinned isolated Blender worker
V1-4 Mapping / Compatibility
V1-5 Transfer / QC / Derived Variant
V1-6 engine-independent Preview
V1-7 optional artifact/output if a concrete consumer requires it
V1-8 real-asset hardening / release audit
```

## 9. Evidence strength and real-asset ledger

### 9.1 Evidence-strength matrix

| Claim | Strength | Scope / interpretation |
| --- | --- | --- |
| Rest/bind/IBM, identity, axes/units, time-domain, and loss distinctions matter | `RESEARCH_REVIEWED` | Accepted cross-format semantic research |
| Mapping, Compatibility, Policy, and QC are separate Product concerns | `RESEARCH_REVIEWED` + `ARCHITECTURAL_DECISION` | Shapes the thin Product Domain |
| Full Canonical-first runtime is not V1 critical path | `ARCHITECTURAL_DECISION` | W0-SR + ADR-0001 |
| Core language is selected | `OPEN` | POC-CORE-01 was `INCONCLUSIVE` |
| ufbx can inspect the tested FBX corpus with guards | `REAL_ASSET_POC` | Two Level-3 files plus lower-level fixtures |
| Native ufbx importer is mandatory in V1 | `OPEN` / not established | Current scope defers the choice |
| Blender can execute the frozen cross-Skeleton transfer path | `REAL_ASSET_POC` | One humanoid pair, one pinned Blender build |
| Blender works for all Character/Motion or non-humanoid cases | `OPEN` / not established | Later real-asset evidence required |
| Durable transfer contracts can avoid Blender semantics for the tested slice | `REAL_ASSET_POC` + `ARCHITECTURAL_DECISION` | Future backend replacement remains an inference, not a second implementation |
| Character, Motion, and Derived Variant can use the tested derived Preview path | `BROWSER_POC` | One research set in one Edge version |
| GLB or model-viewer should be permanent V1 technology | `OPEN` / not established | Explicitly not selected |
| Product contracts can remain non-humanoid-capable | `RESEARCH_REVIEWED` + `ARCHITECTURAL_DECISION` | Contract generality |
| Real non-humanoid transfer/Preview works | `OPEN` | Not yet validated |

### 9.2 Identity-critical real-asset ledger

| Evidence | Identity / pin | Decision relevance |
| --- | --- | --- |
| POC-FBX humanoid L3 | `Knight_Male.fbx`; SHA-256 `fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f` | Real humanoid FBX inspection |
| POC-FBX non-humanoid L3 | `Wolf.fbx`; SHA-256 `91d5c31678fa291571f82abb029fde5c14fdd5932a1569c3afaa5e9602bebf94` | Structural inspection without humanoid-only assumptions; not transfer E2E |
| POC-FBX Maya pivot L2 | `maya_pivots_7500_ascii.fbx`; SHA-256 `6bedb4ebb449e42624a5b698f3f1c76d88e1790fe53b96158b00946c57ff6b2d` | Authored pivot recipe differs from plain local TRS |
| Blender E2E Character | `Knight_Male.fbx`; SHA-256 `fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f` | Frozen target Character |
| Blender E2E Motion | `UAL2_Standard.fbx`; SHA-256 `d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d`; clip `Armature\|Armature\|Walk_Carry_Loop` | Frozen source Motion / Skeleton context |
| Blender E2E backend | Blender `5.2.1 LTS`; build `9e2066aef7ef` | Exact tested execution context |
| Preview lineage | Same frozen Knight + UAL2 source and accepted Mapping / Policy / Job lineage | Preview evidence attributes its persistence representation to that lineage; exact byte correspondence with E2E §15 is not established from repository-only evidence |
| Preview browser | Microsoft Edge `151.0.4129.107` | One qualified desktop browser only |
| Preview harness | `@google/model-viewer` `4.3.1` | Research harness only / not selected |

## 10. Product generality audit

```text
Engine-specific Product authority: NO
Format-specific Product authority: NO
Blender Product authority: NO
FBX Product authority: NO
GLB Product authority: NO
Humanoid-only durable Domain: NO
Auto-Rig required: NO
Raw mesh required: NO
Specific viewer required: NO
Core language selected: NO
```

Contract generality and evidence coverage are different:

| Area | Contract direction | Real evidence coverage |
| --- | --- | --- |
| Character/Motion identity | Format- and engine-independent | FBX used as the frozen vehicle |
| Mapping / Skeleton Summary | Structural, profile-capable, not humanoid-only | Wolf inspected structurally; transfer E2E used one humanoid pair |
| Worker boundary | Backend-neutral durable contract | Only Blender implemented/tested |
| Preview | Product-bound, payload-neutral, generation-replaceable | One GLB/browser research path |
| Non-humanoid | Allowed and must not be structurally excluded | Real transfer and Preview E2E not yet validated |

Generality is a requirement on durable contracts. It is not evidence that
untested assets already work.

## 11. Consolidated open and deferred work

| Topic | Current state | Why not decided now | Required evidence | Earliest stage | Can block release? |
| --- | --- | --- | --- | --- | --- |
| Core language | `OPEN`; POC result `INCONCLUSIVE` | Tested paths had no decisive margin | Concrete Domain/worker integration, ownership, packaging, build, and maintenance evidence | V1-1 planning | Yes, implementation choice; not a W0 direction blocker |
| GUI framework | `OPEN` | Preview harness is not Product UI evidence | Workbench interaction, accessibility, packaging, offline/local I/O, and maintenance spike | V1-2 planning | Yes |
| Database / catalog storage | `OPEN` | Product schema and scale are not frozen | Local-first catalog transactions, migration, provenance, recovery, and concurrency needs | V1-1/V1-2 | Yes |
| Production schema | `OPEN` | PoC fixtures are research contracts only | Versioning, migrations, backend replacement, exact lineage, and failure semantics | V1-1 | Yes |
| Viewer library | `OPEN` | One research harness cannot select V1 technology | Product UI integration, animation/control, security, offline packaging, fidelity, and platform qualification | V1-6 | Yes |
| Preview payload format | `OPEN`; GLB candidate only | Tested transport is not source/round-trip authority | Required asset shapes, animation, overlays, losses, size/performance, packaging, and regeneration | V1-6 | Yes |
| Native ufbx inspector | `DEFER / optional` | Technical credibility does not prove Product necessity | Measured need versus Blender inspection; unsupported-feature policy and corpus breadth | V1-3/V1-4 | No unless required facts/performance cannot be met otherwise |
| Native glTF stack | `DEFER` | Original critical path was replanned | Concrete Preview/export consumer and semantic-loss tests | V1-6/V1-7 | Only if selected payload/output needs it |
| OpenUSD | `DEFER` | No current V1 workflow requires it | Concrete composed-USD workflow, resolver context, packaging, and value | Post-V1 or V1-7 exception | No for current V1 |
| Auto-Mapping algorithm | `OPEN` | Frozen E2E used reviewed Mapping | Real compatible, ambiguous, incompatible, twist/helper, and non-humanoid pairs | V1-4 | Yes |
| Non-humanoid support | Contractually allowed; real E2E `OPEN` | Wolf inspection is not transfer/Preview proof | Real non-humanoid Character + Motion + Mapping + Transfer + QC + Preview | V1-4/V1-8 | Yes for claimed support and release scope |
| Advanced QC / foot contact | Later hardening | One clip and structural checks cannot set universal quality thresholds | Locomotion/contact corpus, spaces, thresholds, false-positive review, user acceptance | V1-5/V1-8 | Depends on release quality claims |
| Worker crash/timeout/retry/cancel | `OPEN` | PoC tested ordinary fail-closed behavior only | Fault injection, process kill, cleanup, idempotency, retry, cancel, partial-output campaigns | V1-3/V1-8 | Yes |
| Concurrency / worker pool | `OPEN` | One-process slice did not test scheduling | Resource isolation, fairness, throughput, cancellation, crash containment | V1-3/V1-8 | Yes for release scale |
| Blender packaging | `OPEN` | Research machine pin is not distributable Product packaging | Reproducible install, size, updates, security, platform support, rollback | V1-3/V1-8 | Yes |
| Blender licensing / distribution legal review | `OPEN / legal review required` | Technical PoC is not legal clearance | Distribution architecture, licenses/notices, worker-script and dependency obligations, counsel review | Before distribution | Yes |
| Material / texture fidelity | Incomplete | Preview PoC simplified appearance | Representative materials/textures, color management, unsupported features, loss UX | V1-6/V1-8 | Depends on acceptance criteria |
| Export Artifact formats | `OPEN / optional` | No concrete downstream consumer selected | Consumer requirements, interop, round-trip expectations, loss declaration | V1-7 | No unless a release consumer requires export |
| Multiple Export Profiles | `DEFER / implementation-gated` | Premature without consumers | At least two concrete consumers with incompatible requirements | Post-first consumer | No for core V1 |
| Direct engine integrations | `DROP_FROM_V1` | Not part of accepted transaction | Changed Product requirement plus engine-specific value/maintenance evidence | Post-V1 | No |
| MCP | `DEFER` | Does not shape current Product Domain | Stable Domain API/CLI and a concrete automation use case | Post-Domain / post-V1 decision | No |

Future production provenance must distinguish `DerivedVariantVersion` from
the version and byte identity of each Persistence Artifact and Preview
Artifact. Regenerating a representation may change its bytes without creating
a new Product identity unless Product policy explicitly says otherwise.
Exact production versioning rules remain **OPEN**.

Open topics must not be represented as solved merely because a historical
candidate was ranked or a research harness used one implementation.

## 12. Decision-relevant risks

| Risk | Known guard | Open exposure |
| --- | --- | --- |
| Blender operational dependency / version churn | Pin build, isolate process, record context, fail closed | Upgrade rehearsal, packaging, rollback, supported platforms |
| Importer semantic loss | Source hashes, authored/evaluated distinction, capability/loss declarations | Broader exporter families, layered animation, unsupported constructs |
| Domain too thin | Add only Product facts proven necessary; preserve provenance and evidence | Later Mapping/QC/backend replacement may expose missing facts |
| Domain too rich / DCC clone recreated | Keep durable concepts workflow-oriented and backend-neutral | Implementation can still over-copy scene/source state |
| Mapping ambiguity hidden by one-click UX | Compatibility gate, progressive disclosure, explicit confirmation | Auto-Mapping quality and ambiguous-pair UX untested |
| Worker success confused with QC success | Separate result, reopen, validation, QC, publication | Production thresholds and fault campaigns incomplete |
| Humanoid first-pair overgeneralization | Non-humanoid-neutral contracts; explicit evidence labels | Real non-humanoid transfer/Preview absent |
| Preview becoming authority | Product validates Preview; distinct references; deletion invariant | Production persistence/schema not designed |
| Stale or misbound Preview payload | Exact binding and integrity validation; visible fail-closed behavior | Production transport, cache invalidation, signing/trust model open |
| Material / texture fidelity | Declare loss and do not claim fidelity | Broader representative evidence absent |
| Packaging / GPL / worker-script obligations | Keep boundary explicit; require legal review | Distribution model and legal clearance open |
| Core / GUI / database premature lock-in | Keep selections open until concrete implementation evidence | Choices become blockers when implementation planning begins |

## 13. Required later real-asset checkpoints

Future V1 work must preserve evidence-driven decision gates:

1. Auto-Mapping and Compatibility across real compatible pairs.
2. Incompatible and ambiguous pairs with visible, fail-closed explanations.
3. Real non-humanoid Character + Motion Mapping, Transfer, QC, persistence,
   reopen, and Preview.
4. Transfer/QC breadth across rest poses, proportions, twist/helper layouts,
   root/trajectory policies, missing channels, and locomotion/contact cases.
5. Preview across broader Character, Motion, Derived Variant, material,
   texture, hierarchy, duration, and payload shapes.
6. Blender/package upgrade qualification with semantic comparison, failure
   campaigns, redistribution review, and rollback.

Observed results may narrow, reverse, or add constraints to the technical
direction. These checkpoints are not paperwork exercises.

## 14. Decisions not to reopen without new evidence

Reopen these only with material new evidence or changed Product requirements:

- Do not restore heavy Canonical-first V1 merely because historical research
  uses Canonical terminology.
- Do not require native FBX/glTF/USD stacks merely because test assets or
  Preview payloads use those formats.
- Do not make Blender, a Blender scene, or worker output Product authority.
- Do not make GLB or `@google/model-viewer` permanent because the Preview PoC
  used them.
- Do not select Core language from the language of a research harness.
- Do not make Auto-Rig, raw mesh processing, custom evaluator/retarget runtime,
  multiple DCCs, or direct engines part of V1 without a changed requirement.
- Do not treat one humanoid pair, one browser, or one research set as broad
  qualification.

This is scope discipline, not a ban on evidence-driven reversal.

## 15. V1 implementation gates and lifecycle

Current state:

```text
W0: ACTIVE
W0-RS: COMPLETE / PASS / BASELINED
IA-1: READY / NOT STARTED
product implementation: NOT STARTED
```

Required sequence:

```text
W0-RS
COMPLETE / PASS / BASELINED
        ↓
IA-1
READY / NOT STARTED
        ↓
V1 implementation
ONLY IF IA-1 PASS / CLOSED
```

This synthesis does not claim `W0 COMPLETE`. IA-1 is authorized to start and
has NOT been executed here. Product implementation remains unauthorized until
IA-1 is PASS / CLOSED.

## 16. IA-1 audit challenge set

The independent auditor must be free to reject, constrain, or require
correction. At minimum IA-1 should challenge:

1. Product-definition consistency and fidelity to the browse/select/Transfer/
   Derived Variant/Preview transaction.
2. Separation of historical evidence, adopted scope revision, current
   contracts, and synthesis interpretation.
3. Scope discipline: exclusions/deferred work have not silently returned.
4. Authority boundaries for source facts, Product truth, worker state,
   Derived Variant, QC, Preview, and exports.
5. Thin-domain sufficiency without reconstructing a DCC clone.
6. Blender coupling leakage into durable identity, Mapping, Policy, Job,
   Result, QC, or provenance.
7. Preview authority leakage, stale binding, payload trust, and deletion
   invariants.
8. Evidence scope: one real humanoid cross-Skeleton pair, one browser, and one
   Preview research set.
9. Contract generality versus actual non-humanoid evidence.
10. Unsupported claim inflation or inference presented as observation.
11. Whether Blender and Preview guards are complete, enforceable, and not
    silently dropped between documents.
12. Whether deferred/open work is incorrectly treated as solved.
13. Consistency of V1-1 through V1-8 planning with implementation gates.
14. Packaging, redistribution, dependency, and legal risks.

IA-1 is not performed in this document and no outcome is predicted.

## 17. W0-RS findings and consistency result

```text
material canonical contradictions: 0
unsupported universal claims in this synthesis: 0
W0-RS review findings: 3 / 3 RESOLVED
retained evidence limitation:
  exact E2E ↔ Preview persistence representation byte correspondence
  NOT ESTABLISHED
```

No accepted Product or Architecture contract required modification during
this synthesis. External focused review resolved the interpretation of the
three discrepancies inside immutable accepted evidence. The historical
reports remain unchanged.

### W0-RS-FINDING-001 — Derived persistence hash lineage

**Resolution:** `RESOLVED / NON_DECISION_CRITICAL_PROVENANCE_LIMITATION`

`POC_BLENDER_E2E_01.md` §15 records:

```text
derived_result.blend
SHA-256 882e2c0a9e3fec121abdabb3807eec4bc12bf9283776532f70f12fb4bc8762eb
```

`POC_PREVIEW_01R.md` §4 says it reused the accepted unchanged E2E lineage but
records:

```text
persistence blend SHA-256
fef863e208b51e2fcf70aa278ff9041e39b93c1e9f479940a8db6670d970a532
```

The Mapping, Retarget Policy, and Job Spec hashes align with the accepted
lineage. Exact persistence-representation byte continuity across the E2E and
Preview historical records is **NOT ESTABLISHED FROM REPOSITORY-ONLY
EVIDENCE**. Neither hash is silently chosen as universally correct.

This retained limitation is not architecture-reversing:

```text
DerivedVariant identity
!=
PersistenceArtifact bytes
```

The Product / architecture decision impact is **NONE**. Future production
provenance must version artifact representations and their byte identities
separately from Derived Variant Product identity. Exact versioning rules
remain open.

### W0-RS-FINDING-002 — E2E semantic-run hash labels

**Resolution:** `CLOSED / DOCUMENT_VERSIONING`

The E2E report contains three differently scoped values:

```text
Rev2 closeout 3/3:
f2d650ec8a79f47a30bf485af58321e17258daac2756a30b4f12c83dd3aeaf82

Rev1 closeout 3/3:
1c9d191f6a11f129323c3443e9a734d667f36e2b489e1e74b6fa9bc934136c91

§17 body 3/3:
76d68b4598fc58a1c686b0c983f1bbdc36cfcf72f5c81967871442b3b2e521e0
```

Interpretation:

- `f2d650ec...` is the current final Rev2 semantic-run hash.
- `1c9d191f...` is the historical Rev1 semantic-run hash.
- `76d68b45...` is a legacy unqualified historical body value and is not
  current Rev2 authority.

Revision-scoped semantic consistency remains accepted evidence. Equality
across revisions is not claimed.

### W0-RS-FINDING-003 — Inspect provenance wording

**Resolution:** `CLOSED / STALE_HISTORICAL_WORDING`

The E2E Rev1 closeout says inspect JSON is regenerated by the
repository-controlled worker path and that there is no hidden external
inspect script. The later §22 external-script statement is stale historical
wording. Final accepted provenance is:

```text
run_e2e.py
        ↓
repository blender_worker.py
        ↓
inspect mode / inspect(job)
        ↓
raw inspect JSON
```

The final repository worker owns inspect, retarget, bake, and artifact
generation. Hidden external inspect prerequisite: **NO**. The historical
report is preserved unchanged.

Remaining architectural tensions are explicit open risks rather than hidden
canonical contradictions:

- backend-neutral contracts versus one currently tested Blender backend;
- non-humanoid-capable contracts versus humanoid-only transfer/Preview E2E;
- derived Preview architecture versus open production viewer/payload/schema;
- technically credible ufbx evidence versus no mandatory native importer;
- accepted execution direction versus open packaging/legal qualification.

## 18. IA-1 source index

Start with this synthesis and
[W0_RS_TRACEABILITY.md](W0_RS_TRACEABILITY.md), then inspect only the
evidence needed for a challenge:

### Current truth

- [README](../../../README.md)
- [Agent entry](../../../AGENTS.md)
- [Product Vision](../../product/PRODUCT_VISION.md)
- [V1 Scope](../../product/V1_SCOPE.md)
- [Architecture](../../architecture/README.md)
- [Blender-backed architecture](../../architecture/V1_BLENDER_BACKED_ARCHITECTURE.md)
- [Workflow Domain](../../architecture/V1_WORKFLOW_DOMAIN.md)
- [ADR-0001](../../architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md)
- [Roadmap](../../development/ROADMAP.md)

### Accepted research

- [W0.1 Canonical Foundations](../foundations/W0_1_CANONICAL_FOUNDATIONS.md)
- [W0.2 Mapping / Compatibility / Retarget](../retargeting/W0_2_MAPPING_COMPAT_RETARGET.md)
- [W0.3 Adapter / Infrastructure](../infrastructure/W0_3_ADAPTER_INFRASTRUCTURE.md)
- [W0.4 Product / Technology Decisions](W0_4_PRODUCT_TECH_DECISIONS.md)
- [POC-CORE-01](../poc/POC_CORE_01.md)
- [POC-FBX-01](../poc/POC_FBX_01.md)
- [POC-BLENDER-E2E-01](../poc/POC_BLENDER_E2E_01.md)
- [POC-PREVIEW-01R](../poc/POC_PREVIEW_01R.md)

### Scope revision provenance

- [W0 scope revision proposal](W0_SCOPE_REVISION_PROPOSAL.md)
- [V1 Product scope revision proposal](../../product/V1_PRODUCT_SCOPE_REVISION_PROPOSAL.md)
- [Blender architecture proposal](../../architecture/V1_BLENDER_BACKED_ARCHITECTURE_PROPOSAL.md)
- [Workflow Domain proposal](../../architecture/V1_WORKFLOW_DOMAIN_PROPOSAL.md)
- [Roadmap scope revision proposal](../../development/ROADMAP_SCOPE_REVISION_PROPOSAL.md)
- [W0-P replan proposal](../poc/W0P_REPLAN_PROPOSAL.md)

No chat history is required to audit this baseline.
