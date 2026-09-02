# RigForge V1-3 production Blender worker.
# Promoted execute/reopen semantics from POC-BLENDER-E2E-01 blender_worker.py.
# Not the research harness. Does not import experiments/.../run_e2e.py.
# Blender object names are execution diagnostics, not Product identity.

from __future__ import annotations

import hashlib
import json
import math
import sys
import traceback
from pathlib import Path

import bpy
from mathutils import Matrix, Quaternion, Vector

WORKER_VERSION = "rigforge-blender-worker/0.1.0"
ENVELOPE_SCHEMA = "rigforge.blender_worker.envelope.v1"
TOL_ROT_RAD = 0.01
TOL_LOC = 0.001
SCALE_TOL = 1e-4
QUAT_POST_NORM_TOL = 1e-5
SAMPLE_TIMES = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
PREFERRED_TRACE = [
    ("root", "Bone"),
    ("pelvis", "Body"),
    ("spine_03", "Torso"),
    ("upperarm_l", "UpperArm.L"),
    ("thigh_l", "UpperLeg.L"),
    ("Head", "Head"),
]
PROVEN_POLICY = {
    "root_policy": "copy_world_translation_delta",
    "channel_policy": "rotation_only_mapped_non_root",
    "quaternion_normalization_policy": "normalize_before_key",
    "quaternion_continuity_policy": "consecutive_hemisphere",
    "quaternion_interpolation_policy": "backend_interpolation_after_normalized_keys",
    "time_bake_policy": "every_source_frame",
    "rest_alignment_policy": "rest_relative_world_delta",
    "unmapped_target_joints": "remain_at_target_rest",
    "missing_source_joint": "fail_closed_do_not_invent",
    "scale_policy": "keep_target_rest_scale",
    "ik_policy": "explicit_no_ik",
}


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


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def blender_build_hash() -> str:
    h = bpy.app.build_hash
    if isinstance(h, (bytes, bytearray)):
        return h.decode("ascii")
    return str(h)


def round_list(values, nd=6):
    out = []
    for v in values:
        if isinstance(v, (int, float)):
            out.append(round(float(v), nd))
        else:
            out.append(v)
    return out


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
    for action in bpy.data.actions:
        if action.name == name or action.name.endswith(name) or name.endswith(action.name):
            return action
    needle = name.split("|")[-1]
    for action in bpy.data.actions:
        if action.name.endswith(needle):
            return action
    return None


def bone_depth(arm_obj, name: str) -> int:
    bone = arm_obj.data.bones[name]
    n = 0
    while bone.parent:
        n += 1
        bone = bone.parent
    return n


def mute_pose_constraints(obj) -> int:
    n = 0
    for pb in obj.pose.bones:
        for constraint in pb.constraints:
            constraint.mute = True
            n += 1
    return n


def quat_angular_error(a, b) -> float:
    qa = Quaternion(a).normalized()
    qb = Quaternion(b).normalized()
    d = min(1.0, max(0.0, abs(qa.dot(qb))))
    return 2.0 * math.acos(d)


def quat_list(q) -> list:
    return round_list([q.w, q.x, q.y, q.z])


def reset_pose_to_rest(obj) -> None:
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
        pb.location = Vector((0.0, 0.0, 0.0))
        pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
        pb.scale = Vector((1.0, 1.0, 1.0))


def loc_rot_without_scale(mat: Matrix) -> Matrix:
    loc, rot, _scl = mat.decompose()
    return Matrix.LocRotScale(loc, rot, Vector((1.0, 1.0, 1.0)))


def apply_root_keep_target_rest_scale(target, tpb, src_rest, src_now, tgt_rest) -> bool:
    """Transfer root translation/rotation without propagating source scale."""
    src_delta = loc_rot_without_scale(src_rest).inverted() @ loc_rot_without_scale(src_now)
    tgt_rest_ns = loc_rot_without_scale(tgt_rest)
    desired_ns = tgt_rest_ns @ src_delta
    loc, rot, _ = desired_ns.decompose()
    _tloc, _trot, tgt_rest_scl = tgt_rest.decompose()
    desired = Matrix.LocRotScale(loc, rot, tgt_rest_scl)
    if not finite_matrix(desired):
        return False
    tpb.matrix = target.matrix_world.inverted() @ desired
    tpb.scale = Vector((1.0, 1.0, 1.0))
    return True


def audit_keep_target_rest_scale(scene, target, ordered, tgt_rest, baked, f0, f1) -> dict:
    scale_fcurves = []
    for fc in iter_action_fcurves(baked):
        path = fc.data_path
        if path.endswith("scale") and 'pose.bones["' in path:
            name = path.split('pose.bones["', 1)[1].split('"]', 1)[0]
            scale_fcurves.append(name)
    max_pose_err = 0.0
    max_world_err = 0.0
    frames = sorted({int(round(f0 + u * (f1 - f0))) for u in SAMPLE_TIMES} | {int(f0), int(f1)})
    mapped = [e for e in ordered if e.get("target") in target.pose.bones]
    for frame in frames:
        scene.frame_set(int(frame))
        bpy.context.view_layer.update()
        for e in mapped:
            name = e["target"]
            tpb = target.pose.bones[name]
            pose_err = max(abs(float(tpb.scale[i]) - 1.0) for i in range(3))
            max_pose_err = max(max_pose_err, pose_err)
            _loc, _rot, now_scl = world_pose_matrix(target, tpb).decompose()
            _rloc, _rrot, rest_scl = tgt_rest[name].decompose()
            world_err = max(abs(float(now_scl[i]) - float(rest_scl[i])) for i in range(3))
            max_world_err = max(max_world_err, world_err)
    status = "PASS"
    if scale_fcurves or max_pose_err > SCALE_TOL or max_world_err > SCALE_TOL:
        status = "FAIL"
    return {
        "status": status,
        "scale_fcurves": scale_fcurves,
        "max_pose_scale_error": round(max_pose_err, 8),
        "max_world_vs_rest_scale_error": round(max_world_err, 8),
        "mapped_root_included": any(e.get("role") == "root" for e in mapped),
    }


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


def finite_matrix(mat: Matrix) -> bool:
    for row in mat:
        for value in row:
            if not math.isfinite(value):
                return False
    return True


def iter_action_fcurves(action):
    if action is None:
        return
    if hasattr(action, "layers"):
        for layer in action.layers:
            for strip in getattr(layer, "strips", []):
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


def classify_by_mapping(obj, mapping_entries) -> str | None:
    names = {b.name for b in obj.data.bones}
    required = [e for e in mapping_entries if e.get("required")]
    src_ok = all(e["source"] in names for e in required)
    tgt_ok = all(e["target"] in names for e in required)
    if tgt_ok and not src_ok:
        return "target"
    if src_ok and not tgt_ok:
        return "source"
    return None


def trace_pairs(entries):
    have = {(e["source"], e["target"]) for e in entries}
    pairs = [p for p in PREFERRED_TRACE if p in have]
    if not pairs:
        pairs = [(e["source"], e["target"]) for e in entries if e.get("required")][:6]
    return pairs


def apply_quaternion_policy(stored_keys) -> dict:
    sign_flips = 0
    keys_checked = 0
    max_post_norm_err = 0.0
    min_dot = None
    prev_q = {}
    for _frame, rec in stored_keys:
        for name, item in rec.items():
            q = Quaternion(item["rotation_quaternion"])
            mag = q.magnitude
            if mag <= 0.0:
                raise RuntimeError(f"FAIL / REPORT: zero quaternion on {name}")
            q = q.normalized()
            max_post_norm_err = max(max_post_norm_err, abs(q.magnitude - 1.0))
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
    if keys_checked <= 0 or max_post_norm_err > QUAT_POST_NORM_TOL:
        status = "FAIL"
    if min_dot is not None and min_dot < 0.0:
        status = "FAIL"
    return {
        "status": status,
        "samples_or_keys_checked": keys_checked,
        "sign_flips_applied": sign_flips,
        "post_normalization_max_abs_norm_minus_1": round(max_post_norm_err, 12),
        "minimum_consecutive_dot_after_continuity": None if min_dot is None else round(min_dot, 12),
    }


def fail_envelope(job, failure_class: str, errors: list[str]) -> dict:
    expected = job.get("expected_backend") or {}
    return {
        "schema": ENVELOPE_SCHEMA,
        "status": "FAIL",
        "failure_class": failure_class,
        "job_spec_id": job.get("job_spec_id"),
        "attempt_id": job.get("attempt_id"),
        "backend": {
            "kind": expected.get("kind", "Blender"),
            "version": bpy.app.version_string,
            "build": blender_build_hash(),
        },
        "adapter_version": WORKER_VERSION,
        "staged_blend": None,
        "measurements": [],
        "diagnostics": errors[:],
        "errors": errors,
    }


def assert_supported_policy(job: dict) -> None:
    policy = job.get("policy") or {}
    for key, expected in PROVEN_POLICY.items():
        found = policy.get(key)
        if found != expected:
            raise PolicyUnsupported(f"unsupported Policy/{key}: {found!r}")


class PolicyUnsupported(RuntimeError):
    pass


def assert_sources(job: dict) -> None:
    for key in ("character", "motion"):
        item = job[key]
        path = Path(item["path"])
        if not path.is_file():
            raise FileNotFoundError(f"missing source input: {path}")
        digest = sha256_file(path)
        expected = str(item["sha256"]).lower()
        if digest != expected:
            raise SourceDigestMismatch(f"{key} digest mismatch: expected {expected} found {digest}")


class SourceDigestMismatch(RuntimeError):
    pass


def write_envelope(job: dict, payload: dict) -> None:
    write_json(Path(job["outputs"]["result_envelope"]), payload)


def execute(job: dict) -> dict:
    try:
        assert_supported_policy(job)
    except PolicyUnsupported as exc:
        payload = fail_envelope(job, "unsupported_policy", [str(exc)])
        write_envelope(job, payload)
        raise SystemExit(1)
    expected = job.get("expected_backend") or {}
    if blender_build_hash() != expected.get("build"):
        payload = fail_envelope(
            job,
            "backend_build_mismatch",
            [f"build {blender_build_hash()} != {expected.get('build')}"],
        )
        write_envelope(job, payload)
        raise SystemExit(1)
    try:
        assert_sources(job)
    except FileNotFoundError as exc:
        payload = fail_envelope(job, "missing_source_input", [str(exc)])
        write_envelope(job, payload)
        raise SystemExit(1)
    except SourceDigestMismatch as exc:
        payload = fail_envelope(job, "source_digest_mismatch", [str(exc)])
        write_envelope(job, payload)
        raise SystemExit(1)

    mapping_entries = list((job.get("mapping") or {}).get("entries") or [])
    out = job["outputs"]
    bpy.ops.wm.read_factory_settings(use_empty=True)
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

    target = source = None
    for obj in list(bpy.data.objects):
        if obj.type != "ARMATURE":
            continue
        kind = classify_by_mapping(obj, mapping_entries)
        if kind == "target":
            target = obj
        elif kind == "source":
            source = obj
    if target is None or source is None:
        raise RuntimeError(f"armature classify failed target={target} source={source}")

    scene = bpy.context.scene
    motion = job["motion"]
    fps = int(motion["fps_num"])
    scene.render.fps = fps
    scene.render.fps_base = max(int(motion.get("fps_den") or 1), 1)
    f0 = int(motion["frame_start"])
    f1 = int(motion["frame_end"])
    scene.frame_start = f0
    scene.frame_end = f1

    tpose = None
    for name in job.get("source_rest_action_candidates") or ["A_TPose"]:
        tpose = find_action(name)
        if tpose is not None:
            break
    clip = find_action(motion["clip_id"])
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

    required = [e for e in mapping_entries if e.get("required")]
    optional = [e for e in mapping_entries if not e.get("required")]
    all_entries = required + optional
    for e in required:
        if e["source"] not in source.pose.bones:
            raise RuntimeError(f"missing required source joint: {e['source']}")
        if e["target"] not in target.pose.bones:
            raise RuntimeError(f"missing required target joint: {e['target']}")

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

    nan_count = 0
    ordered = sorted(all_entries, key=lambda e: bone_depth(target, e["target"]))
    stored_keys = []
    pairs = trace_pairs(ordered)
    sample_frames = {int(round(f0 + u * (f1 - f0))): u for u in SAMPLE_TIMES}

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
            if e.get("role") == "root":
                ok = apply_root_keep_target_rest_scale(
                    target, tpb, src_r, src_now, tgt_r
                )
                if not ok:
                    nan_count += 1
                    continue
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
            if e.get("role") == "root":
                item["location"] = list(tpb.location)
            rec[e["target"]] = item
        stored_keys.append((frame, rec))

    quat_audit = apply_quaternion_policy(stored_keys)
    if quat_audit["status"] != "PASS":
        raise RuntimeError(f"FAIL / REPORT: quaternion Policy audit {quat_audit}")

    baked = bpy.data.actions.new("CharacterArmatureAction")
    assign_action(target, baked)
    for frame, rec in stored_keys:
        for name, item in rec.items():
            tpb = target.pose.bones[name]
            tpb.rotation_quaternion = Quaternion(item["rotation_quaternion"])
            tpb.scale = Vector((1.0, 1.0, 1.0))
            tpb.keyframe_insert("rotation_quaternion", frame=frame)
            if "location" in item:
                tpb.location = Vector(item["location"])
                tpb.keyframe_insert("location", frame=frame)
    for pb in target.pose.bones:
        pb.scale = Vector((1.0, 1.0, 1.0))
    assign_action(target, baked)
    bpy.ops.object.mode_set(mode="OBJECT")

    scale_audit = audit_keep_target_rest_scale(
        scene, target, ordered, tgt_rest, baked, f0, f1
    )
    if scale_audit["status"] != "PASS":
        raise RuntimeError(f"FAIL / REPORT: keep_target_rest_scale audit {scale_audit}")

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
    audit_pass = True
    for e in ordered:
        name = e["target"]
        tpb = target.pose.bones[name]
        loc_mag = float(tpb.location.length)
        loc_keyed = bool(loc_paths.get(name))
        rot_keyed = bool(rot_paths.get(name))
        if e.get("role") == "root":
            conform = loc_keyed and rot_keyed
        else:
            conform = (not loc_keyed) and rot_keyed and loc_mag <= TOL_LOC
        if not conform:
            audit_pass = False

    samples_src = []
    samples_tgt = []
    tgt_names = [p[1] for p in pairs]
    src_names = [p[0] for p in pairs]
    for u in SAMPLE_TIMES:
        frame = int(round(f0 + u * (f1 - f0)))
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        rec_s = sample_pose(source, src_names, frame)
        rec_t = sample_pose(target, tgt_names, frame)
        rec_s["u"] = u
        rec_t["u"] = u
        samples_src.append(rec_s)
        samples_tgt.append(rec_t)

    loop_pass = True
    for src_name, tgt_name in pairs:
        src0 = next(s[src_name] for s in samples_src if s["_frame"] == f0)
        src1 = next(s[src_name] for s in samples_src if s["_frame"] == f1)
        tgt0 = next(s[tgt_name] for s in samples_tgt if s["_frame"] == f0)
        tgt1 = next(s[tgt_name] for s in samples_tgt if s["_frame"] == f1)
        if not src0.get("present") or not tgt0.get("present"):
            continue
        src_err = quat_angular_error(src0["quat"], src1["quat"])
        bake_err = quat_angular_error(tgt0["quat"], tgt1["quat"])
        if src_err <= TOL_ROT_RAD and bake_err > TOL_ROT_RAD:
            loop_pass = False

    source_armatures = [
        obj
        for obj in list(bpy.data.objects)
        if obj.type == "ARMATURE" and classify_by_mapping(obj, mapping_entries) == "source"
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

    blend_path = Path(out["staged_blend"])
    blend_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

    payload = {
        "schema": ENVELOPE_SCHEMA,
        "status": "SUCCESS",
        "failure_class": None,
        "job_spec_id": job["job_spec_id"],
        "attempt_id": job["attempt_id"],
        "backend": {
            "kind": "Blender",
            "version": bpy.app.version_string,
            "build": blender_build_hash(),
        },
        "adapter_version": WORKER_VERSION,
        "staged_blend": str(blend_path),
        "measurements": [
            {"name": "phase_execute", "value": "PASS"},
            {"name": "rotation_only_audit", "value": "PASS" if audit_pass else "FAIL"},
            {"name": "loop_closure", "value": "PASS" if loop_pass else "FAIL"},
            {"name": "quaternion_policy", "value": quat_audit["status"]},
            {"name": "keep_target_rest_scale", "value": scale_audit["status"]},
            {"name": "root_scale_audit", "value": scale_audit["status"]},
            {"name": "nan_inf_desired_matrices", "value": str(nan_count)},
            {"name": "frame_start", "value": str(f0)},
            {"name": "frame_end", "value": str(f1)},
            {"name": "fps", "value": str(fps)},
            {"name": "required_mapping_count", "value": str(len(required))},
            {"name": "target_bone_count", "value": str(len(target.data.bones))},
            {"name": "muted_pose_constraints", "value": str(muted)},
            {"name": "clip_id", "value": motion["clip_id"]},
            {"name": "baked_action", "value": baked.name},
            {
                "name": "quaternion_keys_checked",
                "value": str(quat_audit["samples_or_keys_checked"]),
            },
        ],
        "diagnostics": [
            "per-frame rest reset; rest-relative world rotation delta; "
            "ROTATION_ONLY local location=0; root loc/rot without source scale; "
            "KeepTargetRestScale; no root scale FCurves; "
            "NORMALIZE_BEFORE_KEY + CONSECUTIVE_HEMISPHERE before key insertion; "
            "no IK; no add-on; staged blend is not Product publication"
        ],
        "errors": [],
    }
    if not audit_pass or not loop_pass:
        payload["status"] = "FAIL"
        payload["failure_class"] = "structured_worker_fail"
        payload["errors"].append("execute structural audit failed")
    write_envelope(job, payload)
    write_json(Path(out["workspace"]) / "execute_measurements.json", payload["measurements"])
    return payload


def reopen(job: dict) -> dict:
    blend = Path(job["outputs"]["staged_blend"])
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    mapping_entries = list((job.get("mapping") or {}).get("entries") or [])
    target = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_by_mapping(obj, mapping_entries) == "target":
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
    pairs = trace_pairs(mapping_entries)
    tgt_names = [p[1] for p in pairs] or list(target.pose.bones.keys())[:6]
    samples = []
    for u in SAMPLE_TIMES:
        frame = int(round(f0 + u * (f1 - f0)))
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        rec = sample_pose(target, tgt_names, frame)
        rec["u"] = u
        samples.append(rec)
    root_scale_audit = "FAIL"
    if action is not None:
        bpy.context.view_layer.objects.active = target
        target.select_set(True)
        bpy.ops.object.mode_set(mode="POSE")
        if target.animation_data:
            target.animation_data.action = None
        reset_pose_to_rest(target)
        bpy.context.view_layer.update()
        rest = capture_rest_world(target)
        assign_action(target, action)
        bpy.ops.object.mode_set(mode="OBJECT")
        scale_audit = audit_keep_target_rest_scale(
            scene, target, mapping_entries, rest, action, f0, f1
        )
        root_scale_audit = scale_audit["status"]
    payload = {
        "status": "SUCCESS" if action is not None and root_scale_audit == "PASS" else "FAIL",
        "target_present": True,
        "baked_action": action.name if action else None,
        "frame_start": f0,
        "frame_end": f1,
        "fps": scene.render.fps,
        "bone_count": len(target.data.bones),
        "root_scale_audit": root_scale_audit,
        "samples_target": samples,
        "diagnostics": ["fresh-process reopen; execution scene was not reused"],
    }
    write_json(Path(job["outputs"]["reopen_verification"]), payload)
    return payload


def inspect_armature(arm_obj):
    joints = []
    for bone in arm_obj.data.bones:
        parent = bone.parent.name if bone.parent is not None else None
        deform = "deforming" if bool(getattr(bone, "use_deform", True)) else "helper"
        rest = "head=({:.6f},{:.6f},{:.6f}) tail=({:.6f},{:.6f},{:.6f})".format(
            bone.head_local[0],
            bone.head_local[1],
            bone.head_local[2],
            bone.tail_local[0],
            bone.tail_local[1],
            bone.tail_local[2],
        )
        joints.append(
            {
                "joint_key": bone.name,
                "display_name": bone.name,
                "parent_key": parent,
                "is_root": parent is None,
                "deform_observation": deform,
                "rest_evidence": rest,
            }
        )
    joints.sort(key=lambda item: item["joint_key"])
    return joints


def inspect_skeleton(job: dict) -> dict:
    expected = job.get("expected_digest")
    source = Path(job["source_path"])
    if not source.is_file():
        raise FileNotFoundError(str(source))
    digest = sha256_file(source)
    if expected and digest != expected:
        raise RuntimeError(f"source digest mismatch {digest} != {expected}")
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(
        filepath=str(source),
        use_anim=False,
        automatic_bone_orientation=False,
    )
    armatures = [obj for obj in bpy.data.objects if obj.type == "ARMATURE"]
    if not armatures:
        raise RuntimeError("no armature found")
    arm = max(armatures, key=lambda obj: len(obj.data.bones))
    joints = inspect_armature(arm)
    payload = {
        "status": "SUCCESS",
        "kind": "skeleton_observation",
        "armature_display_name": arm.name,
        "joint_count": len(joints),
        "joints": joints,
        "source_digest": digest,
        "diagnostics": [
            "inspect only; no retarget execution",
            "joint_key is source-local evidence, not Product identity",
        ],
    }
    write_json(Path(job["outputs"]["inspect_envelope"]), payload)
    return payload


def inspect_qc(job: dict) -> dict:
    """Read-only Product QC inspection of persisted bytes. Must not save."""
    blend = Path(job["artifact_path"])
    expected = str(job.get("expected_digest") or "").lower()
    envelope_path = Path(job["outputs"]["inspect_envelope"])
    if not blend.is_file():
        payload = {
            "status": "FAIL",
            "failure_class": "missing_artifact",
            "digest_before": None,
            "digest_after": None,
            "structurally_readable": False,
            "finite_transforms": False,
            "present_joint_keys": [],
            "baked_animation_present": False,
            "duration_s": None,
            "gross_scale_sane": False,
            "root_trajectory_sane": False,
            "saved": False,
            "diagnostics": ["persisted artifact file is missing"],
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    digest_before = sha256_file(blend)
    if expected and digest_before != expected:
        payload = {
            "status": "FAIL",
            "failure_class": "digest_mismatch",
            "digest_before": digest_before,
            "digest_after": digest_before,
            "structurally_readable": False,
            "finite_transforms": False,
            "present_joint_keys": [],
            "baked_animation_present": False,
            "duration_s": None,
            "gross_scale_sane": False,
            "root_trajectory_sane": False,
            "saved": False,
            "diagnostics": [f"digest mismatch expected {expected} found {digest_before}"],
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    mapping_entries = list((job.get("mapping") or {}).get("entries") or [])
    target = None
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE" and classify_by_mapping(obj, mapping_entries) == "target":
            target = obj
            break
    if target is None:
        for obj in bpy.data.objects:
            if obj.type == "ARMATURE":
                target = obj
                break
    structurally_readable = target is not None
    present_joint_keys = []
    finite_transforms = True
    baked_animation_present = False
    duration_s = None
    gross_scale_sane = False
    root_trajectory_sane = False
    diagnostics = [
        "inspect_qc is read-only; must not save the .blend",
        "QC evaluates persisted bytes, not WorkerResult measurements",
    ]
    if target is not None:
        present_joint_keys = [bone.name for bone in target.data.bones]
        scene = bpy.context.scene
        f0 = int(scene.frame_start)
        f1 = int(scene.frame_end)
        fps = float(scene.render.fps) / float(max(1, scene.render.fps_base))
        if fps > 0:
            duration_s = abs(f1 - f0) / fps
        action = None
        if target.animation_data:
            action = target.animation_data.action
        baked_animation_present = action is not None
        if action is not None:
            assign_action(target, action)
        bpy.context.view_layer.objects.active = target
        target.select_set(True)
        bpy.ops.object.mode_set(mode="POSE")
        frames = sorted({int(f0), int(f1), int(round((f0 + f1) / 2))})
        max_scale_err = 0.0
        root_name = None
        for bone in target.data.bones:
            if bone.parent is None:
                root_name = bone.name
                break
        root_locs = []
        for frame in frames:
            scene.frame_set(int(frame))
            bpy.context.view_layer.update()
            for pb in target.pose.bones:
                mat = world_pose_matrix(target, pb)
                if not finite_matrix(mat):
                    finite_transforms = False
                loc, _rot, scl = mat.decompose()
                for component in (loc.x, loc.y, loc.z, scl.x, scl.y, scl.z):
                    if not math.isfinite(component):
                        finite_transforms = False
                pose_err = max(abs(float(pb.scale[i]) - 1.0) for i in range(3))
                max_scale_err = max(max_scale_err, pose_err)
                if pb.name == root_name:
                    root_locs.append((float(loc.x), float(loc.y), float(loc.z)))
        bpy.ops.object.mode_set(mode="OBJECT")
        gross_scale_sane = finite_transforms and max_scale_err < 10.0
        root_trajectory_sane = finite_transforms and len(root_locs) >= 1
        if root_trajectory_sane and len(root_locs) >= 2:
            dx = root_locs[-1][0] - root_locs[0][0]
            dy = root_locs[-1][1] - root_locs[0][1]
            dz = root_locs[-1][2] - root_locs[0][2]
            dist = math.sqrt(dx * dx + dy * dy + dz * dz)
            root_trajectory_sane = math.isfinite(dist) and dist < 1.0e6
        diagnostics.append(f"target={target.name}")
        diagnostics.append(f"baked_action={action.name if action else None}")
        diagnostics.append(f"max_pose_scale_error={round(max_scale_err, 8)}")
    digest_after = sha256_file(blend)
    if digest_after != digest_before:
        finite_transforms = False
        diagnostics.append("digest changed during inspect_qc; inspection must not mutate bytes")
    payload = {
        "status": "SUCCESS" if structurally_readable else "FAIL",
        "digest_before": digest_before,
        "digest_after": digest_after,
        "structurally_readable": structurally_readable,
        "finite_transforms": finite_transforms,
        "present_joint_keys": present_joint_keys,
        "baked_animation_present": baked_animation_present,
        "duration_s": duration_s,
        "gross_scale_sane": gross_scale_sane,
        "root_trajectory_sane": root_trajectory_sane,
        "saved": False,
        "diagnostics": diagnostics,
    }
    write_json(envelope_path, payload)
    if digest_after != digest_before:
        raise SystemExit(1)
    return payload


def main() -> int:
    args = parse_after_dash()
    mode = args[0]
    job = load_json(Path(args[1]))
    try:
        if mode == "execute":
            execute(job)
        elif mode == "reopen":
            reopen(job)
        elif mode == "inspect":
            inspect_skeleton(job)
        elif mode == "inspect_qc":
            inspect_qc(job)
        else:
            raise SystemExit(f"unknown mode {mode}")
        return 0
    except SystemExit:
        raise
    except Exception:
        workspace = Path((job.get("outputs") or {}).get("workspace") or ".")
        write_json(
            workspace / "worker_exception.json",
            {"status": "FAIL", "mode": mode, "traceback": traceback.format_exc()},
        )
        if mode == "execute":
            payload = fail_envelope(
                job,
                "worker_script_exception",
                [traceback.format_exc().splitlines()[-1]],
            )
            payload["diagnostics"].append(traceback.format_exc())
            write_envelope(job, payload)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
