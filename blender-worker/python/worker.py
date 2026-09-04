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
from fractions import Fraction
from pathlib import Path

import bpy
from mathutils import Matrix, Quaternion, Vector

WORKER_VERSION = "rigforge-blender-worker/0.1.2"
ENVELOPE_SCHEMA = "rigforge.blender_worker.envelope.v1"
SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))
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


def finite_quat(q) -> bool:
    return all(math.isfinite(float(v)) for v in (q.w, q.x, q.y, q.z))


class BodyFrameError(RuntimeError):
    """Fail-closed body-semantic frame / alignment construction."""

    def __init__(self, reason: str, details: dict | None = None):
        super().__init__(reason)
        self.reason = reason
        self.details = details or {}


BODY_FRAME_COLLAPSE = 1e-5
BODY_FRAME_COLLINEAR = 0.995
BODY_FRAME_DET_TOL = 1e-3
BILATERAL_ROLES = ("thigh", "upper_arm", "clavicle")
UP_LOWER_ROLES = ("pelvis", "pelvis_central", "spine_01")
UP_UPPER_ROLES = ("spine_03", "neck", "head", "spine_02")
TORSO_CHAIN_ROLES = {
    "spine",
    "spine_01",
    "spine_02",
    "spine_03",
    "neck",
    "head",
}


def identity_quat() -> Quaternion:
    return Quaternion((1.0, 0.0, 0.0, 0.0))


def _as_vector(value) -> Vector:
    if isinstance(value, Vector):
        return Vector((float(value.x), float(value.y), float(value.z)))
    return Vector((float(value[0]), float(value[1]), float(value[2])))


def _finite_vec(v: Vector) -> bool:
    return all(math.isfinite(float(c)) for c in (v.x, v.y, v.z))


def _normalize(v: Vector) -> Vector | None:
    if not _finite_vec(v) or float(v.length) < BODY_FRAME_COLLAPSE:
        return None
    return v.normalized()


def rest_translation(mat: Matrix) -> Vector:
    loc, _rot, _scl = mat.decompose()
    return Vector((float(loc.x), float(loc.y), float(loc.z)))


def entries_with_role(mapping_entries, role: str) -> list:
    return [e for e in mapping_entries if e.get("role") == role]


def unique_role_entry(mapping_entries, role: str):
    found = entries_with_role(mapping_entries, role)
    if not found:
        return None
    if len(found) != 1:
        raise BodyFrameError(
            f"role {role!r} is not unique ({len(found)} entries)",
            {"role": role, "count": len(found)},
        )
    return found[0]


def _lex_key(v: Vector) -> tuple:
    return (round(float(v.x), 8), round(float(v.y), 8), round(float(v.z), 8))


def paired_laterality_vector(mapping_entries, role: str, src_rest: dict, tgt_rest: dict):
    """Map two same-role joints to a signed laterality vector without bone names.

    The source rest positions are ordered lexicographically. The mapped target
    joints follow that pairing so source and target share one laterality sign.
    Swapping the order on both skeletons leaves A invariant.
    """
    found = entries_with_role(mapping_entries, role)
    if len(found) != 2:
        return None
    a, b = found[0], found[1]
    sa, sb = a["source"], b["source"]
    ta, tb = a["target"], b["target"]
    if sa not in src_rest or sb not in src_rest or ta not in tgt_rest or tb not in tgt_rest:
        raise BodyFrameError(
            f"missing rest matrices for bilateral role {role!r}",
            {"role": role},
        )
    pa = rest_translation(src_rest[sa])
    pb = rest_translation(src_rest[sb])
    if _lex_key(pa) == _lex_key(pb):
        raise BodyFrameError(
            "left/right landmarks collapse to the same point",
            {"role": role},
        )
    if _lex_key(pa) > _lex_key(pb):
        src_plus, src_minus = pa, pb
        tgt_plus = rest_translation(tgt_rest[ta])
        tgt_minus = rest_translation(tgt_rest[tb])
    else:
        src_plus, src_minus = pb, pa
        tgt_plus = rest_translation(tgt_rest[tb])
        tgt_minus = rest_translation(tgt_rest[ta])
    src_raw = src_plus - src_minus
    tgt_raw = tgt_plus - tgt_minus
    src_n = _normalize(src_raw)
    tgt_n = _normalize(tgt_raw)
    if src_n is None or tgt_n is None:
        raise BodyFrameError(
            "left/right landmarks collapse to the same point",
            {"role": role},
        )
    return {
        "role": role,
        "source": src_n,
        "target": tgt_n,
    }


def combine_laterality(parts: list) -> tuple[Vector, Vector, list[str]]:
    if not parts:
        raise BodyFrameError(
            "required semantic landmarks are missing",
            {"missing": "bilateral_pair"},
        )
    src_ref = parts[0]["source"]
    tgt_ref = parts[0]["target"]
    for part in parts[1:]:
        src_dot = float(src_ref.dot(part["source"]))
        tgt_dot = float(tgt_ref.dot(part["target"]))
        if src_dot < 0.0 or tgt_dot < 0.0:
            raise BodyFrameError(
                "bilateral landmark axes disagree; reflection/mirror is unsupported",
                {
                    "roles": [p["role"] for p in parts],
                    "source_dot": src_dot,
                    "target_dot": tgt_dot,
                    "reflection_detected": True,
                },
            )
    src_sum = Vector((0.0, 0.0, 0.0))
    tgt_sum = Vector((0.0, 0.0, 0.0))
    for part in parts:
        src_sum += part["source"]
        tgt_sum += part["target"]
    src_n = _normalize(src_sum)
    tgt_n = _normalize(tgt_sum)
    if src_n is None or tgt_n is None:
        raise BodyFrameError("combined laterality vector is degenerate")
    return src_n, tgt_n, [p["role"] for p in parts]


def central_up_vector(mapping_entries, rest: dict, side: str) -> tuple[Vector, str, str]:
    lower = None
    lower_role = None
    for role in UP_LOWER_ROLES:
        entry = unique_role_entry(mapping_entries, role)
        if entry is not None:
            lower = entry
            lower_role = role
            break
    if lower is None:
        raise BodyFrameError(
            "required semantic landmarks are missing",
            {"missing": "central_up_lower"},
        )
    upper = None
    upper_role = None
    for role in UP_UPPER_ROLES:
        entry = unique_role_entry(mapping_entries, role)
        if entry is not None and entry[side] != lower[side]:
            upper = entry
            upper_role = role
            break
    if upper is None:
        candidates = [
            e
            for e in mapping_entries
            if e.get("role") in TORSO_CHAIN_ROLES and e[side] != lower[side]
        ]
        if not candidates:
            raise BodyFrameError(
                "required semantic landmarks are missing",
                {"missing": "central_up_upper", "lower": lower_role},
            )
        lower_pos = rest_translation(rest[lower[side]])

        def height_key(entry):
            return float((rest_translation(rest[entry[side]]) - lower_pos).length)

        upper = max(candidates, key=height_key)
        upper_role = str(upper.get("role") or "torso")
    p0 = rest_translation(rest[lower[side]])
    p1 = rest_translation(rest[upper[side]])
    raw = p1 - p0
    if not _finite_vec(raw):
        raise BodyFrameError("up vector is non-finite")
    return raw, lower_role, upper_role


def frame_from_right_up(right: Vector, up_raw: Vector, forward_mode: str = "up_cross_right") -> Matrix:
    right_n = _normalize(right)
    if right_n is None:
        raise BodyFrameError("laterality vector is degenerate")
    up_ortho = up_raw - right_n * float(up_raw.dot(right_n))
    up_n = _normalize(up_ortho)
    if up_n is None:
        raise BodyFrameError("up vector is degenerate after removing the laterality component")
    if abs(float(right_n.dot(up_n))) > BODY_FRAME_COLLINEAR:
        raise BodyFrameError("up and right are nearly collinear")
    if forward_mode == "up_cross_right":
        forward = _normalize(up_n.cross(right_n))
        if forward is None:
            raise BodyFrameError("forward vector is degenerate")
        columns = (right_n, forward, up_n)
    elif forward_mode == "right_cross_up":
        forward = _normalize(right_n.cross(up_n))
        if forward is None:
            raise BodyFrameError("forward vector is degenerate")
        columns = (right_n, up_n, forward)
    else:
        raise BodyFrameError(f"unknown forward_mode {forward_mode!r}")
    mat = Matrix(
        (
            (columns[0].x, columns[1].x, columns[2].x),
            (columns[0].y, columns[1].y, columns[2].y),
            (columns[0].z, columns[1].z, columns[2].z),
        )
    )
    if not finite_matrix(mat):
        raise BodyFrameError("body frame is non-finite")
    det = float(mat.determinant())
    if det < 0.0:
        raise BodyFrameError(
            "body frame determinant is negative; reflection is unsupported",
            {"determinant": det, "reflection_detected": True},
        )
    if abs(det - 1.0) > BODY_FRAME_DET_TOL:
        raise BodyFrameError(
            "body frame determinant is invalid",
            {"determinant": det},
        )
    return mat


def alignment_from_frames(source_frame: Matrix, target_frame: Matrix) -> Matrix:
    a = target_frame @ source_frame.inverted()
    if not finite_matrix(a):
        raise BodyFrameError("alignment A is non-finite")
    det = float(a.determinant())
    if det < 0.0:
        raise BodyFrameError(
            "alignment A is an improper rotation / reflection",
            {"determinant": det, "reflection_detected": True},
        )
    if abs(det - 1.0) > BODY_FRAME_DET_TOL:
        raise BodyFrameError(
            "alignment A is not a proper rotation",
            {"determinant": det},
        )
    return a


def resolve_body_alignment(
    mapping_entries,
    src_rest: dict,
    tgt_rest: dict,
    *,
    forward_mode: str = "up_cross_right",
) -> dict:
    """Rest body frames from Mapping roles + rest world positions. No clip, no names."""
    parts = []
    used = []
    for role in BILATERAL_ROLES:
        part = paired_laterality_vector(mapping_entries, role, src_rest, tgt_rest)
        if part is not None:
            parts.append(part)
            used.append(role)
    src_right, tgt_right, laterality_roles = combine_laterality(parts)
    src_up_raw, src_lower, src_upper = central_up_vector(mapping_entries, src_rest, "source")
    tgt_up_raw, tgt_lower, tgt_upper = central_up_vector(mapping_entries, tgt_rest, "target")
    source_frame = frame_from_right_up(src_right, src_up_raw, forward_mode)
    target_frame = frame_from_right_up(tgt_right, tgt_up_raw, forward_mode)
    a = alignment_from_frames(source_frame, target_frame)
    aq = a.to_quaternion().normalized()
    if not finite_quat(aq):
        raise BodyFrameError("alignment quaternion is non-finite")
    axis, angle = aq.to_axis_angle()
    src_det = float(source_frame.determinant())
    tgt_det = float(target_frame.determinant())
    a_det = float(a.determinant())
    return {
        "status": "resolved",
        "landmark_resolution": "mapping_roles_rest_world_positions",
        "laterality_roles": laterality_roles,
        "source_up_roles": [src_lower, src_upper],
        "target_up_roles": [tgt_lower, tgt_upper],
        "forward_mode": forward_mode,
        "source_frame": source_frame,
        "target_frame": target_frame,
        "source_determinant": src_det,
        "target_determinant": tgt_det,
        "alignment": a,
        "alignment_determinant": a_det,
        "alignment_quat": aq,
        "alignment_angle_deg": math.degrees(float(angle)),
        "alignment_axis": [float(axis.x), float(axis.y), float(axis.z)],
        "reflection_detected": False,
        "finite": True,
        "motion_used": False,
        "name_heuristics_used": False,
    }


def body_alignment_measurements(resolved: dict) -> list:
    q = resolved["alignment_quat"]
    return [
        {"name": "body_frame_alignment_status", "value": resolved["status"]},
        {"name": "body_frame_landmark_resolution", "value": resolved["landmark_resolution"]},
        {"name": "body_frame_laterality_roles", "value": ",".join(resolved["laterality_roles"])},
        {
            "name": "body_frame_source_up_roles",
            "value": ",".join(resolved["source_up_roles"]),
        },
        {
            "name": "body_frame_target_up_roles",
            "value": ",".join(resolved["target_up_roles"]),
        },
        {"name": "body_frame_source_determinant", "value": f"{resolved['source_determinant']:.6f}"},
        {"name": "body_frame_target_determinant", "value": f"{resolved['target_determinant']:.6f}"},
        {
            "name": "body_frame_alignment_determinant",
            "value": f"{resolved['alignment_determinant']:.6f}",
        },
        {
            "name": "body_frame_alignment_angle_deg",
            "value": f"{resolved['alignment_angle_deg']:.4f}",
        },
        {
            "name": "body_frame_alignment_quat",
            "value": ",".join(f"{float(c):.8f}" for c in (q.w, q.x, q.y, q.z)),
        },
        {"name": "body_frame_finite", "value": "YES" if resolved["finite"] else "NO"},
        {
            "name": "body_frame_reflection_detected",
            "value": "YES" if resolved["reflection_detected"] else "NO",
        },
        {"name": "body_frame_motion_used", "value": "NO"},
        {"name": "body_frame_name_heuristics_used", "value": "NO"},
    ]


def rest_relative_world_delta_quat(src_rest_q, src_now_q, tgt_rest_q, body_a=None) -> Quaternion:
    """RestRelativeWorldDelta in a shared source→target semantic body frame.

    `Dsrc = Rsa * inverse(Rsr)` is the source joint's rest-relative world
    rotation. `Dtgt = A * Dsrc * inverse(A)` expresses that delta in the
    target body-semantic world. Then `Rt = Dtgt * Rtr`.
    `A` is identity when source and target rest body frames already match.
    """
    src_r = src_rest_q.normalized()
    src_n = src_now_q.normalized()
    tgt_r = tgt_rest_q.normalized()
    a = (body_a or identity_quat()).normalized()
    dsrc = src_n @ src_r.inverted()
    dtgt = a @ dsrc @ a.inverted()
    return (dtgt @ tgt_r).normalized()


def apply_desired_world_rotation(target, tpb, desired_q) -> bool:
    """Write a desired world rotation as a pose-local quaternion.

    `world0` is this bone's world orientation with identity pose (rest local
    under the *current* parent world). Parent joints must already be applied.
    """
    if not finite_quat(desired_q):
        return False
    tpb.location = Vector((0.0, 0.0, 0.0))
    tpb.scale = Vector((1.0, 1.0, 1.0))
    tpb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    bpy.context.view_layer.update()
    world0 = world_pose_matrix(target, tpb).to_quaternion()
    pose_q = world0.inverted() @ desired_q
    if not finite_quat(pose_q):
        return False
    tpb.rotation_quaternion = pose_q.normalized()
    tpb.location = Vector((0.0, 0.0, 0.0))
    tpb.scale = Vector((1.0, 1.0, 1.0))
    return True


def apply_rest_relative_world_rotation(
    target, tpb, src_rest, src_now, tgt_rest, body_a=None
) -> bool:
    """Non-root mapped joint: body-aligned world-space rest-relative rotation."""
    desired_q = rest_relative_world_delta_quat(
        src_rest.to_quaternion(),
        src_now.to_quaternion(),
        tgt_rest.to_quaternion(),
        body_a,
    )
    return apply_desired_world_rotation(target, tpb, desired_q)


def apply_root_keep_target_rest_scale(
    target, tpb, src_rest, src_now, tgt_rest, body_a=None
) -> bool:
    """Transfer root translation/rotation as a body-aligned rest-relative delta.

    `Dsrc = LocRot(src_now) * inverse(LocRot(src_rest))` with source scale removed.
    `Dtgt = A4 * Dsrc * inverse(A4)` maps that rigid delta into the target
    body-semantic world. `M_desired = Dtgt * LocRot(tgt_rest)` then restores
    target rest scale. Identity `A` preserves the previous world-delta path.
    """
    src_delta = loc_rot_without_scale(src_now) @ loc_rot_without_scale(src_rest).inverted()
    a = (body_a or identity_quat()).normalized()
    a4 = a.to_matrix().to_4x4()
    tgt_delta = a4 @ src_delta @ a4.inverted()
    tgt_rest_ns = loc_rot_without_scale(tgt_rest)
    desired_ns = tgt_delta @ tgt_rest_ns
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

    try:
        body_alignment = resolve_body_alignment(mapping_entries, src_rest, tgt_rest)
    except BodyFrameError as exc:
        payload = fail_envelope(job, "body_frame_alignment_unresolved", [str(exc)])
        extra = []
        if isinstance(exc.details, dict):
            for key, value in exc.details.items():
                extra.append({"name": f"body_frame_{key}", "value": str(value)})
        extra.append({"name": "body_frame_alignment_status", "value": "unresolved"})
        extra.append({"name": "body_frame_reflection_detected", "value": "YES" if exc.details.get("reflection_detected") else "NO"})
        extra.append({"name": "body_frame_motion_used", "value": "NO"})
        extra.append({"name": "body_frame_name_heuristics_used", "value": "NO"})
        payload["measurements"] = extra
        write_envelope(job, payload)
        raise SystemExit(1)
    body_a = body_alignment["alignment_quat"]

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
        # Parent-first, one pass. Each mapped joint's desired *world* rotation
        # is independent of siblings. After the parent pose is written and the
        # view layer is updated, world0 for the child is rest-local under the
        # already-animated parent, so pose_q = inverse(world0) * desired_world
        # is the exact local key. A second pass is not a mathematical solver.
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
                    target, tpb, src_r, src_now, tgt_r, body_a
                )
            else:
                ok = apply_rest_relative_world_rotation(
                    target, tpb, src_r, src_now, tgt_r, body_a
                )
            if not ok:
                nan_count += 1
                continue
            bpy.context.view_layer.update()

    for frame in range(f0, f1 + 1):
        scene.frame_set(frame)
        bpy.context.view_layer.update()
        reset_pose_to_rest(target)
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
        ]
        + body_alignment_measurements(body_alignment),
        "diagnostics": [
            "per-frame rest reset; rest-relative world rotation delta conjugated "
            "through rest body-semantic alignment A; "
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


MSG_NO_SKELETON = "No skeleton was found in this FBX."
MSG_MULTIPLE_SKELETONS = (
    "Multiple usable skeletons were found. "
    "R1 currently requires one unambiguous skeleton per source file."
)
MSG_NO_CLIPS = "No usable animation clips were found."
MSG_FRACTIONAL_FRAMES = (
    "This animation uses fractional frame endpoints that are not currently supported."
)
MSG_UNUSABLE_TIMING = (
    "This Motion source has timing that RigForge cannot represent exactly."
)
U32_MAX = 2**32 - 1


class UniqueArmatureError(RuntimeError):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


def rational_fps(fps, fps_base) -> tuple[int, int]:
    fps_i = int(fps)
    if fps_i <= 0:
        raise ValueError("scene.render.fps must be > 0")
    if isinstance(fps_base, bool) or not isinstance(fps_base, (int, float)):
        raise ValueError("scene.render.fps_base is not a real number")
    if not math.isfinite(fps_base) or fps_base <= 0:
        raise ValueError("scene.render.fps_base must be finite and > 0")
    try:
        base = Fraction(str(fps_base))
    except (ValueError, ZeroDivisionError) as exc:
        raise ValueError(f"str(fps_base) did not parse as Fraction: {fps_base!r}") from exc
    if base <= 0:
        raise ValueError("fps_base Fraction must be > 0")
    effective = Fraction(fps_i, 1) / base
    num, den = effective.numerator, effective.denominator
    if num <= 0 or den <= 0:
        raise ValueError("effective FPS must be a positive rational")
    if num > U32_MAX or den > U32_MAX:
        raise ValueError("reduced FPS exceeds Product u32/u32")
    return int(num), int(den)


def usable_armature_objects():
    out = []
    for obj in bpy.data.objects:
        if obj.type != "ARMATURE":
            continue
        data = getattr(obj, "data", None)
        if data is None:
            continue
        bones = getattr(data, "bones", None)
        if bones is None or len(bones) <= 0:
            continue
        out.append(obj)
    return out


def require_unique_usable_armature(armatures):
    if not armatures:
        raise UniqueArmatureError("no_skeleton", MSG_NO_SKELETON)
    if len(armatures) > 1:
        raise UniqueArmatureError("multiple_skeletons", MSG_MULTIPLE_SKELETONS)
    return armatures[0]


def extract_pose_bone_name(data_path: str) -> str | None:
    """Extract the exact quoted bone key from a Blender pose RNA path.

    Supported forms:
      pose.bones["pelvis"].rotation_quaternion
      pose.bones['upperarm_l'].location

    Numeric or unquoted pose.bones[...] paths are not association proof.
    Comparison against Armature bones is exact membership, not a substring.
    """
    text = data_path or ""
    marker = "pose.bones["
    start = text.find(marker)
    if start < 0:
        return None
    i = start + len(marker)
    if i >= len(text):
        return None
    quote = text[i]
    if quote not in "\"'":
        return None
    i += 1
    chars = []
    while i < len(text):
        ch = text[i]
        if ch == "\\" and i + 1 < len(text):
            chars.append(text[i + 1])
            i += 2
            continue
        if ch == quote:
            return "".join(chars)
        chars.append(ch)
        i += 1
    return None


def classify_pose_bone_names(names, armature_bones: set[str]) -> tuple[list[str], list[str]]:
    resolved = []
    unresolved = []
    seen_resolved = set()
    seen_unresolved = set()
    for name in names:
        if name in armature_bones:
            if name not in seen_resolved:
                resolved.append(name)
                seen_resolved.add(name)
        elif name not in seen_unresolved:
            unresolved.append(name)
            seen_unresolved.add(name)
    return resolved, unresolved


def pose_channels_qualifies(resolved, unresolved) -> bool:
    """Fail-closed pose_channels proof for one slot/channelbag.

    If the bag contains pose-bone paths, at least one must exist on the
    unique Armature and every pose-bone target used as proof must resolve.
    Mixed resolved + unresolved paths do not qualify pose_channels.
    """
    return bool(resolved) and not unresolved


def pose_channels_association_from_paths(paths, armature_bones: set[str]) -> dict:
    kept = []
    names = []
    for path in paths:
        name = extract_pose_bone_name(path or "")
        if name is None:
            continue
        kept.append(path)
        names.append(name)
    resolved, unresolved = classify_pose_bone_names(names, armature_bones)
    return {
        "qualifies": pose_channels_qualifies(resolved, unresolved),
        "resolved_pose_bones": resolved,
        "unresolved_pose_bones": unresolved,
        "pose_data_path_samples": kept[:4],
    }


def strong_association_kinds(
    *,
    direct_action: bool,
    nla_strip: bool,
    pose_channels: bool,
) -> list[str]:
    kinds = []
    if direct_action:
        kinds.append("direct_action")
    if nla_strip:
        kinds.append("nla_strip")
    if pose_channels:
        kinds.append("pose_channels")
    return kinds


def integral_frame(value):
    try:
        number = float(value)
    except (TypeError, ValueError):
        return None
    if not math.isfinite(number) or not number.is_integer():
        return None
    return int(number)


def action_display_label(name: str) -> str:
    tail = (name or "").split("|")[-1]
    return " ".join(tail.replace("_", " ").replace("-", " ").split())


def iter_channelbags(action, hint_slot=None):
    layers = list(getattr(action, "layers", None) or [])
    slots = list(getattr(action, "slots", None) or [])
    if hint_slot is not None and layers:
        try:
            bag = layers[0].strips[0].channelbag(hint_slot)
        except Exception:
            bag = None
        if bag is not None:
            yield hint_slot, bag
            return
    if slots and layers:
        try:
            strip = layers[0].strips[0]
        except Exception:
            strip = None
        if strip is not None:
            any_bag = False
            for slot in slots:
                try:
                    bag = strip.channelbag(slot)
                except Exception:
                    bag = None
                if bag is not None:
                    any_bag = True
                    yield slot, bag
            if any_bag:
                return
    fcurves = getattr(action, "fcurves", None)
    if fcurves:
        yield None, fcurves


def fcurves_of(bag_or_fcurves):
    if bag_or_fcurves is None:
        return []
    if hasattr(bag_or_fcurves, "fcurves"):
        return list(bag_or_fcurves.fcurves)
    return list(bag_or_fcurves)


def independent_channelbags(action):
    layers = list(getattr(action, "layers", None) or [])
    slots = list(getattr(action, "slots", None) or [])
    if slots and layers:
        try:
            strip = layers[0].strips[0]
        except Exception:
            strip = None
        if strip is not None:
            any_bag = False
            for slot in slots:
                try:
                    bag = strip.channelbag(slot)
                except Exception:
                    bag = None
                if bag is not None:
                    any_bag = True
                    yield slot, bag
            if any_bag:
                return
    fcurves = getattr(action, "fcurves", None)
    if fcurves:
        yield None, fcurves


def hinted_channelbag(action, hint_slot):
    if hint_slot is None:
        return
    layers = list(getattr(action, "layers", None) or [])
    if not layers:
        return
    try:
        bag = layers[0].strips[0].channelbag(hint_slot)
    except Exception:
        bag = None
    if bag is not None:
        yield hint_slot, bag


def armature_bone_name_set(arm) -> set[str]:
    data = getattr(arm, "data", None)
    bones = getattr(data, "bones", None) if data is not None else None
    if not bones:
        return set()
    return {bone.name for bone in bones}


def fcurve_data_paths(bag_or_fcurves) -> list[str]:
    return [getattr(curve, "data_path", "") or "" for curve in fcurves_of(bag_or_fcurves)]


def pose_proof_for_bag(bag, armature_bones: set[str]) -> dict:
    return pose_channels_association_from_paths(fcurve_data_paths(bag), armature_bones)


def evaluate_pose_channels(action, armature_bones: set[str], hint_slot=None) -> dict:
    empty = {
        "qualifies": False,
        "slot": None,
        "resolved_pose_bones": [],
        "unresolved_pose_bones": [],
        "pose_data_path_samples": [],
    }
    if hint_slot is not None:
        bags = list(hinted_channelbag(action, hint_slot))
        if not bags:
            return empty
        slot, bag = bags[0]
        proof = pose_proof_for_bag(bag, armature_bones)
        proof["slot"] = slot
        return proof
    evidence = dict(empty)
    for slot, bag in independent_channelbags(action):
        proof = pose_proof_for_bag(bag, armature_bones)
        if proof["qualifies"]:
            proof["slot"] = slot
            return proof
        if (proof["resolved_pose_bones"] or proof["unresolved_pose_bones"]) and not (
            evidence["resolved_pose_bones"] or evidence["unresolved_pose_bones"]
        ):
            evidence = dict(proof)
            evidence["slot"] = slot
    return evidence


def action_frame_span(action, hint_slot=None):
    xs = []
    for _slot, bag in iter_channelbags(action, hint_slot):
        for curve in fcurves_of(bag):
            points = getattr(curve, "keyframe_points", None) or []
            for key in points:
                co = getattr(key, "co", None)
                if co is None or len(co) < 1:
                    continue
                xs.append(float(co[0]))
    if not xs:
        frame_range = getattr(action, "frame_range", None)
        if frame_range is not None and len(frame_range) >= 2:
            return float(frame_range[0]), float(frame_range[1])
        return None
    return min(xs), max(xs)


def nla_clips_for_action(arm, action) -> list:
    ad = getattr(arm, "animation_data", None)
    if ad is None:
        return []
    found = []

    def walk(strips):
        for strip in strips:
            strip_type = getattr(strip, "type", None)
            if strip_type == "META":
                walk(getattr(strip, "strips", []) or [])
            elif strip_type == "CLIP" and getattr(strip, "action", None) == action:
                found.append(strip)

    for track in getattr(ad, "nla_tracks", []) or []:
        walk(getattr(track, "strips", []) or [])
    return found


def slot_identifier(slot) -> str | None:
    if slot is None:
        return None
    ident = getattr(slot, "identifier", None)
    if ident:
        return str(ident)
    name = getattr(slot, "name_display", None) or getattr(slot, "name", None)
    return str(name) if name else None


def collect_animation_candidates(arm, fps_num, fps_den, timing_ok: bool) -> list[dict]:
    candidates = []
    armature_bones = armature_bone_name_set(arm)
    for action in list(bpy.data.actions):
        ad = getattr(arm, "animation_data", None)
        assigned_slot = None
        has_direct = ad is not None and getattr(ad, "action", None) == action
        if has_direct:
            assigned_slot = getattr(ad, "action_slot", None)
        nla_strips = nla_clips_for_action(arm, action)
        nla_slot = None
        has_nla = bool(nla_strips)
        if has_nla:
            nla_slot = getattr(nla_strips[0], "action_slot", None)
        hint_slot = assigned_slot or nla_slot
        pose = evaluate_pose_channels(action, armature_bones, hint_slot)
        kinds = strong_association_kinds(
            direct_action=has_direct,
            nla_strip=has_nla,
            pose_channels=bool(pose.get("qualifies")),
        )
        if not kinds:
            continue
        association_kind = kinds[0]
        span = action_frame_span(action, hint_slot)
        unusable_reason = None
        start_frame = None
        end_frame = None
        usable = True
        if not timing_ok:
            usable = False
            unusable_reason = "unusable_timing"
        elif span is None:
            usable = False
            unusable_reason = "missing_frames"
        else:
            start_frame = integral_frame(span[0])
            end_frame = integral_frame(span[1])
            if start_frame is None or end_frame is None:
                usable = False
                unusable_reason = "fractional_frames"
        weak_notes = []
        slots = list(getattr(action, "slots", None) or [])
        for slot in slots:
            target = getattr(slot, "target_id_type", None)
            if target:
                weak_notes.append(f"slot_target_id_type={target}")
        id_root = getattr(action, "id_root", None)
        if id_root:
            weak_notes.append(f"id_root={id_root}")
        evidence = {
            "kinds_present": kinds,
            "assigned_slot_identifier": slot_identifier(assigned_slot),
            "nla_track_name": None,
            "nla_strip_name": None,
            "nla_slot_identifier": slot_identifier(nla_slot),
            "pose_slot_identifier": slot_identifier(pose.get("slot")),
            "pose_data_path_samples": list(pose.get("pose_data_path_samples") or []),
            "resolved_pose_bones": list(pose.get("resolved_pose_bones") or []),
            "unresolved_pose_bones": list(pose.get("unresolved_pose_bones") or []),
            "weak_notes": weak_notes,
        }
        if nla_strips:
            strip = nla_strips[0]
            evidence["nla_strip_name"] = getattr(strip, "name", None)
            track = getattr(strip, "id_data", None)
            evidence["nla_track_name"] = getattr(track, "name", None)
        clip_identity = action.name
        candidates.append(
            {
                "clip_identity": clip_identity,
                "display_label": action_display_label(clip_identity),
                "source_skeleton_local_key": arm.name,
                "association_kind": association_kind,
                "association_evidence": evidence,
                "start_frame": start_frame,
                "end_frame": end_frame,
                "fps_num": fps_num,
                "fps_den": fps_den,
                "usable": usable,
                "unusable_reason": unusable_reason,
            }
        )
    candidates.sort(key=lambda item: item["clip_identity"])
    return candidates


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
    armatures = usable_armature_objects()
    try:
        arm = require_unique_usable_armature(armatures)
    except UniqueArmatureError as exc:
        payload = {
            "status": "FAIL",
            "kind": "skeleton_observation",
            "user_code": exc.code,
            "user_message": str(exc),
            "joints": [],
            "source_digest": digest,
            "diagnostics": [
                "unique usable Armature required; silent max-bones selection is forbidden",
                str(exc),
            ],
        }
        write_json(Path(job["outputs"]["inspect_envelope"]), payload)
        raise SystemExit(1)
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
            "unique usable Armature required; silent max-bones selection is forbidden",
        ],
    }
    write_json(Path(job["outputs"]["inspect_envelope"]), payload)
    return payload


def inspect_source(job: dict) -> dict:
    expected = job.get("expected_digest")
    source = Path(job["source_path"])
    envelope_path = Path(job["outputs"]["inspect_envelope"])
    if not source.is_file():
        payload = {
            "status": "FAIL",
            "kind": "source_observation",
            "user_code": "missing_file",
            "user_message": "The selected file is missing.",
            "source_path": str(source),
            "source_digest": None,
            "size_bytes": None,
            "usable_armature_count": 0,
            "skeleton_candidates": [],
            "animation_candidates": [],
            "timing_context": None,
            "diagnostics": ["source file is missing"],
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    digest = sha256_file(source)
    size_bytes = source.stat().st_size
    if expected and digest != expected:
        payload = {
            "status": "FAIL",
            "kind": "source_observation",
            "user_code": "digest_mismatch",
            "user_message": "This file changed after it was inspected. Inspect it again.",
            "source_path": str(source),
            "source_digest": digest,
            "size_bytes": size_bytes,
            "usable_armature_count": 0,
            "skeleton_candidates": [],
            "animation_candidates": [],
            "timing_context": None,
            "diagnostics": [f"source digest mismatch {digest} != {expected}"],
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(
        filepath=str(source),
        use_anim=True,
        automatic_bone_orientation=False,
    )
    armatures = usable_armature_objects()
    diagnostics = [
        "inspect_source only; no retarget execution",
        "joint_key and clip_identity are source-local evidence, not Product identity",
        "slot_suitable alone is not eligibility proof",
        "producer=Blender {} adapter={}".format(
            getattr(bpy.app, "version_string", ""),
            WORKER_VERSION,
        ),
    ]
    if not armatures:
        payload = {
            "status": "FAIL",
            "kind": "source_observation",
            "user_code": "no_skeleton",
            "user_message": MSG_NO_SKELETON,
            "source_path": str(source),
            "source_digest": digest,
            "size_bytes": size_bytes,
            "observed_media_type": "application/octet-stream",
            "usable_armature_count": 0,
            "skeleton_candidates": [],
            "animation_candidates": [],
            "timing_context": None,
            "diagnostics": diagnostics,
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    if len(armatures) > 1:
        payload = {
            "status": "FAIL",
            "kind": "source_observation",
            "user_code": "multiple_skeletons",
            "user_message": MSG_MULTIPLE_SKELETONS,
            "source_path": str(source),
            "source_digest": digest,
            "size_bytes": size_bytes,
            "observed_media_type": "application/octet-stream",
            "usable_armature_count": len(armatures),
            "skeleton_candidates": [],
            "animation_candidates": [],
            "timing_context": None,
            "diagnostics": diagnostics + [f"usable_armature_count={len(armatures)}"],
        }
        write_json(envelope_path, payload)
        raise SystemExit(1)
    arm = armatures[0]
    joints = inspect_armature(arm)
    timing_ok = True
    timing_context = None
    fps_num = None
    fps_den = None
    try:
        scene = bpy.context.scene
        fps_num, fps_den = rational_fps(scene.render.fps, scene.render.fps_base)
        timing_context = {"fps_num": fps_num, "fps_den": fps_den}
    except Exception as exc:
        timing_ok = False
        diagnostics.append(f"rational_fps failed: {exc}")
    candidates = collect_animation_candidates(arm, fps_num, fps_den, timing_ok)
    payload = {
        "status": "SUCCESS",
        "kind": "source_observation",
        "user_code": None,
        "user_message": None,
        "source_path": str(source),
        "source_digest": digest,
        "size_bytes": size_bytes,
        "observed_media_type": "application/octet-stream",
        "usable_armature_count": 1,
        "skeleton_candidates": [
            {
                "source_local_key": arm.name,
                "display_name": arm.name,
                "joint_count": len(joints),
                "joints": joints,
            }
        ],
        "animation_candidates": candidates,
        "timing_context": timing_context,
        "diagnostics": diagnostics,
    }
    write_json(envelope_path, payload)
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
        elif mode == "inspect_source":
            inspect_source(job)
        elif mode == "inspect_qc":
            inspect_qc(job)
        elif mode in (
            "preview_character",
            "preview_motion",
            "preview_derived",
            "preview_motion_synthetic",
        ):
            from preview_gen import run_preview

            result = run_preview(mode, job)
            write_json(Path(job["outputs"]["generation_json"]), result)
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
        if mode.startswith("preview_"):
            gen = (job.get("outputs") or {}).get("generation_json")
            if gen:
                write_json(
                    Path(gen),
                    {
                        "status": "FAILURE",
                        "mode": mode,
                        "traceback": traceback.format_exc(),
                    },
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
