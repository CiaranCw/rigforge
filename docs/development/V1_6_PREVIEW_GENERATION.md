# V1-6 Preview Generation

Generation-time Preview producer. Blender is not a viewer requirement and
not Product authority.

Status: `COMPLETE / PASS / BASELINED`

## Recipe

```text
PREVIEW_RECIPE_VERSION = v1-6-preview-1
PREVIEW_GENERATOR_ID   = rigforge-preview-generator/0.1.0
PREVIEW_MEDIA_TYPE     = model/gltf-binary
```

Production generator: sealed pinned Blender 5.2.1 LTS build `9e2066aef7ef`
(`app/src/pinned_preview.rs`). This path ignores
`RIGFORGE_BLENDER_EXECUTABLE`, matching sealed QC.

Tests inject `MemoryPreviewGenerator`. Production
`Application::generate_preview` always uses the Blender generator.

## Modes

| Mode | Subject | Source | Animation |
| --- | --- | --- | --- |
| `preview_character` | CharacterAssetVersion | exact Character FBX | stripped |
| `preview_motion` | MotionAssetVersion | Motion + Source Skeleton FBX | required; generic hierarchy proxy |
| `preview_derived` | Published DerivedVariantVersion | published PersistenceArtifact `.blend` | required; open exact bake |
| `preview_motion_synthetic` | MotionAssetVersion (architecture fixture) | 4-bone non-humanoid proxy | required |

Motion Preview does **not** require a target Character.

`PreviewDescriptor.has_animation` is generator / UX metadata. Generation
may fail early when it is `false` for Motion / Derived. A `true` value
does not prove that the viewer-loaded GLB contains an animation; the
viewer fails closed when the actual payload has none.

Derived Preview requires:

```text
Published DerivedVariantVersion
PersistenceArtifact
PersistenceVerification PASS
```

It opens the exact durable `.blend`. It does not rerun Transfer, Mapping,
retarget, QC, or publication.

## Source integrity

Non-synthetic generation verifies source digest and size before and after.
Mutation of source or persistence bytes denies generation.

## Declared losses (not silent)

Typical declared losses include simplified materials/textures, FBX
pivot/layer drop, Character animation strip, Motion hierarchy proxy (not
Canonical Skeleton identity), and “Preview is not an export contract”.
Synthetic non-humanoid uses `base_joint` / `mid_a_joint` / `mid_b_joint` /
`tip_joint` and forbids hips/spine/UE/Mixamo names. `classify_armature` is
frozen-pair routing only, not product Skeleton identity.

## Isolation from Transfer

Preview generation is not a Transfer `JobRun`. It does not produce
`WorkerResult`, QC, or publication. Failure leaves Product truth unchanged.
