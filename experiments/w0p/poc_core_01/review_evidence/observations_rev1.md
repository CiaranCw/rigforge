# POC-CORE-01 Rev1 observations

RESEARCH ONLY. Classification: OBSERVED unless marked RIGFORGE_INFERENCE.

See also `F:\NewResearch\rigforge_w0p_evidence\poc_core_01\observations_rev1.md`.

## Primary raw-C (retained)

Same ABI, ctypes, worker JSON, stdin/stdout, prebuilt ufbx.lib v0.23.0,
same Domain algorithm. C++ includes `ufbx.h`. Rust raw-C uses
`ufbx_access.c` (~198 physical LOC with header).

Functional after Rev1: C++ PASS, Rust raw-C PASS (minimal_chain +
long_name + worker). Worker stdout JSON bytes identical across C++, Rust
raw-C, and 01S.

## Official binding status

ufbx maintains an official Rust binding (`ufbx/ufbx-rust`). ufbx v0.23.0
changelog pairs that tag with `ufbx-rust 0.11.2`. [OFFICIAL_DOC]
This is not characterized as historical/stale.

## POC-CORE-01S

Pin: crates.io `ufbx =0.11.2` (not 0.11.3). Bundled C is ufbx 0.23.0.
`cc` compiles `ufbx/ufbx.c`. Consumer libclang/bindgen not required.
RigForge handwritten C shim: 0. Functional PASS on the same assertions.

Build milliseconds and DLL size versus the raw-C run are different
integration strategies and are not used as a language contest.

## Semantic equivalence

Original Rust `[u8; 1024]` truncated names >1023 bytes. Two-pass length
query restores C++ `node->name.length` behavior. Empty → `unnamed`
preserved. long_name.fbx: 1105-byte name; all three candidates PASS.

## ABI unwind

Every C ABI export: C++ `try/catch (...)`; Rust `catch_unwind`.
PoC-only. Residual: invalid pointers remain UB.

## Handle registry

Pointer-address live set is a research harness. Immediate post-destroy,
single-threaded only. Not a production stale-handle / ABA / concurrent
design. Shared; not scored.

## Lifetime wording

No differentiating C++ ownership/lifetime failure was observed in the
single-threaded tested slice. That is not formal memory-safety validation.

## Decision

Primary raw-C favors C++ for raw-C integration.
01S removes the material handwritten-FFI tax for ufbx.
Final recorded outcome: INCONCLUSIVE.
