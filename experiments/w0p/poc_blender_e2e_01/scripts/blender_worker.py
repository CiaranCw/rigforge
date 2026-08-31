# RESEARCH ONLY / W0-P / POC-BLENDER-E2E-01 Rev2
# Isolated Blender worker: inspect, rest-relative retarget, bake, artifacts.
# Not a production worker. Core language is not selected by this file.

from __future__ import annotations

import json
import math
import sys
import time
import traceback
from pathlib import Path

import bpy
from mathutils import Matrix, Quaternion, Vector


SAMPLE_TIMES = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
SAMPLE_JOINTS_SOURCE = ["root", "pelvis", "spine_03", "upperarm_l", "thigh_l", "Head"]
SAMPLE_JOINTS_TARGET = ["Bone", "Body", "Torso", "UpperArm.L", "UpperLeg.L", "Head"]
TRACE_PAIRS = [
    ("root", "Bone"),
    ("pelvis", "Body"),
    ("spine_03", "Torso"),
    ("upperarm_l", "UpperArm.L"),
    ("thigh_l", "UpperLeg.L"),
    ("Head", "Head"),
]
WORKER_VERSION = "v1-rev2"
INSPECTOR_VERSION = "v1-rev1"
TOL_ROT_RAD = 0.01
TOL_LOC = 0.001
SUPPORTED_QUAT_NORMALIZATION = frozenset({"NORMALIZE_BEFORE_KEY"})
SUPPORTED_QUAT_CONTINUITY = frozenset({"CONSECUTIVE_HEMISPHERE"})
QUAT_POST_NORM_TOL = 1e-5


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
    out = []
    for v in values:
        if isinstance(v, (int, float)):
            out.append(round(float(v), nd))
        else:
            out.append(v)
    return out


def blender_build_hash() -> str:
    h = bpy.app.build_hash
    if isinstance(h, (bytes, bytearray)):
        return h.decode("ascii")
    return str(h)


def world_pose_matrix(obj, pbone) -> Matrix:
    return obj.matrix_world @ pbone.matrix


def assign_action(obj, action) -> None:
    ad = obj.animation_data_create()
    ad.action = action
    slots = getattr(action, "slots", None)
    if slots is not None and len(slots) > 0:
        try:
            ad.action_slot = slots[0]
        except Exception:
            pass


def find_action(name: str):
    exact = bpy.data.actions.get(name)
    if exact is not None:
        return exact
    for a in bpy.data.actions:
        if a.name == name or a.name.endswith(name) or name.endswith(a.name):
            return a
    needle = name.split("|")[-1]
    for a in bpy.data.actions:
        if a.name.endswith(needle):
            return a
    return None


def classify_armature(obj) -> str | None:
    names = {b.name for b in obj.data.bones}
    if "UpperArm.L" in names and "Hips" in names and "Bone" in names:
        return "target"
    if "thigh_l" in names and "pelvis" in names and "root" in names:
        return "source"
    return None


def bone_depth(arm_obj, name: str) -> int:
    b = arm_obj.data.bones[name]
    n = 0
    while b.parent:
        n += 1
        b = b.parent
    return n


def look_at(obj, target: Vector):
    direction = target - obj.location
    obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def mute_pose_constraints(obj) -> int:
    n = 0
    for pb in obj.pose.bones:
        for c in pb.constraints:
            c.mute = True
            n += 1
    return n


def quat_angular_error(a, b) -> float:
    qa = Quaternion(a).normalized()
    qb = Quaternion(b).normalized()
    d = abs(qa.dot(qb))
    d = min(1.0, max(0.0, d))
    return 2.0 * math.acos(d)


def quat_list(q) -> list:
    return round_list([q.w, q.x, q.y, q.z])


def resolve_quaternion_policy(policy: dict) -> dict:
    q = policy.get("quaternion")
    if not isinstance(q, dict):
        raise RuntimeError("FAIL / REPORT: frozen Policy missing quaternion object")
    normalization = q.get("normalization")
    continuity = q.get("continuity")
    if normalization not in SUPPORTED_QUAT_NORMALIZATION:
        raise RuntimeError(
            "FAIL / REPORT: unsupported quaternion.normalization "
            f"{normalization!r}; supported={sorted(SUPPORTED_QUAT_NORMALIZATION)}"
        )
    if continuity not in SUPPORTED_QUAT_CONTINUITY:
        raise RuntimeError(
            "FAIL / REPORT: unsupported quaternion.continuity "
            f"{continuity!r}; supported={sorted(SUPPORTED_QUAT_CONTINUITY)}"
        )
    return {
        "normalization": normalization,
        "continuity": continuity,
        "continuity_rule": q.get("continuity_rule"),
        "interpolation": q.get("interpolation"),
        "worker_supported_normalization": sorted(SUPPORTED_QUAT_NORMALIZATION),
        "worker_supported_continuity": sorted(SUPPORTED_QUAT_CONTINUITY),
        "recognized": True,
        "supported": True,
    }


def apply_quaternion_policy(stored_keys, quat_pol: dict) -> dict:
    """Execute the frozen quaternion Policy on collected keys.

    Compact Adapter step only. Not a generic policy engine.
    """
    sign_flips = 0
    keys_checked = 0
    max_post_norm_err = 0.0
    min_dot = None
    prev_q = {}
    executed_normalization = quat_pol["normalization"]
    executed_continuity = quat_pol["continuity"]

    for _frame, rec in stored_keys:
        for name, item in rec.items():
            q = Quaternion(item["rotation_quaternion"])
            if executed_normalization == "NORMALIZE_BEFORE_KEY":
                mag = q.magnitude
                if mag <= 0.0:
                    raise RuntimeError(f"FAIL / REPORT: zero quaternion on {name}")
                q = q.normalized()
            max_post_norm_err = max(max_post_norm_err, abs(q.magnitude - 1.0))
            if executed_continuity == "CONSECUTIVE_HEMISPHERE":
                if name in prev_q:
                    d = float(prev_q[name].dot(q))
                    if d < 0.0:
                        q = -q
                        sign_flips += 1
                        d = float(prev_q[name].dot(q))
                    min_dot = d if min_dot is None else min(min_dot, d)
                prev_q[name] = Quaternion((q.w, q.x, q.y, q.z))
            item["rotation_quaternion"] = [float(q.w), float(q.x), float(q.y), float(q.z)]
            keys_checked += 1

    status = "PASS"
    if keys_checked <= 0:
        status = "FAIL"
    if max_post_norm_err > QUAT_POST_NORM_TOL:
        status = "FAIL"
    if min_dot is not None and min_dot < 0.0:
        status = "FAIL"
    return {
        "policy_normalization": quat_pol["normalization"],
        "policy_continuity": quat_pol["continuity"],
        "worker_supported_normalization": quat_pol["worker_supported_normalization"],
        "worker_supported_continuity": quat_pol["worker_supported_continuity"],
        "requested_mode_recognized": True,
        "requested_mode_supported": True,
        "requested_mode_executed": True,
        "executed_normalization": executed_normalization,
        "executed_continuity": executed_continuity,
        "samples_or_keys_checked": keys_checked,
        "sign_flips_applied": sign_flips,
        "post_normalization_max_abs_norm_minus_1": round(max_post_norm_err, 12),
        "minimum_consecutive_dot_after_continuity": (
            None if min_dot is None else round(min_dot, 12)
        ),
        "interpolation": quat_pol.get("interpolation"),
        "status": status,
    }


def iter_action_fcurves(action):
    if action is None:
        return
    if hasattr(action, "layers"):
        for layer in action.layers:
            strips = getattr(layer, "strips", [])
            for strip in strips:
                bags = []
                if hasattr(strip, "channelbags"):
                    bags = list(strip.channelbags)
                elif hasattr(strip, "channelbag"):
                    try:
                        bags = [strip.channelbag]
                    except Exception:
                        bags = []
                for bag in bags:
                    fcs = getattr(bag, "fcurves", None)
                    if fcs is not None:
                        yield from list(fcs)
    fcs = getattr(action, "fcurves", None)
    if fcs is not None:
        try:
            yield from list(fcs)
        except Exception:
            return


def action_summary(action):
    fcs = list(iter_action_fcurves(action))
    frames = []
    for fc in fcs:
        for kp in fc.keyframe_points:
            frames.append(float(kp.co[0]))
    fr = None
    if hasattr(action, "curve_frame_range"):
        try:
            fr = [float(action.curve_frame_range[0]), float(action.curve_frame_range[1])]
        except Exception:
            fr = None
    return {
        "name": action.name,
        "is_action_layered": bool(getattr(action, "is_action_layered", False)),
        "layer_count": len(action.layers) if hasattr(action, "layers") else None,
        "slot_count": len(action.slots) if hasattr(action, "slots") else None,
        "fcurve_count": len(fcs),
        "frame_min": min(frames) if frames else (fr[0] if fr else None),
        "frame_max": max(frames) if frames else (fr[1] if fr else None),
        "curve_frame_range": fr,
    }


def evaluated_mesh_aabb():
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mins = Vector((1e9, 1e9, 1e9))
    maxs = Vector((-1e9, -1e9, -1e9))
    found = False
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
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
        return Vector((0.0, 0.0, 1.0)), Vector((1.0, 1.0, 2.0))
    return mins, maxs


def union_sample_aabb(scene, f0: int, f1: int):
    mins = Vector((1e9, 1e9, 1e9))
    maxs = Vector((-1e9, -1e9, -1e9))
    for u in SAMPLE_TIMES:
        scene.frame_set(int(round(f0 + u * (f1 - f0))))
        bpy.context.view_layer.update()
        a, b = evaluated_mesh_aabb()
        mins.x = min(mins.x, a.x)
        mins.y = min(mins.y, a.y)
        mins.z = min(mins.z, a.z)
        maxs.x = max(maxs.x, b.x)
        maxs.y = max(maxs.y, b.y)
        maxs.z = max(maxs.z, b.z)
    center = (mins + maxs) * 0.5
    extent = maxs - mins
    radius = max(extent.length * 0.5, 1.0)
    return center, radius


def capture_rest_world(obj) -> dict:
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    out = {}
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
        out[pb.name] = world_pose_matrix(obj, pb).copy()
    bpy.ops.object.mode_set(mode="OBJECT")
    return out


def reset_pose_to_rest(obj) -> None:
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
        pb.location = Vector((0.0, 0.0, 0.0))
        pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
        pb.scale = Vector((1.0, 1.0, 1.0))


def sample_pose(obj, names, frame: int) -> dict:
    rec = {}
    for name in names:
        if name not in obj.pose.bones:
            rec[name] = {"present": False}
            continue
        pb = obj.pose.bones[name]
        mat = world_pose_matrix(obj, pb)
        loc, rot, scl = mat.decompose()
        rec[name] = {
            "present": True,
            "location": round_list(loc),
            "quat": round_list(rot),
            "scale": round_list(scl),
        }
    rec["_frame"] = frame
    return rec


def finite_matrix(mat: Matrix) -> bool:
    for row in mat:
        for v in row:
            if not math.isfinite(v):
                return False
    return True


def inspect(job: dict) -> dict:
    src = Path(job["asset_path"])
    out = Path(job["outputs"]["inspect_json"])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(
        filepath=str(src),
        use_anim=True,
        automatic_bone_orientation=False,
    )
    scene = bpy.context.scene
    armatures = []
    for obj in bpy.data.objects:
        if obj.type != "ARMATURE":
            continue
        bones = []
        for b in obj.data.bones:
            rest = obj.matrix_world @ b.matrix_local
            loc, rot, scl = rest.decompose()
            bones.append(
                {
                    "name": b.name,
                    "parent": b.parent.name if b.parent else None,
                    "use_deform": bool(b.use_deform),
                    "head_local": round_list(b.head_local),
                    "tail_local": round_list(b.tail_local),
                    "rest_world_location": round_list(loc),
                    "rest_world_quat": round_list(rot),
                    "rest_world_scale": round_list(scl),
                    "rest_source": "blender_data_bone_matrix_local_times_object_world",
                    "bind_pose": "UNKNOWN",
                    "inverse_bind": "NOT_OBSERVED",
                    "geometry_bind": "NOT_REQUIRED_FOR_THIS_POC",
                }
            )
        assigned = None
        if obj.animation_data and obj.animation_data.action:
            assigned = obj.animation_data.action.name
        armatures.append(
            {
                "object_name_ephemeral": obj.name,
                "data_name_ephemeral": obj.data.name,
                "assigned_action": assigned,
                "bone_count": len(obj.data.bones),
                "bones": bones,
            }
        )
    payload = {
        "source_file": str(src),
        "source_asset_sha256": job.get("asset_sha256"),
        "fps": scene.render.fps,
        "fps_base": scene.render.fps_base,
        "frame_start": int(scene.frame_start),
        "frame_end": int(scene.frame_end),
        "armature_count": len(armatures),
        "armatures": armatures,
        "actions": [action_summary(a) for a in bpy.data.actions],
        "objects": [
            {"name": o.name, "type": o.type, "parent": o.parent.name if o.parent else None}
            for o in bpy.data.objects
        ],
        "provenance": {
            "blender_version": bpy.app.version_string,
            "blender_build_hash": blender_build_hash(),
            "inspector_id": job.get("inspector_id", "poc-blender-e2e-01.blender_worker.inspect"),
            "inspector_version": job.get("inspector_version", INSPECTOR_VERSION),
            "inspector_source_sha256": job.get("inspector_source_sha256"),
            "factory_startup": True,
        },
        "classification": "RESEARCH_ONLY / W0-P / NON-PRODUCTION",
        "core_language_selected": False,
    }
    write_json(out, payload)
    return payload


def execute(job: dict) -> dict:
    t0 = time.perf_counter()
    timings = {}
    mapping = load_json(Path(job["mapping_path"]))
    policy = load_json(Path(job["policy_path"]))
    quat_pol = resolve_quaternion_policy(policy)
    out = job["outputs"]
    workspace = Path(out["workspace"])
    workspace.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    t_imp = time.perf_counter()
    bpy.ops.import_scene.fbx(
        filepath=job["character"]["path"],
        use_anim=False,
        automatic_bone_orientation=False,
    )
    bpy.ops.import_scene.fbx(
        filepath=job["motion"]["path"],
        use_anim=True,
        automatic_bone_orientation=False,
    )
    timings["import_s"] = round(time.perf_counter() - t_imp, 3)

    target = source = None
    for obj in list(bpy.data.objects):
        if obj.type != "ARMATURE":
            continue
        kind = classify_armature(obj)
        if kind == "target":
            target = obj
        elif kind == "source":
            source = obj
    if target is None or source is None:
        raise RuntimeError(f"armature classify failed target={target} source={source}")

    scene = bpy.context.scene
    fps = int(policy["bake_sample"]["fps"])
    scene.render.fps = fps
    scene.render.fps_base = 1.0
    f0 = int(policy["bake_sample"]["frame_start"])
    f1 = int(policy["bake_sample"]["frame_end"])
    scene.frame_start = f0
    scene.frame_end = f1

    tpose = find_action("Armature|Armature|A_TPose") or find_action("A_TPose")
    clip = find_action(job["motion"]["clip_id"])
    if tpose is None or clip is None:
        raise RuntimeError(
            f"missing actions tpose={tpose} clip={clip} available={[a.name for a in bpy.data.actions]}"
        )

    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    muted = mute_pose_constraints(target)
    reset_pose_to_rest(target)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.context.view_layer.update()
    tgt_rest = capture_rest_world(target)

    assign_action(source, tpose)
    scene.frame_set(f0)
    bpy.context.view_layer.update()
    src_rest = capture_rest_world(source)

    required = [e for e in mapping["entries"] if e.get("required")]
    optional = [e for e in mapping["entries"] if not e.get("required")]
    all_entries = required + optional
    for e in required:
        if e["source"] not in source.pose.bones:
            raise RuntimeError(f"unmapped required source joint missing in worker: {e['source']}")
        if e["target"] not in target.pose.bones:
            raise RuntimeError(f"unmapped required target joint missing in worker: {e['target']}")

    assign_action(source, clip)
    if target.animation_data:
        target.animation_data.action = None
        if hasattr(target, "animation_data_clear"):
            target.animation_data_clear()

    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    for pb in target.pose.bones:
        pb.rotation_mode = "QUATERNION"

    t_ret = time.perf_counter()
    nan_count = 0
    ordered = sorted(all_entries, key=lambda e: bone_depth(target, e["target"]))
    sample_frames = {int(round(f0 + u * (f1 - f0))): u for u in SAMPLE_TIMES}
    stored_keys = []
    traces = []
    desired_endpoints = {f0: {}, f1: {}}

    def apply_entries() -> None:
        nonlocal nan_count
        for e in ordered:
            if e["source"] not in source.pose.bones or e["target"] not in target.pose.bones:
                continue
            spb = source.pose.bones[e["source"]]
            tpb = target.pose.bones[e["target"]]
            src_now = world_pose_matrix(source, spb)
            src_r = src_rest[e["source"]]
            tgt_r = tgt_rest[e["target"]]
            if e["role"] == "root":
                desired = tgt_r @ (src_r.inverted() @ src_now)
                if not finite_matrix(desired):
                    nan_count += 1
                    continue
                tpb.matrix = target.matrix_world.inverted() @ desired
            else:
                src_dq = src_r.to_quaternion().inverted() @ src_now.to_quaternion()
                desired_q = tgt_r.to_quaternion() @ src_dq
                tpb.location = Vector((0.0, 0.0, 0.0))
                tpb.scale = Vector((1.0, 1.0, 1.0))
                tpb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
                bpy.context.view_layer.update()
                world0 = world_pose_matrix(target, tpb).to_quaternion()
                pose_q = world0.inverted() @ desired_q
                tpb.rotation_quaternion = pose_q.normalized()
                tpb.location = Vector((0.0, 0.0, 0.0))
                tpb.scale = Vector((1.0, 1.0, 1.0))
            bpy.context.view_layer.update()

    for frame in range(f0, f1 + 1):
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        reset_pose_to_rest(target)
        bpy.context.view_layer.update()
        apply_entries()
        bpy.context.view_layer.update()
        apply_entries()
        bpy.context.view_layer.update()

        rec = {}
        for e in ordered:
            if e["target"] not in target.pose.bones:
                continue
            tpb = target.pose.bones[e["target"]]
            q = Quaternion(tpb.rotation_quaternion)
            item = {"rotation_quaternion": [q.w, q.x, q.y, q.z]}
            if e["role"] == "root":
                item["location"] = list(tpb.location)
            rec[e["target"]] = item
        stored_keys.append((frame, rec))

        if frame in sample_frames or frame in (f0, f1):
            snap = {"frame": frame, "u": sample_frames.get(frame), "joints": []}
            for src_name, tgt_name in TRACE_PAIRS:
                e = next((x for x in ordered if x["source"] == src_name and x["target"] == tgt_name), None)
                if e is None:
                    continue
                spb = source.pose.bones[src_name]
                tpb = target.pose.bones[tgt_name]
                src_now = world_pose_matrix(source, spb)
                src_r = src_rest[src_name]
                tgt_r = tgt_rest[tgt_name]
                src_now_q = src_now.to_quaternion()
                src_rest_q = src_r.to_quaternion()
                src_delta_q = src_rest_q.inverted() @ src_now_q
                desired_q = tgt_r.to_quaternion() @ src_delta_q
                after_q = world_pose_matrix(target, tpb).to_quaternion()
                joint = {
                    "source": src_name,
                    "target": tgt_name,
                    "role": e["role"],
                    "source_current_world_rotation": quat_list(src_now_q),
                    "source_rest_world_rotation": quat_list(src_rest_q),
                    "source_rest_relative_delta": quat_list(src_delta_q),
                    "desired_target_world_rotation": quat_list(desired_q),
                    "after_adapter_apply_world_rotation": quat_list(after_q),
                    "after_apply_vs_desired_rad": round(quat_angular_error(desired_q, after_q), 6),
                }
                if e["role"] == "root":
                    joint["source_current_world_location"] = round_list(src_now.to_translation())
                    joint["source_rest_world_location"] = round_list(src_r.to_translation())
                    desired_root = tgt_r @ (src_r.inverted() @ src_now)
                    joint["desired_target_world_location"] = round_list(desired_root.to_translation())
                    joint["after_adapter_apply_world_location"] = round_list(
                        world_pose_matrix(target, tpb).to_translation()
                    )
                snap["joints"].append(joint)
                if frame in (f0, f1):
                    desired_endpoints[frame][tgt_name] = {
                        "desired_q": quat_list(desired_q),
                        "source_q": quat_list(src_now_q),
                    }
            if frame in sample_frames:
                traces.append(snap)

    quat_audit = apply_quaternion_policy(stored_keys, quat_pol)
    if quat_audit["status"] != "PASS":
        raise RuntimeError(f"FAIL / REPORT: quaternion Policy audit {quat_audit}")

    baked = bpy.data.actions.new("CharacterArmatureAction")
    assign_action(target, baked)
    for frame, rec in stored_keys:
        for name, item in rec.items():
            tpb = target.pose.bones[name]
            tpb.rotation_quaternion = Quaternion(item["rotation_quaternion"])
            tpb.keyframe_insert("rotation_quaternion", frame=frame)
            if "location" in item:
                tpb.location = Vector(item["location"])
                tpb.keyframe_insert("location", frame=frame)
    assign_action(target, baked)
    bpy.ops.object.mode_set(mode="OBJECT")
    timings["retarget_bake_s"] = round(time.perf_counter() - t_ret, 3)
    baked_action = baked.name

    samples_src = []
    samples_tgt = []
    baked_by_frame = {}
    for u in SAMPLE_TIMES:
        frame = int(round(f0 + u * (f1 - f0)))
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        rec_s = sample_pose(source, SAMPLE_JOINTS_SOURCE, frame)
        rec_t = sample_pose(target, SAMPLE_JOINTS_TARGET, frame)
        rec_s["u"] = u
        rec_t["u"] = u
        samples_src.append(rec_s)
        samples_tgt.append(rec_t)
        baked_by_frame[frame] = rec_t

    for snap in traces:
        frame = snap["frame"]
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        for joint in snap["joints"]:
            tpb = target.pose.bones[joint["target"]]
            baked_q = world_pose_matrix(target, tpb).to_quaternion()
            joint["after_bake_world_rotation"] = quat_list(baked_q)
            joint["bake_vs_desired_rad"] = round(
                quat_angular_error(joint["desired_target_world_rotation"], baked_q), 6
            )
            if joint["role"] == "root":
                joint["after_bake_world_location"] = round_list(
                    world_pose_matrix(target, tpb).to_translation()
                )

    max_bake_err = 0.0
    max_apply_err = 0.0
    bake_pass = True
    for snap in traces:
        for joint in snap["joints"]:
            max_apply_err = max(max_apply_err, joint["after_apply_vs_desired_rad"])
            max_bake_err = max(max_bake_err, joint["bake_vs_desired_rad"])
            if joint["bake_vs_desired_rad"] > TOL_ROT_RAD:
                bake_pass = False
            if joint["role"] == "root":
                d = joint["desired_target_world_location"]
                b = joint["after_bake_world_location"]
                dist = math.sqrt(sum((d[i] - b[i]) ** 2 for i in range(3)))
                joint["bake_vs_desired_location"] = round(dist, 6)
                if dist > TOL_LOC:
                    bake_pass = False

    loc_paths = {}
    rot_paths = {}
    for fc in iter_action_fcurves(baked):
        path = fc.data_path
        if "pose.bones[" not in path:
            continue
        name = path.split('pose.bones["', 1)[1].split('"]', 1)[0]
        if path.endswith("location"):
            loc_paths[name] = True
        if path.endswith("rotation_quaternion"):
            rot_paths[name] = True

    scene.frame_set(f0)
    bpy.context.view_layer.update()
    audit_rows = []
    audit_pass = True
    for e in ordered:
        name = e["target"]
        tpb = target.pose.bones[name]
        loc_mag = float(tpb.location.length)
        loc_keyed = bool(loc_paths.get(name))
        rot_keyed = bool(rot_paths.get(name))
        if e["role"] == "root":
            conform = loc_keyed and rot_keyed
        else:
            conform = (not loc_keyed) and rot_keyed and loc_mag <= TOL_LOC
        if not conform:
            audit_pass = False
        audit_rows.append(
            {
                "target": name,
                "role": e["role"],
                "location_channel_keyed": loc_keyed,
                "rotation_channel_keyed": rot_keyed,
                "persistent_pose_location_magnitude": round(loc_mag, 6),
                "policy_conformant": conform,
            }
        )

    loop_rows = []
    loop_pass = True
    for src_name, tgt_name in TRACE_PAIRS:
        e = next((x for x in ordered if x["source"] == src_name and x["target"] == tgt_name), None)
        if e is None:
            continue
        src0 = next(s[src_name] for s in samples_src if s["_frame"] == f0)
        src1 = next(s[src_name] for s in samples_src if s["_frame"] == f1)
        tgt0 = next(s[tgt_name] for s in samples_tgt if s["_frame"] == f0)
        tgt1 = next(s[tgt_name] for s in samples_tgt if s["_frame"] == f1)
        src_err = quat_angular_error(src0["quat"], src1["quat"])
        des0 = desired_endpoints[f0][tgt_name]["desired_q"]
        des1 = desired_endpoints[f1][tgt_name]["desired_q"]
        des_err = quat_angular_error(des0, des1)
        bake_err = quat_angular_error(tgt0["quat"], tgt1["quat"])
        source_closes = src_err <= TOL_ROT_RAD
        row_pass = True
        if source_closes:
            row_pass = des_err <= TOL_ROT_RAD and bake_err <= TOL_ROT_RAD
        if not row_pass:
            loop_pass = False
        loop_rows.append(
            {
                "source": src_name,
                "target": tgt_name,
                "source_endpoint_angular_error_rad": round(src_err, 6),
                "desired_target_endpoint_angular_error_rad": round(des_err, 6),
                "baked_target_endpoint_angular_error_rad": round(bake_err, 6),
                "source_closes": source_closes,
                "status": "PASS" if row_pass else "FAIL",
            }
        )

    src_root0 = next(s["root"] for s in samples_src if s["_frame"] == f0)
    src_root1 = next(s["root"] for s in samples_src if s["_frame"] == f1)
    tgt_root0 = next(s["Bone"] for s in samples_tgt if s["_frame"] == f0)
    tgt_root1 = next(s["Bone"] for s in samples_tgt if s["_frame"] == f1)
    def loc_span(a, b):
        return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(3)))

    src_root_delta = loc_span(src_root0["location"], src_root1["location"])
    tgt_root_delta = loc_span(tgt_root0["location"], tgt_root1["location"])
    root_policy_pass = abs(tgt_root_delta - src_root_delta) <= max(TOL_LOC, 0.05 * max(src_root_delta, TOL_LOC))

    source_armatures = [
        obj for obj in list(bpy.data.objects) if obj.type == "ARMATURE" and classify_armature(obj) == "source"
    ]
    for obj in list(bpy.data.objects):
        if obj.type != "MESH":
            continue
        for mod in obj.modifiers:
            if getattr(mod, "type", "") == "ARMATURE" and getattr(mod, "object", None) in source_armatures:
                bpy.data.objects.remove(obj, do_unlink=True)
                break
        else:
            if obj.parent in source_armatures:
                bpy.data.objects.remove(obj, do_unlink=True)
    for obj in source_armatures:
        bpy.data.objects.remove(obj, do_unlink=True)

    t_save = time.perf_counter()
    blend_path = Path(out["persistence_blend"])
    blend_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
    timings["save_blend_s"] = round(time.perf_counter() - t_save, 3)

    t_glb = time.perf_counter()
    glb_path = Path(out["preview_glb"])
    glb_ok = False
    glb_error = None
    try:
        bpy.ops.export_scene.gltf(
            filepath=str(glb_path),
            export_format="GLB",
            export_animations=True,
            export_skins=True,
            export_apply=False,
        )
        glb_ok = glb_path.exists()
    except Exception as exc:
        glb_error = str(exc)
    timings["glb_export_s"] = round(time.perf_counter() - t_glb, 3)

    t_img = time.perf_counter()
    frames_dir = Path(out["frames_dir"])
    frames_dir.mkdir(parents=True, exist_ok=True)
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = 640
    scene.render.resolution_y = 360
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    center, radius = union_sample_aabb(scene, f0, f1)
    dist = max(radius * 3.6, 10.0)
    cam_data = bpy.data.cameras.new("poc_cam")
    cam_data.lens = 35.0
    cam = bpy.data.objects.new("poc_cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    cam.location = center + Vector((dist * 0.55, -dist, dist * 0.42))
    look_at(cam, center)
    light_data = bpy.data.lights.new("poc_light", "SUN")
    light = bpy.data.objects.new("poc_light", light_data)
    scene.collection.objects.link(light)
    light.location = center + Vector((dist * 0.4, -dist * 0.25, dist * 0.8))
    image_paths = []
    for u in SAMPLE_TIMES:
        frame = int(round(f0 + u * (f1 - f0)))
        scene.frame_set(frame)
        fp = frames_dir / f"frame_u{str(u).replace('.', '')}_f{frame:03d}.png"
        scene.render.filepath = str(fp)
        bpy.ops.render.render(write_still=True)
        image_paths.append(str(fp))
    timings["render_s"] = round(time.perf_counter() - t_img, 3)
    timings["total_worker_s"] = round(time.perf_counter() - t0, 3)

    policy_trace = {
        "tolerance_rotation_rad": TOL_ROT_RAD,
        "tolerance_location": TOL_LOC,
        "per_frame_reset_to_rest": True,
        "rotation_only_non_root": True,
        "samples": traces,
        "max_after_apply_vs_desired_rad": round(max_apply_err, 6),
        "max_bake_vs_desired_rad": round(max_bake_err, 6),
        "desired_vs_baked": "PASS" if bake_pass else "FAIL",
    }
    loop_doc = {
        "tolerance_rotation_rad": TOL_ROT_RAD,
        "frame_start": f0,
        "frame_end": f1,
        "entries": loop_rows,
        "status": "PASS" if loop_pass else "FAIL",
        "reopen_status": "PENDING",
    }
    audit_doc = {
        "tolerance_location": TOL_LOC,
        "evaluated_at_frame": f0,
        "entries": audit_rows,
        "status": "PASS" if audit_pass else "FAIL",
    }
    quat_audit_path = Path(out.get("quaternion_policy_audit") or (workspace / "quaternion_policy_audit.json"))
    write_json(Path(out["policy_execution_trace"]), policy_trace)
    write_json(Path(out["loop_closure"]), loop_doc)
    write_json(Path(out["pose_channel_audit"]), audit_doc)
    write_json(quat_audit_path, quat_audit)
    out["quaternion_policy_audit"] = str(quat_audit_path)

    result = {
        "status": "SUCCESS",
        "worker_id": "poc-blender-e2e-01.blender_worker",
        "worker_version": WORKER_VERSION,
        "backend_identity": "blender",
        "blender_version": bpy.app.version_string,
        "blender_build_hash": blender_build_hash(),
        "classification": "RESEARCH_ONLY / W0-P / NON-PRODUCTION",
        "core_language_selected": False,
        "inputs": {
            "character_sha256": job["character"]["sha256"],
            "motion_sha256": job["motion"]["sha256"],
            "clip_id": job["motion"]["clip_id"],
            "mapping_sha256": job["mapping_sha256"],
            "policy_sha256": job["policy_sha256"],
            "job_spec_sha256": job.get("job_spec_sha256"),
        },
        "observed": {
            "source_clip": clip.name,
            "target_baked_action": baked_action,
            "fps": fps,
            "frame_start": f0,
            "frame_end": f1,
            "duration_s": round((f1 - f0) / float(fps), 6),
            "target_armature_bone_count": len(target.data.bones),
            "required_mapping_count": len(required),
        },
        "measurements": {
            "nan_inf_desired_matrices": nan_count,
            "samples_source": samples_src,
            "samples_target": samples_tgt,
            "policy_desired_vs_baked": "PASS" if bake_pass else "FAIL",
            "loop_closure": "PASS" if loop_pass else "FAIL",
            "rotation_only_audit": "PASS" if audit_pass else "FAIL",
            "root_source_endpoint_delta": round(src_root_delta, 6),
            "root_target_endpoint_delta": round(tgt_root_delta, 6),
            "root_policy": "PASS" if root_policy_pass else "FAIL",
            "quaternion_policy_execution": quat_audit["status"],
        },
        "artifacts": {
            "persistence_blend": str(blend_path),
            "preview_glb": str(glb_path) if glb_ok else None,
            "preview_glb_error": glb_error,
            "frames": image_paths,
            "policy_execution_trace": out.get("policy_execution_trace"),
            "loop_closure": out.get("loop_closure"),
            "pose_channel_audit": out.get("pose_channel_audit"),
            "quaternion_policy_audit": out.get("quaternion_policy_audit"),
        },
        "timings_s": timings,
        "adapter_diagnostics": {
            "note": "ephemeral Blender object names only; not product identity",
            "target_object_name": target.name,
            "method": (
                "per-frame rest reset; rest-relative world rotation delta; "
                "ROTATION_ONLY local location=0; root rest-relative world copy; "
                "frozen quaternion Policy NORMALIZE_BEFORE_KEY + CONSECUTIVE_HEMISPHERE "
                "applied before key insertion; keys written after independent "
                "per-frame solves; no IK; no add-on"
            ),
            "quaternion_policy_executed": {
                "normalization": quat_pol["normalization"],
                "continuity": quat_pol["continuity"],
            },
            "muted_pose_constraints": muted,
            "per_frame_independent_of_previous_pose": True,
        },
        "warnings": [],
        "errors": [],
    }
    if nan_count:
        result["warnings"].append(f"non-finite desired matrices skipped: {nan_count}")
    write_json(Path(out["worker_result"]), result)
    return result


def reopen(job: dict) -> dict:
    blend = Path(job["outputs"]["persistence_blend"])
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    target = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_armature(obj) == "target":
            target = obj
            break
    if target is None:
        for obj in bpy.data.objects:
            if obj.type == "ARMATURE":
                target = obj
                break
    if target is None:
        raise RuntimeError("reopen: no armature")
    scene = bpy.context.scene
    f0 = int(scene.frame_start)
    f1 = int(scene.frame_end)
    action = None
    if target.animation_data:
        action = target.animation_data.action
        if action is not None:
            assign_action(target, action)
    samples = []
    for u in SAMPLE_TIMES:
        frame = int(round(f0 + u * (f1 - f0)))
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        rec = sample_pose(target, SAMPLE_JOINTS_TARGET, frame)
        rec["u"] = u
        samples.append(rec)
    endpoint = {}
    for frame, label in ((f0, "start"), (f1, "end")):
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        endpoint[label] = {}
        for _, tgt_name in TRACE_PAIRS:
            if tgt_name not in target.pose.bones:
                continue
            mat = world_pose_matrix(target, target.pose.bones[tgt_name])
            loc, rot, _scl = mat.decompose()
            endpoint[label][tgt_name] = {"quat": round_list(rot), "location": round_list(loc)}
    payload = {
        "status": "SUCCESS",
        "target_present": True,
        "baked_action": action.name if action else None,
        "frame_start": f0,
        "frame_end": f1,
        "fps": scene.render.fps,
        "samples_target": samples,
        "endpoint_world": endpoint,
        "bone_count": len(target.data.bones),
    }
    write_json(Path(job["outputs"]["reopen_verification"]), payload)
    return payload


def main() -> int:
    args = parse_after_dash()
    mode = args[0]
    job = load_json(Path(args[1]))
    try:
        if mode == "inspect":
            inspect(job)
        elif mode == "execute":
            execute(job)
        elif mode == "reopen":
            reopen(job)
        else:
            raise SystemExit(f"unknown mode {mode}")
        return 0
    except Exception:
        err_path = Path(job.get("outputs", {}).get("workspace") or ".") / "worker_exception.json"
        write_json(
            err_path,
            {
                "status": "FAILURE",
                "mode": mode,
                "traceback": traceback.format_exc(),
            },
        )
        result_path = Path(job.get("outputs", {}).get("worker_result") or err_path)
        if mode == "execute":
            write_json(
                result_path,
                {
                    "status": "FAILURE",
                    "worker_id": "poc-blender-e2e-01.blender_worker",
                    "traceback": traceback.format_exc(),
                    "errors": [traceback.format_exc().splitlines()[-1]],
                    "published_derived_variant": False,
                },
            )
        raise


if __name__ == "__main__":
    raise SystemExit(main())
