# V1-8 Release Qualification

Qualify whether implemented V1 can credibly become a release candidate.

Status: `COMPLETE / PASS / BASELINED`

This is not a Gate D report. Gate D is **READY / NOT STARTED**.

## Classification legend

| Class | Meaning |
| --- | --- |
| REQUIRED_FOR_V1_RELEASE | Must work or fail closed for a V1 candidate |
| QUALIFICATION_ONLY | Measured/documented; not a feature mandate |
| FIX_IF_REPRODUCED | Change only if a concrete defect appears |
| ACCEPTED_LIMITATION | Known and allowed for current V1 |
| POST_V1 | After Gate D / later packaging |

## Product workflow to qualify

```text
Asset Browser
        ↓
select Character + Motion
        ↓
Mapping proposal + Compatibility
        ↓
explicit Mapping acceptance
        ↓
Transfer
        ↓
isolated worker
        ↓
candidate
        ↓
QC / persistence verification
        ↓
Published Derived Variant
        ↓
engine-independent Preview
```

## Qualification matrix

| Item | Class | Result owner |
| --- | --- | --- |
| Frozen Knight / UAL2 Mapping + Transfer + QC + Preview | REQUIRED_FOR_V1_RELEASE | retained V1-4/V1-5/V1-6 tests |
| Representative real-asset Mapping/Compatibility campaign | REQUIRED_FOR_V1_RELEASE | `blender-worker/tests/v1_8_real_assets.rs` |
| Honest non-humanoid / ambiguous failure | REQUIRED_FOR_V1_RELEASE | Horse + UAL2 row |
| Native Workbench Mapping/Compatibility/Transfer wiring | REQUIRED_FOR_V1_RELEASE | Workbench handlers + draw path |
| Catalog reopen retains Published Product | REQUIRED_FOR_V1_RELEASE | existing reopen tests + V1-8 runtime tests |
| Spawn-to-RUNNING crash window | REQUIRED_FOR_V1_RELEASE | dispatch intent + fail closed |
| RUNNING after process restart | REQUIRED_FOR_V1_RELEASE | fail closed; no publication |
| Worker SUCCESS cannot publish | REQUIRED_FOR_V1_RELEASE | unchanged V1-5 |
| Worker script / Preview viewer integrity | REQUIRED_FOR_V1_RELEASE | SHA-256 checks on Transfer, Preview, QC inspect, and fresh reopen |
| Relocatable runtime resource root | REQUIRED_FOR_V1_RELEASE | `app/src/runtime.rs` + relocation tests |
| Exact Published RetargetPolicy selection | REQUIRED_FOR_V1_RELEASE | unique-or-explicit; multi-policy fail closed |
| PreviewHost bounded lifecycle | REQUIRED_FOR_V1_RELEASE | Drop/replace joins listener thread |
| Pinned Blender 5.2.1 LTS `9e2066aef7ef` | REQUIRED_FOR_V1_RELEASE | existing pin + V1-8 constants |
| Offline after deps present | QUALIFICATION_ONLY | `cargo test --offline`; vendored viewer; no CDN |
| Clean-environment assumptions | QUALIFICATION_ONLY | documented; no accidental `target/` / `node_modules` Product identity |
| Performance sanity | QUALIFICATION_ONLY | no pathological optimization phase |
| RUNNING cancellation | ACCEPTED_LIMITATION | NOT REQUIRED FOR V1 |
| Automatic retry | ACCEPTED_LIMITATION | manual new attempt |
| Worker pool | ACCEPTED_LIMITATION | one fresh process per attempt |
| Polished installer | POST_V1 | release bundle notes only |
| Blender GPL redistribution | HUMAN / LEGAL REVIEW REQUIRED | inventory only |
| Universal FBX/humanoid/GPU/browser claims | unsupported | must not be claimed |

## Retry / cancellation / pool decisions

```text
RUNNING cancellation: NOT REQUIRED FOR V1
Automatic retry:      NOT REQUIRED FOR V1
Worker pool:          NOT REQUIRED FOR V1
```

Manual re-run creates a new JobRun/attempt. Old WorkerResult cannot
authorize a new JobRun (`app/tests/orchestration.rs`).

Lack of RUNNING cancellation can leave a native Transfer blocking the UI
thread until collect returns. That is an operational limitation, not
Product-authority corruption. A restarted app fails leftover RUNNING
closed.

## Supported V1 envelope (summary)

See [V1_8_REAL_ASSET_MATRIX.md](V1_8_REAL_ASSET_MATRIX.md) for tested scope.

```text
Character: already rigged only
Motion: requires Source Skeleton context
execution frames: integral Blender frame points
backend: pinned Blender 5.2.1 LTS only
Mapping: deterministic proposal + explicit review
Preview: GLB / model-viewer path; non-authoritative
engine integration: none
Auto-Rig: unsupported
direct Unity/Unreal export: not part of current V1
V1-7 export: SKIPPED
```

Claim bound:

```text
PASS FOR TESTED V1 QUALIFICATION MATRIX
NOT A CLAIM OF UNIVERSAL ASSET SUPPORT
```

## Failure semantics (must not false-succeed)

Retained and/or covered by existing plus V1-8 tests:

- invalid Product graph → reject before execution where possible
- worker launch failure → no publication
- worker terminal failure → durable FAILED
- bad WorkerResult correlation → reject + FAILED
- fractional frame → fail before Blender
- fresh reopen / root scale / missing staged / persistence / QC failure → no publication
- Preview generation failure → Product remains published
- Preview payload corruption / missing Preview → Product remains valid
- spawn-to-RUNNING crash → DISPATCHABLE+intent becomes FAILED on reopen
- RUNNING after restart → FAILED; cannot publish
- tampered/missing runtime worker package → no QC evidence, no fresh reopen PASS, no Transfer/Preview production launch

## Packaging qualification bound

```text
runtime resource relocation: TESTED
release bundle layout: DEFINED
polished installer: POST_V1
third-party clean VM installer SKU: NOT TESTED
```

Do not treat relocatable-resource tests as a clean-machine installer SKU.

