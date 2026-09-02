# V1-6 Preview Artifact Contract

Derived Preview identity, exact Product binding, and payload integrity.
Not a Product format. Not an export contract.

Status: `COMPLETE / PASS / BASELINED`

## Flags

A `PreviewArtifact` is always:

```text
derived = true
rebuildable = true
authoritative = false
```

Validation rejects any other combination.

## Exact Product binding

Exactly one Product kind:

```text
CharacterAssetVersion XOR
MotionAssetVersion XOR
DerivedVariantVersion
```

Multi-binding and no-binding are Domain errors (`PreviewBinding`).
`PreviewArtifact.id` is not the payload digest.

Preferred direction:

```text
PreviewArtifact → exact Product Version
```

Published Product records are not mutated to store Preview pointers.
Catalog `latest_preview_for_exact_version` is a query, not Product truth.

## Payload

V1 Preview payload media type:

```text
model/gltf-binary
```

Storage:

```text
{catalog_parent}/rigforge-previews/<PreviewArtifactId>/preview.glb
{catalog_parent}/rigforge-previews/<PreviewArtifactId>/descriptor.json
```

`descriptor.json` is UX / generation metadata. It is not Domain.

## Persist / display order

Before Catalog insertion of a **location-bearing** `PreviewArtifact`:

1. exact Product version exists in the same Catalog
2. `PreviewArtifact.producer_id` resolves to an existing
   `BackendExecutionContext` in the same Catalog
3. payload file exists
4. media type is `model/gltf-binary`
5. size matches `PreviewArtifact.size_bytes`
6. SHA-256 matches `PreviewArtifact.digest`

A typed but nonexistent `BackendExecutionContextId` is **FAIL CLOSED**.
Catalog must not silently create a producer. Producer creation remains an
Application / generator responsibility.

This check applies to `persist_preview_artifact` / `persist_preview_on` and
to the generic `put_validated` Preview guard. Product binding and payload
integrity checks are not weakened.

Application validates **before** the viewer may display:

1. exact Product version exists
2. PreviewArtifact is bound to that exact version
3. payload file exists
4. size matches `PreviewArtifact.size_bytes`
5. SHA-256 matches `PreviewArtifact.digest`

`PreviewDescriptor.has_animation` is generator / UX metadata. It is not
Product truth and cannot prove that the loaded GLB contains an animation.

On any Application validation failure the session is invalid, `payload.glb`
is omitted, and the viewer must fail closed.

## Delete / regenerate

Delete and regenerate create a new Preview identity. They must not change
Character / Motion / Derived Variant Product bytes, lifecycle, or
publication evidence.

## Location-less records

Historical Catalog fixtures may store a location-less `PreviewArtifact`.
A location-bearing persist requires the bound Product version, an existing
`BackendExecutionContext` for `producer_id`, and a verified
`model/gltf-binary` payload. Location-less fixtures skip producer and
payload file checks.
