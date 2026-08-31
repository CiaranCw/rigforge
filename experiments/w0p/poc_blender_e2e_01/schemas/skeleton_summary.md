# Skeleton Summary (research)

Backend-neutral inspection evidence. Not a production schema.

Required fields for this PoC:

- `skeleton_id`
- `source_asset_sha256`
- `joints[]`: `id`, `parent`, `deforming` (or UNKNOWN)
- rest/base observation with `rest_source`
- `bind_pose` / `inverse_bind` / `geometry_bind` explicitly
  UNKNOWN / NOT_OBSERVED / NOT_REQUIRED_FOR_THIS_POC when not copied
- `fingerprint_sha256`

Forbidden as durable fields:

- `bpy.types.*`
- Blender object IDs / datablock pointers
