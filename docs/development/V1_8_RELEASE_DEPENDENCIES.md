# V1-8 Release Dependencies

Runtime/release dependency and license inventory. Not a legal opinion.

Status: `COMPLETE / PASS / BASELINED`

Classify:

```text
license identified
notice requirement identified
distribution question identified
needs human/legal review
```

Do not treat this file as counsel. Where interpretation is required:

```text
HUMAN / LEGAL REVIEW REQUIRED
```

## Product / first-party

| Component | Relationship | License note |
| --- | --- | --- |
| RigForge Domain / App / Workbench / blender-worker Rust crates | shipped source | `UNLICENSED` in crate manifests; project-owned |
| `blender-worker/python/worker.py` | execution script | project-owned; integrity SHA-256 pinned |
| `blender-worker/python/preview_gen.py` | Preview generation-time only | project-owned; integrity SHA-256 pinned |
| Workbench `preview-viewer/` HTML/JS/CSS | Preview surface, not Product authority | project-owned except vendored model-viewer |

Worker script digest is a **package integrity** check. It is not Product
identity. Transfer, Preview, QC inspect, and fresh reopen share
`app/src/runtime.rs::verify_runtime_worker_package`.

## Rust / SQLite

Workspace uses Cargo. Representative direct dependencies:

| Crate | Role | License (from crate metadata; license identified) |
| --- | --- | --- |
| `rusqlite` 0.32.1 bundled | Catalog | MIT/Apache-2.0 typical; bundled SQLite amalgamation is a **distribution question** |
| `serde` / `serde_json` | JSON records | MIT OR Apache-2.0 |
| `uuid` | IDs | MIT OR Apache-2.0 |
| `sha2` | digests | MIT OR Apache-2.0 |
| `eframe` / `egui` 0.31.1 | Workbench GUI | MIT OR Apache-2.0 |

Exact transitive crate licenses must be taken from `cargo metadata` at
release time. This file does not freeze a complete legal SBOM.

SQLite amalgamation via `rusqlite` `bundled`: license identified as the
SQLite blessing / public-domain-style grant **plus** the rusqlite crate
license. **distribution question identified** for the combined binary.

## Blender runtime

```text
version: 5.2.1 LTS
build:   9e2066aef7ef
archive SHA-256:
0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c
```

Blender is a pinned isolated execution dependency, not Product authority
(ADR-0001, ADR-0005).

GPL and redistribution of Blender binaries with or beside RigForge:

```text
HUMAN / LEGAL REVIEW REQUIRED
distribution question identified
notice requirement identified
```

V1-8 does not conclude whether a given installer/bundle form is
GPL-compliant. Missing or wrong Blender/build must fail diagnostically
without falling back to arbitrary user Blender.

## Preview viewer

```text
@google/model-viewer 4.3.1
vendored: workbench/preview-viewer/vendor/model-viewer.min.js
local SHA-256:
283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b
no runtime CDN
```

Package license: Apache-2.0 [official npm]. The minified bundle also
contains Lit BSD-3-Clause headers. Notices: `workbench/preview-viewer/NOTICE.txt`.

```text
license identified
notice requirement identified
```

Decoder CDN defaults are redirected locally so view time does not fetch
Draco/KTX2/Lottie. Browser/OS WebView remains an implementation detail, not
Product authority. Local browser launch failure is a Preview display
failure, not Product corruption.

## Browser / OS runtime assumptions

- Windows local sidecar: `cmd /C start` of `http://127.0.0.1:<port>/`
- No claim of all browsers, all GPUs, or all Windows machines
- `node_modules` is not a runtime requirement; vendor file is committed

## Runtime resource layout

Production Transfer, Preview generation, QC inspect/reopen, worker scripts,
and the Preview viewer resolve from one backend-neutral runtime root.

Resolution order:

1. Thread-local test bind (`bind_thread_runtime_root`), if present
2. Environment `RIGFORGE_RUNTIME_ROOT`, if non-empty
3. Directory containing the current executable

Production resolution does **not** use:

```text
CARGO_MANIFEST_DIR
the original source checkout
F:\NewResearch\... hard-coded developer trees
RIGFORGE_BLENDER_EXECUTABLE as an arbitrary unpinned fallback
```

Defined release layout (relocatable; not a polished installer):

```text
<RigForge runtime root>/
    rigforge_workbench.exe
    blender-worker/
        worker.py
        preview_gen.py
    preview-viewer/
        index.html
        app.js
        app.css
        vendor/model-viewer.min.js
    runtime/
        blender-5.2.1-windows-x64/
            blender.exe
            ...
```

Local V1-8 development may set `RIGFORGE_RUNTIME_ROOT` once to a directory
in that layout. The existing accepted Blender 5.2.1 LTS tree may be
junctioned or copied into `runtime/blender-5.2.1-windows-x64/`. Runtime
location is replaceable. Pinned content is still verified:

```text
Blender 5.2.1 LTS build 9e2066aef7ef
worker.py SHA-256
preview_gen.py SHA-256
model-viewer.min.js SHA-256
```

Product identity remains independent of these paths and digests.

Qualification bound:

```text
runtime resource relocation: TESTED
release bundle layout: DEFINED
polished installer: POST_V1
third-party clean VM installer SKU: NOT TESTED
```

```text
release bundle / reproducible package
!=
final polished installer
```

Gate D needs to know what files are shipped, external runtimes, integrity
checks, and clean-machine expectations. A polished installer is **POST_V1**
unless later evidence says otherwise.

Minimum clean-machine expectation (project claim, not a tested SKU):

1. RigForge Workbench binary + Catalog schema
2. Runtime root containing pinned Blender 5.2.1 LTS under
   `runtime/blender-5.2.1-windows-x64/`
3. `blender-worker/worker.py` and `preview_gen.py` matching pinned SHA-256
4. Vendored model-viewer matching pinned SHA-256
5. No Internet required after those are present
6. No dependence on `target/`, developer `node_modules`, user Blender add-ons,
   compile-time crate directories, or untracked scripts as Product identity

## Integrity pins (package, not Product IDs)

```text
worker.py SHA-256:
01105c47d6ecfa1cc77407d996670b7afb455ce2cb7a45afc3399ccae2089a5d

preview_gen.py SHA-256:
c84aada5a3785dfc5a6c7114dce270616c1fe0640a4fa390dda39d4b8b3db795

model-viewer.min.js SHA-256:
283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b
```
