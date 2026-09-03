# Gate D — Findings

Audit date: 2026-09-02

Auditor: new independent clean-context GPT-5.6 Sol High agent

Baseline: `ecbbb28d2ae59cdd5c38abbd1db1cdf0f0a26a8a`

Result: `GATE_D_FINDINGS`

## Finding inventory

```text
MAJOR:       2
MINOR:       3
OBSERVATION: 11

OPEN MAJOR:  2
OPEN MINOR:  3
```

The Product-authority core held under every probe this audit ran. No path was
found that could falsely assert Mapping acceptance, Compatibility readiness,
worker success, QC PASS, persistence verification, or publication, and no path
was found that mutates a Published Product version. Both MAJOR findings are on
the native product surface — a missing entry point and a stale-state defect in
the Workbench flow — not corrupted Domain or Catalog invariants.

## GATE-D-MAJOR-001 — No production path can register a Character or Motion asset

Severity: `MAJOR`

Affected files:

```text
app/src/application.rs
workbench/src/lib.rs
workbench/src/main.rs
workbench/Cargo.toml
```

Contract violated:

```text
docs/product/V1_SCOPE.md — "In scope": Character Asset management,
    Motion Asset management, Asset Browser
docs/product/V1_SCOPE.md — happy path begins "select Character + select Motion"
docs/development/V1_8_RELEASE_QUALIFICATION.md — qualified workflow begins
    "Asset Browser -> select Character + Motion"
docs/development/V1_1_DOMAIN_CONTRACT.md — CharacterAssetVersion is created on
    "New ingest/edit intended as history"
Gate D brief section 26 — required native Workbench Product E2E flow
```

Reproduction (source-confirmed):

1. Read `app/src/application.rs`. Enumerate the public surface. It exposes
   list/load/resolve queries, `enqueue_job`, `job_status`,
   `mark_dispatchable`, `dispatch`, `collect`, `complete_success`,
   `complete_failure`, `assemble_worker_dispatch_request`, plus the mapping,
   preflight, transfer, and preview workflows reached through the same type.
   There is no operation that creates a `CharacterAsset`,
   `CharacterAssetVersion`, `MotionAsset`, or `MotionAssetVersion` from a
   source file, and none that builds `SourceArtifactEvidence`.
2. Search `workbench/src/` for `CharacterAsset`, `MotionAsset`,
   `SourceArtifactEvidence`, `put_validated`, `FileDialog`, `rfd`, or
   `pick_file`. There are no matches. `workbench/Cargo.toml` declares no file
   dialog dependency.
3. Search the repository (excluding `experiments/`) for
   `CharacterAssetVersion::draft` and `MotionAssetVersion::draft`. Every hit
   is either the constructor definition in `domain/src/assets.rs`, an
   integration test, a `#[cfg(test)]` module inside `app/src/preview.rs`, or
   `app/src/test_graph.rs`, which is compiled only under `#[cfg(test)]` or the
   non-default `test-support` feature. `app/tests/transfer.rs::
   production_app_and_workbench_do_not_enable_test_support` asserts that the
   shipped Workbench does not enable that feature.
4. Read `workbench/src/main.rs`. The only shipped product binary is
   `rigforge_workbench`; `WorkbenchHost::open_default` opens or creates
   `rigforge-catalog.sqlite` and hands the Application to `WorkbenchApp::draw`.
   The Asset Browser panels render `Application::list_characters()`,
   `list_motions()`, and `list_derived_variants()`.
5. Confirm the shipped binary set: the workspace declares exactly two
   `[[bin]]` targets outside `experiments/` — `rigforge_workbench` and
   `rigforge_fake_blender` (a test double). There is no CLI or importer.

Consequence: a user who installs the release bundle and launches
`rigforge_workbench.exe` against a fresh catalog gets an empty Asset Browser
and has no supported way to add a Character or Motion. Selection, Mapping
proposal, Mapping acceptance, Compatibility, Transfer, QC, publication, and
Preview are all correctly wired but unreachable, because every one of them
begins from an existing exact `CharacterAssetVersion` and
`MotionAssetVersion`. Every real-asset result in the V1-8 matrix and in this
audit's reproduction was produced by test code constructing those records
directly.

Impact:

Blocks required Product E2E usability for the defined V1 technical release
candidate. It does not corrupt Product authority, provenance, durability, or
publication safety: nothing in the implemented chain is wrong, the entrance
to the chain is absent.

Why this is not classified as an accepted limitation:

`docs/development/V1_8_RELEASE_QUALIFICATION.md` lists
`ACCEPTED_LIMITATION` items (RUNNING cancellation, automatic retry, worker
pool) and `POST_V1` items (polished installer). Asset registration appears in
neither list, is not marked `DROP_FROM_V1` or `DEFER` in
`docs/product/V1_SCOPE.md` — where it is instead listed *in scope* — and no
V1-1..V1-8 stage plan claims or defers ownership of it. The repository
therefore presents a release candidate whose documented entry point has no
implementation.

Required correction scope (for the Main Agent; do not implement in Gate D):

Add a Product-owned asset registration path and expose it on the native
Workbench, or explicitly and honestly rescope V1 to declare that catalog
population is out of the V1 product surface and state how a V1 user is
expected to obtain a populated catalog. If the first option is chosen, the
new path must build `SourceArtifactEvidence` (location, digest, size, media
type) through Domain validation, must keep filesystem paths as location
evidence rather than Product identity, and must not weaken the existing
`Validated<T>` ingress boundary. Owner: Product surface / Workbench
integration.

Gate effect:

```text
blocking: YES
false Product success: NO
OPEN MAJOR remains: 1
```

## GATE-D-MAJOR-002 — Character/Motion selection does not invalidate Compatibility or Transfer authorization

Severity: `MAJOR`

Affected files:

```text
workbench/src/lib.rs
```

Contract violated:

```text
docs/development/V1_8_RELEASE_QUALIFICATION.md — REQUIRED_FOR_V1_RELEASE:
    "ReadyWithWarnings requires explicit acknowledgement"
docs/product/V1_SCOPE.md — happy path is a sequential flow bound to the
    selected Character + Motion
Gate D brief section 26 — the native flow must be bound to the current
    exact selection
```

Reproduction (source-confirmed):

1. `select_character_version` and `select_motion_version`
   (`workbench/src/lib.rs:204-212`) set the new selection and call only
   `clear_preview_presentation()`, which touches `preview_host`,
   `preview.occupied`, `preview_valid`, and `preview_status`.
2. The Asset Browser click handler (`lib.rs:838-846` for Characters,
   `lib.rs:863-869` for Motions) then calls
   `bind_published_mapping_for_current_selection`, which resets `mapping_id`,
   `mapping_version_id`, `accepted_mapping_version_id`, `mapping_entries`,
   `unmapped`, and `ambiguities` (`lib.rs:147-170`).
3. Nothing in either path clears `compatibility_id`, `compatibility_summary`,
   `transfer_auth`, or `warnings_acknowledged`.
4. `transfer_available()` (`lib.rs:398-403`) reads `transfer_auth.eligible`,
   so the rendered `add_enabled(self.transfer_available(), Button::new(
   "Transfer"))` control (`lib.rs:997-999`) stays enabled after the selection
   changes.
5. `request_transfer` (`lib.rs:423-453`) re-authorizes and then transfers using
   the retained `self.compatibility_id` — the previous pair — not the current
   selection.

Two distinct consequences:

- **Stale Transfer.** With Character A evaluated to an eligible summary, a user
  who then clicks Character B sees Character B selected, an empty Mapping
  panel, and an enabled Transfer button. Clicking it runs a real isolated
  Blender attempt for Character A and publishes a Derived Variant for
  Character A. Nothing in the Workbench indicates that the transferred pair is
  not the displayed pair.
- **Carried acknowledgement.** `on_evaluate_compatibility_clicked`
  (`lib.rs:499-537`) never resets `warnings_acknowledged`, and neither does
  either selection setter. An acknowledgement ticked once for an earlier
  `ReadyWithWarnings` pair persists, so a later `ReadyWithWarnings` result is
  authorized immediately by `refresh_transfer_authorization`
  (`lib.rs:366-376`) with the checkbox already satisfied. The user never
  acknowledges warnings for the pair actually being transferred.

Impact:

The published Product record remains internally coherent — the JobSpec binds
the exact `CompatibilityResult` and its recorded `warnings_acknowledged`, so
provenance truthfully describes what was executed. The defect is on the native
product surface: the Workbench authorizes and executes a Transfer that does
not correspond to the user's current selection, and it consumes a stale
acknowledgement to satisfy a REQUIRED_FOR_V1_RELEASE gate. On a Final Release
Readiness audit of the native flow, publishing a Derived Variant for an asset
the user is not looking at is a user-facing wrong-result defect, not a
cosmetic one.

Existing coverage does not catch it:
`workbench/tests/shell.rs::ready_with_warnings_requires_explicit_acknowledgement`
and `::warnings_checkbox_handler_refreshes_authorization` exercise a single
pair, and `::selection_change_clears_previous_valid_preview` asserts only the
Preview reset. No test changes selection after authorization.

Required correction scope (for the Main Agent; do not implement in Gate D):

Selection change must invalidate derived workflow state that is bound to the
previous exact selection — at minimum `compatibility_id`,
`compatibility_summary`, `compatibility_notes`, `transfer_auth`, and
`warnings_acknowledged` — and a fresh Compatibility evaluation must require a
fresh acknowledgement. A regression test must change selection after
authorization and assert that Transfer becomes unavailable. Owner: Workbench
product flow.

Gate effect:

```text
blocking: YES
false Product success: NO (published provenance stays truthful)
wrong-pair Product result reachable from the native UI: YES
```

## GATE-D-MINOR-001 — Current product and ADR-0001 text still assert unselected, unauthorized, and OPEN

Severity: `MINOR`

Affected files:

```text
docs/architecture/decisions/ADR-0001-v1-scope-and-blender-backed-execution.md
docs/product/V1_SCOPE.md
docs/product/PRODUCT_VISION.md
docs/architecture/V1_WORKFLOW_DOMAIN.md
docs/architecture/V1_BLENDER_BACKED_ARCHITECTURE.md
```

`AGENTS.md` names these as current Source of Truth, and ADR-0001 as the
current V1 decision. At this baseline they still state:

```text
ADR-0001:78     "Core language remains not selected. POC-CORE-01 remains
                INCONCLUSIVE."
ADR-0001:149    "Product implementation remains unauthorized until W0-RS and
                IA-1 complete the remaining required gates."
ADR-0001:151    "Viewer library and Preview payload format are not selected."
ADR-0001:185    "Viewer technology, production Preview payload, Core language,
                database, and exact schemas remain OPEN or DEFER."
V1_SCOPE:15     implementation "remains unauthorized until W0-RS and IA-1"
V1_SCOPE:37     Preview "viewer library OPEN"
V1_SCOPE:79     "Core language selection | DEFER"
```

Every one of those items is closed: ADR-0002 selected Rust, ADR-0003 selected
SQLite/rusqlite bundled, ADR-0004 selected egui/eframe, ADR-0006 selected
`@google/model-viewer 4.3.1` with a GLB payload, and V1-1 through V1-8 are
baselined with `schema_version` 1 in the shipped Domain. All four ADRs are
`Accepted`.

Impact: a release reviewer reading the documents that `AGENTS.md` designates
as current authority is told the Core language, database, viewer, payload, and
schema are still open, and that implementation is not yet authorized. Two
mutually contradictory "current" contracts exist at a final release gate. No
code is affected and no capability is misrepresented in the direction of
overclaiming.

Required correction: reconcile the current product and architecture files with
the later Accepted ADRs, or mark the superseded passages as historical W0
rationale in place. Do not rewrite the historical Gate or IA-1 packages.

## GATE-D-MINOR-002 — V1_SCOPE release-hardening requirement for real non-humanoid E2E is unmet and undisposed

Severity: `MINOR`

Affected files:

```text
docs/product/V1_SCOPE.md (lines 153-157)
docs/development/V1_8_REAL_ASSET_MATRIX.md
docs/architecture/decisions/ADR-0001-... (open question 3)
```

`docs/product/V1_SCOPE.md` states: "Real non-humanoid E2E remains a release
hardening requirement, not the first architecture PoC." ADR-0001 open question
3 still lists real non-humanoid E2E as later mandatory validation. W0-RS
checkpoint 3 and IA-1 Observation 3 scoped that requirement as Mapping **plus**
Transfer, QC, persistence/reopen, and Preview.

At this baseline the requirement is unmet. The Horse row reaches
`Unsupported` with blocking unmapped target joints, Transfer is
`transfer_attempted: NO`, and
`blender-worker/tests/v1_8_real_assets.rs` never attempts Transfer for any
non-frozen row. V1-8 itself records `real non-humanoid retarget quality: NOT
YET PROVEN`.

This is not a false claim — the matrix and qualification documents state the
bound honestly, and the `Unsupported` verdict is correct fail-closed behavior
rather than a failure to try. It is an explicitly stated *release hardening*
requirement that is still open at the final release gate, and Gate D is the
audit at which it must be dispositioned.

Required correction: either satisfy the requirement with a non-humanoid pair
that can legitimately reach Ready, or amend `V1_SCOPE.md` and ADR-0001 to
state explicitly that real non-humanoid E2E is deferred past V1 with the
reasoning recorded. Do not leave a stated release-hardening requirement
silently unmet.

## GATE-D-MINOR-003 — Transfer publication is not bound into Preview Derived Variant selection

Severity: `MINOR`

Affected files:

```text
workbench/src/lib.rs
```

`apply_transfer_outcome` (`lib.rs:599-621`) records the published
`derived_variant_version_id` for display, and `reload_from_application`
(`lib.rs:575-592`) refreshes the asset lists, but neither sets
`selected_derived_variant_version`. The "Preview Derived Variant" control
(`lib.rs:1086-1097`) reads only `selected_derived_variant_version`.

Consequently, immediately after a successful Transfer the Preview Derived
control either does nothing (no Derived selected) or previews a *different*
Derived version that `WorkbenchApp::from_application` auto-selected
(`lib.rs:132-140`), while the status panel displays the newly published ID.
The user must find and click the new row manually.

Impact: the qualified sequence "Transfer → Published Derived Variant →
Preview" is not continuous on the native surface, and a stale Derived
selection can silently preview the wrong version. Preview is derived and
non-authoritative, so no Product truth is affected.

Required correction: bind the published Derived Variant version from the
Transfer outcome into the Preview selection, or make the Preview Derived
control operate on the displayed publication result.

## GATE-D-OBS-001 — Generic Catalog Draft replacement can perform the Mapping acceptance transition

Severity: `OBSERVATION`

Affected files:

```text
app/src/catalog.rs (may_replace, reject_public_authority_bypass)
app/src/mapping_workflow.rs (accept_mapping_version)
domain/src/mapping.rs (validate_review_history)
```

Reproduction (`REPRODUCED`,
`review_evidence/gate_d_mapping_authority_repro/`, output in
`review_evidence/gate_d_generic_path_probe.txt`):

1. Persist an automatic candidate `BoneMappingVersion`
   (`lifecycle = Draft`, `review_kind = automatic_candidate`,
   `generated_from_candidates = true`).
2. Serialize it, edit the JSON to `lifecycle = published`,
   `reviewed = true`, `review_kind = automatic_confirmed`, and re-enter it
   through the documented untrusted ingress `ingest_validated`.
3. `catalog_mut().put_validated(...)` accepts it, because
   `may_replace(Some("draft"), _)` is `true` for every record type.
4. Compatibility preflight then returns `ReadyWithWarnings` and Transfer is
   authorized.

Why this is an OBSERVATION and not a finding:

- In-place Draft replacement is intended, documented behavior. `Draft` is the
  mutable lifecycle, and `domain/tests/contract.rs::
  draft_may_be_replaced_in_place` asserts it.
- The forged record was accepted only because it was self-consistent with its
  own durable history. `BoneMappingVersion::validate_review_history` rejects a
  Published version that is not `reviewed`, that carries
  `automatic_candidate`, or whose `review_kind` differs from
  `derived_accepted_kind()` computed from `generated_from_candidates` and
  `user_modified`. That is exactly the guarantee
  `docs/development/V1_4_MAPPING_WORKFLOW.md` states, and it holds.
- Asserting `reviewed = true` is the same authority a caller already exercises
  by invoking `accept_mapping_version`, which itself writes
  `reviewed = true` with the reason "explicit Product acceptance". No Domain
  rule can verify human intent, and none claims to.
- Published, Ready, and Invalidated records are refused
  (`AppError::ImmutablePublished`), so this latitude cannot rewrite Product
  truth once frozen.

Residual difference worth recording: `accept_mapping_version` additionally
validates that the bound source and target `SkeletonSummary` subjects match
the mapping's `SourceSkeletonReference` and `CharacterAssetVersion`. The
generic path skips those checks. The decision-critical bindings
(`target_character_version_id`, `source_skeleton_ref_id`) are still
revalidated in `validate_compatibility_graph_on`, `validate_job_inputs`, and
`validate_publication_lineage`, so no publication can proceed on a
mismatched graph.

Required correction: none for Gate D. If the project wants the acceptance
transition to be workflow-exclusive, `reject_public_authority_bypass` could
refuse a generic `BoneMappingVersion` write that changes lifecycle from
`Draft` to `Published`. That is a hardening preference, not a contract defect.

## GATE-D-OBS-002 — Shipped worker crate exposes un-gated test seams that skip pin and package verification

Severity: `OBSERVATION`

Affected files:

```text
blender-worker/src/adapter.rs
```

Reproduction (source-confirmed):

1. `BlenderWorker::for_fake_executable(executable, workspace_root)` is a
   plain `pub fn` with no `#[cfg(test)]` and no feature gate. It sets
   `script = harness_worker_script()`, which is built from
   `env!("CARGO_MANIFEST_DIR")`, and sets `test = Some(TestHarness { .. })`.
2. `pin_verification_is_mandatory()` returns `self.test.is_none()`, so
   `dispatch_resolved` skips both `enforce_pin` and
   `verify_worker_package_integrity` for such an instance.
3. `set_test_behavior`, `enable_source_verification_for_test`,
   `skip_reopen_for_test`, and `set_reopen_executable_for_test` are likewise
   ungated `pub fn`s.
4. `workbench/Cargo.toml` depends on `rigforge_blender_worker` as a normal
   (non-dev) dependency, so the shipped Workbench links this surface.

Why this is an OBSERVATION:

No shipped code path constructs a harness worker.
`workbench/src/native_exec.rs::complete_native_transfer` uses
`BlenderWorker::production()` exclusively, which resolves the runtime root,
enforces the pin, and verifies the worker package. The reproduced tests
`adapter_fake.rs::production_worker_cannot_disable_pin_verification` and
`::production_worker_cannot_disable_source_verification` assert that a
production instance cannot be downgraded. The Gate D section 22 question —
"can a separate code path execute unverified production `worker.py`?" — is
answered no for every production entry point: Transfer, Preview generation,
QC inspect, and fresh reopen all route through
`verify_runtime_worker_package`.

Related sealing asymmetry (same class, same disposition): `rigforge_app`
seals its QC seams properly — `ArtifactInspector`, `PersistenceReopener`,
`evaluate_and_bind_qc_for_test`, and `finalize_transfer_for_test` are all
behind `cfg(any(test, feature = "test-support"))`, and the Workbench binary
does not enable that feature. Several non-QC seams are not sealed the same
way: `Application::generate_preview_with` / `regenerate_preview_with` with
`PreviewGeneratorPort` and `MemoryPreviewGenerator`,
`propose_and_store_mapping_for_selection` with `MemorySkeletonInspector`,
`Application::dispatch<W: WorkerPort>` with the public `FakeWorker`, and
`Application::complete_success`. None is reached from a Workbench control —
the UI uses `generate_preview` with `BlenderPreviewGenerator` and
`BlenderSkeletonInspector::production` — and none can publish, because
publication still requires the sealed pinned QC inspect and fresh reopen.
`Application::complete_success` can mark a run SUCCEEDED without the
`collect` envelope and execute-path scale audit, but it still enforces
JobSpec, attempt, `worker_execution_ref`, and `worker_success` correlation,
and publication remains gated.

Required correction: none for Gate D. Optional hygiene: gate the harness
constructor and setters behind a non-default `test-support` feature, matching
how `rigforge_app` isolates `test_graph` and its inspector seams, extend the
same sealing to the Preview, mapping-inspector, worker-port, and
`complete_success` seams, and add a source-surface assertion equivalent to
`app/tests/transfer.rs::test_memory_inspector_not_available_on_production_surface`.

## GATE-D-OBS-003 — Blender GPL redistribution remains an external human prerequisite

Severity: `OBSERVATION` / `EXTERNAL RELEASE PREREQUISITE`

Affected files:

```text
docs/development/V1_8_RELEASE_DEPENDENCIES.md
```

The repository records Blender GPL redistribution as
`HUMAN / LEGAL REVIEW REQUIRED`, with the distribution question and notice
requirement identified and no conclusion drawn. It also flags the bundled
SQLite amalgamation as a distribution question and states that a release-time
full transitive license sweep is still required.

Gate D verified the specific risk it was asked to test: the repository does
**not** claim legal clearance anywhere. Because no false claim exists, this is
not a technical finding. It does mean a Gate D technical PASS candidate would
not authorize public redistribution of third-party software.

Required correction: none technical. Human/legal decision before any public
distribution model is chosen.

## GATE-D-OBS-004 — Release bundle is defined and relocatable; no installer exists or is claimed

Severity: `OBSERVATION`

Affected files:

```text
docs/development/V1_8_RELEASE_DEPENDENCIES.md
docs/development/V1_8_RELEASE_QUALIFICATION.md
app/src/runtime.rs
```

The layout is defined, materializable
(`materialize_runtime_bundle`), relocatable, and internally consistent, and
the reproduced relocation tests ran real pinned Blender from a relocated root.
The documents label `polished installer: POST_V1` and
`third-party clean VM installer SKU: NOT TESTED`. This audit performed no
installer test and makes no installer claim. The labels are honest.

## GATE-D-OBS-005 — Real non-humanoid retarget quality is not proven

Severity: `OBSERVATION`

The reproduced campaign shows `Horse + UAL2` reaching
`Unsupported` with the note "blocking unmapped target joint(s) remain", and
Transfer was never attempted for it. That is architectural failure honesty,
not evidence of non-humanoid retarget capability.
`docs/development/V1_8_REAL_ASSET_MATRIX.md` and
`docs/development/V1_8_RELEASE_QUALIFICATION.md` bound the claim to
`PASS FOR TESTED V1 QUALIFICATION MATRIX / NOT A CLAIM OF UNIVERSAL ASSET
SUPPORT`, which matches the reproduced evidence.

## GATE-D-OBS-006 — Native Transfer waits on the UI thread during collect

Severity: `OBSERVATION`

Affected files:

```text
workbench/src/lib.rs (on_transfer_action)
workbench/src/native_exec.rs
```

`complete_native_transfer` runs `mark_dispatchable`, `dispatch`, `collect`,
and `finalize_transfer` synchronously inside the egui event handler, so the
window is unresponsive for the duration of one Blender attempt (roughly ten
seconds per attempt in the reproduced real runs). Assessment:
**operationally imperfect but still V1-usable**, not release-blocking. Launch
and collect remain separate durable operations, `RUNNING` is persisted before
collect, and a crash or restart during the wait fails closed rather than
publishing. This is an accepted limitation in
`docs/development/V1_8_RELEASE_QUALIFICATION.md` and Gate D agrees with that
classification.

## GATE-D-OBS-007 — Publication reopen parser does not independently require `root_scale_audit`

Severity: `OBSERVATION`

Affected files:

```text
app/src/pinned_qc.rs
blender-worker/src/envelope.rs
blender-worker/python/worker.py
```

Transfer collect deserializes the full `ReopenEnvelope`, in which
`root_scale_audit` is a required field (`envelope.rs:42-57`), and requires it
to be `PASS` before a successful outcome (`adapter.rs:785-804`). The
publication-critical reopen in `pinned_qc.rs:333-362` uses a narrower
`ReopenEnvelopeLite` carrying only `status`, `target_present`, and
`baked_action`, and derives both `fresh` and `structural` Pass without
consulting `root_scale_audit` at all.

Not currently reachable: the only production producer of that file is
`blender-worker/python/worker.py`, which sets
`"status": "SUCCESS" if action is not None and root_scale_audit == "PASS"`
(`worker.py:765`) and is pinned by SHA-256
`01105c47…89a5d`, verified before every reopen. A tampered script fails the
package check first. So an envelope claiming SUCCESS with a failed root scale
audit cannot be produced on the production path.

This is an independent-enforcement gap rather than a bypass: the
`KeepTargetRestScale` guarantee that closed `GATE-B-MAJOR-004` is enforced
twice on the Transfer collect path but only transitively — through the pinned
script's own gating — on the publication reopen path.

Required correction: none for Gate D. Optional hardening: have
`ReopenEnvelopeLite` require `root_scale_audit` and demand `PASS`, so the Rust
side enforces the policy independently of the script's internal gating.

## GATE-D-OBS-008 — Real-asset matrix specifics are not regression-locked

Severity: `OBSERVATION`

Affected files:

```text
blender-worker/tests/v1_8_real_assets.rs
.gitignore
```

The documented Horse outcome — 6 mapped, 59 unmapped, confirmation required,
`Unsupported`, Transfer denied — is asserted only as
`assert!(horse["outcome"].is_string())` and
`assert_ne!(horse["outcome"], "ASSET_DIGEST_MISMATCH")`
(`v1_8_real_assets.rs:338-340`). The non-Workbench path records
`MAPPING_COMPAT_COMPLETED` whenever preflight returns a value, including
`Unsupported`, so the summary itself is not locked. The frozen-pair assertion
accepts `Ready`, `ReadyWithWarnings`, or `MappingConfirmationRequired`
(`:330-336`), which is weaker than the documented `ReadyWithWarnings` and
weaker than the Gate C frozen-pair expectation. The written matrix artifact
lands in the gitignored `review_evidence/`.

This audit independently regenerated and read that artifact and observed the
documented values, so the *behavior* is confirmed at this baseline. What is
missing is a lock that would fail if the behavior drifted.

Required correction: none for Gate D. Optional hardening: assert the Horse
summary is `Unsupported` with Transfer denied and pin the frozen-pair summary
to `ReadyWithWarnings`.

## GATE-D-OBS-009 — Generic enqueue can execute a JobSpec with no Compatibility authorization

Severity: `OBSERVATION`

Affected files:

```text
app/src/application.rs, app/src/catalog.rs, domain/src/execution.rs
```

`JobSpec::new` leaves `compatibility_result_id` as `None`, and
`validate_job_graph_on` applies Transfer eligibility checks only when that
field is `Some`. Public `Application::enqueue_job` therefore persists a
JobSpec and JobRun with no Compatibility authorization, and that run can be
marked dispatchable and executed by a real worker.

This is the documented historical-execution allowance rather than a defect:
`GATE_C_PRODUCT_CORE_AUDIT.md` records that "historical JobSpecs may omit V1-5
fields for historical execution, but V1-5 candidate ingestion and publication
fail closed when target or compatibility authorization is missing", and
`publish_derived_on` does require the JobSpec's `compatibility_result_id`.
The Workbench never uses this route — `request_transfer` always goes through
`start_transfer`, which binds the exact authorizing result.

Required correction: none. Recorded so a future reader does not mistake
worker execution for Transfer authorization.

## GATE-D-OBS-010 — Residual documentation drift around release qualification

Severity: `OBSERVATION`

Independent of `GATE-D-MINOR-001`, these current documents lag the V1-8
outcome. None overclaims capability; each could mislead a release reviewer:

```text
ROADMAP.md (deferred Gate B list)   still defers spawn-to-RUNNING crash
                                    recovery and release worker-script
                                    integrity to V1-8, which implemented both
NOTICE.txt / V1_6_VIEWER_CONTRACT   still say "legal release clearance
                                    remains V1-8"; V1-8 inventoried rather
                                    than cleared, so clearance is still open
V1_3_PROCESS_LIFECYCLE / V1_5_QC    still say worker execute/collect may
                                    honor RIGFORGE_BLENDER_EXECUTABLE; the
                                    V1-8 source has no production reader
V1_8_IMPLEMENTATION_PLAN            records stage baseline 9002a2d…, not the
                                    closeout SHA ecbbb28d…, unlike the way
                                    Gate B and Gate C recorded audit HEADs
docs/development/audits/            contains no independent Gate A package;
                                    Gate A PASS is asserted only in the V1-1
                                    plan and ADR-0002 narrative
```

Required correction: none blocking. Fold into whatever documentation pass
addresses `GATE-D-MINOR-001`.

## GATE-D-OBS-011 — Native flow gaps that are not defects

Severity: `OBSERVATION`

- No RetargetPolicy picker exists in `draw()`; `select_policy_version` is an
  API-level operation. With two or more Published policies the flow fails
  closed with "policy selection required" rather than picking one, which is
  the correct contract behavior, but a GUI-only user cannot then proceed.
- "Regenerate Preview" prefers Derived, then Character, then Motion, so with
  any Derived selected the Character and Motion regenerate paths are
  unreachable from that one control. The dedicated Preview buttons still work.
- `override_mapping_entry` is wired at the `WorkbenchApp` level but has no
  rendered control, so manual per-joint override is not reachable from the
  GUI.
- `PreviewHost::shutdown` joins the listener thread on the UI thread, so a
  Preview replacement can block for up to the in-flight request timeout. The
  host count stays bounded; only latency is unbounded.

Required correction: none for Gate D. The policy-picker item becomes relevant
if a V1 user is ever expected to hold more than one Published policy.

## Historical finding closure re-check

Gate D did not modify, delete, or re-severity any historical finding. It
re-checked whether their accepted closure semantics remain true at this
baseline.

| Historical finding | Original severity | Closure semantics at `ecbbb28d` |
| --- | --- | --- |
| `GATE-B-MAJOR-001` WorkerResult not bound to its attempt | MAJOR | remains closed; attempt/execution correlation enforced and reproduced |
| `GATE-B-MAJOR-002` rejected failure could leave RUNNING | MAJOR | remains closed; terminal failure is durable, restart reconciliation fails closed |
| `GATE-B-MAJOR-003` rational frame provenance lost at dispatch | MAJOR | remains closed; rational `TimePoint` preserved, fractional fails before launch |
| `GATE-B-MAJOR-004` root execution could violate KeepTargetRestScale | MAJOR | remains closed; reproduced with real Blender in `root_scale.rs` |
| `GATE-B-MINOR-001` completed attempts retained in adapter memory | MINOR | closed; `terminal_collect_removes_attempt_bookkeeping` reproduced |
| `GATE-B-MINOR-002` Unicode stderr could panic collection | MINOR | closed; `unicode_stderr_tail_does_not_panic` and `unicode_stderr_collect_does_not_panic` reproduced |
| `GATE-B-OBS-001` spawn-to-RUNNING crash window | OBSERVATION | addressed by V1-8 dispatch intent; fails closed on reopen |
| `GATE-B-OBS-002` worker script bound to the development tree | OBSERVATION | addressed; production resolves from the relocatable runtime root with pinned digests |
| `GATE-B-OBS-003` packaged test breakdown miscounted app tests | OBSERVATION | evidence-only; this audit records exact per-target totals |
| `GATE-C-OBS-001` native Workbench was an unwired shell | OBSERVATION | addressed; `WorkbenchHost` retains the Application and every rendered control calls an Application-backed handler. Verified in current source, not taken on trust |

`GATE-C-OBS-001` remains historical evidence. The Gate C independent result
remains `GATE_C_PASS_CANDIDATE` and project acceptance remains
`Gate C PASS / CLOSED`. Nothing in the Gate C reports was rewritten.

## Required future flow

```text
these same Gate D findings
        |
Main Agent surgical correction of GATE-D-MAJOR-001
        |
focused closure of the SAME Gate D
```

Do not open Gate D2. This initial Gate D report is immutable.
