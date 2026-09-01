# Preview Artifact (research)

Thin derived/non-authoritative preview contract. Not product identity.

A Preview Artifact records:

- `preview_artifact_id` / version
- source product kind (`CHARACTER` | `MOTION` | `DERIVED_VARIANT`)
- source product id and binding hashes
- generator identity / backend / build
- payload type, external path reference, SHA-256, size, MIME
- scene/bounds observations
- animation inventory and duration when applicable
- camera/framing hint
- declared losses
- generation recipe hash
- generated-from provenance

Viewer validation is against independent Product fixture truth (id, version,
complete expected_source_binding), not against a catalog copy of this
manifest. Payload `payload_sha256` / `payload_size` are enforced before
render.

It MUST NOT become authority for Asset identity, Source Skeleton identity,
Mapping, Retarget Policy, QC, Derived Variant lineage, or source file truth.

Deleting it must not destroy product state. Rebuilding it must not create a
new Character / Motion / Derived identity.

Payload candidate in this PoC is GLB. GLB is not selected as a permanent
RigForge Preview format.
