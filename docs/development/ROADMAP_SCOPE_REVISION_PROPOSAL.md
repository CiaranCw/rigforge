# Roadmap Scope Revision Proposal

**Stage:** W0-SR  
**Status:** `REVIEW_PENDING`  
**Replaces current roadmap:** NO — proposed replacement for external review

## 1. Replanning rule

The clarified product workflow changes the critical path. RigForge should not
continue the old PoC sequence mechanically or begin W1's heavy Canonical
contract. The next evidence must test the actual product path: select a rigged
Character and a cross-skeleton Motion, map, transfer through a hidden Blender
worker, QC, preview, version, and re-open the result.

Historical W0 work remains accepted evidence. W0-SR changes the proposed V1
destination and sequence, not past observations.

## 2. Proposed stage map

| Stage | Purpose | Status / gate |
| --- | --- | --- |
| W0.1–W0.4 | Historical semantic, mapping, infrastructure, and technology research | `COMPLETE` — preserve |
| POC-CORE-01 | Equivalent language slice | `COMPLETE / PASS`; impact `INCONCLUSIVE` |
| POC-FBX-01 | ufbx technical/evidence validation | externally accepted `TECHNICAL / EVIDENCE PASS`; impact `KEEP_UFBX_WITH_GUARDS`; accepted files still uncommitted at W0-SR preflight |
| W0-SR | Reconcile clarified workflow with W0 evidence | `REVIEW_PENDING` |
| POC-BLENDER-E2E-01 | Test the proposed real V1 execution path and thin domain boundary | next proposed Mandatory PoC after W0-SR review |
| POC-PREVIEW-01R | Click-to-preview and Derived Variant viewer using generated Preview Artifacts | Mandatory; follows/overlaps E2E artifact output without confounding renderer choice |
| W0-RS | Revised W0 synthesis | Decide product scope, worker placement, thin-domain sufficiency, remaining PoCs, and implementation gates |
| IA-1 | Formal independent architecture/research audit | After accepted W0-SR + required revised PoCs + W0-RS |
| V1-1 | Workflow Domain and version/provenance contract | Start only after IA-1 acceptance/gate |
| V1-2 | Local-first Asset Catalog and job orchestration | Asset Browser foundations; technology chosen at implementation gate |
| V1-3 | Pinned isolated Blender worker integration | Import/evaluate/retarget/bake/output under versioned Job Spec |
| V1-4 | Skeleton Mapping and compatibility workflow | Rich mapping, evidence, confirmation, product-facing states |
| V1-5 | Transfer, QC, and Derived Variant lifecycle | One-button transfer after gate; validation, versioning, artifacts |
| V1-6 | Engine-independent Preview | Source and derived click-to-preview; rebuildable artifact |
| V1-7 | Optional artifact/output support | Add an Export Artifact/Profile only when a concrete downstream consumer requires it |
| V1-8 | Real-asset hardening and release audit | Humanoid + non-humanoid, negative cases, upgrades, recovery, packaging |

Names after W0 are proposals, not frozen lifecycle identifiers.

## 3. Simplified critical path

```text
W0-SR external focused review
        |
        v
POC-BLENDER-E2E-01
  real Character + different-source-skeleton Motion
  frozen Mapping -> basic Compatibility -> Blender -> Bake -> basic QC
  -> Preview Artifact -> re-open verification
        |
        +---- may invalidate Blender backend or thin domain
        v
POC-PREVIEW-01R
  click-to-preview + derived viewer + inspection overlays
        |
        v
Revised W0 synthesis
        |
        v
IA-1 (new independent Agent)
        |
        v
V1 implementation
```

POC-BLENDER-E2E-01 may produce one practical persistence/preview artifact. It
does not qualify Export Profiles and has no engine-specific output
requirement.

### Architecture validation versus product hardening

```text
PRE-IMPLEMENTATION ARCHITECTURE VALIDATION

W0-SR
  -> POC-BLENDER-E2E-01 (one minimal vertical slice)
  -> POC-PREVIEW-01R (actual click-to-preview)
  -> revised W0 synthesis
  -> IA-1

V1 PRODUCT IMPLEMENTATION / HARDENING

Workflow Domain + Asset Browser
  -> Mapping / Compatibility validation
  -> worker reliability
  -> Transfer / QC validation
  -> real-asset + mandatory non-humanoid hardening
  -> release qualification
```

Do not move the implementation/hardening matrix back into the initial
architecture PoC.

## 4. V1 implementation sequencing rationale

1. **Workflow contract before technology breadth.** Define Asset Version,
   Skeleton Summary, Mapping, Policy, Job Spec, Derived Variant, QC, Preview
   Artifact, and Export Artifact boundaries without production schemas for
   every format.
2. **Catalog before polish.** Asset identity/version relationships make the
   browser and generated results coherent.
3. **One backend before abstraction zoo.** Implement only the Blender worker,
   behind a boundary already challenged by the E2E PoC.
4. **Automatic preflight before one-click.** Ready exposes Transfer directly.
   Mapping/Compatibility explanation or confirmation is progressively
   disclosed only for exceptions; advanced users may open it voluntarily.
5. **QC and provenance with execution.** They are not later observability
   decoration.
6. **Preview is core.** Source and derived click-to-preview ships in the V1
   path, but the artifact/viewer technology remains evidence-gated.
7. **Concrete output need before profiles.** Keep an optional Export Artifact
   boundary, but do not implement multiple profiles without a downstream
   consumer. Direct engine plugins remain outside V1.

## 5. Real-asset validation points

| Gate | Required evidence |
| --- | --- |
| POC-BLENDER-E2E-01 | One real compatible cross-Skeleton transfer using a frozen reviewed Mapping; backend-neutral Job Spec; bake; basic structural QC; Derived Variant; candidate Preview Artifact; fresh-process re-open; clean-process consistency |
| POC-PREVIEW-01R | Source Character, source Motion metadata/binding, and Derived Variant preview; skeleton/mapping/QC visibility; large real asset |
| V1-3 worker integration | Crash/timeout/malformed-envelope handling, retry/cancellation/recovery, staged-output cleanup, concurrency/pool/resource decisions |
| V1-4 Mapping/Compatibility | Auto-Mapping quality, ambiguity UX, missing source Skeleton UX, incompatible pairs, voluntary Mapping editing |
| V1-5 transfer/QC | Identity oracles, different bone counts, twist/helpers/end bones, root policy, advanced discontinuity/contact signals, artistic review boundary |
| V1-8 release hardening | Multiple exporter families/formats as supported, mandatory real non-humanoid E2E, unsupported cases, Blender upgrade rehearsal, complete packaging/distribution/legal qualification |

No test may infer genericity from a single humanoid/Mixamo-like family.

## 6. Independent audit placement

IA-1 remains formal and independent. Recommended trigger:

```text
W0-SR accepted
+ POC-BLENDER-E2E-01 complete
+ POC-PREVIEW-01R complete
+ any blocker-driven follow-up complete
+ revised W0 synthesis complete
        -> IA-1
```

IA-1 must use a new independent Agent, not the research/implementation Agent:

```text
GPT-5.6 Sol High / 1M High
or
Claude Opus 5 High
```

IA-1 should assess whether the new scope satisfies the product workflow,
whether Blender remains an adapter, whether the domain is too thin or too
heavy, whether PoC evidence supports the backend, and whether implementation
may begin.

## 7. Decision timing

| Decision | Timing |
| --- | --- |
| Product identity, rigged inputs, Asset Browser, Derived Variant, version/provenance | W0-SR `DECIDE_NOW` |
| Blender backend, thin-domain sufficiency | POC-BLENDER-E2E-01 |
| Preview Artifact and viewer path | E2E artifact lane + POC-PREVIEW-01R |
| Core language | Defer through revised PoCs; decide only when V1-1/worker boundary evidence demands it |
| Database/storage | V1-2 design, after local/team requirements |
| Native parser optimization | Only after measured need |
| Optional Export Artifact/Profile | V1-7 only if a concrete downstream consumer requires it |
| Engine integration | Post-V1 |
| MCP / AI mapping | Post-V1 or later evidence |

The Blender-backed process boundary reduces urgency of the previous
C++-versus-Rust Core decision. POC-CORE-01 remains PASS with
`INCONCLUSIVE` impact; language can be deferred until the thinner Workflow
Domain and worker protocol are accepted.

## 8. Lifecycle proposal

```text
W0: ACTIVE
W0-SR: REVIEW_PENDING
next original PoC: PAUSED pending scope-revision review
W1: NOT STARTED
IA-1: NOT STARTED
```

Do not mark W0-SR PASS and do not execute POC-BLENDER-E2E-01 from this
proposal.
