# POC-PREVIEW-01R research harness

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

Question: can RigForge provide useful click-to-preview for Character, Motion,
and Derived Variant assets using rebuildable Preview Artifacts and an
engine-independent viewer, without a full RigForge animation runtime?

This directory is not a production viewer, not an Asset Browser, and not a
Core language / renderer / GLB selection.

```text
POC-PREVIEW-01R: COMPLETE / PASS / BASELINED
Decision Impact: ACCEPT_DERIVED_PREVIEW_PATH_WITH_GUARDS
viewer library: RESEARCH HARNESS ONLY / NOT SELECTED
payload candidate: GLB (not a permanent format selection)
```

Independent Product truth: `config/product_fixture.json` (not generated from
Preview Manifest). Catalog expected binding is copied from that fixture.

Viewer harness: `@google/model-viewer` **4.3.1** (Apache-2.0), bytes kept
outside git.

```text
python experiments/w0p/poc_preview_01r/scripts/generate_previews.py
python experiments/w0p/poc_preview_01r/scripts/preview_selftest.py
```
