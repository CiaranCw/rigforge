# R1 UAT Plan

Mandatory real-human first-use checkpoint as part of R1-V (historical R1-FR + R1-UAT).

Status: `FINDINGS / awaiting re-UAT`

R1-V Human UAT confirmed `R1-V-MAJOR-001`. R1-FIX is an implementation
complete candidate. Re-UAT uses
[R1_INTEGRATED_VALIDATION.md](R1_INTEGRATED_VALIDATION.md).

R1-2 is `COMPLETE / PASS / BASELINED`. Transfer execute, QC inspect, and
fresh-reopen waits are off the egui thread. UAT still confirms that a real
user can complete first-use without a frozen window.

This is not a speed contest. Do not set an artificial time-to-success
target before measuring a baseline.

R1-FIX is authorized only by issues observed or clearly reproduced here.

## Frozen task

A first-time user with the local supported pair must:

```text
add Character
add Motion
select / accept Mapping
Compatibility
Transfer
Preview Derived
```

without being told:

```text
Skeleton name
clip ID
frame range
FPS
```

## UAT-1 — fresh Catalog, Knight + UAL2

Assets (already permitted local test material; do not redownload):

```text
Knight_Male.fbx
UAL2_Standard.fbx
```

Start:

```text
fresh Catalog
only those two files as user-visible inputs
```

Do not pre-seed Character or Motion records.

Record:

| Field | How |
| --- | --- |
| manual text fields actually required | count fields the user typed (not Browse) |
| points of confusion | observer notes |
| misclicks | observer notes |
| errors | Product-facing message + whether recovered |
| blocking waits | which operations froze the window, duration |
| time to first successful Preview | wall clock from first Browse to Derived Preview |

Expected if R1-1 and R1-2 landed:

- Browse instead of typed paths
- no typed skeleton / clip / frames / FPS when those observations are unique
  or (clips) explicitly selected from a list
- 2+ Armatures fail closed with the R1 diagnostic (Knight/UAL2 are expected
  unique-Armature files; multi-Armature picking is not a UAT-1 requirement)
- UI remains usable during Transfer (spinner / Running, not a frozen window)
- exact Published Derived is the Preview selection

Pass/fail of Product authority is not “fast”. Fail if the user cannot
complete the path, if Product truth was guessed, or if a Derived published
without Mapping acceptance / Compatibility / QC / PersistenceVerification.

## UAT-2 — multiple animation candidates

Historical PoC evidence (`POC-BLENDER-E2E-01`) records that
`UAL2_Standard.fbx` is not a single plain Walk take; it listed at least
`Walk_Carry_Loop` and `Zombie_Walk_Fwd_Loop`.

UAT-2 uses that **same already permitted file** if R1-1 discovery reports
two or more usable clips **strongly associated with the unique Armature**
(`direct_action`, `nla_strip`, or `pose_channels`).

`action_suitable_slots` / slot type alone must not list a clip.
Camera, material, mesh/object-only, and unrelated imported Actions must
not appear as Motion choices. The selected clip identity must equal
registered `TimeDomainProvenance.clip_identity_evidence`.

Validate:

```text
RigForge detects multiple clips
  → user understands the selector
  → exact selected clip is the registered TimeDomainProvenance.clip_identity_evidence
```

If discovery on that local file reports fewer than two usable clips:

```text
UAT-2: NOT TESTED
```

Do not download random assets to satisfy this checkbox.

## Observer script (human)

1. Give the user only the two FBX files and RigForge Workbench.
2. Ask them to get a preview of the transferred animation on the Knight.
3. Do not hint field names.
4. If they stall, wait before helping; record the stall.
5. After success or abandonment, copy the registered Motion clip identity
   from Catalog/Workbench (operator, not user) for UAT-2 correlation.

## R1-FIX authorization

Default: only UAT-observed or clearly reproduced issues.
Historical name: R1-D.

Examples that *may* become R1-FIX if seen:

- unclear label
- bad default
- missing busy state
- misleading error
- layout issue
- Cancel becoming necessary because waits are long

Do not pre-load R1-FIX with speculative features.

## Evidence

Store UAT notes as review evidence according to repository policy
(`review_evidence/` is gitignored unless a later baseline explicitly tracks
a path). Do not commit FBX, GLB, `.blend`, or Blender distributions.
