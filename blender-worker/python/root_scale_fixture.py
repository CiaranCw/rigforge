# Deterministic KeepTargetRestScale fixture. Not a Product Character/Motion.
# Procedural armatures only. Does not download or parse FBX.

from __future__ import annotations

import json
import sys
import traceback
from pathlib import Path

import bpy
from mathutils import Matrix, Quaternion, Vector

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import worker  # noqa: E402


def parse_after_dash():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit("missing -- args")
    return argv[argv.index("--") + 1 :]


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def make_armature(name: str, bone_name: str):
    arm = bpy.data.armatures.new(name + "Data")
    obj = bpy.data.objects.new(name, arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm.edit_bones.new(bone_name)
    eb.head = (0.0, 0.0, 0.0)
    eb.tail = (0.0, 1.0, 0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.mode_set(mode="POSE")
    obj.pose.bones[bone_name].rotation_mode = "QUATERNION"
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def assign_action(obj, action):
    if obj.animation_data is None:
        obj.animation_data_create()
    obj.animation_data.action = action
    worker.assign_action(obj, action)


def apply_forbidden_full_matrix(target, tpb, src_rest, src_now, tgt_rest) -> None:
    desired = tgt_rest @ (src_rest.inverted() @ src_now)
    tpb.matrix = target.matrix_world.inverted() @ desired


def setup_scene():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = 10
    scene.render.fps = 30
    source = make_armature("SrcArm", "root")
    target = make_armature("TgtArm", "Bone")
    bpy.context.view_layer.objects.active = source
    source.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    spb = source.pose.bones["root"]
    action = bpy.data.actions.new("SrcRootScale")
    assign_action(source, action)
    for frame, scale, loc_y in ((1, 1.0, 0.0), (10, 3.0, 2.0)):
        scene.frame_set(frame)
        spb.location = Vector((0.0, loc_y, 0.0))
        spb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
        spb.scale = Vector((scale, scale, scale))
        spb.keyframe_insert("location", frame=frame)
        spb.keyframe_insert("rotation_quaternion", frame=frame)
        spb.keyframe_insert("scale", frame=frame)
    bpy.ops.object.mode_set(mode="OBJECT")
    return scene, source, target


def transfer(scene, source, target, violate: bool) -> dict:
    mapping = [{"source": "root", "target": "Bone", "role": "root", "required": True}]
    src_action = source.animation_data.action if source.animation_data else None
    if source.animation_data:
        source.animation_data.action = None
    if target.animation_data:
        target.animation_data.action = None
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
    for frame in range(1, 11):
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        worker.reset_pose_to_rest(target)
        bpy.context.view_layer.update()
        spb = source.pose.bones["root"]
        tpb = target.pose.bones["Bone"]
        src_now = worker.world_pose_matrix(source, spb)
        if violate:
            apply_forbidden_full_matrix(target, tpb, src_rest["root"], src_now, tgt_rest["Bone"])
        else:
            worker.apply_root_keep_target_rest_scale(
                target, tpb, src_rest["root"], src_now, tgt_rest["Bone"]
            )
        bpy.context.view_layer.update()
        stored.append(
            (
                frame,
                list(tpb.location),
                list(tpb.rotation_quaternion),
                list(tpb.scale),
            )
        )
    baked = bpy.data.actions.new("CharacterArmatureAction")
    worker.assign_action(target, baked)
    for frame, loc, quat, _scale in stored:
        tpb = target.pose.bones["Bone"]
        tpb.location = Vector(loc)
        tpb.rotation_quaternion = Quaternion(quat)
        tpb.scale = Vector((1.0, 1.0, 1.0)) if not violate else Vector(_scale)
        tpb.keyframe_insert("location", frame=frame)
        tpb.keyframe_insert("rotation_quaternion", frame=frame)
        if violate:
            tpb.keyframe_insert("scale", frame=frame)
    audit = worker.audit_keep_target_rest_scale(
        scene, target, mapping, tgt_rest, baked, 1, 10
    )
    scene.frame_set(10)
    bpy.context.view_layer.update()
    tpb = target.pose.bones["Bone"]
    loc, _rot, world_scl = worker.world_pose_matrix(target, tpb).decompose()
    _rloc, _rrot, rest_scl = tgt_rest["Bone"].decompose()
    return {
        "audit": audit,
        "pose_scale_end": [float(tpb.scale[i]) for i in range(3)],
        "world_scale_end": [float(world_scl[i]) for i in range(3)],
        "world_location_end": [float(loc[i]) for i in range(3)],
        "rest_scale": [float(rest_scl[i]) for i in range(3)],
        "location_end": [float(tpb.location[i]) for i in range(3)],
        "source_scale_end": 3.0,
        "scale_fcurves": audit["scale_fcurves"],
        "mapped_root_included": audit["mapped_root_included"],
        "baked_action": baked.name,
    }


def main() -> int:
    args = parse_after_dash()
    mode = args[0]
    out_dir = Path(args[1])
    out_dir.mkdir(parents=True, exist_ok=True)
    try:
        if mode == "reopen":
            blend = Path(args[2])
            bpy.ops.wm.open_mainfile(filepath=str(blend))
            target = bpy.data.objects.get("TgtArm")
            if target is None:
                for obj in bpy.data.objects:
                    if obj.type == "ARMATURE":
                        target = obj
                        break
            scene = bpy.context.scene
            action = target.animation_data.action if target.animation_data else None
            bpy.context.view_layer.objects.active = target
            target.select_set(True)
            bpy.ops.object.mode_set(mode="POSE")
            if target.animation_data:
                target.animation_data.action = None
            worker.reset_pose_to_rest(target)
            bpy.context.view_layer.update()
            rest = worker.capture_rest_world(target)
            if action is not None:
                worker.assign_action(target, action)
            mapping = [{"source": "root", "target": "Bone", "role": "root", "required": True}]
            audit = worker.audit_keep_target_rest_scale(
                scene, target, mapping, rest, action, int(scene.frame_start), int(scene.frame_end)
            )
            scene.frame_set(int(scene.frame_end))
            bpy.context.view_layer.update()
            tpb = target.pose.bones["Bone"]
            payload = {
                "mode": "reopen",
                "status": "PASS" if audit["status"] == "PASS" else "FAIL",
                "root_scale_audit": audit["status"],
                "pose_scale_end": [float(tpb.scale[i]) for i in range(3)],
                "mapped_root_included": audit["mapped_root_included"],
                "scale_fcurves": audit["scale_fcurves"],
            }
            write_json(out_dir / "root_scale_reopen.json", payload)
            return 0 if audit["status"] == "PASS" else 1

        scene, source, target = setup_scene()
        violate = mode == "violate"
        result = transfer(scene, source, target, violate=violate)
        blend_path = out_dir / "root_scale.blend"
        bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
        status = result["audit"]["status"]
        if violate:
            worker_success = False
            expected = "FAIL"
        else:
            worker_success = status == "PASS"
            expected = "PASS"
        payload = {
            "mode": mode,
            "status": status,
            "expected": expected,
            "worker_success": worker_success and status == expected,
            "keep_target_rest_scale": status,
            "root_scale_audit": status,
            "mapped_root_included": result["mapped_root_included"],
            "pose_scale_end": result["pose_scale_end"],
            "world_scale_end": result["world_scale_end"],
            "world_location_end": result["world_location_end"],
            "rest_scale": result["rest_scale"],
            "location_end": result["location_end"],
            "source_scale_end": result["source_scale_end"],
            "scale_fcurves": result["scale_fcurves"],
            "staged_blend": str(blend_path),
        }
        write_json(out_dir / "root_scale_result.json", payload)
        if violate:
            return 0 if status == "FAIL" else 1
        return 0 if status == "PASS" else 1
    except Exception:
        write_json(
            out_dir / "root_scale_exception.json",
            {"traceback": traceback.format_exc()},
        )
        raise


if __name__ == "__main__":
    raise SystemExit(main())
