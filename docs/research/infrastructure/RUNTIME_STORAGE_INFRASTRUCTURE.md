# Runtime and Storage Infrastructure — W0.3

Access date: **2026-08-30**.

Two **separate** concerns. Neither defines Canonical Character / Skeleton / Motion.

```text
Asset Identity / Resolve / Publish
        ≠
Runtime Cook / Compression
```

---

## A. Asset identity / resolve / publish

### OpenAssetIO

Official: OpenAssetIO is a **bridge** between a **host** and an **asset management system**. It is **not** a database, asset manager, file format, pipeline framework, storage system, or replacement for tracking tools [OFFICIAL_DOC] https://docs.openassetio.org/OpenAssetIO/ and https://github.com/OpenAssetIO/OpenAssetIO

| Concept | Official meaning | RigForge use |
| --- | --- | --- |
| Entity reference | Opaque URI owned by the manager | Optional identity string; not a joint id |
| Traits / specifications | Additive property groups for resolve/publish | Must **not** become Canonical schema |
| Resolve | Reference → trait data (URLs, etc.) | Storage adapter |
| Publish | Register produced data | Publish adapter |
| Relationships | Related entities | Optional |
| hostApi / managerApi | Two sides of the bridge | Optional integration |
| Thread/reentrancy | ManagerInterface is thread-safe/reentrant [OFFICIAL_DOC] | Infra constraint |
| C++ / Python | Official language surfaces | W0.4 |

**Mandatory V1 runtime dependency?** Evidence says **no** — matches V1_SCOPE “OpenAssetIO mandatory runtime dependency” as non-core. Useful as **architectural reference** and **optional future adapter**.

**Must influence:** location independence; versioned identity; resolve-before-use; publish as an explicit step; the idea that **resolution context affecting output must be explicit provenance/input**.

OpenAssetIO `Context` / Manager State is official evidence that reproducible resolution may require explicit contextual state [OFFICIAL_DOC] https://docs.openassetio.org/OpenAssetIO/classopenassetio_1_1v1_1_1_context.html — **do not** copy that type into RigForge Core (AD-DET-001).

**Must not influence:** joint semantics, retarget math, Preview renderer, Canonical fields.

### AYON

Official glossary: Folder → **Product** → **Version** → **Representation** (on-disk extract); publish is export to an immutable versioned file [OFFICIAL_DOC] https://help.ayon.app/articles/3030530-ayon-glossary

Developer model: Create / Collect / Validate / Extract / Integrate (pyblish plugins) [OFFICIAL_DOC] ayon-documentation `dev_addon_intro.md`

**Classification:** reference architecture + optional pipeline adapter. **Not** Canonical taxonomy. Do not import AYON product-type names into Core.

**Must influence:** versioned publish; representation ≠ product; path templates as **storage policy**.

**Must not influence:** Skeleton/Motion schema.

---

## B. Runtime cook / compression

Accepted (architecture + V1_SCOPE): ozz is **not** Canonical. ACL is **not** Canonical.

```text
Canonical Motion
        ↓
Cook policy (versioned, derived)
        ↓
runtime-specific clip / skeleton
```

Derived data must remain derived, record provenance + cook version/policy, and be **rebuildable**.

### Generic Runtime Cook Adapter (requirements, not API)

Capability areas a contract should be able to declare: sampling; track reduction; compression; quantization; runtime skeleton representation; streaming; random access; decompression; error metric; platform profile.

### ozz-animation

[OFFICIAL_DOC] https://github.com/guillaumeblanc/ozz-animation README (master, access 2026-08-30)

| Fact | Value |
| --- | --- |
| What it solves | Runtime load/sample/blend; offline convert from DCC formats to **ozz structures** |
| What it does not | Canonical authoring; portable additive layers; format-faithful FBX recipe |
| Runtime deps | C++17 + stdlib; **no OS-specific** runtime; renderer/engine agnostic |
| Offline tools | May depend on tinygltf, **FBX SDK**, etc. — **not** needed to ship runtime |
| Platforms | Linux, macOS, Windows, **WebAssembly** CI |
| License | MIT |
| Semantic loss | Bake/sample to ozz tracks; DCC richness discarded |

**WASM:** official CI column. Useful for **derived** browser playback experiments in W0-P — not Preview Canonical.

### Animation Compression Library (ACL)

[OFFICIAL_DOC] https://github.com/nfrechette/acl README (`develop`, access 2026-08-30)

| Fact | Value |
| --- | --- |
| What it solves | Compress/decompress animation tracks; error-aware quality |
| What it does not | Authoring schema; skeleton mapping; retarget |
| Integration | **Header-only** C++11; game-provided allocator |
| Input | Intermediate clip format (documented) — **cook input**, not Canonical |
| Platforms | CI includes Windows/Linux/macOS + **Emscripten WASM** |
| License | MIT |
| Engine note | Informational only — **not** a Core reason to adopt ACL. Official plugin repo: “As of Unreal Engine **5.3**, this plugin comes out of the box and is distributed and maintained by Epic in their fork” [SOURCE_CONFIRMED] https://github.com/nfrechette/acl-ue4-plugin (access 2026-08-30). Author blog same claim [PROJECT_CLAIM] https://nfrechette.github.io/2023/09/17/acl_in_ue/. ACL **main** README (`develop`) wording “since UE 5.13” is a **[PROJECT_CLAIM]** inconsistent with the plugin repo; treat as an **apparent upstream documentation typo**. Do not propagate `5.13` as fact. Epic 5.3 release-note text was not independently retrieved this pass (docs host 403). Later-tree presence of `ACLPlugin` was not independently inspected. |

Compression/cook ≠ Canonical Motion representation. Do not derive Canonical fields from ACL clip layout.

---

## Other discoveries (not deep-dived)

| Candidate | Class | Note |
| --- | --- | --- |
| Khronos glTF Validator | `USEFUL_REFERENCE` | Validator ≠ parser |
| meshoptimizer / gltfpack | `DEFER W0.4` | glTF mesh/anim optimize |
| Draco | `DEFER W0.4` | Required-ext geometry cook |
| OpenUSD Hydra | `OUT_OF_SCOPE` as Preview engine | Rendering hydra ≠ Domain |

---

## Placement

| System | Layer |
| --- | --- |
| OpenAssetIO | Optional storage/identity adapter; reference |
| AYON | Optional pipeline adapter; reference |
| ozz | Optional Runtime Cook backend |
| ACL | Optional compression backend |
| Unreal/Unity cookers | Host-side derived only |
