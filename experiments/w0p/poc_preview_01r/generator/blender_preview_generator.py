# RESEARCH ONLY / W0-P / POC-PREVIEW-01R
# Isolated Blender Preview Generator. Not a production service.
# Viewer runtime does not import this file.

from __future__ import annotations

import json
import math
import sys
import time
import traceback
from pathlib import Path

import bpy
from mathutils import Vector


GENERATOR_VERSION = "v1-preview-01r"


def parse_after_dash():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit("missing -- args")
    return argv[argv.index("--") + 1 :]


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def round_list(values, nd=6):
    return [round(float(v), nd) for v in values]


def blender_build_hash() -> str:
    h = bpy.app.build_hash
    if isinstance(h, (bytes, bytearray)):
        return h.decode("ascii")
    return str(h)


def find_action(name: str):
    exact = bpy.data.actions.get(name)
    if exact is not None:
        return exact
    needle = name.split("|")[-1]
    for a in bpy.data.actions:
        if a.name == name or a.name.endswith(name) or name.endswith(a.name) or a.name.endswith(needle):
            return a
    return None


def assign_action(obj, action) -> None:
    ad = obj.animation_data_create()
    ad.action = action
    slots = getattr(action, "slots", None)
    if slots is not None and len(slots) > 0:
        try:
            ad.action_slot = slots[0]
        except Exception:
            pass


def classify_armature(obj) -> str | None:
    """Test-pair generator routing only. Not Preview Artifact contract.

    These bone-name checks pick source vs target armature inside the frozen
    Knight + UAL2 pair. They must not become product Skeleton identity,
    Canonical roles, or Motion-proxy semantics.
    """
    names = {b.name for b in obj.data.bones}
    if "UpperArm.L" in names and "Hips" in names and "Bone" in names:
        return "target"
    if "thigh_l" in names and "pelvis" in names and "root" in names:
        return "source"
    return None


def evaluated_mesh_aabb():
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mins = Vector((1e9, 1e9, 1e9))
    maxs = Vector((-1e9, -1e9, -1e9))
    found = False
    node_count = 0
    mat_names = []
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        node_count += 1
        for slot in obj.material_slots:
            if slot.material and slot.material.name not in mat_names:
                mat_names.append(slot.material.name)
        eval_obj = obj.evaluated_get(depsgraph)
        try:
            mesh = eval_obj.to_mesh()
        except Exception:
            continue
        try:
            for v in mesh.vertices:
                w = eval_obj.matrix_world @ v.co
                mins.x = min(mins.x, w.x)
                mins.y = min(mins.y, w.y)
                mins.z = min(mins.z, w.z)
                maxs.x = max(maxs.x, w.x)
                maxs.y = max(maxs.y, w.y)
                maxs.z = max(maxs.z, w.z)
                found = True
        finally:
            eval_obj.to_mesh_clear()
    if not found:
        return {
            "valid": False,
            "min": [0, 0, 0],
            "max": [1, 1, 1],
            "center": [0.5, 0.5, 0.5],
            "radius": 1.0,
            "mesh_object_count": node_count,
            "material_names": mat_names[:24],
        }
    center = (mins + maxs) * 0.5
    extent = maxs - mins
    radius = max(extent.length * 0.5, 0.25)
    return {
        "valid": True,
        "min": round_list(mins),
        "max": round_list(maxs),
        "center": round_list(center),
        "radius": round(float(radius), 6),
        "mesh_object_count": node_count,
        "material_names": mat_names[:24],
    }


def action_inventory():
    out = []
    for a in bpy.data.actions:
        frames = []
        fcs = list(getattr(a, "fcurves", []) or [])
        if hasattr(a, "layers"):
            for layer in a.layers:
                for strip in getattr(layer, "strips", []) or []:
                    bags = []
                    if hasattr(strip, "channelbags"):
                        bags = list(strip.channelbags)
                    for bag in bags:
                        fcs.extend(list(getattr(bag, "fcurves", []) or []))
        for fc in fcs:
            for kp in getattr(fc, "keyframe_points", []) or []:
                frames.append(float(kp.co[0]))
        item = {"name": a.name, "key_count_curves": len(fcs)}
        if frames:
            item["frame_min"] = min(frames)
            item["frame_max"] = max(frames)
        out.append(item)
    return out


def preview_make_visible_materials() -> dict:
    """Force opaque, visible preview albedo. Not source-material authority.

    Observed FBX/glTF path: MASK + baseColor alpha 0 made Knight invisible
    in the viewer. Source textures are not present on this FBX import.
    """
    preview_albedo = {
        "Skin": (0.72, 0.52, 0.40, 1.0),
        "Armor": (0.42, 0.44, 0.48, 1.0),
        "Armor_Dark": (0.18, 0.19, 0.22, 1.0),
        "Detail": (0.50, 0.32, 0.16, 1.0),
        "Red": (0.62, 0.10, 0.08, 1.0),
    }
    touched = []
    for mat in bpy.data.materials:
        if mat.name == "proxy_mat":
            continue
        rgb = preview_albedo.get(mat.name, (0.55, 0.55, 0.58, 1.0))
        if hasattr(mat, "blend_method"):
            try:
                mat.blend_method = "OPAQUE"
            except Exception:
                pass
        if hasattr(mat, "surface_render_method"):
            try:
                mat.surface_render_method = "DITHERED"
            except Exception:
                pass
        mat.diffuse_color = rgb
        if mat.use_nodes and mat.node_tree:
            for node in mat.node_tree.nodes:
                if node.type != "BSDF_PRINCIPLED":
                    continue
                for sock_name in ("Alpha", "Base Color"):
                    sock = node.inputs.get(sock_name)
                    if sock is None:
                        continue
                    for link in list(sock.links):
                        mat.node_tree.links.remove(link)
                if "Alpha" in node.inputs:
                    node.inputs["Alpha"].default_value = 1.0
                if "Base Color" in node.inputs:
                    node.inputs["Base Color"].default_value = rgb
                if "Metallic" in node.inputs:
                    node.inputs["Metallic"].default_value = 0.0
                if "Roughness" in node.inputs:
                    node.inputs["Roughness"].default_value = 0.55
        touched.append(mat.name)
    return {
        "forced_opaque": True,
        "simplified_albedo": True,
        "materials": touched[:24],
    }


def export_glb(path: Path, animations: bool) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        export_animations=animations,
        export_skins=True,
        export_apply=False,
        export_cameras=False,
        export_extras=False,
    )


def ortho(axis: Vector) -> Vector:
    axis = axis.normalized()
    helper = Vector((0.0, 0.0, 1.0)) if abs(axis.z) < 0.9 else Vector((0.0, 1.0, 0.0))
    side = axis.cross(helper)
    if side.length < 1e-8:
        side = axis.cross(Vector((1.0, 0.0, 0.0)))
    return side.normalized()


def build_hierarchy_proxy(arm_obj) -> dict:
    """Generic joint/segment proxy from parent/transform hierarchy only.

    No pelvis/humanoid/Mixamo/UE role table. Not a Canonical Skeleton.
    """
    mesh = bpy.data.meshes.new("motion_proxy")
    obj = bpy.data.objects.new("motion_proxy", mesh)
    bpy.context.scene.collection.objects.link(obj)
    verts = []
    faces = []
    groups = {b.name: [] for b in arm_obj.data.bones}

    def add_octa(center: Vector, scale: float, bone_name: str) -> None:
        s = max(scale, 0.012)
        base = len(verts)
        verts.extend(
            [
                center + Vector((s, 0, 0)),
                center + Vector((-s, 0, 0)),
                center + Vector((0, s, 0)),
                center + Vector((0, -s, 0)),
                center + Vector((0, 0, s)),
                center + Vector((0, 0, -s)),
            ]
        )
        faces.extend(
            [
                (base + 4, base + 0, base + 2),
                (base + 4, base + 2, base + 1),
                (base + 4, base + 1, base + 3),
                (base + 4, base + 3, base + 0),
                (base + 5, base + 2, base + 0),
                (base + 5, base + 1, base + 2),
                (base + 5, base + 3, base + 1),
                (base + 5, base + 0, base + 3),
            ]
        )
        groups[bone_name].extend(range(base, base + 6))

    def add_segment(h: Vector, t: Vector, bone_name: str) -> None:
        d = t - h
        length = d.length
        if length < 1e-6:
            return
        axis = d.normalized()
        side = ortho(axis)
        up = axis.cross(side).normalized()
        r = max(min(length * 0.07, 0.035), 0.006)
        base = len(verts)
        for end in (h, t):
            for sx, sy in ((1, 1), (1, -1), (-1, -1), (-1, 1)):
                verts.append(end + side * (sx * r) + up * (sy * r))
        faces.extend(
            [
                (base + 0, base + 1, base + 5, base + 4),
                (base + 1, base + 2, base + 6, base + 5),
                (base + 2, base + 3, base + 7, base + 6),
                (base + 3, base + 0, base + 4, base + 7),
            ]
        )
        groups[bone_name].extend(range(base, base + 8))

    for bone in arm_obj.data.bones:
        add_octa(bone.head_local, 0.018, bone.name)
        add_segment(bone.head_local, bone.tail_local, bone.name)

    mesh.from_pydata(verts, [], faces)
    mesh.update()
    for name, idxs in groups.items():
        vg = obj.vertex_groups.new(name=name)
        if idxs:
            vg.add(list(idxs), 1.0, "REPLACE")
    obj.parent = arm_obj
    mod = obj.modifiers.new("Armature", "ARMATURE")
    mod.object = arm_obj
    mat = bpy.data.materials.new("proxy_mat")
    mat.diffuse_color = (0.82, 0.78, 0.55, 1.0)
    obj.data.materials.append(mat)
    return {
        "proxy_kind": "hierarchy_joint_and_segment_mesh",
        "humanoid_role_table": False,
        "canonical_skeleton": False,
        "joint_count": len(arm_obj.data.bones),
        "vertex_count": len(verts),
        "face_count": len(faces),
    }


def generate_character(job: dict) -> dict:
    t0 = time.perf_counter()
    src = Path(job["source_path"])
    out = Path(job["outputs"]["payload"])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(
        filepath=str(src),
        use_anim=False,
        automatic_bone_orientation=False,
    )
    stripped_actions = len(bpy.data.actions)
    for obj in bpy.data.objects:
        if obj.animation_data:
            obj.animation_data_clear()
    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = 1
    scene.frame_set(1)
    bpy.context.view_layer.update()
    mat_fix = preview_make_visible_materials()
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    export_glb(out, animations=False)
    return {
        "kind": "CHARACTER",
        "presentation": "rest_or_imported_reference_no_source_clip",
        "stripped_leftover_action_count": stripped_actions,
        "material_preview_fix": mat_fix,
        "bounds": bounds,
        "animation_inventory": [],
        "default_animation": None,
        "duration_s": None,
        "fps": None,
        "had_source_mesh": bounds["mesh_object_count"] > 0,
        "elapsed_s": round(time.perf_counter() - t0, 3),
        "payload": str(out),
        "warnings": [
            "source Character clip library not played",
            "FBX pivot/layer recipes not preserved",
            "source MASK/clip alpha forced opaque for preview visibility",
            "source textures not present on this FBX import; simplified opaque albedo",
        ],
    }


def generate_motion(job: dict) -> dict:
    t0 = time.perf_counter()
    src = Path(job["source_path"])
    clip_id = job["clip_id"]
    out = Path(job["outputs"]["payload"])
    fps = int(job.get("fps", 30))
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(
        filepath=str(src),
        use_anim=True,
        automatic_bone_orientation=False,
    )
    source = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_armature(obj) == "source":
            source = obj
            break
    if source is None:
        for obj in bpy.data.objects:
            if obj.type == "ARMATURE":
                source = obj
                break
    if source is None:
        raise RuntimeError("motion: no armature")
    had_mesh = any(o.type == "MESH" for o in bpy.data.objects)
    mesh_names = [o.name for o in bpy.data.objects if o.type == "MESH"]
    for obj in list(bpy.data.objects):
        if obj.type == "MESH":
            bpy.data.objects.remove(obj, do_unlink=True)
    clip = find_action(clip_id)
    if clip is None:
        raise RuntimeError(f"clip not found: {clip_id} available={[a.name for a in bpy.data.actions]}")
    assign_action(source, clip)
    for action in list(bpy.data.actions):
        if action != clip:
            bpy.data.actions.remove(action)
    scene = bpy.context.scene
    scene.render.fps = fps
    scene.render.fps_base = 1.0
    scene.frame_start = 1
    scene.frame_end = 61
    scene.frame_set(1)
    bpy.context.view_layer.update()
    proxy = build_hierarchy_proxy(source)
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    export_glb(out, animations=True)
    duration = (61 - 1) / float(fps)
    return {
        "kind": "MOTION",
        "clip_requested": clip_id,
        "clip_observed": clip.name,
        "had_source_renderable_mesh": had_mesh,
        "source_mesh_names_ephemeral": mesh_names[:20],
        "source_mesh_used_as_preview": False,
        "proxy": proxy,
        "bounds": bounds,
        "animation_inventory": action_inventory(),
        "default_animation": clip.name,
        "frame_start": 1,
        "frame_end": 61,
        "fps": fps,
        "duration_s": duration,
        "elapsed_s": round(time.perf_counter() - t0, 3),
        "payload": str(out),
        "warnings": [
            "synthetic hierarchy proxy; not a production rig",
            "source mesh discarded even if present",
            "FBX layers/pivots not preserved",
            "not Canonical Skeleton identity",
        ],
    }


def generate_derived(job: dict) -> dict:
    t0 = time.perf_counter()
    blend = Path(job["source_path"])
    out = Path(job["outputs"]["payload"])
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    scene = bpy.context.scene
    fps = int(scene.render.fps or 30)
    f0 = int(scene.frame_start)
    f1 = int(scene.frame_end)
    scene.frame_set(f0)
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    default = None
    target = None
    keep = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_armature(obj) == "target":
            target = obj
            break
    if target and target.animation_data and target.animation_data.action:
        keep = target.animation_data.action
        default = keep.name
        assign_action(target, keep)
    for action in list(bpy.data.actions):
        if keep is None or action != keep:
            bpy.data.actions.remove(action)
    if default is None and bpy.data.actions:
        default = bpy.data.actions[0].name
    mat_fix = preview_make_visible_materials()
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    inv = action_inventory()
    export_glb(out, animations=True)
    duration = (f1 - f0) / float(fps) if fps else None
    return {
        "kind": "DERIVED_VARIANT",
        "opened_blend": True,
        "material_preview_fix": mat_fix,
        "bounds": bounds,
        "animation_inventory": inv,
        "default_animation": default,
        "frame_start": f0,
        "frame_end": f1,
        "fps": fps,
        "duration_s": duration,
        "elapsed_s": round(time.perf_counter() - t0, 3),
        "payload": str(out),
        "warnings": [
            "Preview of accepted Derived Variant bake; not Mapping/QC authority",
            "GLB may drop FBX pivot/layer semantics",
            "source MASK/clip alpha forced opaque for preview visibility",
            "source textures not present on this FBX import; simplified opaque albedo",
        ],
    }


def main() -> int:
    args = parse_after_dash()
    mode = args[0]
    job = load_json(Path(args[1]))
    out_meta = Path(job["outputs"]["generation_json"])
    try:
        if mode == "character":
            result = generate_character(job)
        elif mode == "motion":
            result = generate_motion(job)
        elif mode == "derived":
            result = generate_derived(job)
        else:
            raise SystemExit(f"unknown mode {mode}")
        result["status"] = "SUCCESS"
        result["generator_id"] = "poc-preview-01r.blender_preview_generator"
        result["generator_version"] = GENERATOR_VERSION
        result["blender_version"] = bpy.app.version_string
        result["blender_build_hash"] = blender_build_hash()
        write_json(out_meta, result)
        return 0
    except Exception:
        write_json(
            out_meta,
            {
                "status": "FAILURE",
                "mode": mode,
                "traceback": traceback.format_exc(),
            },
        )
        raise


if __name__ == "__main__":
    raise SystemExit(main())
