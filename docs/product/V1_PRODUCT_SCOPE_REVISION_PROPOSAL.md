# V1 Product Scope Revision Proposal

**Stage:** W0-SR — W0 Scope Revision  
**Status:** `REVIEW_PENDING`  
**Authority:** proposal only; the accepted historical documents remain unchanged  
**Decision basis date:** 2026-08-31

## 1. Revised product definition

RigForge is proposed as a **Character Animation Asset Workbench**. It gives
users an asset-browser-like interface for organizing and previewing rigged
characters, motions, and generated variants. A user selects a rigged Character
Asset and a Motion Asset. RigForge runs Mapping and Compatibility preflight
automatically. On the happy path the user clicks Transfer immediately and
receives a previewable, versioned Derived Variant with provenance and QC.
Export is an optional derived capability, not a mandatory happy-path step.

A Character Asset is already rigged:

```text
Character Asset
= mesh
+ skeleton / armature
+ skin weights
```

It is analogous at the product level to an Unreal Engine Skeletal Mesh asset,
but it is not an Unreal `USkeletalMesh`, does not adopt Unreal identity or
serialization, and does not require Unreal. A raw unrigged mesh is not a V1
transfer target.

A Motion Asset is not an arbitrary bag of tracks:

```text
Motion Asset
= animation clip
+ source Skeleton reference or sufficient source Skeleton context
```

Multiple motions may share one source Skeleton version. RigForge therefore
does not need to duplicate the same source Skeleton for every clip.

This product definition is `DECIDE_NOW` based on
`CLARIFIED_REQUIREMENT`. It supersedes the earlier V1 framing as a generic
Character/Skeleton/Motion infrastructure platform, while preserving the
research that makes this narrower workflow reliable.

## 2. Core user flow

```text
Asset Browser
    |
    +-- select Character Asset A (already rigged)
    |
    +-- select Motion Asset B (with source Skeleton context)
    v
Transfer Tray / Selection Panel
    v
automatic Mapping + Compatibility preflight
    |
    +-- Ready ----------------------> [ Transfer ]
    |
    `-- exception
        +-- warning
        +-- ambiguity
        +-- missing correspondence
        `-- unsupported policy
                |
                v
        Compatibility explanation
        and/or Mapping confirmation
                |
                `-------------------> [ Transfer ] when resolved
    v
Job progress
    v
Derived Variant C
    +-- Validation / QC report
    +-- Preview Artifact
    `-- optional Export Artifact
    v
Save version / compare / preview / use
```

The state labels are illustrative, not frozen enums. Mapping and Compatibility
remain first-class internally. They use **progressive disclosure**: no
mandatory Mapping editor appears on the Ready path. Advanced users may inspect
or edit Mapping voluntarily.

## 3. Primary V1 objects visible to users

### Character Asset

- A logical asset with immutable or append-only versions.
- Each version resolves to source representations containing mesh, skeleton,
  and skin weights.
- It may have no desired animation.
- It can own many Derived Variants without duplicating full geometry for every
  transferred motion.

### Motion Asset

- A logical animation asset with versions.
- Each version identifies the clip and references its source Skeleton context.
- Several motions can reference one source Skeleton version.
- It is diagnosable when its Skeleton reference is missing or unresolved, but
  generic skeletal playback and retarget cannot silently invent that context.

### Derived Variant

- A first-class result, not merely “another FBX file.”
- A version records the exact Character version, Motion version, source
  Skeleton, Mapping version, Retarget Policy version, execution backend and
  version, QC result, generation context, and produced artifacts.
- The logical variant may receive new versions after mapping, policy, backend,
  or source inputs change.
- A full exported character file may be one representation, but it is not the
  only internal identity of the result.

### Preview Artifact

- A rebuildable derivative for click-to-preview.
- It references the asset or Derived Variant version and generator version.
- It may later be GLB plus RigForge inspection metadata, but neither GLB nor a
  viewer technology is selected here.
- It cannot become authority for Character, Motion, Mapping, or QC.

### Export Artifact

- A supported optional derivative when persistence or a concrete downstream
  consumer requires one.
- It records generator/backend version, diagnostics, losses, and the Derived
  Variant version it represents.
- Multiple versioned Export Profiles are deferred / implementation-gated until
  a real downstream requirement justifies them.
- No engine-specific Export Profile is required by the V1 happy path.

## 4. Version relationships

```text
Knight (logical Character Asset)
|-- Character v1 ---- source representation(s)
|-- Character v2 ---- source representation(s)
`-- Derived Variants
    |-- Walk transfer
    |   |-- v1 -> target v1 + motion Walk v3 + mapping v2 + policy v1
    |   `-- v2 -> target v2 + motion Walk v3 + mapping v2 + policy v2
    |-- Run transfer
    |   `-- v1
    `-- Attack transfer
        `-- v1

Mixamo-like Skeleton S1 (source Skeleton context)
|-- Walk v3
|-- Run v1
|-- Jump v4
`-- Attack v2
```

The example does not make Mixamo a dependency or authority. It demonstrates
shared source Skeleton context.

## 5. V1 in scope

| Capability | V1 intent | Basis | Class |
| --- | --- | --- | --- |
| Asset Browser for Character, Motion, and Derived Variant | Core product surface | `CLARIFIED_REQUIREMENT` | `DECIDE_NOW` |
| Click-to-preview source and derived assets | Core product surface | `CLARIFIED_REQUIREMENT` | `DECIDE_NOW` |
| Rigged Character input | Required input contract | `CLARIFIED_REQUIREMENT` | `DECIDE_NOW` |
| Motion plus source Skeleton context | Required retarget input | `CLARIFIED_REQUIREMENT` + `EXISTING_RESEARCH` | `DECIDE_NOW` |
| Rich Skeleton Mapping | Automatic preflight plus reusable, inspectable, confirmable Mapping; editor shown only when useful | `CLARIFIED_REQUIREMENT` + W0.2 | `DECIDE_NOW` |
| Layered compatibility UX | Automatic gate; explanation progressively disclosed on warnings/ambiguity/unsupported states | `CLARIFIED_REQUIREMENT` + W0.2 | `DECIDE_NOW` |
| Version/provenance model | Makes generated results reproducible and manageable | `CLARIFIED_REQUIREMENT` + W0.3 | `DECIDE_NOW` |
| Retarget Policy | Product-owned intent separate from execution | W0.2 + `ARCHITECTURAL_INFERENCE` | `DECIDE_NOW` |
| Hidden execution worker | Transfer/bake implementation boundary | `ARCHITECTURAL_INFERENCE` | `DECIDE_NOW` for boundary |
| Blender as first worker | V1 implementation hypothesis | `NEEDS_POC` | `PROVISIONAL_POC_GATED` |
| QC and Validation Report | Exit code is insufficient | `CLARIFIED_REQUIREMENT` + W0.2 | `DECIDE_NOW` |
| Derived Preview Artifact | Engine-independent preview path | `CLARIFIED_REQUIREMENT` + W0.3 | `DECIDE_NOW` for role |
| Optional Export Artifact concept | Allows later persistence/downstream representation without defining the happy path | `ARCHITECTURAL_INFERENCE` | `DECIDE_NOW` for concept |
| Multiple versioned Export Profiles | Requires a concrete downstream consumer | `NEEDS_POC` / implementation evidence | `DEFER` |
| At least one real cross-skeleton E2E path | Validates actual product promise | W0.2 + `NEEDS_POC` | `PROVISIONAL_POC_GATED` |

## 6. V1 out of scope

| Excluded capability | Reason | Basis | Class |
| --- | --- | --- | --- |
| Auto-Rig / automatic Skeleton generation | Target Character is already rigged | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| Automatic skinning | Skin weights already exist | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| Raw mesh topology analysis for rig generation | Not part of transfer workflow | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| Full DCC editing environment | Blender is hidden execution, not user workspace | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| RigForge-owned full animation evaluation runtime | Blender can evaluate and bake in V1 | `ARCHITECTURAL_INFERENCE` + `NEEDS_POC` | `DROP_FROM_V1` if E2E PoC passes |
| RigForge-owned custom retarget runtime | Own policy, not necessarily execution | `ARCHITECTURAL_INFERENCE` + `NEEDS_POC` | `DROP_FROM_V1` if E2E PoC passes |
| Mandatory native FBX/glTF parser stack | Not needed unless latency, metadata, or interchange evidence requires it | `POC_EVIDENCE` + `ARCHITECTURAL_INFERENCE` | `DEFER` |
| Direct Unreal/Unity/Godot integrations | No engine integration is required for the accepted Character→Derived Variant workflow | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| Multiple execution backends | Boundaries provide generality without multiple implementations | `CLARIFIED_REQUIREMENT` | `DROP_FROM_V1` |
| MCP | Underlying job API can be reused later | `CLARIFIED_REQUIREMENT` + W0.4 | `DEFER` |
| Mandatory AI mapping/retarget | Deterministic evidence and confirmation remain available | W0.4 | `DEFER` |

## 7. Product-facing compatibility

Compatibility is explanatory product state, not one boolean:

```text
Mapping completeness
    != structural / semantic compatibility
    != method / policy eligibility
    != Motion suitability
    != result acceptability
```

It may consider required/optional mapping coverage, hierarchy, rest/base pose,
proportions, root/trajectory policy, helper/twist differences, source Motion
requirements, and worker capabilities. It must fail closed on missing required
correspondence and may require user confirmation for ambiguity. It runs
automatically. A Ready result exposes Transfer directly; warnings,
ambiguities, missing correspondences, or unsupported policy disclose the
explanation and/or Mapping confirmation path. Exact states and thresholds
remain open.

## 8. Requirements-drift check

| Question | Result |
| --- | --- |
| Can users browse Character/Motion assets? | Yes — core Asset Browser |
| Can users click and preview? | Yes — derived, rebuildable Preview Artifact |
| Can users select one Character and one Motion? | Yes — Transfer Tray |
| Can the system assess mapping/compatibility? | Yes — first-class product capabilities |
| Can one button start transfer? | Yes — Ready exposes Transfer without a mandatory Mapping editor |
| Does the result become a Derived Variant? | Yes — first-class, versioned |
| Can variants be previewed and versioned? | Yes — mandatory happy path |
| Can optional export be added without engine coupling? | Yes — supported artifact concept; profiles deferred |
| Does V1 require Auto-Rig? | No |
| Is the product tied to a game engine? | No |
| Can Blender be replaced without changing product semantics? | Yes, if the proposed Job/Worker boundary passes PoC |

## 9. Open product questions

Only questions requiring evidence remain open:

1. Which evidence lets automatic preflight reach Ready, and when must the
   exception path request Mapping confirmation?
2. Which QC signals are useful enough for V1, and which are warnings rather
   than blockers?
3. What preview payload gives acceptable load time and inspection coverage?
4. Does a concrete downstream consumer require an Export Artifact or versioned
   Export Profile in V1?
5. Can the asset catalog remain local-first in V1, or does validated workflow
   require shared/team storage?

Database choice, UI toolkit, exact enum names, and screen layout are later
design decisions, not scope questions.
