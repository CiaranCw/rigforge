# V1-6 Viewer Contract

Engine-independent Preview display. Implementation detail, not Product
authority.

Status: `COMPLETE / PASS / BASELINED`

ADR: [ADR-0006](../architecture/decisions/ADR-0006-v1-engine-independent-preview.md)
(`Accepted` at V1-6 focused review / focused closure).

## Selection

```text
viewer library:  @google/model-viewer
version:         4.3.1
surface:         Workbench sidecar local HTTP + OS browser window
Preview payload: model/gltf-binary (GLB)
offline runtime: YES (vendored min.js; no runtime CDN)
```

Official npm integrity:

```text
sha512-GP+inXhAtY31E8rILVmByA6z8CZZjdlNajddppyI1/j1eIaSQiZcMRaUqTFe7+jv4mzRzwKIOiKBud0apiv+WQ==
```

Local SHA-256 of `workbench/preview-viewer/vendor/model-viewer.min.js`:

```text
283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b
```

License: package Apache-2.0; the minified bundle also contains Lit
BSD-3-Clause headers. Recorded in
[NOTICE.txt](../../workbench/preview-viewer/NOTICE.txt). Legal release
clearance is V1-8.

This is **not** inferred from ADR-0004 (egui/eframe). The GUI crate is not
the viewer. `wry` is not selected. `WorkbenchApp::requires_blender()` stays
`false`. View time does not require Unreal, Unity, Blender UI, or Maya.

## Integrity before display

The viewer fetches `/session.json` first.

- If `valid !== true` or `payload_filename` is absent: no payload fetch as
  a successful display; explicit invalid / blocked UI.
- If valid: fetch payload, re-check size and SHA-256, then create a blob
  URL. Product truth has already been checked by Application.

Wrong Product version, stale Preview, digest mismatch, size mismatch, and
missing payload fail closed. Selection change in the Workbench clears the
previous valid presentation.

After `model-viewer` has actually loaded the GLB:

```text
MotionAssetVersion / DerivedVariantVersion:
  viewer.availableAnimations.length == 0 → FAIL CLOSED
```

This display-validity gate ignores `descriptor.has_animation` and
`session.has_animation`. Descriptor metadata cannot override contradictory
loaded-payload evidence. Character Preview may have zero animations.

If Application uses the descriptor pre-display:

```text
has_animation == false → may fail early
has_animation == true  → never sufficient to prove actual animation
```

The viewer is not Product authority. Preview failure does not revoke a
Published Product.

## Controls

Measured on the actual `model-viewer` element, not HTML attributes alone:

- Character: orbit, zoom, reset
- Motion / Derived: play, pause, seek, restart (`currentTime` movement)

Automation hook: `window.__rigforgePreview`.

## What the viewer is not

- not Product identity
- not a V1-7 export (`Download GLB` / engine publish is forbidden here)
- not Transfer / QC / publication authority
- not a runtime CDN client
