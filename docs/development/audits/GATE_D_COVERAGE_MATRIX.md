# Gate D — Coverage Matrix

Audit date: 2026-09-02

Baseline: `ecbbb28d2ae59cdd5c38abbd1db1cdf0f0a26a8a`

Result: `GATE_D_FINDINGS`

Evidence classes:

```text
SOURCE      = current main implementation read and traced to enforcing code
TEST        = asserted by a named test rerun unfiltered in this audit's 490-test suite
REAL        = real pinned Blender 5.2.1 LTS 9e2066aef7ef execution in this audit's rerun
PROBE       = negative probe written and executed by this audit
DOC         = normative contract text
NOT TESTED  = no evidence exists
EXTERNAL    = human/legal prerequisite
```

## A. Product identity — COVERED

Typed UUIDv7 IDs in `domain/src/identity.rs`; paths, digests, rowids, UUIDv4,
Blender datablocks, FBX object names, `.blend`, GLB, and Preview paths are
never Product identity. `SOURCE + TEST` (`identity::tests::path_is_not_an_id`,
`::digest_is_not_an_id`, `::uuid_v4_is_rejected_on_parse`,
`catalog::catalog_surface_tests::product_ids_are_not_sqlite_rowids`,
`v1_8_runtime.rs::path_is_not_product_identity`).

## B. Exact version resolution — COVERED

`CharacterAssetVersion`, `MotionAssetVersion`, `SourceSkeletonReference`,
`BoneMappingVersion`, `RetargetPolicyVersion`, `CompatibilityResult`,
`JobSpec`, `DerivedVariantVersion`, `PersistenceArtifact`,
`PersistenceVerification`, and `PreviewArtifact` are all loaded by exact ID.
`assemble_worker_dispatch_request` is documented and implemented as the exact
projection that never resolves latest/current, and the worker refuses a
JobSpec-only dispatch so it cannot resolve anything itself.
`SOURCE + TEST + REAL` (`adapter_fake.rs::job_spec_only_dispatch_is_refused`,
`application.rs::selected_version_is_exact_and_job_status_does_not_mutate_product`).

Residual latitude reviewed and accepted: `latest_preview_for_exact_version`
and `latest_compatibility_for_exact_set` order candidates *within* an already
exact Product key set; they do not substitute a different Product version.
Reopened Workbench mapping selection picks the greatest `BoneMappingVersionId`
among versions that already match the exact selection, which
`docs/development/V1_4_MAPPING_WORKFLOW.md` states explicitly and
`shell.rs::multiple_published_mappings_do_not_use_arbitrary_unrelated_first`
asserts.

## C. Published immutability — COVERED

`may_replace` refuses `published` / `ready` / `invalidated` with
`AppError::ImmutablePublished`; `reject_public_authority_bypass` blocks
generic `DerivedVariantVersion` and logical pointer writes outright. Later
Preview generation, QC, fresh reopen, and V1-8 release hardening do not touch
a Published version. `SOURCE + TEST + REAL + PROBE`
(`rerun_existing_variant_preserves_old_published_version`,
`generic_put_cannot_repoint_published_pointer`,
`db_reopen_preserves_published_lineage`,
`assets::tests::published_cannot_return_to_draft`, and the audit probe's
`ImmutablePublished` rejection of a forged `CompatibilityResult`).

## D. Mapping authority — COVERED WITH OBSERVATION

Automatic proposals are candidates; `BoneMappingVersion::publish` requires
`reviewed = true` and rejects `automatic_candidate`;
`validate_review_history` forces a Published version's `review_kind` to equal
`derived_accepted_kind()` derived from `generated_from_candidates` and
`user_modified`. Ready and ReadyWithWarnings both require a Published mapping,
so a reviewed Draft stays `MappingConfirmationRequired`.
`SOURCE + DOC + TEST + REAL + PROBE`. `GATE-D-OBS-001` records that the
generic Draft-replacement path can perform the acceptance lifecycle transition
outside `accept_mapping_version` and therefore skips its optional
`SkeletonSummary` side checks; it cannot create a Published mapping that
contradicts its own durable review history.

## E. Compatibility authority — COVERED

Five typed dimensions; `summary` must equal `derive_summary(...)` or Domain
validation fails; `validate_compatibility_graph_on` re-checks every referenced
exact record, requires a Published mapping for Ready/ReadyWithWarnings, and
requires the mapping and motion Source Skeletons to agree. A caller cannot
replace a persisted result in place. `SOURCE + TEST + REAL + PROBE`.

## F. Retarget Policy selection — COVERED

`ensure_published_proven_policy`: zero Published policies create the exact
proven V1 default, exactly one is used as an exact version, two or more fail
closed demanding explicit selection. `require_published_policy_version`
rejects a non-Published selection. No production path selects first / latest /
lexicographically first. `SOURCE + TEST`
(`shell.rs::multiple_published_policies_do_not_select_arbitrary_first`,
`::explicit_policy_selection_binds_compatibility_and_transfer`).

## G. Transfer authorization — COVERED WITH MAJOR FINDING

`start_transfer` consumes one exact persisted `CompatibilityResult` and
derives Character, Motion, Mapping, and Policy from it; no caller substitute
is accepted. `Unsupported` and `MappingConfirmationRequired` are refused;
`ReadyWithWarnings` requires durable acknowledgement recorded on the JobSpec.
`SOURCE + TEST + REAL + PROBE`.

At the Application layer this is sound. At the Workbench layer the
authorization is not invalidated when the selection changes, and the
acknowledgement flag is never reset per evaluation — `GATE-D-MAJOR-002`.
Separately, a JobSpec enqueued through the generic route with no
`compatibility_result_id` can be executed by a worker, which is the documented
historical-execution allowance and still fails closed at publication —
`GATE-D-OBS-009`.

## H. Job / Worker correlation — COVERED

`JobSpec` is distinct from `JobRun`; attempt IDs are orchestrator-owned;
`dispatch` rejects a receipt whose `attempt_id` differs; `WorkerResult`
carries `ExecutionCorrelation`; one WorkerResult cannot authorize two runs or
create two candidate versions. `SOURCE + TEST + REAL`.

## I. Launch / collect lifecycle — COVERED

`resolve exact graph -> write dispatch intent -> launch -> DispatchReceipt ->
durable RUNNING -> collect -> WorkerResult -> terminal JobRun`. `dispatch`
does not collapse launch and terminal execution: RUNNING is persisted from the
receipt, and the intent file is cleared only afterwards. `SOURCE + TEST + REAL`.

## J. Failure durability — COVERED

Valid terminal failure becomes durable `FAILED`; rejected or mismatched
failure evidence is discarded while the run still becomes durably `FAILED`.
No ordinary failure leaves Product truth indefinitely `RUNNING`.
`SOURCE + TEST` (`orchestration.rs`, `adapter_fake.rs` invalid/missing
envelope tests).

## K. Restart / crash recovery — COVERED

Intent is persisted before spawn. On reopen, `DISPATCHABLE` + leftover intent
becomes durable `FAILED`, and `RUNNING` becomes durable `FAILED` with no
WorkerResult. Recovery fabricates no evidence, publishes nothing, and attaches
to no process. For any other state a leftover intent file is merely deleted,
so a valid completed run cannot be failed. `reconcile_interrupted_runtime`
runs from both `SqliteCatalog::open` and `open_in_memory`.
`SOURCE + TEST` (`runtime_recovery.rs`).

## L. Frame provenance — COVERED

Rational `TimePoint` is preserved; `is_integral_frame` gates execution;
fractional frames fail before Blender launch with no truncate, round, floor,
or ceil. `SOURCE + TEST + PROBE`
(`adapter_fake.rs::fractional_frame_never_reaches_blender_worker`; the audit
probe observed `method_eligibility = Fail` for a 3/2..5/2 frame motion).

## M. Retarget scale policy — COVERED

`KeepTargetRestScale` is projected and executed; source animated root scale
does not propagate; target rest scale is preserved; fresh reopen must report
`root_scale_audit = PASS`, and missing / empty / FAIL / skipped all fail
closed. `SOURCE + TEST + REAL` (`root_scale.rs` 4 real-Blender tests plus the
`adapter_fake.rs` reopen-audit family).

Enforcement asymmetry recorded as `GATE-D-OBS-007`: Transfer collect requires
`root_scale_audit` as a mandatory envelope field and demands `PASS`, while the
publication reopen parser in `pinned_qc.rs` omits the field and relies on the
SHA-pinned `worker.py` gating `status: SUCCESS` on that audit itself.

## N. Persistence / publication — COVERED

`PersistenceArtifact` is distinct from `DerivedVariantVersion`, and file
existence is not verification. Publication requires the exact bound QcReport
with verdict Pass, the exact PersistenceVerification with structural and fresh
reopen Pass, JobRun SUCCEEDED, worker success, eligible Compatibility with
acknowledgement where required, the exact JobSpec target, and the exact draft
pointer — then freezes the version and binds the logical pointer in one
transaction. Worker success alone, candidate file alone, QC alone, or reopen
alone never publish. `SOURCE + TEST + REAL`.

## O. QC authority and non-mutation — COVERED

QC is Product-owned: the sealed pinned inspect path produces measurements, the
Product evaluator derives the verdict from seven typed required checks, and
bytes are hashed before and after inspection. Generic Catalog persistence
rejects `QcReport`. `SOURCE + TEST + REAL`.

## P. Publication-critical package integrity — COVERED

One shared authority, `app/src/runtime.rs::verify_runtime_worker_package`,
pins `worker.py` `01105c47…89a5d` and `preview_gen.py` `c84aada5…3db795`, and
is applied on Transfer dispatch, Preview generation, QC inspect, and fresh
reopen. No production entry point can execute an unverified production
`worker.py`. `SOURCE + TEST + REAL`. `GATE-D-OBS-002` records the un-gated
harness constructor in the shipped worker crate, which no shipped code path
uses.

## Q. Blender pin — COVERED

5.2.1 LTS build `9e2066aef7ef`, archive SHA-256 `0e631dad…cb2c`. Production
resolves Blender only from the runtime root and ignores
`RIGFORGE_BLENDER_EXECUTABLE` for publication-critical inspect/reopen and for
Preview generation. Wrong version, wrong build, wrong adapter version, and
spoofed executables all fail. Relocatability is not weakened.
`SOURCE + TEST + REAL` (observed `Blender 5.2.1 LTS (hash 9e2066aef7ef …)`).

## R. RuntimeLayout / release relocation — COVERED

Production resolution is `RIGFORGE_RUNTIME_ROOT` then the current executable
directory, with a thread-local bind used only by tests. No production consumer
uses `CARGO_MANIFEST_DIR`, the source checkout, or a hard-coded developer
tree. All five consumer families were audited: Transfer worker, skeleton
inspector, Preview generator, QC inspect / fresh reopen, and the viewer.
`SOURCE + TEST + REAL`.

## S. Defined release bundle — COVERED

Layout defined in `V1_8_RELEASE_DEPENDENCIES.md` and `app/src/runtime.rs`,
materializable via `materialize_runtime_bundle`, relocatable, internally
consistent, and sufficiently reproducible for V1. No installer exists and none
was tested or claimed. `SOURCE + DOC + REAL`; installer `NOT TESTED`.

## T. Workbench Product E2E — COVERED WITH MAJOR FINDING

Event wiring is real: `WorkbenchHost` retains the Application and every
rendered control ("Propose Mapping", "Accept Mapping", "Evaluate
Compatibility", warnings checkbox, "Transfer", the three Preview buttons,
"Regenerate Preview", and the three selection lists) calls an
Application-backed handler. `GATE-C-OBS-001` is addressed in the current
implementation, verified in source rather than trusted.
`SOURCE + TEST + REAL` (`shell.rs::native_draw_source_wires_product_actions`,
`::native_handlers_propose_accept_evaluate_and_queue_transfer`, and the
frozen-pair `via_workbench_handlers` campaign row).

The flow is nevertheless unreachable for an end user because no production
path can register a Character or Motion asset — `GATE-D-MAJOR-001`. Where it
is reachable it is not bound to the current selection: changing the selected
Character or Motion leaves `compatibility_id`, `transfer_auth`, and
`warnings_acknowledged` intact, so Transfer stays enabled for the previous
pair and a single acknowledgement satisfies later `ReadyWithWarnings` results
— `GATE-D-MAJOR-002`. The published Derived Variant is also not bound into the
Preview Derived selection — `GATE-D-MINOR-003`.

## U. UI-thread Transfer limitation — COVERED

Assessed as operationally imperfect but V1-usable, not release-blocking.
`GATE-D-OBS-006`. `SOURCE + REAL`.

## V. Preview binding — COVERED

All three kinds (`CharacterAssetVersion`, `MotionAssetVersion`,
`DerivedVariantVersion`) require exact binding validation before display; a
Preview for Product Version A never satisfies Version B; selection change
clears a previously valid Preview. `SOURCE + TEST + REAL`.

## W. Preview payload integrity order — COVERED

Order enforced: the selected Product is known independently, the
`PreviewArtifact` is resolved, exact binding is verified, the payload is read,
size is checked, SHA-256 is checked, and only then does the viewer receive the
bytes. The viewer never loads first and validates afterwards.
`SOURCE + TEST + REAL`.

## X. Preview animation validity — COVERED

For Motion and Derived Variant, the actual loaded
`viewer.availableAnimations.length` must be nonzero, and descriptor/session
metadata cannot override contradictory payload evidence. Character may have
zero animations, and Character generation strips unintended animation.
`SOURCE + TEST + REAL`.

## Y. PreviewHost lifecycle — COVERED

`PreviewHost` owns an `Arc<AtomicBool>` stop flag and the listener
`JoinHandle`; `shutdown` sets the flag and joins; `Drop` calls `shutdown`. The
Workbench owns one bounded active host, and repeated Preview/regenerate does
not accumulate detached servers. `SOURCE + TEST + REAL`
(`preview_host_shutdown_terminates_server_thread`,
`preview_host_replacement_shuts_down_previous`,
`repeated_preview_host_replace_stays_bounded`).

## Z. Viewer package integrity — COVERED

`@google/model-viewer 4.3.1` vendored; pinned SHA-256
`283b0672…f033b` independently rehashed and matching; no CDN required at
runtime; a corrupt viewer yields Preview unavailable without corrupting
Product truth. `SOURCE + TEST + REAL`.

## AA. Preview / publication independence — COVERED

Preview generation failure does not revoke a Published Derived Variant;
Preview deletion does not revoke it; Preview regeneration creates a new
derivative identity rather than a new Product version. `SOURCE + TEST + REAL`.

## AB. V1-7 boundary — COVERED

No Download GLB, Export GLB, Export FBX, Unity export, Unreal export, Maya
integration, or output-profile implementation exists. The only GLB surface is
the loopback Preview host serving `payload.glb` to the local viewer, and the
viewer's `createObjectURL` feeds `model-viewer` rather than any download
control. `SOURCE`.

## AC. Real-asset evidence — COVERED (bounded)

Frozen Knight_Male + UAL2 `Walk_Carry_Loop` reproduced through Mapping,
explicit acceptance, Compatibility, Transfer, real isolated worker, QC, fresh
reopen, PersistenceVerification, Published Derived Variant, and Preview.
Additional matrix reproduced: Goblin_Male `ReadyWithWarnings`, Mannequin_F
`Ready`, Horse `Unsupported` honest reject. `REAL`. Claim bound: tested V1
matrix only, not universal asset support.

Two qualifications. The matrix specifics are not regression-locked — the Horse
assertion only checks that the outcome is a string and is not
`ASSET_DIGEST_MISMATCH`, and the frozen-pair assertion accepts
`MappingConfirmationRequired` (`GATE-D-OBS-008`). This audit regenerated and
read the artifact, so the behavior is confirmed at this baseline but not
pinned. Separately, `V1_SCOPE.md` still states real non-humanoid E2E is a
release hardening requirement, which an `Unsupported` Horse row does not
satisfy (`GATE-D-MINOR-002`).

## AD. Generality claims — COVERED

No claim of all FBX, all humanoids, all non-humanoids, all browsers, all
Windows machines, all GPUs, or solved Auto-Mapping appears in the current
contracts. The bounded claim is stated in both
`V1_8_RELEASE_QUALIFICATION.md` and `V1_8_REAL_ASSET_MATRIX.md`. `DOC + REAL`.

## AE. Accepted limitations — COVERED

RUNNING cancellation, automatic retry, and worker pool are honestly
classified as not required for V1: manual re-run creates a new attempt ID, an
old WorkerResult cannot authorize it, failed history is retained, inability to
cancel corrupts nothing, and one fresh process per attempt is an accepted
isolation strategy. `SOURCE + TEST + DOC`.

## AF. Offline behavior — COVERED

No network probing was performed. The suite ran fully offline;
`requires_network()` is false for both Application and Workbench; the viewer
is vendored with decoder CDNs redirected locally; no runtime CDN,
third-party hosted JavaScript, network service, or online engine is required
once dependencies and assets are present. `SOURCE + TEST + DOC`.

## AG. Clean runtime assumptions — COVERED

Production resolution does not rely on the Cargo target cache,
`node_modules`, the source checkout, user Blender startup or add-ons,
developer temp files, old Preview outputs, or untracked scripts. Worker
processes run with factory startup, isolated TEMP, and
`RIGFORGE_BLENDER_EXECUTABLE` removed from the child environment on sealed
paths. No clean-VM installer test is claimed. `SOURCE + TEST`.

## AH. Product path independence — COVERED

`RuntimeLayout` paths and package digests are operational runtime evidence.
Product truth survives moving runtime resource locations, and location
evidence is stored separately from Product identity. `SOURCE + TEST + REAL`.

## AI. Dependency / license inventory — COVERED as inventory

Rust dependencies, bundled SQLite via `rusqlite`, Blender, worker Python,
`model-viewer 4.3.1`, Lit notices, and browser/runtime assumptions are
inventoried with license, notice, and distribution questions classified rather
than resolved. No unsupported legal conclusion is drawn here.
`DOC + SOURCE`; see `review_evidence/gate_d_dependency_review.md`.

## AJ. Human / legal review — COVERED

Blender GPL redistribution remains `HUMAN / LEGAL REVIEW REQUIRED`. The
repository makes no false clearance claim, so there is no technical finding.
`EXTERNAL`.

## AK. Untested-bypass hunting — COVERED

Public and generic persistence routes were inspected directly rather than
trusted through named tests. `reject_public_authority_bypass` and
`may_replace` were read line by line; `put_validated_on` graph revalidation
was traced for `JobSpec`, `CompatibilityResult`, `QcReport`,
`PersistenceArtifact`, `PersistenceVerification`, `DerivedVariantVersion`, and
`PreviewArtifact`; and two negative probes were written and executed against
the real public API. One documented latitude was found and classified
(`GATE-D-OBS-001`); one forgery attempt was rejected by the Catalog.
`SOURCE + PROBE`.

## Scope discipline — COVERED

No production source was modified. No V1-7 export, engine integration, second
DCC backend, Auto-Rig, new Auto-Mapping system, artistic scoring, worker pool,
or installer was introduced. No security, adversarial, or network testing was
performed. Gate A, Gate B, and Gate C reports were read and left unchanged.
