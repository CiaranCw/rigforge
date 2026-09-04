# Deterministic rest body-frame alignment regressions. Not a Product Character.
# Procedural armatures only. Does not use asset, clip, or Host names.

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

PI = math.pi
FLEX_RAD = math.radians(25.0)
ROOT_TRAVEL = 1.3


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


def quat_abs_dot(a: Quaternion, b: Quaternion) -> float:
    return abs(float(a.normalized().dot(b.normalized())))


def mapping():
    return [
        {"source": "Root", "target": "Root", "role": "root", "required": True},
        {"source": "Pelvis", "target": "Pelvis", "role": "pelvis", "required": True},
        {"source": "Chest", "target": "Chest", "role": "spine_03", "required": True},
        {"source": "Thigh.A", "target": "Thigh.A", "role": "thigh", "required": True},
        {"source": "Thigh.B", "target": "Thigh.B", "role": "thigh", "required": True},
        {"source": "Arm.A", "target": "Arm.A", "role": "upper_arm", "required": True},
        {"source": "Arm.B", "target": "Arm.B", "role": "upper_arm", "required": True},
    ]


def make_humanoid(name: str, *, yaw_rad: float = 0.0, roll: float = 0.0, mirror_shoulders: bool = False, omit_limbs: bool = False):
    arm = bpy.data.armatures.new(name + "Data")
    obj = bpy.data.objects.new(name, arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    def bone(bone_name, head, tail, parent=None):
        eb = arm.edit_bones.new(bone_name)
        eb.head = head
        eb.tail = tail
        eb.roll = roll
        if parent is not None:
            eb.parent = parent
        return eb
    root = bone("Root", (0.0, 0.0, 0.0), (0.0, 0.0, 0.1))
    pelvis = bone("Pelvis", (0.0, 0.0, 0.9), (0.0, 0.0, 1.05), root)
    bone("Chest", (0.0, 0.0, 1.35), (0.0, 0.0, 1.5), pelvis)
    if not omit_limbs:
        bone("Thigh.A", (-0.12, 0.0, 0.85), (-0.12, 0.0, 0.45), pelvis)
        bone("Thigh.B", (0.12, 0.0, 0.85), (0.12, 0.0, 0.45), pelvis)
        if mirror_shoulders:
            bone("Arm.A", (0.25, 0.0, 1.4), (0.55, 0.0, 1.4), pelvis)
            bone("Arm.B", (-0.25, 0.0, 1.4), (-0.55, 0.0, 1.4), pelvis)
        else:
            bone("Arm.A", (-0.25, 0.0, 1.4), (-0.55, 0.0, 1.4), pelvis)
            bone("Arm.B", (0.25, 0.0, 1.4), (0.55, 0.0, 1.4), pelvis)
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.rotation_euler = (0.0, 0.0, yaw_rad)
    bpy.context.view_layer.update()
    bpy.ops.object.mode_set(mode="POSE")
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def rest_world(obj):
    worker.reset_pose_to_rest(obj)
    bpy.context.view_layer.update()
    return worker.capture_rest_world(obj)


def torso_lean_deg(obj, rest, fwd, up) -> float:
    chest = worker.rest_translation(worker.world_pose_matrix(obj, obj.pose.bones["Chest"]))
    pelvis = worker.rest_translation(worker.world_pose_matrix(obj, obj.pose.bones["Pelvis"]))
    v = chest - pelvis
    f = float(v.dot(fwd))
    u = float(v.dot(up))
    return math.degrees(math.atan2(f, u))


def body_axes(frame: Matrix):
    right = Vector(frame.col[0])
    forward = Vector(frame.col[1])
    up = Vector(frame.col[2])
    return right, forward, up


def transfer_pose(source, target, src_rest, tgt_rest, body_a, frames, animate):
    ordered = sorted(mapping(), key=lambda e: worker.bone_depth(target, e["target"]))
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    stored = []
    nan_count = 0
    scene = bpy.context.scene
    for frame in frames:
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        animate(source, frame)
        bpy.context.view_layer.update()
        worker.reset_pose_to_rest(target)
        bpy.context.view_layer.update()
        for e in ordered:
            if e["source"] not in source.pose.bones or e["target"] not in target.pose.bones:
                continue
            spb = source.pose.bones[e["source"]]
            tpb = target.pose.bones[e["target"]]
            src_now = worker.world_pose_matrix(source, spb)
            if e.get("role") == "root":
                ok = worker.apply_root_keep_target_rest_scale(
                    target, tpb, src_rest[e["source"]], src_now, tgt_rest[e["target"]], body_a
                )
            else:
                ok = worker.apply_rest_relative_world_rotation(
                    target, tpb, src_rest[e["source"]], src_now, tgt_rest[e["target"]], body_a
                )
            if not ok:
                nan_count += 1
            bpy.context.view_layer.update()
        rec = {}
        for e in ordered:
            if e["target"] not in target.pose.bones:
                continue
            tpb = target.pose.bones[e["target"]]
            rec[e["target"]] = {
                "rotation_quaternion": list(tpb.rotation_quaternion),
                "location": list(tpb.location),
            }
        stored.append((frame, rec))
    bpy.ops.object.mode_set(mode="OBJECT")
    return stored, nan_count


def flex_pelvis(source, frame: int) -> None:
    worker.reset_pose_to_rest(source)
    bpy.context.view_layer.update()
    pb = source.pose.bones["Pelvis"]
    if frame <= 1:
        pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    else:
        pb.rotation_quaternion = Quaternion(Vector((1.0, 0.0, 0.0)), FLEX_RAD)
    pb.location = Vector((0.0, 0.0, 0.0))
    pb.scale = Vector((1.0, 1.0, 1.0))


def travel_root(source, frame: int) -> None:
    worker.reset_pose_to_rest(source)
    bpy.context.view_layer.update()
    pb = source.pose.bones["Root"]
    t = 0.0 if frame <= 1 else ROOT_TRAVEL
    rest = worker.world_pose_matrix(source, pb)
    loc, rot, scl = rest.decompose()
    desired = Matrix.LocRotScale(loc + Vector((0.0, t, 0.0)), rot, scl)
    pb.matrix = source.matrix_world.inverted() @ desired
    pb.scale = Vector((1.0, 1.0, 1.0))


def inplace_root(source, frame: int) -> None:
    worker.reset_pose_to_rest(source)
    bpy.context.view_layer.update()
    pb = source.pose.bones["Root"]
    pb.location = Vector((0.0, 0.0, 0.0))
    pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    pb.scale = Vector((1.0, 1.0, 1.0))


def thigh_swing(source, frame: int) -> None:
    worker.reset_pose_to_rest(source)
    bpy.context.view_layer.update()
    pb = source.pose.bones["Thigh.B"]
    if frame <= 1:
        pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    else:
        pb.rotation_quaternion = Quaternion(Vector((1.0, 0.0, 0.0)), math.radians(40.0))
    pb.location = Vector((0.0, 0.0, 0.0))
    pb.scale = Vector((1.0, 1.0, 1.0))


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def setup_pair(yaw_deg: float, roll: float = 0.0, mirror_shoulders: bool = False, omit_limbs: bool = False):
    clear_scene()
    source = make_humanoid("SrcBody", yaw_rad=0.0, roll=0.0, omit_limbs=omit_limbs)
    target = make_humanoid(
        "TgtBody",
        yaw_rad=math.radians(yaw_deg),
        roll=roll,
        mirror_shoulders=mirror_shoulders,
        omit_limbs=omit_limbs,
    )
    src_rest = rest_world(source)
    tgt_rest = rest_world(target)
    return source, target, src_rest, tgt_rest


def case_alignment(yaw_deg: float, roll: float = 0.0) -> dict:
    source, target, src_rest, tgt_rest = setup_pair(yaw_deg, roll=roll)
    resolved = worker.resolve_body_alignment(mapping(), src_rest, tgt_rest)
    alt = worker.resolve_body_alignment(
        mapping(), src_rest, tgt_rest, forward_mode="right_cross_up"
    )
    a = resolved["alignment_quat"]
    invariant = quat_abs_dot(a, alt["alignment_quat"]) > 0.999
    stored, nan_count = transfer_pose(
        source, target, src_rest, tgt_rest, a, [1, 10], flex_pelvis
    )
    bpy.context.scene.frame_set(10)
    flex_pelvis(source, 10)
    bpy.context.view_layer.update()
    src_right, src_fwd, src_up = body_axes(resolved["source_frame"])
    tgt_right, tgt_fwd, tgt_up = body_axes(resolved["target_frame"])
    src_lean = torso_lean_deg(source, src_rest, src_fwd, src_up)
    tgt_lean = torso_lean_deg(target, tgt_rest, tgt_fwd, tgt_up)
    transfer_pose(source, target, src_rest, tgt_rest, a, [1, 10], thigh_swing)
    bpy.context.scene.frame_set(1)
    thigh_swing(source, 1)
    bpy.context.view_layer.update()
    src_tail_rest = (source.matrix_world @ source.pose.bones["Thigh.B"].tail).copy()
    bpy.context.scene.frame_set(10)
    thigh_swing(source, 10)
    bpy.context.view_layer.update()
    src_tail_now = (source.matrix_world @ source.pose.bones["Thigh.B"].tail).copy()
    tgt_tail_now = (target.matrix_world @ target.pose.bones["Thigh.B"].tail).copy()
    tgt_tail_rest = worker.rest_translation(tgt_rest["Thigh.B"])
    # Rest tail for target: bone tail at rest pose
    worker.reset_pose_to_rest(target)
    bpy.context.view_layer.update()
    tgt_tail_rest = (target.matrix_world @ target.pose.bones["Thigh.B"].tail).copy()
    transfer_pose(source, target, src_rest, tgt_rest, a, [10], thigh_swing)
    tgt_tail_now = (target.matrix_world @ target.pose.bones["Thigh.B"].tail).copy()
    src_swing = src_tail_now - src_tail_rest
    tgt_swing = tgt_tail_now - tgt_tail_rest
    src_fwd_comp = float(src_swing.dot(src_fwd))
    tgt_fwd_comp = float(tgt_swing.dot(tgt_fwd))
    lean_ok = (src_lean >= 0) == (tgt_lean >= 0) or abs(src_lean) < 1.0
    swing_ok = (src_fwd_comp >= 0) == (tgt_fwd_comp >= 0)
    angle = resolved["alignment_angle_deg"]
    expected = float(abs(yaw_deg) % 360)
    if expected > 180:
        expected = 360 - expected
    angle_ok = abs(angle - expected) < 8.0 or (expected < 1.0 and angle < 8.0)
    det_ok = (
        abs(resolved["source_determinant"] - 1.0) < 1e-3
        and abs(resolved["target_determinant"] - 1.0) < 1e-3
        and abs(resolved["alignment_determinant"] - 1.0) < 1e-3
    )
    status = (
        "PASS"
        if invariant and lean_ok and swing_ok and angle_ok and det_ok and nan_count == 0
        else "FAIL"
    )
    return {
        "status": status,
        "yaw_deg": yaw_deg,
        "roll": roll,
        "alignment_angle_deg": round(angle, 4),
        "alignment_quat": r6([a.w, a.x, a.y, a.z]),
        "source_determinant": round(resolved["source_determinant"], 6),
        "target_determinant": round(resolved["target_determinant"], 6),
        "alignment_determinant": round(resolved["alignment_determinant"], 6),
        "cross_order_invariant": invariant,
        "source_lean_deg": round(src_lean, 4),
        "target_lean_deg": round(tgt_lean, 4),
        "lean_sign_match": lean_ok,
        "source_swing_forward": round(src_fwd_comp, 6),
        "target_swing_forward": round(tgt_fwd_comp, 6),
        "swing_sign_match": swing_ok,
        "nan_count": nan_count,
        "reflection_detected": resolved["reflection_detected"],
        "name_heuristics_used": resolved["name_heuristics_used"],
        "motion_used": resolved["motion_used"],
        "laterality_roles": resolved["laterality_roles"],
        "apply_passes": 1,
    }


def case_degenerate() -> dict:
    source, target, src_rest, tgt_rest = setup_pair(0.0, omit_limbs=True)
    try:
        worker.resolve_body_alignment(mapping(), src_rest, tgt_rest)
        return {"status": "FAIL", "reason": "expected fail-closed"}
    except worker.BodyFrameError as exc:
        return {
            "status": "PASS",
            "reason": str(exc),
            "details": exc.details,
            "failure_class": "body_frame_alignment_unresolved",
        }


def case_reflection() -> dict:
    source, target, src_rest, tgt_rest = setup_pair(0.0, mirror_shoulders=True)
    try:
        worker.resolve_body_alignment(mapping(), src_rest, tgt_rest)
        return {"status": "FAIL", "reason": "expected fail-closed on contradictory laterality"}
    except worker.BodyFrameError as exc:
        reflection = bool(exc.details.get("reflection_detected"))
        return {
            "status": "PASS" if reflection else "FAIL",
            "reason": str(exc),
            "details": exc.details,
            "reflection_detected": reflection,
            "failure_class": "body_frame_alignment_unresolved",
        }


def case_root(yaw_deg: float, animate, expect_travel: bool) -> dict:
    source, target, src_rest, tgt_rest = setup_pair(yaw_deg)
    resolved = worker.resolve_body_alignment(mapping(), src_rest, tgt_rest)
    a = resolved["alignment_quat"]
    transfer_pose(source, target, src_rest, tgt_rest, a, [1, 10], animate)
    bpy.context.scene.frame_set(10)
    animate(source, 10)
    bpy.context.view_layer.update()
    src_root = worker.world_pose_matrix(source, source.pose.bones["Root"]).to_translation()
    tgt_root = worker.world_pose_matrix(target, target.pose.bones["Root"]).to_translation()
    src_rest_t = worker.rest_translation(src_rest["Root"])
    tgt_rest_t = worker.rest_translation(tgt_rest["Root"])
    src_delta = src_root - src_rest_t
    tgt_delta = tgt_root - tgt_rest_t
    _r, tgt_fwd, _u = body_axes(resolved["target_frame"])
    mag = float(tgt_delta.length)
    fwd_comp = float(tgt_delta.dot(tgt_fwd)) if mag > 1e-8 else 0.0
    lateral = float(tgt_delta.dot(_r)) if mag > 1e-8 else 0.0
    if expect_travel:
        direction_ok = fwd_comp > 0.9 * mag and mag > 1.0
        synthetic = False
    else:
        direction_ok = mag < 1e-4
        synthetic = mag >= 1e-4
    status = "PASS" if direction_ok else "FAIL"
    return {
        "status": status,
        "yaw_deg": yaw_deg,
        "source_delta": r6(src_delta),
        "target_delta": r6(tgt_delta),
        "target_forward_component": round(fwd_comp, 6),
        "target_lateral_component": round(lateral, 6),
        "magnitude": round(mag, 6),
        "synthetic_forward_motion": synthetic,
        "alignment_angle_deg": round(resolved["alignment_angle_deg"], 4),
    }


def main() -> int:
    args = parse_after_dash()
    out_dir = Path(args[0])
    out_dir.mkdir(parents=True, exist_ok=True)
    try:
        zero = case_alignment(0.0)
        yaw90 = case_alignment(90.0)
        yaw180 = case_alignment(180.0)
        roll_body = case_alignment(180.0, roll=PI / 2.0)
        degenerate = case_degenerate()
        reflection = case_reflection()
        root_travel = case_root(180.0, travel_root, True)
        root_hold = case_root(180.0, inplace_root, False)
        payload = {
            "status": "PASS"
            if all(
                item["status"] == "PASS"
                for item in (
                    zero,
                    yaw90,
                    yaw180,
                    roll_body,
                    degenerate,
                    reflection,
                    root_travel,
                    root_hold,
                )
            )
            else "FAIL",
            "adapter_version": worker.WORKER_VERSION,
            "zero": zero,
            "yaw90": yaw90,
            "yaw180": yaw180,
            "roll_and_body": roll_body,
            "degenerate": degenerate,
            "reflection": reflection,
            "root_travel": root_travel,
            "root_inplace": root_hold,
        }
        write_json(out_dir / "body_frame_result.json", payload)
        return 0 if payload["status"] == "PASS" else 1
    except Exception:
        write_json(out_dir / "body_frame_exception.json", {"traceback": traceback.format_exc()})
        raise


if __name__ == "__main__":
    raise SystemExit(main())
