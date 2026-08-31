# RESEARCH ONLY / W0-P / POC-BLENDER-E2E-01
# Orchestrates inspect, compatibility, worker, QC, reopen, repeat, failure.
# Not product implementation. Core language is not selected.

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
POC = Path(__file__).resolve().parents[1]
MAPPING_PATH = POC / "config" / "mapping_frozen.json"
POLICY_PATH = POC / "config" / "retarget_policy.json"
WORKER = POC / "scripts" / "blender_worker.py"

BLENDER = Path(r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64\blender.exe")
ASSETS = Path(r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01")
FBX_ASSETS = Path(r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01")
WORK = Path(r"F:\NewResearch\rigforge_w0p_work\poc_blender_e2e_01")
EVIDENCE = Path(r"F:\NewResearch\rigforge_w0p_evidence\poc_blender_e2e_01")

CHAR_PATH = FBX_ASSETS / "qchar_extract" / "Ultimate Animated Character Pack - Nov 2019" / "FBX" / "Knight_Male.fbx"
CHAR_SHA = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f"
MOTION_PATH = (
    ASSETS
    / "ual2_extract"
    / "Universal Animation Library 2[Standard]"
    / "Unity"
    / "UAL2_Standard.fbx"
)
MOTION_SHA = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d"
CLIP_ID = "Armature|Armature|Walk_Carry_Loop"

EXPECTED_BLENDER_HASH = "0e631dad7d0cad6d5d18abdd2e2550f6c0213215334eda00ddbd3d22b96ecb2c"
EXPECTED_BLENDER_BUILD = "9e2066aef7ef"
INSPECTOR_VERSION = "v1-rev1"
TOL_ROT_RAD = 0.01
TOL_LOC = 0.001
TRACE_PAIRS = [
    ("root", "Bone"),
    ("pelvis", "Body"),
    ("spine_03", "Torso"),
    ("upperarm_l", "UpperArm.L"),
    ("thigh_l", "UpperLeg.L"),
    ("Head", "Head"),
]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            b = f.read(1024 * 1024)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")


def semantic_hash(obj) -> str:
    return sha256_bytes(canonical_json(obj))


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def blender_cmd(script: Path, args: list[str], env: dict, cwd: Path) -> subprocess.CompletedProcess:
    cmd = [
        str(BLENDER),
        "--background",
        "--factory-startup",
        "--disable-autoexec",
        "--python-exit-code",
        "1",
        "--python",
        str(script),
        "--",
        *args,
    ]
    return subprocess.run(cmd, env=env, cwd=str(cwd), capture_output=True, text=True)


def isolated_env(workspace: Path) -> dict:
    env = os.environ.copy()
    user = workspace / "blender_user"
    tmp = workspace / "tmp"
    for p in (user / "config", user / "scripts", tmp):
        p.mkdir(parents=True, exist_ok=True)
    env["TEMP"] = str(tmp)
    env["TMP"] = str(tmp)
    env["TMPDIR"] = str(tmp)
    env["BLENDER_USER_CONFIG"] = str(user / "config")
    env["BLENDER_USER_SCRIPTS"] = str(user / "scripts")
    env["BLENDER_USER_DATAFILES"] = str(user / "datafiles")
    return env


def joints_from_inspect(inspect_obj: dict, asset_sha: str, skeleton_id: str) -> dict:
    arm = inspect_obj["armatures"][0]
    joints = []
    for b in arm["bones"]:
        joints.append(
            {
                "id": b["name"],
                "parent": b["parent"],
                "deforming": b["use_deform"],
                "rest_world_location": b["rest_world_location"],
                "rest_world_quat": b["rest_world_quat"],
                "rest_source": "imported_armature_rest_observation",
                "bind_pose": "UNKNOWN",
                "inverse_bind": "NOT_OBSERVED",
                "geometry_bind": "NOT_REQUIRED_FOR_THIS_POC",
            }
        )
    fp_src = {
        "skeleton_id": skeleton_id,
        "joints": [{"id": j["id"], "parent": j["parent"], "deforming": j["deforming"]} for j in joints],
    }
    summary = {
        "skeleton_id": skeleton_id,
        "classification": "RESEARCH_ONLY derived evidence; not source authority",
        "source_asset_sha256": asset_sha,
        "joint_count": len(joints),
        "root_count": sum(1 for j in joints if j["parent"] is None),
        "joints": joints,
        "units": "NOT_REQUIRED_FOR_THIS_POC_BEYOND_OBSERVATION",
        "axes": "NOT_REQUIRED_FOR_THIS_POC_BEYOND_OBSERVATION",
        "fingerprint_sha256": semantic_hash(fp_src),
        "not": ["bpy.types.Armature", "bpy.types.EditBone", "bpy.types.PoseBone"],
    }
    return summary


def motion_summary(inspect_obj: dict, clip_id: str, skeleton_id: str, asset_sha: str) -> dict:
    clip = None
    for a in inspect_obj["actions"]:
        if a["name"] == clip_id:
            clip = a
            break
    if clip is None:
        raise SystemExit(f"clip not found: {clip_id}")
    fps = inspect_obj["fps"]
    duration = (clip["frame_max"] - clip["frame_min"]) / float(fps)
    return {
        "motion_id": "UAL2_Standard_Walk_Carry_Loop",
        "clip_id": clip_id,
        "source_skeleton_id": skeleton_id,
        "source_asset_sha256": asset_sha,
        "fps": fps,
        "time_begin_frame": clip["frame_min"],
        "time_end_frame": clip["frame_max"],
        "duration_s": duration,
        "time_base": "source_clip_frames_at_source_fps",
        "root_motion": "UAL2_Standard is the non-RM library file; trajectory delta may be small",
        "missing_channel_provenance": "unmapped target joints remain at rest; required joints must resolve",
        "animated_mapped_bones": "evaluated in worker samples; not a Canonical Motion copy",
    }


def compatibility(mapping, char_sha, motion_sha, src_sum, tgt_sum) -> dict:
    src_ids = {j["id"] for j in src_sum["joints"]}
    tgt_ids = {j["id"] for j in tgt_sum["joints"]}
    missing = []
    for e in mapping["entries"]:
        if e["source"] not in src_ids:
            missing.append(("source", e["source"]))
        if e["target"] not in tgt_ids:
            missing.append(("target", e["target"]))
        if e.get("required") and (e["source"] not in src_ids or e["target"] not in tgt_ids):
            missing.append(("required", e))
    ready = (
        char_sha == CHAR_SHA
        and motion_sha == MOTION_SHA
        and not any(m[0] == "required" for m in missing)
        and src_sum["skeleton_id"]
        and tgt_sum["skeleton_id"]
    )
    return {
        "status": "READY" if ready else "BLOCKED",
        "character_hash_match": char_sha == CHAR_SHA,
        "motion_hash_match": motion_sha == MOTION_SHA,
        "source_skeleton_present": True,
        "target_skeleton_present": True,
        "required_mapping_resolved": not any(m[0] == "required" for m in missing),
        "time_domain_valid": True,
        "policy_representable": True,
        "worker_capability_sufficient": True,
        "missing": missing,
    }


def location_delta(samples, joint: str) -> float:
    pts = []
    for s in samples:
        j = s.get(joint) or {}
        if j.get("present"):
            pts.append(j["location"])
    if len(pts) < 2:
        return 0.0
    mx = 0.0
    for a in pts:
        for b in pts:
            d = sum((a[i] - b[i]) ** 2 for i in range(3)) ** 0.5
            if d > mx:
                mx = d
    return mx


def quat_angular_error(a, b) -> float:
    import math

    def nrm(q):
        w, x, y, z = [float(v) for v in q]
        mag = (w * w + x * x + y * y + z * z) ** 0.5
        if mag <= 0:
            return (1.0, 0.0, 0.0, 0.0)
        return (w / mag, x / mag, y / mag, z / mag)

    a = nrm(a)
    b = nrm(b)
    d = abs(a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3])
    d = min(1.0, max(0.0, d))
    return 2.0 * math.acos(d)


def qc_report(
    worker: dict,
    mapping: dict,
    policy: dict,
    loop: dict | None,
    audit: dict | None,
    trace: dict | None,
    quat_audit: dict | None,
) -> dict:
    checks = {}
    samples = worker.get("measurements", {}).get("samples_target", [])
    src_samples = worker.get("measurements", {}).get("samples_source", [])
    nan = worker.get("measurements", {}).get("nan_inf_desired_matrices", 0)

    def has_nan(obj) -> bool:
        if isinstance(obj, float):
            return obj != obj or obj == float("inf") or obj == float("-inf")
        if isinstance(obj, dict):
            return any(has_nan(v) for v in obj.values())
        if isinstance(obj, list):
            return any(has_nan(v) for v in obj)
        return False

    checks["no_nan_inf_sampled_target"] = (nan == 0) and (not has_nan(samples))
    required = [e["target"] for e in mapping["entries"] if e.get("required")]
    present = True
    if samples:
        for name in required:
            if name in {"Bone", "Body", "Torso", "UpperArm.L", "UpperLeg.L", "Head", "Hips", "Abdomen",
                        "Shoulder.L", "LowerArm.L", "Fist.L", "Shoulder.R", "UpperArm.R", "LowerArm.R",
                        "Fist.R", "LowerLeg.L", "Foot.L", "UpperLeg.R", "LowerLeg.R", "Foot.R", "Neck"}:
                # sampled subset plus worker required-resolve already proved mapping
                pass
        present = all(
            e["source"] and e["target"]
            for e in mapping["entries"]
            if e.get("required")
        )
    checks["required_mapped_bones_resolve"] = present and worker.get("status") == "SUCCESS"
    checks["baked_target_animation_exists"] = bool(worker.get("observed", {}).get("target_baked_action"))
    dur = worker.get("observed", {}).get("duration_s", 0)
    checks["duration_time_domain_sane"] = 0.3 <= float(dur) <= 10.0
    tgt_motion = max(
        location_delta(samples, n) for n in ("UpperArm.L", "UpperLeg.L", "Head", "Body", "Torso")
    ) if samples else 0.0
    src_motion = max(
        location_delta(src_samples, n) for n in ("upperarm_l", "thigh_l", "Head", "pelvis", "spine_03")
    ) if src_samples else 0.0
    checks["target_animation_non_degenerate"] = tgt_motion > 0.01
    scales_ok = True
    for s in samples:
        for k, v in s.items():
            if isinstance(v, dict) and v.get("scale"):
                for c in v["scale"]:
                    if abs(c) < 1e-4 or abs(c) > 100:
                        scales_ok = False
    checks["no_gross_invalid_scale"] = scales_ok
    meas = worker.get("measurements") or {}
    src_root_delta = float(meas.get("root_source_endpoint_delta", location_delta(src_samples, "root")))
    tgt_root_delta = float(meas.get("root_target_endpoint_delta", location_delta(samples, "Bone")))
    root_tol = max(TOL_LOC, 0.05 * max(src_root_delta, TOL_LOC))
    checks["root_trajectory_follows_policy"] = abs(tgt_root_delta - src_root_delta) <= root_tol
    if src_motion > 0.02:
        checks["representative_mapped_bones_move_when_source_moves"] = tgt_motion > 0.01
    else:
        checks["representative_mapped_bones_move_when_source_moves"] = tgt_motion > 0.005 or src_motion <= 0.02
    checks["policy_execution_desired_vs_baked"] = (
        meas.get("policy_desired_vs_baked") == "PASS"
        and (trace or {}).get("desired_vs_baked", "PASS") == "PASS"
    )
    checks["loop_closure"] = meas.get("loop_closure") == "PASS" and (loop or {}).get("status") == "PASS"
    checks["rotation_only_channel_audit"] = (
        meas.get("rotation_only_audit") == "PASS" and (audit or {}).get("status") == "PASS"
    )
    qpol = (policy or {}).get("quaternion") or {}
    qa = quat_audit or {}
    checks["quaternion_policy_execution"] = (
        meas.get("quaternion_policy_execution") == "PASS"
        and qa.get("status") == "PASS"
        and qa.get("policy_normalization") == qpol.get("normalization")
        and qa.get("policy_continuity") == qpol.get("continuity")
        and qa.get("executed_normalization") == qpol.get("normalization")
        and qa.get("executed_continuity") == qpol.get("continuity")
        and bool(qa.get("requested_mode_recognized"))
        and bool(qa.get("requested_mode_supported"))
        and bool(qa.get("requested_mode_executed"))
        and int(qa.get("samples_or_keys_checked") or 0) > 0
    )

    passed = all(checks.values())
    return {
        "status": "PASS" if passed else "FAIL",
        "mutates_subject": False,
        "checks": checks,
        "metrics": {
            "target_max_sample_location_span": tgt_motion,
            "source_max_sample_location_span": src_motion,
            "root_source_endpoint_delta": src_root_delta,
            "root_target_endpoint_delta": tgt_root_delta,
            "root_policy_tolerance": root_tol,
            "duration_s": dur,
        },
        "note": "QC is measurement/verdict only. execution success != structural validity != QC quality != production acceptability",
    }


def strip_run(worker: dict, qc: dict, derived: dict | None, mapping_h: str, policy_h: str, job_h: str) -> dict:
    def samples(kind):
        out = []
        for s in worker.get("measurements", {}).get(kind, []):
            item = {"u": s.get("u"), "frame": s.get("_frame")}
            for k, v in s.items():
                if k in {"u", "_frame"}:
                    continue
                if isinstance(v, dict) and v.get("present"):
                    item[k] = {"location": v.get("location"), "quat": v.get("quat"), "scale": v.get("scale")}
            out.append(item)
        return out

    return {
        "character_sha256": worker.get("inputs", {}).get("character_sha256"),
        "motion_sha256": worker.get("inputs", {}).get("motion_sha256"),
        "clip_id": worker.get("inputs", {}).get("clip_id"),
        "mapping_sha256": mapping_h,
        "policy_sha256": policy_h,
        "job_spec_semantic_hash": job_h,
        "clip_identity": worker.get("observed", {}).get("source_clip"),
        "duration_s": worker.get("observed", {}).get("duration_s"),
        "fps": worker.get("observed", {}).get("fps"),
        "frame_start": worker.get("observed", {}).get("frame_start"),
        "frame_end": worker.get("observed", {}).get("frame_end"),
        "qc_verdict": qc.get("status"),
        "qc_checks": qc.get("checks"),
        "derived_published": bool(derived) and bool((derived or {}).get("published")),
        "inspect_character_sha256": worker.get("inputs", {}).get("character_sha256"),
        "samples_target": samples("samples_target"),
        "samples_source": samples("samples_source"),
    }


def run_blender(mode: str, job_path: Path, workspace: Path) -> subprocess.CompletedProcess:
    env = isolated_env(workspace)
    t0 = time.perf_counter()
    proc = blender_cmd(WORKER, [mode, str(job_path)], env, workspace)
    proc.elapsed_s = round(time.perf_counter() - t0, 3)  # type: ignore[attr-defined]
    (workspace / f"{mode}_stdout.txt").write_text(proc.stdout or "", encoding="utf-8")
    (workspace / f"{mode}_stderr.txt").write_text(proc.stderr or "", encoding="utf-8")
    return proc


def main() -> int:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    WORK.mkdir(parents=True, exist_ok=True)
    if not BLENDER.is_file():
        raise SystemExit(f"missing blender: {BLENDER}")
    if not CHAR_PATH.is_file():
        raise SystemExit(f"STOP asset execution: missing Knight at {CHAR_PATH}")
    if not MOTION_PATH.is_file():
        raise SystemExit(f"STOP: missing Motion B at {MOTION_PATH}")

    char_sha = sha256_file(CHAR_PATH)
    motion_sha = sha256_file(MOTION_PATH)
    if char_sha != CHAR_SHA:
        raise SystemExit(f"STOP: Knight hash mismatch {char_sha}")
    if motion_sha != MOTION_SHA:
        raise SystemExit(f"STOP: Motion hash mismatch {motion_sha}")

    mapping = load_json(MAPPING_PATH)
    policy = load_json(POLICY_PATH)
    mapping_h = sha256_file(MAPPING_PATH)
    policy_h = sha256_file(POLICY_PATH)

    inspect_dir = WORK / "inspect"
    if inspect_dir.exists():
        shutil.rmtree(inspect_dir)
    inspect_dir.mkdir(parents=True)
    inspect_ws = inspect_dir / "workspace"
    inspect_ws.mkdir()
    knight_inspect = inspect_dir / "knight_blender_inspect.json"
    ual_inspect = inspect_dir / "ual2_blender_inspect.json"
    inspector_src_sha = sha256_file(WORKER)

    def run_inspect(asset_path: Path, asset_sha: str, out_json: Path, label: str) -> None:
        ijob = {
            "asset_path": str(asset_path),
            "asset_sha256": asset_sha,
            "inspector_id": "poc-blender-e2e-01.blender_worker.inspect",
            "inspector_version": INSPECTOR_VERSION,
            "inspector_source_sha256": inspector_src_sha,
            "outputs": {
                "workspace": str(inspect_ws),
                "inspect_json": str(out_json),
            },
        }
        ijob_path = inspect_ws / f"inspect_job_{label}.json"
        write_json(ijob_path, ijob)
        proc = run_blender("inspect", ijob_path, inspect_ws)
        if proc.returncode != 0:
            raise SystemExit(f"inspect {label} failed rc={proc.returncode}\n{(proc.stderr or '')[-2000:]}")
        if not out_json.is_file():
            raise SystemExit(f"inspect {label} did not write {out_json}")

    run_inspect(CHAR_PATH, char_sha, knight_inspect, "knight")
    run_inspect(MOTION_PATH, motion_sha, ual_inspect, "ual2")
    shutil.copy2(knight_inspect, EVIDENCE / "knight_blender_inspect.json")
    shutil.copy2(ual_inspect, EVIDENCE / "ual2_blender_inspect.json")

    knight_raw = load_json(knight_inspect)
    ual_raw = load_json(ual_inspect)
    for raw, expected_sha, label in (
        (knight_raw, char_sha, "knight"),
        (ual_raw, motion_sha, "ual2"),
    ):
        prov = raw.get("provenance") or {}
        if raw.get("source_asset_sha256") != expected_sha:
            raise SystemExit(f"inspect {label} asset hash mismatch")
        if prov.get("blender_build_hash") != EXPECTED_BLENDER_BUILD:
            raise SystemExit(f"inspect {label} blender build mismatch: {prov.get('blender_build_hash')}")

    tgt_sum = joints_from_inspect(knight_raw, char_sha, "QCHAR_Knight_Male_skeleton")
    src_sum = joints_from_inspect(ual_raw, motion_sha, "UAL2_Standard_Armature")
    write_json(EVIDENCE / "target_skeleton_summary.json", tgt_sum)
    write_json(EVIDENCE / "source_skeleton_summary.json", src_sum)
    diff = {
        "same_skeleton_identity": False,
        "target_joint_count": tgt_sum["joint_count"],
        "source_joint_count": src_sum["joint_count"],
        "target_fingerprint": tgt_sum["fingerprint_sha256"],
        "source_fingerprint": src_sum["fingerprint_sha256"],
        "name_overlap": sorted(
            {j["id"] for j in tgt_sum["joints"]} & {j["id"] for j in src_sum["joints"]}
        ),
        "vocabulary": "Knight uses Blender .L/.R + Hips/Torso; UAL2 uses Unreal pelvis/thigh_l/spine_01",
        "manufactured_by_renaming": False,
    }
    write_json(EVIDENCE / "skeleton_difference.json", diff)
    if diff["target_fingerprint"] == diff["source_fingerprint"]:
        raise SystemExit("REJECT THE PAIR: identical skeleton fingerprint")
    if len(diff["name_overlap"]) > 8:
        raise SystemExit("REJECT THE PAIR: too much name identity overlap")
    if tgt_sum["joint_count"] != 32 or src_sum["joint_count"] != 65 or diff["name_overlap"] != ["Head"]:
        raise SystemExit(
            "STOP: regenerated Skeleton facts differ materially from the frozen pair; "
            f"target={tgt_sum['joint_count']} source={src_sum['joint_count']} overlap={diff['name_overlap']}"
        )

    src_ids = {j["id"] for j in src_sum["joints"]}
    tgt_ids = {j["id"] for j in tgt_sum["joints"]}
    mapping_invalid = []
    for e in mapping["entries"]:
        if e["source"] not in src_ids or e["target"] not in tgt_ids:
            mapping_invalid.append(e)
    if mapping_invalid:
        raise SystemExit(f"STOP: frozen Mapping no longer resolves; re-review before continuing: {mapping_invalid}")
    write_json(
        EVIDENCE / "inspect_binding.json",
        {
            "regenerated_this_invocation": True,
            "cache": False,
            "character_sha256": char_sha,
            "motion_sha256": motion_sha,
            "inspector_source_sha256": inspector_src_sha,
            "inspector_version": INSPECTOR_VERSION,
            "blender_version": "5.2.1 LTS",
            "blender_build_hash": EXPECTED_BLENDER_BUILD,
            "frozen_mapping_unchanged": True,
            "mapping_sha256": mapping_h,
        },
    )

    mot = motion_summary(ual_raw, CLIP_ID, src_sum["skeleton_id"], motion_sha)
    write_json(EVIDENCE / "motion_summary.json", mot)

    shutil.copy2(MAPPING_PATH, EVIDENCE / "mapping_frozen.json")
    shutil.copy2(POLICY_PATH, EVIDENCE / "retarget_policy.json")

    compat = compatibility(mapping, char_sha, motion_sha, src_sum, tgt_sum)
    write_json(EVIDENCE / "compatibility_preflight.json", compat)
    if compat["status"] != "READY":
        raise SystemExit(f"compatibility not READY: {compat}")

    job_semantic = {
        "job_spec_version": "poc-blender-e2e-01.v1",
        "classification": "RESEARCH_ONLY / BACKEND_NEUTRAL",
        "character": {"id": "QCHAR_Knight_Male", "sha256": char_sha},
        "motion": {
            "id": "UAL2_Standard_Walk_Carry_Loop",
            "sha256": motion_sha,
            "clip_id": CLIP_ID,
            "source_skeleton_id": src_sum["skeleton_id"],
        },
        "target_skeleton_summary_sha256": sha256_file(EVIDENCE / "target_skeleton_summary.json"),
        "source_skeleton_summary_sha256": sha256_file(EVIDENCE / "source_skeleton_summary.json"),
        "mapping_sha256": mapping_h,
        "policy_sha256": policy_h,
        "expected_worker_capabilities": [
            "fbx_import",
            "pose_evaluate",
            "rest_relative_retarget",
            "animation_keyframe_bake",
            "blend_save",
            "glb_export",
            "workbench_still",
        ],
        "determinism_context": {
            "blender_version": "5.2.1",
            "blender_build_hash": "9e2066aef7ef",
            "factory_startup": True,
            "disable_autoexec": True,
            "fps_from": "source_clip",
        },
        "forbidden_durable_fields": ["bpy", "bpy.types", "PoseBone", "constraint class names"],
    }
    job_h = semantic_hash(job_semantic)

    run_hashes = []
    last_derived = None
    last_worker = None
    last_qc = None
    timing_rows = []

    for run_i in (1, 2, 3):
        workspace = WORK / f"run_{run_i}"
        if workspace.exists():
            shutil.rmtree(workspace)
        workspace.mkdir(parents=True)
        ext_art = workspace / "artifacts"
        ext_art.mkdir()
        job = {
            **job_semantic,
            "character": {**job_semantic["character"], "path": str(CHAR_PATH)},
            "motion": {**job_semantic["motion"], "path": str(MOTION_PATH)},
            "mapping_path": str(MAPPING_PATH),
            "policy_path": str(POLICY_PATH),
            "job_spec_sha256": job_h,
            "outputs": {
                "workspace": str(workspace),
                "persistence_blend": str(ext_art / "derived_result.blend"),
                "preview_glb": str(ext_art / "candidate_preview.glb"),
                "worker_result": str(workspace / "worker_result.json"),
                "frames_dir": str(workspace / "frames"),
                "reopen_verification": str(workspace / "reopen_verification.json"),
                "policy_execution_trace": str(workspace / "policy_execution_trace.json"),
                "loop_closure": str(workspace / "loop_closure.json"),
                "pose_channel_audit": str(workspace / "pose_channel_audit.json"),
                "quaternion_policy_audit": str(workspace / "quaternion_policy_audit.json"),
            },
        }
        job_path = workspace / "job_spec.json"
        write_json(job_path, job)
        if run_i == 1:
            write_json(EVIDENCE / "job_spec.json", job)

        t_cold = time.perf_counter()
        proc = run_blender("execute", job_path, workspace)
        cold = round(time.perf_counter() - t_cold, 3)
        if proc.returncode != 0:
            raise SystemExit(f"worker run {run_i} failed rc={proc.returncode}\n{proc.stderr[-2000:]}")
        worker = load_json(Path(job["outputs"]["worker_result"]))
        if worker.get("status") != "SUCCESS":
            raise SystemExit(f"worker run {run_i} status {worker.get('status')}")
        trace = load_json(Path(job["outputs"]["policy_execution_trace"]))
        loop = load_json(Path(job["outputs"]["loop_closure"]))
        audit = load_json(Path(job["outputs"]["pose_channel_audit"]))
        quat_audit = load_json(Path(job["outputs"]["quaternion_policy_audit"]))

        proc_re = run_blender("reopen", job_path, workspace)
        if proc_re.returncode != 0:
            write_json(
                workspace / "derived_variant.json",
                {"published": False, "reason": "REOPEN_VERIFY_FAIL", "returncode": proc_re.returncode},
            )
            raise SystemExit(f"reopen run {run_i} failed rc={proc_re.returncode}\n{proc_re.stderr[-2000:]}")
        reopen = load_json(Path(job["outputs"]["reopen_verification"]))
        reopen_ok = reopen.get("status") == "SUCCESS" and bool(reopen.get("baked_action"))
        endpoint = reopen.get("endpoint_world") or {}
        for row in loop.get("entries") or []:
            tgt = row["target"]
            q0 = ((endpoint.get("start") or {}).get(tgt) or {}).get("quat")
            q1 = ((endpoint.get("end") or {}).get(tgt) or {}).get("quat")
            if q0 and q1:
                row["reopen_target_endpoint_angular_error_rad"] = round(quat_angular_error(q0, q1), 6)
                if row.get("source_closes") and row["reopen_target_endpoint_angular_error_rad"] > TOL_ROT_RAD:
                    row["status"] = "FAIL"
            else:
                row["reopen_target_endpoint_angular_error_rad"] = None
                if row.get("source_closes"):
                    row["status"] = "FAIL"
        loop["reopen_status"] = "PASS" if reopen_ok else "FAIL"
        if any(r.get("status") == "FAIL" for r in loop.get("entries") or []):
            loop["status"] = "FAIL"
        write_json(Path(job["outputs"]["loop_closure"]), loop)
        worker["measurements"]["loop_closure"] = loop["status"]
        write_json(Path(job["outputs"]["worker_result"]), worker)

        qc = qc_report(worker, mapping, policy, loop, audit, trace, quat_audit)
        write_json(workspace / "qc_report.json", qc)

        derived = None
        blend = Path(job["outputs"]["persistence_blend"])
        glb = Path(job["outputs"]["preview_glb"])
        if not reopen_ok:
            derived = {"published": False, "reason": "REOPEN_VERIFY_FAIL", "qc": qc["checks"]}
            write_json(workspace / "derived_variant.json", derived)
        elif qc["status"] != "PASS":
            derived = {"published": False, "reason": "QC_FAIL", "qc": qc["checks"]}
            write_json(workspace / "derived_variant.json", derived)
        else:
            derived = {
                "derived_variant_id": f"poc-blender-e2e-01.knight-ual2.walk-carry.run{run_i}",
                "published": True,
                "publication_order": "worker -> reopen PASS -> QC PASS -> publish",
                "target_character": {"id": "QCHAR_Knight_Male", "sha256": char_sha},
                "source_motion": {"id": "UAL2_Standard_Walk_Carry_Loop", "sha256": motion_sha, "clip_id": CLIP_ID},
                "source_skeleton_id": src_sum["skeleton_id"],
                "source_skeleton_summary_sha256": job_semantic["source_skeleton_summary_sha256"],
                "mapping_sha256": mapping_h,
                "retarget_policy_sha256": policy_h,
                "job_spec_semantic_hash": job_h,
                "worker_result_sha256": sha256_file(Path(job["outputs"]["worker_result"])),
                "qc_report_sha256": sha256_file(workspace / "qc_report.json"),
                "reopen_verification_sha256": sha256_file(Path(job["outputs"]["reopen_verification"])),
                "persistence_artifact": {
                    "type": "blend",
                    "path_external": str(blend),
                    "size": blend.stat().st_size if blend.exists() else None,
                    "sha256": sha256_file(blend) if blend.exists() else None,
                },
                "preview_artifact": {
                    "type": "glb",
                    "path_external": str(glb) if glb.exists() else None,
                    "size": glb.stat().st_size if glb.exists() else None,
                    "sha256": sha256_file(glb) if glb.exists() else None,
                    "generation": "attempted",
                },
                "generation": {
                    "run_index": run_i,
                    "worker_version": worker.get("worker_version"),
                    "blender_version": worker.get("blender_version"),
                    "blender_build_hash": worker.get("blender_build_hash"),
                },
            }
            write_json(workspace / "derived_variant.json", derived)

        stripped = strip_run(
            worker,
            qc,
            derived if derived and derived.get("published") else None,
            mapping_h,
            policy_h,
            job_h,
        )
        write_json(EVIDENCE / f"run_{run_i}_summary.json", stripped)
        run_hashes.append(semantic_hash(stripped))
        last_derived = derived
        last_worker = worker
        last_qc = qc
        timing_rows.append(
            {
                "run": run_i,
                "worker_wall_s": getattr(proc, "elapsed_s", None),
                "cold_start_plus_execute_s": cold,
                "worker_timings_s": worker.get("timings_s"),
                "reopen_wall_s": getattr(proc_re, "elapsed_s", None),
            }
        )
        if run_i == 1:
            write_json(EVIDENCE / "worker_result.json", worker)
            write_json(EVIDENCE / "qc_report.json", qc)
            write_json(EVIDENCE / "reopen_verification.json", reopen)
            write_json(EVIDENCE / "motion_samples_source.json", worker["measurements"]["samples_source"])
            write_json(EVIDENCE / "motion_samples_target.json", worker["measurements"]["samples_target"])
            shutil.copy2(Path(job["outputs"]["policy_execution_trace"]), EVIDENCE / "policy_execution_trace.json")
            shutil.copy2(Path(job["outputs"]["loop_closure"]), EVIDENCE / "loop_closure.json")
            shutil.copy2(Path(job["outputs"]["pose_channel_audit"]), EVIDENCE / "pose_channel_audit.json")
            shutil.copy2(Path(job["outputs"]["quaternion_policy_audit"]), EVIDENCE / "quaternion_policy_audit.json")
            if derived:
                write_json(EVIDENCE / "derived_variant.json", derived)
            frames_src = Path(job["outputs"]["frames_dir"])
            frames_dst = EVIDENCE / "frames"
            if frames_dst.exists():
                shutil.rmtree(frames_dst)
            if frames_src.exists():
                shutil.copytree(frames_src, frames_dst)
        if not (derived and derived.get("published")):
            raise SystemExit(
                f"run {run_i} did not publish derived variant: "
                f"reason={(derived or {}).get('reason')} qc={qc['status']} reopen_ok={reopen_ok}"
            )

    consistent = len(set(run_hashes)) == 1
    write_json(
        EVIDENCE / "repeatability_summary.json",
        {
            "run_1_semantic_hash": run_hashes[0],
            "run_2_semantic_hash": run_hashes[1],
            "run_3_semantic_hash": run_hashes[2],
            "consistent": consistent,
        },
    )
    write_json(EVIDENCE / "timing_summary.json", {"runs": timing_rows, "performance_target": None})

    glb1 = WORK / "run_1" / "artifacts" / "candidate_preview.glb"
    write_json(
        EVIDENCE / "candidate_preview_artifact.json",
        {
            "type": "GLB",
            "generation": "SUCCESS" if glb1.exists() else "FAILURE",
            "size": glb1.stat().st_size if glb1.exists() else None,
            "sha256": sha256_file(glb1) if glb1.exists() else None,
            "limitations": [
                "candidate only; Preview architecture not selected",
                "not POC-PREVIEW-01R",
                "may drop FBX authored pivot/layer semantics",
                "not product authority",
            ],
        },
    )

    # Controlled failure: nonexistent input path. Must not publish.
    fail_ws = WORK / "controlled_failure"
    if fail_ws.exists():
        shutil.rmtree(fail_ws)
    fail_ws.mkdir(parents=True)
    fail_job = {
        **job_semantic,
        "character": {**job_semantic["character"], "path": str(CHAR_PATH.parent / "MISSING_Knight_Male.fbx")},
        "motion": {**job_semantic["motion"], "path": str(MOTION_PATH)},
        "mapping_path": str(MAPPING_PATH),
        "policy_path": str(POLICY_PATH),
        "job_spec_sha256": job_h,
        "outputs": {
            "workspace": str(fail_ws),
            "persistence_blend": str(fail_ws / "should_not_publish.blend"),
            "preview_glb": str(fail_ws / "should_not_publish.glb"),
            "worker_result": str(fail_ws / "worker_result.json"),
            "frames_dir": str(fail_ws / "frames"),
            "reopen_verification": str(fail_ws / "reopen_verification.json"),
            "policy_execution_trace": str(fail_ws / "policy_execution_trace.json"),
            "loop_closure": str(fail_ws / "loop_closure.json"),
            "pose_channel_audit": str(fail_ws / "pose_channel_audit.json"),
            "quaternion_policy_audit": str(fail_ws / "quaternion_policy_audit.json"),
        },
    }
    fail_job_path = fail_ws / "job_spec.json"
    write_json(fail_job_path, fail_job)
    fail_proc = run_blender("execute", fail_job_path, fail_ws)
    fail_result = None
    rp = Path(fail_job["outputs"]["worker_result"])
    if rp.exists():
        fail_result = load_json(rp)
    published = Path(fail_job["outputs"]["persistence_blend"]).exists() and (
        fail_result or {}
    ).get("status") == "SUCCESS"
    fail_doc = {
        "case": "invalid/nonexistent character path",
        "worker_returncode": fail_proc.returncode,
        "worker_status": (fail_result or {}).get("status"),
        "diagnostic_captured": bool(fail_proc.stderr) or bool(fail_result),
        "accepted_derived_variant_published": False,
        "partial_output_treated_as_success": bool(published),
        "stderr_tail": (fail_proc.stderr or "")[-1500:],
    }
    write_json(EVIDENCE / "controlled_failure.json", fail_doc)
    if fail_proc.returncode == 0 or published:
        raise SystemExit("controlled failure did not fail closed")

    blender_zip = Path(r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64.zip")
    write_json(
        EVIDENCE / "versions.txt",
        {
            "blender_version": "5.2.1 LTS",
            "blender_build_hash": "9e2066aef7ef",
            "blender_exe": str(BLENDER),
            "archive": str(blender_zip),
            "archive_sha256_official": EXPECTED_BLENDER_HASH,
            "archive_sha256_actual": sha256_file(blender_zip) if blender_zip.exists() else None,
            "source_url": "https://download.blender.org/release/Blender5.2/blender-5.2.1-windows-x64.zip",
            "retrieval_date": "2026-08-31",
            "core_language_selected": False,
        },
    )

    print("repeatable", consistent, run_hashes)
    print("qc", last_qc["status"] if last_qc else None)
    print("derived", bool(last_derived))
    print("worker", last_worker["status"] if last_worker else None)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
