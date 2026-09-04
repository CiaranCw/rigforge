# Deterministic RestRelativeWorldDelta basis regression. Not a Product Character.
# Procedural armatures only. Source and target share semantic rest limb layout
# but differ in local bone roll. Does not use Synty or UAL2 names.

from __future__ import annotations

import json
import math
import sys
import traceback
from pathlib import Path

import bpy
from mathutils import Matrix, Quaternion, Vector

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import worker  # noqa: E402

PI_2 = math.pi / 2.0
FLEX_RAD = math.radians(60.0)


def parse_after_dash():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit("missing -- args")
    return argv[argv.index("--") + 1 :]


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def r6(v):
    return [round(float(x), 6) for x in v]


def angle_deg(a: Vector, b: Vector) -> float:
    na = a.normalized()
    nb = b.normalized()
    d = max(-1.0, min(1.0, float(na.dot(nb))))
    return math.degrees(math.acos(d))


def legacy_desired_q(src_rest, src_now, tgt_rest) -> Quaternion:
    src_dq = src_rest.to_quaternion().inverted() @ src_now.to_quaternion()
    return (tgt_rest.to_quaternion() @ src_dq).normalized()


def make_chain(name: str, parent_name: str, child_name: str, left_name: str, right_name: str, roll: float):
    arm = bpy.data.armatures.new(name + "Data")
    obj = bpy.data.objects.new(name, arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    parent = arm.edit_bones.new(parent_name)
    parent.head = (0.0, 0.0, 0.0)
    parent.tail = (0.0, 1.0, 0.0)
    parent.roll = roll
    child = arm.edit_bones.new(child_name)
    child.head = (0.0, 1.0, 0.0)
    child.tail = (0.0, 2.0, 0.0)
    child.parent = parent
    child.use_connect = True
    child.roll = roll
    left = arm.edit_bones.new(left_name)
    left.head = (0.2, 0.8, 0.0)
    left.tail = (1.2, 0.8, 0.0)
    left.parent = parent
    left.roll = roll
    right = arm.edit_bones.new(right_name)
    right.head = (-0.2, 0.8, 0.0)
    right.tail = (-1.2, 0.8, 0.0)
    right.parent = parent
    right.roll = roll
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.mode_set(mode="POSE")
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def bone_tail_world(obj, name: str) -> Vector:
    pb = obj.pose.bones[name]
    return (obj.matrix_world @ pb.tail).copy()


def bone_world_quat(obj, name: str) -> Quaternion:
    return worker.world_pose_matrix(obj, obj.pose.bones[name]).to_quaternion()


def pose_scale(obj, name: str):
    return [float(obj.pose.bones[name].scale[i]) for i in range(3)]


def setup_scene():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = 10
    scene.render.fps = 30
    source = make_chain("SrcArm", "Parent", "Child", "Upper.L", "Upper.R", 0.0)
    target = make_chain("TgtArm", "Hip", "Shin", "Arm.L", "Arm.R", PI_2)
    bpy.context.view_layer.objects.active = source
    source.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    spb = source.pose.bones["Parent"]
    action = bpy.data.actions.new("SrcFlex")
    worker.assign_action(source, action)
    rest_q = Quaternion((1.0, 0.0, 0.0, 0.0))
    flex_q = Quaternion(Vector((1.0, 0.0, 0.0)), FLEX_RAD)
    for frame, quat in ((1, rest_q), (10, flex_q)):
        scene.frame_set(frame)
        spb.location = Vector((0.0, 0.0, 0.0))
        spb.rotation_quaternion = quat
        spb.scale = Vector((1.0, 1.0, 1.0))
        spb.keyframe_insert("location", frame=frame)
        spb.keyframe_insert("rotation_quaternion", frame=frame)
        spb.keyframe_insert("scale", frame=frame)
    bpy.ops.object.mode_set(mode="OBJECT")
    return scene, source, target, action


def transfer(scene, source, target, formula: str) -> dict:
    mapping = [
        {"source": "Parent", "target": "Hip", "role": "spine_01", "required": True},
        {"source": "Child", "target": "Shin", "role": "spine_02", "required": True},
        {"source": "Upper.L", "target": "Arm.L", "role": "upper_arm", "required": True},
        {"source": "Upper.R", "target": "Arm.R", "role": "upper_arm", "required": True},
    ]
    ordered = sorted(mapping, key=lambda e: worker.bone_depth(target, e["target"]))
    src_action = source.animation_data.action if source.animation_data else None
    worker.reset_pose_to_rest(source)
    worker.reset_pose_to_rest(target)
    bpy.context.view_layer.update()
    src_rest = worker.capture_rest_world(source)
    tgt_rest = worker.capture_rest_world(target)
    if src_action is not None:
        worker.assign_action(source, src_action)
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    stored = []
    nan_count = 0
    for frame in range(1, 11):
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        worker.reset_pose_to_rest(target)
        bpy.context.view_layer.update()
        for e in ordered:
            spb = source.pose.bones[e["source"]]
            tpb = target.pose.bones[e["target"]]
            src_now = worker.world_pose_matrix(source, spb)
            src_r = src_rest[e["source"]]
            tgt_r = tgt_rest[e["target"]]
            if formula == "legacy":
                desired_q = legacy_desired_q(src_r, src_now, tgt_r)
                ok = worker.apply_desired_world_rotation(target, tpb, desired_q)
            else:
                ok = worker.apply_rest_relative_world_rotation(
                    target, tpb, src_r, src_now, tgt_r
                )
            if not ok:
                nan_count += 1
            bpy.context.view_layer.update()
        rec = {}
        for e in ordered:
            tpb = target.pose.bones[e["target"]]
            rec[e["target"]] = {
                "rotation_quaternion": list(tpb.rotation_quaternion),
            }
        stored.append((frame, rec))
    quat_audit = worker.apply_quaternion_policy(stored)
    baked = bpy.data.actions.new("CharacterArmatureAction")
    worker.assign_action(target, baked)
    for frame, rec in stored:
        for name, item in rec.items():
            tpb = target.pose.bones[name]
            tpb.location = Vector((0.0, 0.0, 0.0))
            tpb.rotation_quaternion = Quaternion(item["rotation_quaternion"])
            tpb.scale = Vector((1.0, 1.0, 1.0))
            tpb.keyframe_insert("rotation_quaternion", frame=frame)
    scene.frame_set(10)
    bpy.context.view_layer.update()
    src_child = bone_tail_world(source, "Child")
    tgt_child = bone_tail_world(target, "Shin")
    src_left = bone_tail_world(source, "Upper.L")
    tgt_left = bone_tail_world(target, "Arm.L")
    src_right = bone_tail_world(source, "Upper.R")
    tgt_right = bone_tail_world(target, "Arm.R")
    src_parent_dir = (bone_tail_world(source, "Parent") - Vector((0.0, 0.0, 0.0))).normalized()
    tgt_parent_dir = (bone_tail_world(target, "Hip") - Vector((0.0, 0.0, 0.0))).normalized()
    endpoint_err_deg = angle_deg(src_child, tgt_child)
    parent_err_deg = angle_deg(src_parent_dir, tgt_parent_dir)
    finite = all(
        math.isfinite(v)
        for v in [*src_child, *tgt_child, *src_left, *tgt_left]
    )
    scales = [pose_scale(target, n) for n in ("Hip", "Shin", "Arm.L", "Arm.R")]
    scale_ok = all(abs(v - 1.0) < 1e-4 for row in scales for v in row)
    laterality_ok = float(tgt_left.x) > 0.0 and float(tgt_right.x) < 0.0
    angular_ok = endpoint_err_deg < 8.0 and parent_err_deg < 8.0
    status = "PASS" if finite and scale_ok and laterality_ok and angular_ok and nan_count == 0 else "FAIL"
    return {
        "formula": formula,
        "status": status,
        "nan_count": nan_count,
        "finite": finite,
        "keep_target_rest_scale": "PASS" if scale_ok else "FAIL",
        "quaternion_policy": quat_audit["status"],
        "laterality_ok": laterality_ok,
        "endpoint_err_deg": round(endpoint_err_deg, 4),
        "parent_dir_err_deg": round(parent_err_deg, 4),
        "source_child_tail": r6(src_child),
        "target_child_tail": r6(tgt_child),
        "source_left_tail": r6(src_left),
        "target_left_tail": r6(tgt_left),
        "source_right_tail": r6(src_right),
        "target_right_tail": r6(tgt_right),
        "source_parent_dir": r6(src_parent_dir),
        "target_parent_dir": r6(tgt_parent_dir),
        "pose_scale": scales,
        "quaternion_keys_checked": quat_audit["samples_or_keys_checked"],
        "apply_passes": 1,
        "source_roll": 0.0,
        "target_roll_deg": 90.0,
    }


def rest_basis_angles(source, target) -> dict:
    worker.reset_pose_to_rest(source)
    worker.reset_pose_to_rest(target)
    bpy.context.view_layer.update()
    def bone_y(obj, name):
        bone = obj.data.bones[name]
        head = obj.matrix_world @ bone.head_local
        tail = obj.matrix_world @ bone.tail_local
        return (tail - head).normalized()
    src_y = bone_y(source, "Parent")
    tgt_y = bone_y(target, "Hip")
    src_basis = worker.world_pose_matrix(source, source.pose.bones["Parent"]).to_3x3()
    tgt_basis = worker.world_pose_matrix(target, target.pose.bones["Hip"]).to_3x3()
    # Compare local X columns (roll difference).
    src_x = Vector(src_basis.col[0])
    tgt_x = Vector(tgt_basis.col[0])
    return {
        "semantic_limb_dir_err_deg": round(angle_deg(src_y, tgt_y), 4),
        "local_x_err_deg": round(angle_deg(src_x, tgt_x), 4),
    }


def main() -> int:
    args = parse_after_dash()
    out_dir = Path(args[0])
    out_dir.mkdir(parents=True, exist_ok=True)
    try:
        scene, source, target, _action = setup_scene()
        basis = rest_basis_angles(source, target)
        legacy = transfer(scene, source, target, "legacy")
        # Rebuild a clean target pose for the corrected formula in the same file.
        bpy.ops.wm.read_factory_settings(use_empty=True)
        scene, source, target, _action = setup_scene()
        world = transfer(scene, source, target, "world")
        payload = {
            "status": world["status"],
            "legacy_status": legacy["status"],
            "world_status": world["status"],
            "basis": basis,
            "legacy": legacy,
            "world": world,
            "adapter_version": worker.WORKER_VERSION,
        }
        write_json(out_dir / "basis_alignment_result.json", payload)
        if world["status"] != "PASS" or legacy["status"] != "FAIL":
            return 1
        return 0
    except Exception:
        write_json(out_dir / "basis_alignment_exception.json", {"traceback": traceback.format_exc()})
        raise


if __name__ == "__main__":
    raise SystemExit(main())
