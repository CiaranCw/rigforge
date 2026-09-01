# IA-1 — Independent Audit Findings

**Audit status:** `AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING`

**Verdict:** `PASS`

**Sentinel:** `IA1_AUDIT_PASS_CANDIDATE`

**Baseline:** `main` at
`7a13605952fe85021243928c5d3f0097c86d86e3`

This is the authoritative IA-1 finding inventory. Candidate `PASS` means
external review and closeout may proceed. It does not close IA-1, complete W0,
authorize implementation, select technology, or modify accepted lifecycle
files.

## Counts

```text
OPEN MAJOR: 0
OPEN MINOR: 1
OBSERVATIONS: 6
```

## Open major findings

None.

## Open minor findings

### IA1-MINOR-001

**ID:** IA1-MINOR-001

**Severity:** MINOR

**Title:** Current PoC index retains the pre-synthesis next step

**Affected current claim:** W0-RS traceability classifies
`docs/research/poc/README.md` as a current PoC lifecycle index, but that index
still says W0-RS is next and `READY / NOT STARTED`.

**Evidence:** Current canonical lifecycle sources (`README.md`, `AGENTS.md`,
`docs/development/ROADMAP.md`, and
`docs/research/R1_RESEARCH_BASELINE.md`) record W0-RS as
`COMPLETE / PASS / BASELINED` and IA-1 as `READY / NOT STARTED` at the audit
baseline. In contrast, `docs/research/poc/README.md` says “Next is W0-RS” in
its active-lifecycle narrative, records `W0-RS: READY / NOT STARTED` in the
POC-PREVIEW-01R block, and again directs readers to W0-RS. The immutable
historical lifecycle block in `POC_BLENDER_E2E_01.md` is correctly treated as
historical and is not part of this finding.

**Repository paths / headings:** `docs/research/poc/README.md` — “Active
lifecycle” and “POC-PREVIEW-01R (COMPLETE / PASS / BASELINED)”;
`docs/research/decisions/W0_RS_TRACEABILITY.md` — “2. Current canonical
truth”; `README.md` — “Current Status”; `docs/development/ROADMAP.md` —
“Current lifecycle”.

**Why this is material:** A reviewer following the traceability-designated
current PoC index can be routed backward to an already completed gate and can
misread whether IA-1 was ready. Canonical lifecycle authority is otherwise
clear, so this is a traceability defect rather than an architecture or gate
failure.

**Required correction:** After IA-1 external review determines closeout,
update the current PoC index to the accepted W0-RS and IA-1 lifecycle state.
Do not rewrite historical PoC report lifecycle snapshots.

**Decision impact:** None on Product scope, Blender acceptance, Preview
acceptance, Core-language status, or implementation authorization.

**Can IA-1 pass with finding open?:** YES

## Observations

### Observation 1 — Backend replacement remains an inference

The tested durable transfer concepts avoid `bpy` objects and Blender operator
names, but only the Blender worker was implemented. A second worker consuming
the contract was not demonstrated. This is correctly disclosed and is not a
W0 direction defect.

### Observation 2 — Concrete job instances may name backend and artifact capabilities

The E2E research Job instance pins Blender and requests `blend_save`,
`glb_export`, and `workbench_still`. Those are execution-selection and
artifact capabilities in a PoC instance, not Product identity or schema
authority. Production contract design must keep the schema backend-neutral
and make backend-specific capability selection explicit.

### Observation 3 — Non-humanoid E2E is a release gate, not accepted evidence

Wolf establishes non-humanoid FBX inspection only. Real non-humanoid Mapping,
Transfer, persistence/reopen, QC, and Preview are not validated. Deferral to
V1-4/V1-8 is legitimate if the release scope and hardening gate remain
fail-closed.

### Observation 4 — Exact E2E-to-Preview persistence bytes are not established

The E2E report records
`882e2c0a9e3fec121abdabb3807eec4bc12bf9283776532f70f12fb4bc8762eb`;
the Preview fixture records
`fef863e208b51e2fcf70aa278ff9041e39b93c1e9f479940a8db6670d970a532`.
W0-RS correctly retains this as `NOT ESTABLISHED` and distinguishes
`DerivedVariantVersion` from representation bytes.

### Observation 5 — Packaging and legal clearance remain release blockers

The repository makes no new legal conclusion. Blender packaging,
redistribution, notices, worker-script obligations, supported-platform
delivery, upgrade, and rollback remain open and require engineering evidence
plus legal review before distribution.

### Observation 6 — Production technology and schemas remain intentionally open

Core language, GUI, database, production schemas, viewer, Preview payload,
native format stacks, Auto-Mapping method, and advanced QC remain unresolved
at explicitly assigned implementation stages. Research harness languages,
GLB, `model-viewer`, and ufbx do not select them.

## Closure rule

IA-1 may be closed only by external acceptance of this report and evidence
package. Until then:

```text
W0: ACTIVE
W0-RS: COMPLETE / PASS / BASELINED
IA-1: AUDIT_COMPLETE / EXTERNAL_REVIEW_PENDING
product implementation: NOT STARTED
```
