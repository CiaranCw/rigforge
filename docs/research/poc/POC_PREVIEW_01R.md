# POC-PREVIEW-01R — Engine-independent click-to-preview

**Status:** `COMPLETE / PASS / BASELINED`

**Revision:** Rev1 closeout (independent product binding / measured browser
controls / delete-regenerate + request audit)

**Date:** 2026-08-31

**Closeout:** 2026-09-01

**Classification:** RESEARCH ONLY / W0-P / NON-PRODUCTION

**Decision impact:** `ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS`

This is a W0 Preview architecture validation. It is not a product frontend,
not an Asset Browser, not a production viewer, and not a renderer / GLB /
`<model-viewer>` selection.

Product implementation remains unauthorized until W0-RS and IA-1 complete
the remaining required gates.

---

## Accepted closeout

External focused review accepted this PoC.

```text
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
```

All findings are CLOSED:

```text
PREVIEW-R1-MAJOR-001 independent Product binding + payload integrity
PREVIEW-R1-MAJOR-002 real browser-control evidence
PREVIEW-R1-CLOSEOUT-003 runtime request audit + literal delete/regenerate
```

Rev0 / Rev1 sections below remain historical correction records.

External evidence: `F:\NewResearch\rigforge_w0p_evidence\poc_preview_01r\`

External work/runtime: `F:\NewResearch\rigforge_w0p_work\poc_preview_01r\`

---

## 1. Question

Can RigForge provide useful click-to-preview for Character, Motion, and
Derived Variant assets using rebuildable Preview Artifacts and an
engine-independent viewer, without implementing a full RigForge animation
runtime?

Success requires both:

```text
A. The actual user-facing preview path works for all three V1 asset kinds.
B. Preview remains a derived/non-authoritative layer rather than becoming
   product or source truth.
```

Allowed outcomes:

```text
ACCEPT_DERIVED_PREVIEW_PATH
ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
REJECT_DERIVED_PREVIEW_PATH
INCONCLUSIVE
```

This decision concerns derived Preview Artifact + independent viewer
architecture. It does **not** select GLB, `<model-viewer>`, Three.js, or
Blender as permanent Preview technology.

---

## 2. Architecture Under Test

```text
Authoritative Product Object
        ↓
Preview Request
        ↓
Preview Generator (Blender 5.2.1 LTS, generation time only)
        ↓
Preview Artifact Manifest + Preview Payload
        ↓
Preview Resolver / Binding Check
        ↓
Independent Viewer (browser + local static files)
```

Product objects in this PoC:

```text
CHARACTER          QCHAR_Knight_Male
MOTION             UAL2_Standard_Walk_Carry_Loop
DERIVED_VARIANT    poc-blender-e2e-01.knight-ual2.walk-carry.run1
```

Cards are product-level refs. They are not `.glb` filenames.

Independent product truth lives in
`experiments/w0p/poc_preview_01r/config/product_fixture.json`.
The catalog copies `expected_source_binding` from that fixture. The Preview
Generator writes a derived Manifest from the same fixture (not from the
catalog, and the catalog is not copied from the Manifest). The viewer
validates the **selected catalog entry** (id, version, complete expected
binding) then fetches payload bytes and checks size + SHA-256 before render.

**SOURCE / OBSERVED FACT.** The research mock implements that resolver path
in `experiments/w0p/poc_preview_01r/viewer/app.js`.

**INTERPRETATION.** This is enough to test the architecture. It is not a
product Asset Browser.

---

## 3. Environment / Tool Pins

| Pin | Value |
| --- | --- |
| OS | Windows 11 10.0.26100 |
| Python | 3.13.12 (Anaconda, MSC v.1942 x64) |
| Preview generator | Blender 5.2.1 LTS / `9e2066aef7ef` |
| Browser | Microsoft Edge 151.0.4129.107 (one browser only) |
| Browser automation | Playwright 1.62.0, `channel=msedge` |
| Viewer harness | `@google/model-viewer` 4.3.1 |
| Viewer license | Apache-2.0 (dependency evidence only) |
| npm integrity | `sha512-GP+inXhAtY31E8rILVmByA6z8CZZjdlNajddppyI1/j1eIaSQiZcMRaUqTFe7+jv4mzRzwKIOiKBud0apiv+WQ==` |
| npm tarball | `https://registry.npmjs.org/@google/model-viewer/-/model-viewer-4.3.1.tgz` |
| Local `model-viewer.min.js` SHA-256 | `283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b` (1,068,903 bytes) |
| Local static server | Python `ThreadingHTTPServer` (transport only) |
| Runtime CDN | NO |
| Retrieval | official npm registry; 2026-08-31 |

Vendor bytes remain outside git:
`F:\NewResearch\rigforge_w0p_work\poc_preview_01r\vendor\`

Viewer library: **RESEARCH HARNESS ONLY / NOT SELECTED**.

---

## 4. Product Asset Identities

### Character

```text
kind:     CHARACTER
id:       QCHAR_Knight_Male
file:     Knight_Male.fbx
SHA-256:  fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f
```

External path (not in repo): QChar pack FBX used by POC-FBX-01 /
POC-BLENDER-E2E-01.

### Motion

```text
kind:     MOTION
id:       UAL2_Standard_Walk_Carry_Loop
file:     UAL2_Standard.fbx
SHA-256:  d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d
clip:     Armature|Armature|Walk_Carry_Loop
```

Motion means animation + Source Skeleton context. It is previewed
independently of the Knight Character.

### Derived Variant

Accepted lineage from POC-BLENDER-E2E-01 (unchanged; not retuned):

```text
id:                        poc-blender-e2e-01.knight-ual2.walk-carry.run1
persistence blend SHA-256: fef863e208b51e2fcf70aa278ff9041e39b93c1e9f479940a8db6670d970a532
mapping SHA-256:           683603627bbe431a8438c8b6bacca4d6779046e4ef09ede7fa3218c41e8a602b
retarget policy SHA-256:   43eee4b181ffad617b270eeb8b076e650e39c4e2d7ed8425918d825c9cab1b8b
job spec semantic hash:    bc953822b6498bf3f87c8ca40ddf03365e920adff359a84a291bb27cffc42d80
QC report SHA-256:         692754326cd1fb1c47ad90286a50b0374ca7180f50a5dc56bdf47d421755e8af
```

The accepted `derived_result.blend` was still present externally and was
reused. POC-BLENDER-E2E-01 harness was not invoked and not modified.

---

## 5. Preview Artifact Contract

Thin research contract: `experiments/w0p/poc_preview_01r/schemas/preview_artifact.md`

Each artifact is a JSON manifest plus an external payload. Minimum fields
used in this PoC:

```text
preview_artifact_version
preview_artifact_id
source_product_kind / source_product_id / source_product_version
source_binding hashes
generator identity / backend / version / build
payload type / MIME / ref / SHA-256 / size
scene/bounds observations
animation inventory / default animation / duration
camera_framing_hint
declared_losses
generation_recipe_sha256
generated_from
authority flags (all false)
semantic_hash (normalized; excludes timestamps/PIDs/ports)
```

Payload candidate is GLB. GLB is **not** selected as a permanent RigForge
Preview format. `payload_ref.id` is a catalog payload name, not product
identity.

---

## 6. Character Preview

**SOURCE / OBSERVED FACT.**

| Observation | Value |
| --- | --- |
| loaded | YES |
| bounding / framing valid | YES (mesh AABB valid; viewer auto-framed) |
| renderable nodes/materials | 1 mesh object; materials Skin, Armor, Armor_Dark, Detail, Red |
| animation | not required; inventory empty |
| payload SHA-256 | `d83fa89dc0976f69df17a471e3a0fb96c1ccbb9b871a413f8125836420e36e2a` |
| payload size | 284600 |
| manifest semantic hash | `83b8c7f7010aab5c19b51895d4e37dc99609c413c65fc19db36ef4749582f50f` |
| viewer | Knight visible in rest/T-pose; orbit/zoom via `camera-controls` |

Generation stripped leftover actions (`use_anim=False` plus action clear) so
Character preview is not a source-clip player.

Importer/export originally produced `alphaMode: MASK` with baseColor alpha 0
and no textures. That payload rendered as an empty viewport. The generator
forced opaque materials and a simplified albedo as a declared Preview
choice. See §14.

Screenshot: `review_evidence/frames/character_selected.png`

---

## 7. Motion Preview

Motion preview does **not** require the Knight Character.

**SOURCE / OBSERVED FACT.** UAL2 FBX contained a renderable mesh named
`Mannequin` (`had_source_renderable_mesh: true`). That mesh was discarded.
It is not a general Motion Preview requirement.

**Preferred path tested:**

```text
Motion Asset + Source Skeleton
        ↓
Preview Generator
        ↓
generic derived joint / bone-segment proxy
        ↓
animated Preview Artifact
```

Proxy construction consumes joint hierarchy, parent relations, and rest
head/tail transforms. It does not look up pelvis, human arm/leg, Mixamo, or
UE mannequin roles. `classify_armature` exists only to pick the source
armature inside this specific FBX when more than one armature could exist;
the proxy mesh itself is hierarchy-generic.

This PoC does **not** claim non-humanoid Preview coverage. The Motion used
here is humanoid.

| Observation | Value |
| --- | --- |
| loaded | YES |
| proxy visible | YES |
| default clip | `Armature|Armature|Walk_Carry_Loop` |
| duration (generator) | 2.0 s (frames 1–61 @ 30 fps) |
| duration (viewer) | 2.033 s |
| currentTime advances on play | YES |
| pause stops advancement | YES |
| seek ~0.8 s | YES |
| restart near start | YES |
| payload SHA-256 | `0e50eef5af70be52b9ba99711cf1facb14bd6e267b4996191446b1635d377f3c` |
| payload size | 203916 |
| proxy | 65 joints, 910 verts, 780 faces; `humanoid_role_table: false` |

Screenshots: `motion_selected_start.png` (t=0), `motion_selected_mid.png`
(t=1.0). The mid frame is a different walk-carry pose.

---

## 8. Derived Variant Preview

Input is the accepted POC-BLENDER-E2E-01 persistence blend. Mapping /
Compatibility / Retarget are **not** rerun in the viewer.

| Observation | Value |
| --- | --- |
| loaded | YES |
| target Character visible | YES (Knight mesh) |
| default clip | `CharacterArmatureAction` |
| extra source clips stripped | YES (inventory is that one action) |
| duration (generator) | 2.0 s |
| duration (viewer) | 2.033 s |
| play / pause / seek / restart | YES |
| payload SHA-256 | `e3407a731fdd95985a6e2799a0d30771b99481ac0ad32ce0df45cc773107ba85` |
| payload size | 327848 |

Screenshots: `derived_selected_start.png` (t=0), `derived_selected_mid.png`
(t=1.0, walk-carry pose on Knight).

The viewer does not implement retarget, skinning, or a RigForge animation
runtime. It consumes an already-produced preview-ready payload.

---

## 9. Click-to-Preview Interaction

Playwright drove the real UI cards (not direct function-only selection):

```text
click [CHARACTER] Knight
  → selected_product_kind CHARACTER
  → binding OK
  → character_preview.glb LOADED

click [MOTION] Walk_Carry_Loop
  → selected_product_kind MOTION
  → binding OK
  → motion_preview.glb LOADED
  → play / pause / seek / restart

click [DERIVED_VARIANT] Knight + Walk_Carry
  → selected_product_kind DERIVED_VARIANT
  → binding OK
  → derived_preview.glb LOADED
  → play / pause / seek / restart
```

Catalog identities are product ids. Payload paths are resolved from the
manifest `payload_ref` after binding validation.

Evidence: `interaction_test.json`

---

## 10. Camera / Playback Controls

**Executed tests (Rev1), not capability exposure:**

| Control | Character | Motion | Derived |
| --- | --- | --- | --- |
| camera orbit (pointer drag) | YES, measured | not kind-specifically exercised | YES, measured |
| camera zoom (wheel) | YES, measured | not kind-specifically exercised | YES, measured |
| reset/recenter (`#btn-reset-cam`) | YES, measured | not kind-specifically exercised | not exercised this run |
| play (`#btn-play`) | n/a | YES, measured | YES, measured |
| pause (`#btn-pause`) | n/a | YES, measured | YES, measured |
| restart (`#btn-restart`) | n/a | YES, measured | YES, measured |
| seek (`#seek` input event) | n/a | YES, measured | YES, measured |

Motion shares the same viewer camera widget. This report does **not** claim
a Motion-specific camera test.

`<model-viewer camera-controls>` is capability exposure only until the
pointer/wheel/button path is measured. Evidence: `camera_controls.json`,
`viewer_motion.json`, `viewer_derived.json`.

---

## 11. Binding Diagnostics

Validation is against the **selected Product entry** from the catalog (which
is built from `product_fixture.json`), not `expectedBinding(product_kind)`.

Checks: `source_product_kind`, `source_product_id`, `source_product_version`,
then every key in `expected_source_binding` (Derived includes Mapping,
Retarget Policy, Job Spec, QC, persistence blend, character/motion/clip ids).

Payload: fetch bytes → size → SHA-256 → only then blob URL to the viewer.

Three controlled failures:

```text
wrong character_sha256:
  BINDING_MISMATCH, overlay, payload not shown as valid

wrong source_product_version (hashes otherwise valid):
  BINDING_MISMATCH field=source_product_version
  payload not shown as valid

valid Product binding + wrong payload_sha256:
  PAYLOAD_HASH_MISMATCH
  payload not shown as valid
```

Screenshots: `binding_error.png`, `version_binding_error.png`,
`payload_integrity_error.png`.

This is ordinary local validation, not a security test.

---

## 12. Rebuildability

Literal sequence (Rev1):

```text
Generate A
record Product truth from product_fixture.json + live source SHA-256
delete payloads_a and gen_a Preview outputs
verify Product truth still exists and is unchanged
Generate B from frozen inputs
viewer loads regenerated B (selftest)
```

Product snapshot is **not** reconstructed from deleted Preview Artifacts.

| Kind | Manifest semantic match | Generation semantic match | Raw payload SHA match |
| --- | --- | --- | --- |
| CHARACTER | YES | YES | YES `d83fa89d…` |
| MOTION | YES | YES | YES `0e50eef5…` |
| DERIVED_VARIANT | YES | YES | YES `e3407a73…` |

Raw GLB byte identity happened to match in this run. That is recorded, not
required. Required bar is semantic equivalence (same source binding, kind,
animation inventory/duration semantics, viewer usability on B). That bar was
met.

---

## 13. Runtime Independence

After generation, Blender was terminated. Viewer used only:

```text
browser + static viewer assets + manifests + payloads + localhost transport
```

| Question | Answer |
| --- | --- |
| Blender required at view time | NO |
| Blender process during viewer | NO |
| Unreal required | NO |
| Unity required | NO |
| Maya required | NO |
| network / CDN required to load local artifacts | NO |
| Playwright page request external count | 0 (PASS) |

`environment-image="neutral"` is bundled in the local `@google/model-viewer`
build. Observed origins: `http://127.0.0.1:8765` and `blob:`. Evidence:
`runtime_request_audit.json`.

---

## 14. Artifact Losses / Limitations

Declared (not hidden) losses:

```text
FBX pivot / layer recipes not preserved
Character source clip library not played
UAL2 source mesh discarded; Motion uses synthetic proxy
source MASK/clip alpha forced opaque for Character/Derived visibility
source textures not present on this FBX import; simplified opaque albedo
GLB is a preview payload candidate, not source round-trip
materials are not production/cinematic
proxy is not Canonical Skeleton / production rig / Mapping truth
```

The status panel shows preview type, source id, duration, generator pin, and
warnings.

**Duration classification.** Generator duration is `(frame_end - frame_start) / fps = 2.0`.
Viewer-reported duration is `2.033…` s (≈ 61/30). Difference is within the
declared 0.15 s tolerance. It is a frame-inclusive vs frame-span
representation difference, not silently treated as exact identity.

---

## 15. Browser Evidence

One desktop browser: Microsoft Edge 151.0.4129.107.

Screenshots in `review_evidence/frames/`:

| File | What it shows |
| --- | --- |
| `character_selected.png` | Knight selected, rest pose visible, status LOADED |
| `motion_selected_start.png` | Motion proxy at t=0 |
| `motion_selected_mid.png` | Motion proxy at t=1.0, different pose |
| `derived_selected_start.png` | Knight + baked clip at t=0 |
| `derived_selected_mid.png` | Knight walk-carry pose at t=1.0 |
| `binding_error.png` | Hash fixture, overlay unavailable, BINDING_MISMATCH |
| `version_binding_error.png` | Version fixture, field=source_product_version |
| `payload_integrity_error.png` | Payload SHA fixture, PAYLOAD_HASH_MISMATCH |

Aesthetics are not a PASS criterion.

---

## 16. Timing

Approximate; no performance gate.

| Step | Seconds |
| --- | --- |
| Character preview generation (Blender elapsed) | 0.318 |
| Character wall | 1.032 |
| Motion preview generation | 3.190 |
| Motion wall | 3.945 |
| Derived preview generation | 0.420 |
| Derived wall | 1.152 |
| first Character load (browser) | 2.115 |
| Character → Motion switch | 0.620 |
| Motion → Derived switch | 0.629 |

No architectural feasibility blocker appeared from timing.

---

## 17. Generality Audit

Preview architecture in this PoC does **not** define:

| Claim | Status |
| --- | --- |
| Character == FBX | NO. Character is a product id + source hash. FBX is the frozen input. |
| Motion == humanoid | NO structurally for the proxy. Coverage of non-humanoid Preview is **not** claimed. |
| Derived Variant == `.blend` | NO. Blend is accepted persistence bytes, not product identity. |
| Viewer == Blender | NO. Blender is generation-only. |
| Viewer == Unreal | NO. |
| Preview == GLB identity | NO. GLB is the tested payload candidate; contract is payload-type + hash. |

A later payload replacement remains architecturally open.

`classify_armature()` bone-name checks exist only to pick source vs target
armature inside this frozen Knight+UAL2 pair. They are not Preview Artifact
contract and not Canonical Skeleton identity.

---

## 18. Authority Audit

| Question | Answer |
| --- | --- |
| Preview Artifact is product authority | NO |
| Preview Artifact defines Asset identity | NO |
| Preview Artifact defines Mapping | NO |
| Preview Artifact defines QC | NO |
| deleting Preview destroys product state | NO |

Manifests record `authority.* = false`. Regeneration restores Preview
capability without minting a new Character / Motion / Derived identity.

---

## 19. Gaps / Guards

Acceptable limitations (not by themselves reject reasons):

```text
research viewer UI is a mock
one desktop browser
one viewer library
no mobile / AR / engine integration
Motion proxy is synthetic
Character/Derived albedo is simplified
GLB loses FBX layer/pivot semantics
camera framing uses viewer auto-frame
```

Guards if this architecture is accepted:

1. Do not select `<model-viewer>` for V1 from this PoC.
2. Do not select GLB as permanent Preview format from this PoC.
3. Do not treat Preview payloads as source round-trip or Mapping/QC truth.
4. Keep generation backend replaceable; viewing must stay DCC-free.
5. Keep explicit source binding against **independent Product truth** (id +
   version + complete lineage). Never hardcode card → payload path. Never
   validate by "first asset of this kind".
6. Enforce payload size/SHA before render.
7. Do not claim non-humanoid Preview coverage from this pair.
8. Preserve declared losses in the Preview Artifact (no silent semantic loss).
9. Texture/material fidelity is out of scope for this architecture question.
10. `classify_armature()` remains test-pair generator routing only.

No full RigForge animation runtime was introduced in the browser. If later
Preview needs one, that would be a new decision-relevant experiment.

---

## 20. Decision Impact

Accepted:

```text
ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
```

**SOURCE / OBSERVED FACT.** Click-to-preview worked for Character, Motion
(generic derived proxy, no target Character), and Derived Variant in a
standalone browser viewer after Blender was stopped. Product id/version and
full lineage binding are checked against independent Product fixture truth.
Payload SHA/size are checked before render. Version mismatch and payload-hash
mismatch fail closed. Motion/Derived play/pause/seek/restart are measured
from `#btn-*` / `#seek` UI events. Character and Derived camera orbit/zoom
are measured from pointer/wheel (Character also reset). Rebuild deletes A
outputs, Product truth is unchanged, B loads. Page request audit external
count = 0.

**INTERPRETATION.** A derived, rebuildable, non-authoritative Preview
Artifact plus an independent viewer is sufficient for this V1 preview path
**without** a RigForge animation runtime. Guards above remain in force.
This is not a renderer, GLB, or GUI-framework freeze.

Next: W0-RS. Do not start product implementation. Do not run IA-1 before
W0-RS is externally accepted/baselined.
