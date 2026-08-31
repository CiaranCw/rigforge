# Architecture

Current V1 architecture principles. Implementation language, GUI stack,
serialization format, package layout, viewer library, and database are not
frozen.

Decision: [ADR-0001](decisions/ADR-0001-v1-scope-and-blender-backed-execution.md).
Blender worker: [V1_BLENDER_BACKED_ARCHITECTURE.md](V1_BLENDER_BACKED_ARCHITECTURE.md).
Workflow Domain: [V1_WORKFLOW_DOMAIN.md](V1_WORKFLOW_DOMAIN.md).
Product scope: [../product/V1_SCOPE.md](../product/V1_SCOPE.md).

The earlier **Canonical-first / Adapters around Canonical Core** framing is
**superseded as the current V1 direction**. Historical Canonical research
remains accepted evidence: [../research/R1_RESEARCH_BASELINE.md](../research/R1_RESEARCH_BASELINE.md).

## Stable principles

- **Product semantics independent of execution backend.** Asset identity,
  Mapping, Compatibility, Retarget Policy, Job, Derived Variant, Validation,
  Preview orchestration, and versioning do not use Blender data-block names,
  types, or Python API objects as durable contracts.
- **Thin Workflow Domain.** RigForge owns workflow meaning and evidence. It
  does not need complete copies of mesh arrays, skin weights, FBX authored
  semantics, animation curves, a full evaluator, a custom IK solver, or a
  custom retarget runtime unless later evidence proves a specific fact must
  be product-owned.
- **Authoritative vs Derived separation.** Source files remain source
  authority. Preview and Export artifacts are derived, rebuildable, and
  non-authoritative. A worker result cannot become product truth by itself.
- **Mapping != Compatibility != Result Quality.** Mapping completeness is
  not retarget compatibility and not production acceptability.
- **Worker execution does not authorize product truth.** Worker success is
  not structural validity, not QC quality, and not publication.
- **Blender is provisional / replaceable.** V1 may use a pinned, isolated
  Blender worker (`PROVISIONAL_POC_GATED`). Blender is not product authority.
  A future worker must implement the Job Spec without pretending to be Blender.
- **Engine independence.** Unreal and Unity are not required for Preview,
  Transfer, or V1 identity. Direct engine integrations are out of V1.
- **Preview is derived / non-authoritative.** Engine-independent click-to-preview
  is mandatory. The viewer is not the Blender UI.
- **Deterministic-first where practical.** Determinism claims require a
  declared context. Semantic consistency under that context is the V1 bar;
  byte-identical outputs are not assumed.
- **AI cannot silently become authority.** Suggestions may exist later.
  Deterministic structure and user confirmation remain available.
- **Names are evidence, not identity.** Humanoid profiles are optional
  Mapping profiles, not the Skeleton model.
- **Silent semantic loss is forbidden.** Capability and expected loss are
  declared before work starts.
- **QC must not silently mutate its subject.**

## Execution shape

```text
RigForge Workbench
        ↓
Application / Workflow Domain
        ↓
backend-neutral Job Spec
        ↓
Worker Boundary
        ↓
Blender Worker   (PROVISIONAL_POC_GATED)
```

This placement is not finally proven. POC-BLENDER-E2E-01 must confirm that
the worker boundary holds on a real cross-Skeleton Character + Motion pair.

## Decisions

Record changes of mind as ADRs: [decisions/README.md](decisions/README.md).

Do not treat research candidates as selected shipping dependencies. POC-FBX-01
(`KEEP_UFBX_WITH_GUARDS`) remains valid evidence; a native ufbx inspector is
not a V1 mandate.
