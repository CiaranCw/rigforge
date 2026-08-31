# Adapter Capability and Loss Model — W0.3

**This is not W1 / W2. This is not an API.**

Classification: `MUST` / `SHOULD` / `OPEN`.

Evidence tags follow [docs/research/README.md](../README.md). Access date: **2026-08-30**.

Names (`READ_SKELETON`, `SUPPORTED_WITH_LOSS`, …) are **provisional**.

---

## Authority rule

```text
External representation
        ↓
Adapter
        ↓
Canonical Domain
```

must **not** imply `External == Canonical`.

```text
Canonical
        ↓
Exporter
        ↓
Derived external representation
```

must **not** reverse-authorize Canonical unless a **new import** occurs.

---

## AD-CAP

### AD-CAP-001 — MUST

An Adapter is responsible for **translation and negotiation** between an external representation and the RigForge Domain. It must not become Canonical authority.

Evidence: architecture [docs/architecture/README.md](../../architecture/README.md); W0.1 CR-ASSET-001. [RIGFORGE_INFERENCE]

Reason: Formats, DCCs, engines, and renderers remain interchangeable.

### AD-CAP-002 — MUST

The Domain must never delegate to an Adapter: Canonical identity, **Domain Semantic Mapping** identity, compatibility judgment, retarget policy, or QC thresholds.

Evidence: W0.1 / W0.2 accepted baselines. [PROJECT_CLAIM] product vision.

See AD-CAP-007: an Adapter may **translate** a Domain Semantic Mapping into a host representation. It must not **author** semantic roles as Canonical truth.

### AD-CAP-003 — MUST

A caller must be able to discover, **before mutation**, what an adapter can do in a stated context: operations, source/target versions, required environment, and expected loss class.

Evidence: OpenAssetIO `managementPolicy` / trait capability queries [OFFICIAL_DOC] https://docs.openassetio.org/OpenAssetIO/. glTF `extensionsUsed` / `extensionsRequired` is a format-level analogue [SPEC].

Open implications: API name (`DescribeCapabilities`) OPEN for W2.

### AD-CAP-004 — MUST

Capability must be allowed to vary by adapter version, format subset, configuration, and platform. A single static boolean per adapter is insufficient.

Evidence: ufbx load options change evaluated vs recipe exposure (W0.1). glTF Transform requires **registered** extensions before I/O [OFFICIAL_DOC] https://gltf-transform.dev/extensions. OpenUSD plugin/schema load changes available types [OFFICIAL_DOC].

### AD-CAP-005 — SHOULD

Capability families that a future contract should be able to express (names not frozen): read/write Character, Skeleton, Motion; read/write skin; read/write blendshapes; preserve curves; preserve layering; preserve bind semantics; preserve metadata/extensions; headless; batch; streaming; round-trip.

Evidence: production adapters differ in these axes (ufbx bake vs recipe; cgltf extras vs unknown extensions [OFFICIAL_DOC] cgltf README).

### AD-CAP-006 — OPEN

Exact capability token set and whether capabilities are additive flags vs profiles is OPEN for W2.

### AD-CAP-007 — MUST

```text
DOMAIN SEMANTIC MAPPING
        ≠
HOST BINDING / HOST PROJECTION
```

Domain owns semantic roles, joint/chain correspondence, mapping identity, mapping evidence, and compatibility inputs.

An Engine or DCC Adapter may own only **host-specific** facts such as: Canonical joint → host bone identifier; Canonical Motion → host clip/asset identifier; a Domain Semantic Mapping **translated** into a host retarget/mapping representation; host object/asset references; host import/export projection.

Correct flow:

```text
Domain SemanticMapping
        ↓
Adapter
        ↓
host representation (Unreal / Unity / Godot / custom / DCC)
```

Forbidden: the Adapter independently deciding Canonical semantic roles and becoming their source of truth.

If a host provides useful semantic metadata on import, the Adapter may emit **source evidence / provenance**. Domain mapping process still owns the Mapping decision.

Names `Host Binding` / `Host Projection` are **not** a frozen W2 API.

Evidence: AD-CAP-002; W0.2 mapping ≠ compatibility ≠ QC. [RIGFORGE_INFERENCE] Engine-agnostic gate in [DCC_ENGINE_BOUNDARY.md](DCC_ENGINE_BOUNDARY.md).

---

## AD-LOSS

### AD-LOSS-001 — MUST

Silent semantic loss is **not acceptable**. Every conversion that drops, bakes, or approximates a represented concept must emit a diagnosable loss record.

Evidence: W0.1 — FBX pivot recipe vs evaluated TRS; `ufbx_bake_anim` resamples cubic/Euler [OFFICIAL_DOC]. glTF unknown extensions can be present but unread [SPEC] / [OFFICIAL_DOC] cgltf.

### AD-LOSS-002 — MUST

A loss record must identify at least: **what concept**, **where in the asset**, **why**, **whether expected under the declared policy**, **whether conversion may continue**, and **whether human approval is required**.

Evidence: tinygltf v3 structured `tg3_error_stack` with severity and source location [OFFICIAL_DOC] README. ufbx `ufbx_error` [OFFICIAL_DOC].

### AD-LOSS-003 — MUST

The loss model must distinguish at least these **families** (names provisional): supported; supported with loss; unsupported; ambiguous.

Evidence: glTF `extensionsRequired` vs `extensionsUsed` [SPEC] Khronos extensions README. W0.1 FBX layer additive has no portable clip-level analogue.

### AD-LOSS-004 — SHOULD

Expected-loss examples a future model should classify: FBX layered animation → sampled TRS; FBX pivot recipe → evaluated transforms; glTF unsupported/unknown extension; USD relationship not in V1; Canonical helper metadata with no target field; Canonical Motion → runtime-compressed clip.

### AD-LOSS-005 — OPEN

Exact loss codes and whether opaque payloads are stored vs hashed vs dropped is OPEN. Policy choice among preserve-opaque / declare-loss / refuse-round-trip / strip-under-policy is **not** frozen (see format track).

---

## AD-DIAG

### AD-DIAG-001 — MUST

Adapters must emit diagnostics into a **common envelope**, not per-library string dumps as the only record.

Candidate fields (not a schema): severity, code, source, adapter identity/version, asset location, semantic location, message, evidence, loss, recoverability, suggested action.

Evidence: tinygltf v3 error stack [OFFICIAL_DOC]. OpenAssetIO wraps host/manager for audit [OFFICIAL_DOC].

### AD-DIAG-002 — MUST

Parser success, semantic import success, and Canonical validity are **three different outcomes**.

Evidence: cgltf `cgltf_parse` does not load buffers by default [OFFICIAL_DOC]. A syntactically valid glTF can still fail skin/IBM or extension-required load. USD can open a layer and fail composition/resolve [OFFICIAL_DOC].

### AD-DIAG-003 — SHOULD

Severity families: warning, error, unsupported, partial, loss, external-tool failure, version mismatch.

### AD-DIAG-004 — OPEN

Exact diagnostic schema and localization are OPEN for W2.

---

## AD-VERSION

### AD-VERSION-001 — MUST

Every conversion must record adapter identity, adapter version, supported source/target versions used, capability set / feature flags in force, and required external environment (if any).

Evidence: W0.1 CR-PROVENANCE / CR-ASSET-002. ufbx 0.Y.Z may break source compatibility [OFFICIAL_DOC] README. OpenUSD version flavors (25.05 / 25.08) are not interchangeable [OFFICIAL_DOC] OpenUSD Exchange changelog.

### AD-VERSION-002 — SHOULD

A preflight call should answer: can this adapter perform this operation; what will be lost; what external environment is required.

### AD-VERSION-003 — OPEN

Whether adapters are versioned as plugins, processes, or libraries is a W0.4 / W2 packaging question.

---

## AD-DETERMINISM

### AD-DET-001 — MUST

A determinism claim requires a **declared determinism context**.

For a fully declared execution context, the same logical input, adapter and dependency versions, configuration, resolution/composition context, required host/environment state, Domain contract version, and policy must produce **semantically equivalent** results within a documented tolerance.

Field names are **not** frozen. The essential rule is:

```text
determinism claim
requires
declared determinism context
```

The context must be able to include, when they affect output: USD resolver context; USD composition context; DCC scene/session state; external dependency versions; filesystem/environment; host settings.

Bit-identical bytes are **not** required unless a later policy proves them.

Resolution context that affects output must be explicit **provenance / input**. OpenAssetIO `Context` / Manager State is **evidence** that reproducible resolution may need explicit contextual state [OFFICIAL_DOC] https://docs.openassetio.org/OpenAssetIO/classopenassetio_1_1v1_1_1_context.html — do **not** copy that type into RigForge Core.

Evidence: ufbx CI claims bit-exact results across listed platforms [OFFICIAL_DOC] README. USD composition and asset resolvers can change composed results [OFFICIAL_DOC]. ACL/ozz cooks are lossy by design [OFFICIAL_DOC].

### AD-DET-002 — MUST

Adapters must distinguish semantic determinism, structural determinism, and byte determinism in reports.

### AD-DET-003 — SHOULD

Known context and nondeterminism sources to record when relevant: external DCC session state, dependency version, floating point, threading, filesystem enumeration order, USD composition/resolver context, export object order, runtime compression, asset-manager locale/state tokens.

### AD-DET-004 — OPEN

Numeric tolerances remain OPEN (W0-P / W1). Do not promise GPU/DCC bit-identity.

Provisional future reporting tokens (`CONTROLLED`, `CONTEXT_DEPENDENT`, `UNCONTROLLED`, `UNKNOWN`) are **not** frozen.

### AD-DET-005 — MUST

If an Adapter depends on external state that cannot be captured, fixed, or reconstructed, it **must not** claim full deterministic reproducibility. It must report the limitation.

Examples of such state: interactive DCC session; mutable asset-resolver target; latest/version aliases; environment-dependent plugin discovery; external engine project configuration.

---

## AD-FAILURE

### AD-FAIL-001 — MUST

Destructive or publish mutations must follow fail-closed order: inspect → capability preflight → loss preflight → authorize policy → convert → validate → publish derived output. Adapters must not discover halfway that the target cannot represent required semantics after an irrecoverable write.

Evidence: export `is_automated` patterns exist in engines (Unreal Interchange `ImportAssetParameters.is_automated` [OFFICIAL_DOC]) but do not replace Domain preflight. [RIGFORGE_INFERENCE]

### AD-FAIL-002 — MUST

An exporter must be able to **refuse**, **warn and continue under explicit policy**, or **require policy/approval**, depending on loss class. Exact state names are not frozen.

### AD-FAIL-003 — SHOULD

Temporary output and atomic publication should be used where the target filesystem/manager supports it.

### AD-FAIL-004 — OPEN

Exact error codes and transactional storage semantics are OPEN.

---

## AD-PROVENANCE

### AD-PROV-001 — MUST

Import must retain **source facts**, **conversion facts**, **loss facts**, and **adapter facts** without rewriting Canonical semantics as if they were authored in-format.

Evidence: W0.1 derived ≠ authoritative. Architecture: Canonical != Runtime.

### AD-PROV-002 — MUST

Export output is **derived**. Re-import of that output is a new ingest, not an implicit identity with the previous Canonical object.

### AD-PROV-003 — SHOULD

Publish manifests should list adapter version, cook/policy ids, and loss summaries.

### AD-PROV-004 — OPEN

Manifest field set is OPEN for W1.
