# POC-CORE-01 extraction algorithm

RESEARCH ONLY / W0-P / NON-PRODUCTION. POC ONLY — NOT CANONICAL.

Both implementations must follow this procedure. The independent variable is
the implementation language, not this procedure.

## ufbx load

1. If `path` is NULL, fail with `RF_POC_ERR_NULL`.
2. Zero-initialize `ufbx_load_opts`.
3. Call `ufbx_load_file(path, &opts, &error)`.
4. If the scene is NULL:
   - `UFBX_ERROR_FILE_NOT_FOUND` (or equivalent IO) → `RF_POC_ERR_IO`
   - otherwise → `RF_POC_ERR_PARSE`
5. Walk `scene->nodes` in list order.
6. Skip nodes with `is_root == true` (ufbx implicit scene root).
7. Remaining nodes become research joints in that order.
8. Parent index: if `parent` is NULL or `parent->is_root`, parent is `-1`;
   otherwise the parent is the index of that node pointer/raw-index in the
   joint list.
9. Rest local TRS is copied from `node->local_transform`.
10. Joint names are copied from `node->name` using the **exact** byte length
    (`node->name.length`). Empty names become `unnamed`. Implementations must
    not truncate to a fixed buffer (for example 1023 bytes).
11. Free the ufbx scene before returning. Domain data is owned copies.

## Synthetic research motion (not FBX animation)

Native FBX animation is **not** consumed. That is POC-FBX-01.

Attach one clip:

- name: `poc_walk`
- time range: `[0.0, 1.0]`
- one track per joint, `track[i].joint == i`
- translation keys: `{t=0, rest_t}`, `{t=1, rest_t + (0.1 * i, 0, 0)}`
- rotation keys: for joints `0 .. n-2`, `{t=0, rest_r}`, `{t=1, rest_r}`;
  for the last joint, **empty** rotation key list

## Diagnostics (always on successful load)

| severity | code | message | location |
| --- | --- | --- | --- |
| INFO | `POC_LOAD` | `ufbx load ok` | path |
| INFO | `POC_JOINTS` | `{n} joints extracted` | empty |
| WARN | `POC_SYNTH` | `native FBX animation unused; synthetic research motion attached` | empty |
| INFO | `POC_EMPTY_ROT` | `last joint rotation keys empty by design` | last joint name |

Asset id string: `poc_core_01_asset`

## C ABI unwind boundary (PoC-only)

Every exported C ABI function must have a defined no-exception / no-panic
boundary. This is a research harness rule, not a production exception policy.

- C++: top-level `try/catch (...)` maps unexpected exceptions to
  `RF_POC_ERR_PARSE` (status), `-1` (count), or a swallowed destroy.
- Rust: `catch_unwind` maps panic to the same ABI failure forms.
- Do not treat process-abort-on-panic as a clean ABI error contract.

Residual: invalid pointers remain undefined behavior. `catch_unwind` /
`catch (...)` do not make FFI memory-safe.

## Handle registry limitation (shared, not scored)

The pointer-address live registry is a research harness mechanism.
It validates immediate post-destroy misuse on the single-threaded tested
path. It is **not** a production stale-handle / ABA / concurrent
query-destroy design. This limitation is shared by both language
candidates and by POC-CORE-01S.

## POC-CORE-01S (sensitivity only)

POC-CORE-01S is **not** a new Mandatory W0-P PoC and **not** part of the
primary language-only controlled comparison.

It reuses this Domain algorithm, fixtures, C ABI, ctypes caller, assertions,
and worker protocol. The changed variable is the Rust native-library
integration strategy: official crates.io `ufbx` `=0.11.2` (bundled ufbx
v0.23.0 C source via `cc`) instead of the handwritten raw-C shim.

Load equivalent: `unsafe { ufbx::load_file_raw(path, &ufbx::RawLoadOpts::default()) }`.
Parent identity uses `element.element_id`. Joint names use the binding's
exact `name.length` (no fixed cap).
