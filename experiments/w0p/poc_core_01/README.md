# POC-CORE-01 research harness

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

This directory is not production Core, not W1 Canonical, and not a public ABI.

It implements one equivalent vertical slice in C++ and Rust so POC-CORE-01 can
compare languages with the independent variable held to implementation language.

## Layout

```text
experiments/w0p/poc_core_01/
├── README.md
├── ALGORITHM.md
├── abi/rigforge_poc_core.h
├── cpp/
├── rust/          primary raw-C candidate
├── rust_s/        POC-CORE-01S official ufbx-binding sensitivity
├── python/run_poc.py
├── worker/protocol.md
└── fixture/
    ├── minimal_chain.fbx
    └── long_name.fbx
```

Downloaded ufbx sources, compiler caches, binaries, and logs belong in:

```text
F:\NewResearch\rigforge_w0p_work\poc_core_01\
```

Do not vendor ufbx into this repository. Do not commit build outputs.

## Held constant

See [ALGORITHM.md](ALGORITHM.md) and [worker/protocol.md](worker/protocol.md).

- Same C ABI header
- Same ctypes script (`python/run_poc.py`)
- Same worker JSON + stdin/stdout
- Same pinned ufbx static library
- Same fixtures (`minimal_chain.fbx` primary; `long_name.fbx` string-boundary)
- Equivalent RelWithDebInfo / release-with-debug (`opt-level=2`, `debug=1`)

POC-CORE-01S (`rust_s/`) is a **SENSITIVITY ANALYSIS**. It is not the primary
language-only controlled run. It pins crates.io `ufbx =0.11.2` and must not
be compared to the raw-C run as if packaging were identical.

## Rebuild (Windows x64, this experiment's pins)

Tested environment (this recorded run; not a universal minimum):

- MSVC 19.36.32548.0, toolset 14.36.32532
- Visual Studio 2022 Professional 17.13.7
- CMake 4.4.0
- Isolated Rust 1.98.0 (`RUSTUP_HOME` / `CARGO_HOME` under the work tree)
- ufbx **v0.23.0** extracted under the work tree
- Python 3.13 (stdlib ctypes only)

From a VS x64 Developer Prompt, after `ufbx.lib` and `ufbx_access.lib` exist
in `UFBX_LIB_DIR`:

```text
cmake -S experiments/w0p/poc_core_01/cpp -B <work>/build/cpp -G "NMake Makefiles" ^
  -DCMAKE_BUILD_TYPE=RelWithDebInfo -DUFBX_ROOT=<ufbx-0.23.0> -DUFBX_LIB_DIR=<libdir>
cmake --build <work>/build/cpp --config RelWithDebInfo

set CARGO_TARGET_DIR=<work>/build/rust
set UFBX_LIB_DIR=<libdir>
cargo build --release --manifest-path experiments/w0p/poc_core_01/rust/Cargo.toml

set CARGO_TARGET_DIR=<work>/build/rust_s
cargo build --release --manifest-path experiments/w0p/poc_core_01/rust_s/Cargo.toml
```

Run (same script twice):

```text
python experiments/w0p/poc_core_01/python/run_poc.py --lib <cpp.dll> --worker <cpp_worker.exe> --fixture experiments/w0p/poc_core_01/fixture/minimal_chain.fbx --long-name-fixture experiments/w0p/poc_core_01/fixture/long_name.fbx
python experiments/w0p/poc_core_01/python/run_poc.py --lib <rs.dll> --worker <rs_worker.exe> --fixture experiments/w0p/poc_core_01/fixture/minimal_chain.fbx --long-name-fixture experiments/w0p/poc_core_01/fixture/long_name.fbx
python experiments/w0p/poc_core_01/python/run_poc.py --lib <rs_s.dll> --worker <rs_s_worker.exe> --fixture experiments/w0p/poc_core_01/fixture/minimal_chain.fbx --long-name-fixture experiments/w0p/poc_core_01/fixture/long_name.fbx
```

Expected stdout from each invocation:

```text
PASS
```
