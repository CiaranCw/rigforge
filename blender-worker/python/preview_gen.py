# RigForge V1-6 Preview generator (Blender generation-time only).
# Promoted from POC-PREVIEW-01R blender_preview_generator.py.
# Not Product authority. Not a viewer. Does not retarget, map, QC, or publish.
# bpy types never leave this process as Product contracts.

from __future__ import annotations

import hashlib
import json
import math
import sys
import time
import traceback
from pathlib import Path

import bpy
from mathutils import Vector

GENERATOR_VERSION = "v1-6-preview-1"
GENERATOR_ID = "rigforge-preview-generator/0.1.0"


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


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
    for action in bpy.data.actions:
        if action.name == name or action.name.endswith(name) or name.endswith(action.name) or action.name.endswith(needle):
            return action
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
    """Frozen-pair generator routing only. Not Preview Artifact contract.

    Bone-name checks pick source vs target armature inside the Knight + UAL2
    pair. They must not become product Skeleton identity, Canonical roles,
    or Motion-proxy semantics.
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
    for action in bpy.data.actions:
        frames = []
        fcs = list(getattr(action, "fcurves", []) or [])
        if hasattr(action, "layers"):
            for layer in action.layers:
                for strip in getattr(layer, "strips", []) or []:
                    bags = []
                    if hasattr(strip, "channelbags"):
                        bags = list(strip.channelbags)
                    for bag in bags:
                        fcs.extend(list(getattr(bag, "fcurves", []) or []))
        for fc in fcs:
            for kp in getattr(fc, "keyframe_points", []) or []:
                frames.append(float(kp.co[0]))
        item = {"name": action.name, "key_count_curves": len(fcs)}
        if frames:
            item["frame_min"] = min(frames)
            item["frame_max"] = max(frames)
        out.append(item)
    return out


def preview_make_visible_materials() -> dict:
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
        "bone_names": [b.name for b in arm_obj.data.bones],
    }


def verify_source(job: dict) -> None:
    if job.get("synthetic"):
        return
    src = Path(job["source_path"])
    if not src.is_file():
        raise RuntimeError(f"preview source missing: {src}")
    expected = str(job.get("expected_digest") or "")
    if expected:
        found = sha256_file(src)
        if found != expected:
            raise RuntimeError("preview source digest mismatch; generation refused")
    expected_size = job.get("expected_size")
    if expected_size not in (None, 0, "0"):
        size = src.stat().st_size
        if int(expected_size) != size:
            raise RuntimeError(f"preview source size mismatch: expected {expected_size} found {size}")


def generate_character(job: dict) -> dict:
    t0 = time.perf_counter()
    verify_source(job)
    src = Path(job["source_path"])
    digest_before = sha256_file(src)
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
    if sha256_file(src) != digest_before:
        raise RuntimeError("character preview mutated source bytes")
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
    verify_source(job)
    src = Path(job["source_path"])
    digest_before = sha256_file(src)
    clip_id = job["clip_id"]
    out = Path(job["outputs"]["payload"])
    fps = int(job.get("fps") or 30)
    frame_start = int(job.get("frame_start") or 1)
    frame_end = int(job.get("frame_end") or 61)
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
    scene.frame_start = frame_start
    scene.frame_end = frame_end
    scene.frame_set(frame_start)
    bpy.context.view_layer.update()
    proxy = build_hierarchy_proxy(source)
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    export_glb(out, animations=True)
    if sha256_file(src) != digest_before:
        raise RuntimeError("motion preview mutated source bytes")
    duration = (frame_end - frame_start) / float(fps) if fps else None
    return {
        "kind": "MOTION",
        "clip_requested": clip_id,
        "clip_observed": clip.name,
        "had_source_renderable_mesh": had_mesh,
        "source_mesh_names_ephemeral": mesh_names[:20],
        "source_mesh_used_as_preview": False,
        "required_target_character": False,
        "proxy": proxy,
        "bounds": bounds,
        "animation_inventory": action_inventory(),
        "default_animation": clip.name,
        "frame_start": frame_start,
        "frame_end": frame_end,
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


def generate_motion_synthetic(job: dict) -> dict:
    """Deterministic non-humanoid armature. No hips/spine/UE/Mixamo roles."""
    t0 = time.perf_counter()
    out = Path(job["outputs"]["payload"])
    fps = int(job.get("fps") or 30)
    frame_start = int(job.get("frame_start") or 1)
    frame_end = int(job.get("frame_end") or 31)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    arm_data = bpy.data.armatures.new("probe_armature")
    arm_obj = bpy.data.objects.new("probe_armature", arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    names = ["base_joint", "mid_a_joint", "mid_b_joint", "tip_joint"]
    bones = []
    for i, name in enumerate(names):
        bone = arm_data.edit_bones.new(name)
        bone.head = (0.0, 0.0, i * 0.25)
        bone.tail = (0.0, 0.0, i * 0.25 + 0.25)
        if bones:
            bone.parent = bones[-1]
        bones.append(bone)
    bpy.ops.object.mode_set(mode="POSE")
    action = bpy.data.actions.new("synthetic_proxy")
    assign_action(arm_obj, action)
    pbone = arm_obj.pose.bones["mid_a_joint"]
    arm_obj.animation_data.action = action
    scene = bpy.context.scene
    scene.render.fps = fps
    scene.frame_start = frame_start
    scene.frame_end = frame_end
    for frame, angle in ((frame_start, 0.0), (frame_end, 1.2)):
        scene.frame_set(frame)
        pbone.rotation_mode = "XYZ"
        pbone.rotation_euler = (angle, 0.0, 0.0)
        pbone.keyframe_insert(data_path="rotation_euler", frame=frame)
    scene.frame_set(frame_start)
    bpy.context.view_layer.update()
    proxy = build_hierarchy_proxy(arm_obj)
    forbidden = {"hips", "spine", "spine_01", "upperarm_l", "thigh_l", "pelvis", "mixamo", "ue"}
    found = {n.lower() for n in proxy["bone_names"]}
    if found & forbidden:
        raise RuntimeError(f"synthetic proxy used humanoid names: {found & forbidden}")
    bpy.context.view_layer.update()
    bounds = evaluated_mesh_aabb()
    export_glb(out, animations=True)
    duration = (frame_end - frame_start) / float(fps) if fps else None
    return {
        "kind": "MOTION",
        "clip_requested": "synthetic_proxy",
        "clip_observed": "synthetic_proxy",
        "had_source_renderable_mesh": False,
        "source_mesh_used_as_preview": False,
        "required_target_character": False,
        "synthetic_non_humanoid": True,
        "proxy": proxy,
        "bounds": bounds,
        "animation_inventory": action_inventory(),
        "default_animation": "synthetic_proxy",
        "frame_start": frame_start,
        "frame_end": frame_end,
        "fps": fps,
        "duration_s": duration,
        "elapsed_s": round(time.perf_counter() - t0, 3),
        "payload": str(out),
        "warnings": [
            "deterministic synthetic non-humanoid proxy; not real-asset coverage",
            "generic hierarchy proxy; no humanoid role table",
        ],
    }


def generate_derived(job: dict) -> dict:
    t0 = time.perf_counter()
    verify_source(job)
    blend = Path(job["source_path"])
    digest_before = sha256_file(blend)
    out = Path(job["outputs"]["payload"])
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    scene = bpy.context.scene
    fps = int(scene.render.fps or job.get("fps") or 30)
    f0 = int(job.get("frame_start") or scene.frame_start)
    f1 = int(job.get("frame_end") or scene.frame_end)
    scene.frame_set(f0)
    bpy.context.view_layer.update()
    default = None
    target = None
    keep = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_armature(obj) == "target":
            target = obj
            break
    if target is None:
        for obj in bpy.data.objects:
            if obj.type == "ARMATURE":
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
    if sha256_file(blend) != digest_before:
        raise RuntimeError("derived preview mutated persistence bytes")
    duration = (f1 - f0) / float(fps) if fps else None
    return {
        "kind": "DERIVED_VARIANT",
        "opened_blend": True,
        "retarget_rerun": False,
        "mapping_rerun": False,
        "qc_rerun": False,
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


def run_preview(mode: str, job: dict) -> dict:
    if mode == "preview_character":
        result = generate_character(job)
    elif mode == "preview_motion":
        result = generate_motion(job)
    elif mode == "preview_motion_synthetic":
        result = generate_motion_synthetic(job)
    elif mode == "preview_derived":
        result = generate_derived(job)
    else:
        raise SystemExit(f"unknown preview mode {mode}")
    result["status"] = "SUCCESS"
    result["generator_id"] = GENERATOR_ID
    result["generator_version"] = GENERATOR_VERSION
    result["blender_version"] = bpy.app.version_string
    result["blender_build_hash"] = blender_build_hash()
    return result
