# W0-P Replan Proposal

**Stage:** W0-SR Rev1  
**Status:** `REVIEW_PENDING`  
**Execution in this task:** none

## 1. Replan summary

The clarified product workflow makes one uncertainty highest-information:

> Can Blender serve as a hidden execution backend for the real RigForge
> Character + Motion → Derived Variant workflow without forcing Blender
> semantics into RigForge's durable product model?

POC-BLENDER-E2E-01 is a **minimal vertical architecture slice**, not release
qualification. It freezes one reviewed Bone Mapping and one Retarget Policy
for one meaningful real cross-Skeleton pair. Mapping quality, corpus breadth,
advanced QC, worker hardening, packaging completion, and export breadth move
to later V1 stages.

No original pending PoC should run until W0-SR Rev1 is externally reviewed.

## 2. Original PoC disposition

| Original PoC | Disposition | Revised placement |
| --- | --- | --- |
| POC-GLTF-01 | `MERGE` | Candidate Preview Artifact/re-open path only; native parser selection deferred |
| POC-RETARGET-01 | `MERGE` | Minimal execution path moves into POC-BLENDER-E2E-01; retarget breadth and quality move to V1 validation |
| POC-PREVIEW-01 | `REDEFINE` | POC-PREVIEW-01R tests actual click-to-preview experience after the architecture slice |
| POC-ENGINE-01 | `DOWNGRADE` | Direct engine work remains outside V1; no engine-specific export is required |
| POC-USD-01 | `DEFER` | Post-V1 unless a concrete workflow requires it |
| POC-FBX-02 | `DEFER` | Native FBX write only if a concrete later output requirement needs it |
| POC-DCC-01 | `MERGE` | Only first-slice Blender process observations move into E2E; complete operational qualification moves to V1 worker integration/hardening |
| POC-PREVIEW-02 | `DEFER` | Only if POC-PREVIEW-01R cannot establish a viable viewer path |

## 3. POC-BLENDER-E2E-01

### Primary question

> Can Blender serve as a hidden execution backend for the real RigForge
> Character + Motion → Derived Variant workflow without forcing Blender
> semantics into RigForge's durable product model?

The PoC may confirm, constrain, or reject the Blender-backed hypothesis. It may
show that the thin Workflow Domain must grow, but it must not become a broad
system qualification exercise.

### Required real input

```text
one already-rigged Character A
  mesh + target Skeleton + skin weights

one Motion B
  animation + a genuinely different Source Skeleton

expected-compatible enough for meaningful transfer
license-cleared local assets
exact content hashes
exact tool and worker versions
```

No new corpus breadth is required. The first pair may be humanoid if that is
the most efficient valid architecture test. A passing result must not be
reported as non-humanoid E2E support.

### Frozen Mapping

Use one explicit, reviewed, frozen Bone Mapping for the selected pair.

```text
Mapping location: outside Blender
Mapping form: backend-neutral
Mapping quality: held constant
Automatic Mapping: not tested
```

The worker receives a projection of the frozen Mapping, but Blender data-block
names and constraints do not become Mapping authority. Automatic Mapping
remains a V1 product capability and later validation concern.

### Held constants

- Character and Motion versions/hashes;
- source Skeleton binding;
- thin Skeleton Summaries;
- reviewed Bone Mapping;
- Retarget Policy;
- exact Blender build and worker version;
- import, bake, time-domain, and candidate Preview Artifact settings;
- backend-neutral Job Spec/result-envelope version;
- clean-process repeat method.

The PoC does not compare Blender versions, Mapping algorithms, multiple
retarget methods, renderers, format libraries, or Export Profiles.

### Minimal workflow

```text
Register Character A
Register Motion B + Source Skeleton
        ↓
produce/load thin Skeleton Summaries
        ↓
apply frozen Mapping
        ↓
basic Compatibility preflight
        ↓
freeze Retarget Policy
        ↓
backend-neutral Worker Job Spec
        ↓
launch isolated Blender worker
        ↓
import → retarget → bake
        ↓
basic structural validation
        ↓
create Derived Variant record
        ↓
generate one candidate Preview Artifact
        ↓
fresh-process re-open / verify
        ↓
repeat from clean processes
```

The one practical output artifact exists only as needed to persist, preview,
or re-open the result. No engine-specific output or multiple Export Profile
comparison is required.

### Required acceptance criteria

```text
[ ] real cross-Skeleton Character + Motion transfer succeeds
[ ] source/target identity and hashes are recorded
[ ] Mapping is represented outside Blender
[ ] Retarget Policy is represented outside Blender
[ ] durable Job Spec contains no Blender product semantics
[ ] Blender projections/data-blocks remain ephemeral
[ ] animation is baked onto the target Character
[ ] Derived Variant has explicit provenance
[ ] result produces one candidate Preview Artifact
[ ] result reopens in a fresh process
[ ] basic structural QC passes
[ ] repeated clean-process runs are semantically consistent
[ ] worker success and failure are distinguishable
[ ] non-zero/error diagnostics are captured
[ ] partial output is not automatically published
[ ] Core language is not selected by this PoC
```

Semantic consistency is evaluated under the declared context; byte-identical
files are not required unless the chosen output happens to provide them.

### Basic structural QC

Required:

- no NaN or Inf transforms;
- required mapped bones present;
- sane animation duration and time-domain handling;
- expected target animation exists after bake;
- no gross invalid scale or transform;
- basic root-trajectory sanity under the frozen policy.

Preserve:

```text
execution success
  != structural validity
  != QC quality
  != production acceptability
```

The first PoC does **not** establish production foot-slide thresholds, a
contact-quality model, artistic acceptance, a complete discontinuity policy,
or an advanced QC framework. QC must not silently run corrective IK or mutate
the result it measures.

### Minimal failure evidence

It is sufficient to demonstrate:

- successful worker completion is distinguishable from failure;
- one non-zero/error path produces captured diagnostics;
- partial/staged output is not automatically published.

Crash, timeout, malformed-envelope, retry, cancellation, and recovery
campaigns are later worker-integration/hardening work.

### Contract generality

The Job Spec, Mapping, Retarget Policy, result envelope, Derived Variant, and
Preview Artifact contracts must contain no humanoid-only assumption. This is a
contract inspection requirement, not evidence that non-humanoid E2E works.

Real non-humanoid E2E validation remains mandatory before V1 release.

### Packaging and licensing observation

Record known Blender version, deployment, GPL/worker-script constraints, open
questions, and required future legal/distribution review. Complete shipping
package qualification is not an acceptance criterion.

A material legal or distribution blocker discovered during the slice may
still invalidate Blender. Do not infer legal safety from process isolation.

### Architecture decision outcomes

| Observation | Decision impact |
| --- | --- |
| Real transfer, bake, re-open, and repeat succeed behind neutral contracts | Continue Blender-backed V1 hypothesis |
| Durable product state requires Blender types/operators | Redesign or reject worker boundary |
| Thin summaries cannot carry required policy/provenance | Add only demonstrated Workflow Domain facts |
| Blender introduces material silent loss or cannot perform the pair | Constrain or reject Blender backend |
| Material legal/distribution blocker discovered | Stop Blender adoption pending resolution or alternative |
| Candidate Preview Artifact cannot persist/re-open the result | Revise artifact path; do not expand this PoC into viewer qualification |

## 4. Work moved to later stages

These requirements are retained but do not block POC-BLENDER-E2E-01:

| Later stage | Retained work |
| --- | --- |
| V1 Mapping / Compatibility validation | Auto-Mapping quality, ambiguity UX, missing Source Skeleton UX, incompatible-pair coverage, user Mapping edits |
| V1 worker integration tests | Crash, timeout, malformed envelope, retry, cancellation, recovery, staged-output cleanup |
| V1 worker reliability/hardening | Concurrent scheduling, worker pool, resource scheduler, resource limits, process reuse decisions |
| V1 Transfer / QC validation | Identity oracle suite, advanced discontinuity policy, foot/contact metrics, artistic review model |
| V1 artifact/output implementation | Concrete downstream outputs; any required Export Artifact/Profile |
| V1 real-asset hardening | Corpus breadth, exporter diversity, full non-humanoid E2E, unsupported cases |
| Release qualification | Blender upgrade rehearsal, complete packaging/distribution/legal review, installation/update/uninstall behavior |

## 5. POC-PREVIEW-01R

Question:

> Can users click Character, Motion, and Derived Variant entries and obtain a
> useful engine-independent preview from rebuildable artifacts without a full
> RigForge animation runtime?

This remains the second focused pre-implementation PoC. It tests the actual
click-to-preview experience, artifact coverage, binding diagnostics, and
derived/non-authoritative boundary. Renderer technology and GLB remain open.

POC-BLENDER-E2E-01 only proves that one candidate Preview Artifact can be
generated and re-opened; it does not qualify the complete viewer experience.

## 6. Export position

Export Artifact remains a supported Workflow Domain concept and optional
derivative. Multiple versioned Export Profiles are `DEFER` /
implementation-gated until a concrete downstream consumer requires them.

The first E2E has no engine-specific export requirement and no Export Profile
comparison.

## 7. Decision order

```text
W0-SR Rev1 external review
        ↓
POC-BLENDER-E2E-01
  minimal architecture validation
        ↓
POC-PREVIEW-01R
  click-to-preview validation
        ↓
revised W0 synthesis
        ↓
IA-1
        ↓
V1 implementation and hardening
```

Do not execute either PoC during W0-SR Rev1.
