# Frozen-pair semantic acceptance. Test evidence only. Not Product identity.
# Uses Mapping roles + rest world positions. Does not parse asset-specific names
# as production authority; joint keys come from the accepted Mapping projection.

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

FRAMES = (1, 16, 31, 46, 61)


def parse_after_dash():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit("missing -- args")
    return argv[argv.index("--") + 1 :]


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def corr(a, b) -> float:
    n = len(a)
    if n < 2:
        return 0.0
    ma = sum(a) / n
    mb = sum(b) / n
    num = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    da = math.sqrt(sum((x - ma) ** 2 for x in a))
    db = math.sqrt(sum((y - mb) ** 2 for y in b))
    if da < 1e-12 or db < 1e-12:
        return 0.0
    return num / (da * db)


def frame_axes(mat: Matrix):
    return Vector(mat.col[0]), Vector(mat.col[1]), Vector(mat.col[2])


def offset_in_frame(point: Vector, origin: Vector, right, forward, up):
    d = point - origin
    return {
        "forward": float(d.dot(forward)),
        "lateral": float(d.dot(right)),
        "vertical": float(d.dot(up)),
    }


def torso_lean(chest: Vector, pelvis: Vector, forward, up) -> float:
    v = chest - pelvis
    return math.degrees(math.atan2(float(v.dot(forward)), float(v.dot(up))))


def knee_forwardness(hip, knee, ankle, forward) -> float:
    chord = ankle - hip
    if chord.length < 1e-8:
        return 0.0
    n = chord.normalized()
    off = (knee - hip) - n * float((knee - hip).dot(n))
    return float(off.dot(forward))


def pair_role(entries, role: str) -> list:
    found = [e for e in entries if e.get("role") == role]
    return found


def unique_role(entries, role: str):
    found = pair_role(entries, role)
    return found[0] if len(found) == 1 else None


def loc(obj, name) -> Vector:
    return worker.world_pose_matrix(obj, obj.pose.bones[name]).to_translation()


def classify(obj, entries):
    return worker.classify_by_mapping(obj, entries)


def set_camera(origin, right, up, forward, kind: str):
    cam_data = bpy.data.cameras.new("DiagCam")
    cam = bpy.data.objects.new("DiagCam", cam_data)
    bpy.context.collection.objects.link(cam)
    bpy.context.scene.camera = cam
    dist = 3.4
    if kind == "side":
        cam.location = origin + right * dist + up * 0.15
    else:
        cam.location = origin - forward * dist + up * 0.2
    direction = origin - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    cam_data.lens = 50
    return cam


def ensure_lit():
    if not any(obj.type == "LIGHT" for obj in bpy.data.objects):
        light = bpy.data.lights.new("DiagSun", "SUN")
        light.energy = 3.0
        sun = bpy.data.objects.new("DiagSun", light)
        sun.rotation_euler = (math.radians(45.0), 0.0, math.radians(35.0))
        bpy.context.collection.objects.link(sun)
    world = bpy.context.scene.world
    if world is None:
        world = bpy.data.worlds.new("DiagWorld")
        bpy.context.scene.world = world
    world.use_nodes = True
    bg = world.node_tree.nodes.get("Background")
    if bg is not None:
        bg.inputs[0].default_value = (0.18, 0.18, 0.2, 1.0)
        bg.inputs[1].default_value = 0.4


def render_png(path: Path, engine="BLENDER_EEVEE"):
    ensure_lit()
    scene = bpy.context.scene
    scene.render.engine = engine
    scene.render.resolution_x = 960
    scene.render.resolution_y = 720
    scene.render.filepath = str(path)
    scene.render.image_settings.file_format = "PNG"
    bpy.ops.render.render(write_still=True)


def sample_side(obj, entries, rest, right, forward, up, frames, clip_action):
    pelvis_e = unique_role(entries, "pelvis")
    chest_e = unique_role(entries, "spine_03") or unique_role(entries, "neck")
    hands = pair_role(entries, "hand")
    thighs = pair_role(entries, "thigh")
    shins = pair_role(entries, "shin")
    feet = pair_role(entries, "foot")
    if pelvis_e is None or chest_e is None or len(hands) != 2 or len(feet) != 2 or len(thighs) != 2:
        raise RuntimeError("mapping lacks semantic landmarks for acceptance metrics")
    key = "source" if classify(obj, entries) == "source" else "target"
    pelvis_n = pelvis_e[key]
    chest_n = chest_e[key]
    hand_names = [h[key] for h in hands]
    foot_names = [f[key] for f in feet]
    thigh_names = [t[key] for t in thighs]
    shin_names = [s[key] for s in shins] if len(shins) == 2 else []
    if clip_action is not None:
        worker.assign_action(obj, clip_action)
    out = []
    scene = bpy.context.scene
    for frame in frames:
        scene.frame_set(int(frame))
        bpy.context.view_layer.update()
        pelvis = loc(obj, pelvis_n)
        chest = loc(obj, chest_n)
        rec = {
            "frame": frame,
            "lean": torso_lean(chest, pelvis, forward, up),
            "hands": [],
            "feet_fwd": [],
            "knees": [],
        }
        for hn in hand_names:
            rec["hands"].append(offset_in_frame(loc(obj, hn), chest, right, forward, up))
        for fn in foot_names:
            rec["feet_fwd"].append(offset_in_frame(loc(obj, fn), pelvis, right, forward, up)["forward"])
        if shin_names:
            for th, sh, ft in zip(thigh_names, shin_names, foot_names):
                rec["knees"].append(
                    knee_forwardness(loc(obj, th), loc(obj, sh), loc(obj, ft), forward)
                )
        out.append(rec)
    return {
        "samples": out,
        "hand_names": hand_names,
        "foot_names": foot_names,
        "key": key,
    }


def signs_match(src, tgt) -> bool:
    if abs(src) < 1e-4 and abs(tgt) < 1e-4:
        return True
    return (src >= 0) == (tgt >= 0)


def evaluate(src, tgt) -> dict:
    src_s = src["samples"]
    tgt_s = tgt["samples"]
    lean_src = [s["lean"] for s in src_s]
    lean_tgt = [s["lean"] for s in tgt_s]
    lean_ok = all(signs_match(a, b) and abs(a - b) < 8.0 for a, b in zip(lean_src, lean_tgt))
    hand_fwd_ok = True
    hand_vert_ok = True
    lr_ok = True
    for a, b in zip(src_s, tgt_s):
        for hs, ht in zip(a["hands"], b["hands"]):
            if not signs_match(hs["forward"], ht["forward"]) or hs["forward"] <= 0 or ht["forward"] <= 0:
                hand_fwd_ok = False
            if not signs_match(hs["vertical"], ht["vertical"]):
                hand_vert_ok = False
        if len(a["hands"]) == 2:
            if not signs_match(a["hands"][0]["lateral"], b["hands"][0]["lateral"]):
                lr_ok = False
            if not signs_match(a["hands"][1]["lateral"], b["hands"][1]["lateral"]):
                lr_ok = False
    step = []
    amp = []
    for i in range(len(src_s[0]["feet_fwd"])):
        sa = [s["feet_fwd"][i] for s in src_s]
        ta = [s["feet_fwd"][i] for s in tgt_s]
        step.append(corr(sa, ta))
        amp.append(max(ta) - min(ta))
    step_ok = all(v > 0.90 for v in step)
    amp_ok = all(v >= 0.40 for v in amp)
    knee_ok = True
    knee_corr = []
    if src_s[0]["knees"]:
        for i in range(len(src_s[0]["knees"])):
            sa = [s["knees"][i] for s in src_s]
            ta = [s["knees"][i] for s in tgt_s]
            knee_corr.append(corr(sa, ta))
            if any(not signs_match(x, y) for x, y in zip(sa, ta)):
                knee_ok = False
        if any(v <= 0.90 for v in knee_corr):
            knee_ok = False
    semantic = all([hand_fwd_ok, hand_vert_ok, lean_ok, step_ok, knee_ok, lr_ok, amp_ok])
    return {
        "HAND_FORWARDNESS": "PASS" if hand_fwd_ok else "FAIL",
        "HAND_VERTICAL_SEMANTICS": "PASS" if hand_vert_ok else "FAIL",
        "TORSO_LEAN_SIGN": "PASS" if lean_ok else "FAIL",
        "STEP_DIRECTION_SIGN": "PASS" if step_ok else "FAIL",
        "KNEE_BEND_DIRECTION": "PASS" if knee_ok else "FAIL",
        "LEFT_RIGHT_PRESERVATION": "PASS" if lr_ok else "FAIL",
        "FOOT_SWING_AMPLITUDE": "PASS" if amp_ok else "FAIL",
        "SEMANTIC_FROZEN_PAIR": "PASS" if semantic else "FAIL",
        "lean_source": [round(v, 4) for v in lean_src],
        "lean_derived": [round(v, 4) for v in lean_tgt],
        "step_correlation": [round(v, 6) for v in step],
        "foot_amplitude": [round(v, 6) for v in amp],
        "knee_correlation": [round(v, 6) for v in knee_corr],
        "source_hands_frame1": src_s[0]["hands"],
        "derived_hands_frame1": tgt_s[0]["hands"],
    }


def import_fbx(path: str, anim: bool):
    bpy.ops.import_scene.fbx(
        filepath=path,
        use_anim=anim,
        automatic_bone_orientation=False,
    )


def find_armature(entries, kind: str):
    for obj in list(bpy.data.objects):
        if obj.type == "ARMATURE" and classify(obj, entries) == kind:
            return obj
    return None


def retarget_rm(character_path, rm_path, entries, clip_id, out_dir: Path) -> dict:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    import_fbx(character_path, False)
    import_fbx(rm_path, True)
    target = find_armature(entries, "target")
    source = find_armature(entries, "source")
    if target is None or source is None:
        raise RuntimeError(f"RM classify failed target={target} source={source}")
    scene = bpy.context.scene
    scene.render.fps = 30
    scene.frame_start = 1
    scene.frame_end = 61
    tpose = None
    for name in ("Armature|Armature|A_TPose", "A_TPose"):
        tpose = worker.find_action(name)
        if tpose is not None:
            break
    clip = worker.find_action(clip_id)
    if tpose is None or clip is None:
        raise RuntimeError(f"RM missing actions tpose={tpose} clip={clip}")
    worker.reset_pose_to_rest(target)
    bpy.context.view_layer.update()
    tgt_rest = worker.capture_rest_world(target)
    worker.assign_action(source, tpose)
    scene.frame_set(1)
    bpy.context.view_layer.update()
    src_rest = worker.capture_rest_world(source)
    resolved = worker.resolve_body_alignment(entries, src_rest, tgt_rest)
    body_a = resolved["alignment_quat"]
    worker.assign_action(source, clip)
    ordered = sorted(entries, key=lambda e: worker.bone_depth(target, e["target"]))
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.mode_set(mode="POSE")
    root_e = unique_role(entries, "root")
    samples = []
    for frame in FRAMES:
        scene.frame_set(int(frame))
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
                worker.apply_root_keep_target_rest_scale(
                    target, tpb, src_rest[e["source"]], src_now, tgt_rest[e["target"]], body_a
                )
            else:
                worker.apply_rest_relative_world_rotation(
                    target, tpb, src_rest[e["source"]], src_now, tgt_rest[e["target"]], body_a
                )
            bpy.context.view_layer.update()
        src_root = loc(source, root_e["source"])
        tgt_root = loc(target, root_e["target"])
        samples.append({"frame": frame, "source": [float(c) for c in src_root], "target": [float(c) for c in tgt_root]})
    bpy.ops.object.mode_set(mode="OBJECT")
    _r, tgt_fwd, _u = frame_axes(resolved["target_frame"])
    src0 = Vector(samples[0]["source"])
    src1 = Vector(samples[-1]["source"])
    tgt0 = Vector(samples[0]["target"])
    tgt1 = Vector(samples[-1]["target"])
    src_delta = src1 - src0
    tgt_delta = tgt1 - tgt0
    mag = float(tgt_delta.length)
    fwd = float(tgt_delta.dot(tgt_fwd))
    ok = mag > 1.0 and fwd > 0.8 * mag
    return {
        "status": "PASS" if ok else "FAIL",
        "source_delta": [round(float(c), 6) for c in src_delta],
        "target_delta": [round(float(c), 6) for c in tgt_delta],
        "target_forward_component": round(fwd, 6),
        "magnitude": round(mag, 6),
        "alignment_angle_deg": round(resolved["alignment_angle_deg"], 4),
        "samples": samples,
    }


def main() -> int:
    args = parse_after_dash()
    out_dir = Path(args[0])
    character = args[1]
    motion = args[2]
    derived = args[3]
    mapping_path = Path(args[4])
    clip_id = args[5]
    rm_path = args[6] if len(args) > 6 else ""
    entries = json.loads(mapping_path.read_text(encoding="utf-8"))
    out_dir.mkdir(parents=True, exist_ok=True)
    try:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        import_fbx(character, False)
        import_fbx(motion, True)
        source = find_armature(entries, "source")
        target_ref = find_armature(entries, "target")
        if source is None or target_ref is None:
            raise RuntimeError("import classify failed")
        tpose = None
        for name in ("Armature|Armature|A_TPose", "A_TPose"):
            tpose = worker.find_action(name)
            if tpose is not None:
                break
        clip = worker.find_action(clip_id)
        worker.assign_action(source, tpose)
        bpy.context.scene.frame_set(1)
        bpy.context.view_layer.update()
        src_rest = worker.capture_rest_world(source)
        worker.reset_pose_to_rest(target_ref)
        bpy.context.view_layer.update()
        tgt_rest_import = worker.capture_rest_world(target_ref)
        resolved = worker.resolve_body_alignment(entries, src_rest, tgt_rest_import)
        src_right, src_fwd, src_up = frame_axes(resolved["source_frame"])
        tgt_right, tgt_fwd, tgt_up = frame_axes(resolved["target_frame"])
        worker.assign_action(source, clip)
        src_series = sample_side(source, entries, src_rest, src_right, src_fwd, src_up, FRAMES, clip)

        bpy.ops.wm.open_mainfile(filepath=derived)
        derived_arm = None
        for obj in bpy.data.objects:
            if obj.type == "ARMATURE":
                derived_arm = obj
                break
        if derived_arm is None:
            raise RuntimeError("derived blend has no armature")
        bpy.context.view_layer.objects.active = derived_arm
        derived_arm.select_set(True)
        bpy.ops.object.mode_set(mode="POSE")
        if derived_arm.animation_data and derived_arm.animation_data.action:
            worker.assign_action(derived_arm, derived_arm.animation_data.action)
        tgt_series = sample_side(
            derived_arm, entries, None, tgt_right, tgt_fwd, tgt_up, FRAMES, None
        )
        metrics = evaluate(src_series, tgt_series)
        metrics["alignment_angle_deg"] = round(resolved["alignment_angle_deg"], 4)
        metrics["alignment_quat"] = [
            round(float(c), 8)
            for c in (
                resolved["alignment_quat"].w,
                resolved["alignment_quat"].x,
                resolved["alignment_quat"].y,
                resolved["alignment_quat"].z,
            )
        ]
        metrics["source_determinant"] = round(resolved["source_determinant"], 6)
        metrics["target_determinant"] = round(resolved["target_determinant"], 6)
        metrics["alignment_determinant"] = round(resolved["alignment_determinant"], 6)
        metrics["laterality_roles"] = resolved["laterality_roles"]
        metrics["name_heuristics_used"] = False
        metrics["motion_used_for_body_frame"] = False

        hips = loc(derived_arm, unique_role(entries, "pelvis")["target"])
        bpy.ops.object.mode_set(mode="OBJECT")
        for kind in ("side", "front"):
            for obj in list(bpy.data.objects):
                if obj.type == "CAMERA":
                    bpy.data.objects.remove(obj, do_unlink=True)
            set_camera(hips, tgt_right, tgt_up, tgt_fwd, kind)
            try:
                render_png(out_dir / f"derived_{kind}.png")
            except Exception as exc:
                metrics[f"render_{kind}"] = str(exc)

        rm = None
        if rm_path:
            rm = retarget_rm(character, rm_path, entries, clip_id, out_dir)

        payload = {
            "status": metrics["SEMANTIC_FROZEN_PAIR"],
            "metrics": metrics,
            "rm_root": rm,
            "frames": list(FRAMES),
            "adapter_version": worker.WORKER_VERSION,
        }
        write_json(out_dir / "semantic_acceptance.json", payload)
        if payload["status"] != "PASS":
            return 1
        if rm is not None and rm["status"] != "PASS":
            return 1
        return 0
    except Exception:
        write_json(out_dir / "semantic_exception.json", {"traceback": traceback.format_exc()})
        raise


if __name__ == "__main__":
    raise SystemExit(main())
