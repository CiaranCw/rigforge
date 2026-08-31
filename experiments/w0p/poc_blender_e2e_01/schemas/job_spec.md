# Job Spec (research)

Versioned command to a worker. Not a Blender session.

May carry hashes/paths for Character, Motion, summaries, Mapping, Policy,
outputs, and a declared determinism context.

Must not carry as product semantics:

- `bpy` types
- Blender collections / datablocks / object pointers
- Blender operator names
- Blender constraint class names
