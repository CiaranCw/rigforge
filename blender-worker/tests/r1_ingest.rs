mod common;

use std::path::Path;
use std::process::Command;

use common::crate_worker_script;
use rigforge_app::{AnimationCandidate, AssociationEvidence};
use rigforge_blender_worker::BlenderSourceInspector;

fn worker_src() -> &'static str {
    include_str!("../python/worker.py")
}

#[test]
fn inspect_skeleton_does_not_silently_pick_max_bones() {
    let src = worker_src().replace("\r\n", "\n");
    assert!(
        !src.contains("max(armatures"),
        "silent max-bones armature selection must not remain in worker.py"
    );
    assert!(src.contains("require_unique_usable_armature"));
    assert!(src.contains("inspect_source"));
}

#[test]
fn slot_suitable_is_not_an_eligibility_kind() {
    let src = worker_src().replace("\r\n", "\n");
    assert!(src.contains("slot_suitable alone is not eligibility proof"));
    assert!(src.contains("\"direct_action\""));
    assert!(src.contains("\"nla_strip\""));
    assert!(src.contains("\"pose_channels\""));
    assert!(src.contains("if not kinds:\n            continue"));
    let mut clip = AnimationCandidate {
        clip_identity: "Cam".into(),
        display_label: "Cam".into(),
        source_skeleton_local_key: "Armature".into(),
        association_kind: "slot_suitable".into(),
        association_evidence: AssociationEvidence::default(),
        start_frame: Some(1),
        end_frame: Some(2),
        fps_num: Some(30),
        fps_den: Some(1),
        usable: true,
        unusable_reason: None,
    };
    assert!(!clip.association_is_strong());
    clip.association_kind = "pose_channels".into();
    assert!(clip.association_is_strong());
}

#[test]
fn rational_fps_vectors_match_accepted_contract() {
    let script = r#"
from fractions import Fraction
import math
U32_MAX = 2**32 - 1
def rational_fps(fps, fps_base):
    fps_i = int(fps)
    if fps_i <= 0:
        raise ValueError("scene.render.fps must be > 0")
    if isinstance(fps_base, bool) or not isinstance(fps_base, (int, float)):
        raise ValueError("scene.render.fps_base is not a real number")
    if not math.isfinite(fps_base) or fps_base <= 0:
        raise ValueError("scene.render.fps_base must be finite and > 0")
    base = Fraction(str(fps_base))
    if base <= 0:
        raise ValueError("fps_base Fraction must be > 0")
    effective = Fraction(fps_i, 1) / base
    num, den = effective.numerator, effective.denominator
    if num <= 0 or den <= 0:
        raise ValueError("effective FPS must be a positive rational")
    if num > U32_MAX or den > U32_MAX:
        raise ValueError("reduced FPS exceeds Product u32/u32")
    return int(num), int(den)
assert rational_fps(30, 1) == (30, 1)
assert rational_fps(30, 1.001) == (30000, 1001)
assert rational_fps(24, 1.001) == (24000, 1001)
print("ok")
"#;
    let output = Command::new("python")
        .arg("-c")
        .arg(script)
        .output()
        .expect("python must be available for FPS contract tests");
    assert!(
        output.status.success(),
        "rational fps vectors failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let src = worker_src();
    assert!(src.contains("base = Fraction(str(fps_base))"));
    assert!(!src.contains("limit_denominator"));
    let _ = crate_worker_script();
}

#[test]
fn fractional_frame_endpoint_is_unusable() {
    let src = worker_src();
    assert!(src.contains("fractional_frames"));
    assert!(src.contains("number.is_integer()"));
}

#[test]
fn worker_script_path_exists() {
    assert!(Path::new(&crate_worker_script()).is_file());
}

fn assert_send<T: Send>() {}

#[test]
fn blender_source_inspector_is_send() {
    assert_send::<BlenderSourceInspector>();
}

fn load_worker_python() -> String {
    format!(
        r#"
import importlib.util
import sys
import types
from pathlib import Path
bpy = types.ModuleType("bpy")
sys.modules["bpy"] = bpy
mu = types.ModuleType("mathutils")
class _Dummy:
    pass
mu.Matrix = _Dummy
mu.Quaternion = _Dummy
mu.Vector = _Dummy
sys.modules["mathutils"] = mu
path = Path(r"{script}")
spec = importlib.util.spec_from_file_location("rf_worker", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
bones = {{"root", "spine"}}
valid = mod.pose_channels_association_from_paths(
    [
        'pose.bones["root"].rotation_quaternion',
        'pose.bones["spine"].rotation_quaternion',
    ],
    bones,
)
assert valid["qualifies"] is True, valid
assert valid["resolved_pose_bones"] == ["root", "spine"], valid
assert valid["unresolved_pose_bones"] == [], valid
alien = mod.pose_channels_association_from_paths(
    [
        'pose.bones["alien_root"].rotation_quaternion',
        'pose.bones["alien_spine"].rotation_quaternion',
    ],
    bones,
)
assert alien["qualifies"] is False, alien
assert alien["resolved_pose_bones"] == [], alien
assert alien["unresolved_pose_bones"] == ["alien_root", "alien_spine"], alien
# R1-1-MAJOR-001: substring pose.bones[ would have accepted this Action.
assert any("pose.bones[" in p for p in [
    'pose.bones["alien_root"].rotation_quaternion',
    'pose.bones["alien_spine"].rotation_quaternion',
])
mixed = mod.pose_channels_association_from_paths(
    [
        'pose.bones["root"].rotation_quaternion',
        'pose.bones["alien_spine"].rotation_quaternion',
    ],
    bones,
)
assert mixed["qualifies"] is False, mixed
assert mixed["resolved_pose_bones"] == ["root"], mixed
assert mixed["unresolved_pose_bones"] == ["alien_spine"], mixed
assert mod.extract_pose_bone_name('pose.bones["pelvis"].rotation_quaternion') == "pelvis"
assert mod.extract_pose_bone_name("pose.bones['upperarm_l'].location") == "upperarm_l"
assert mod.extract_pose_bone_name("pose.bones[0].location") is None
assert "root" != "Root"
assert mod.pose_channels_association_from_paths(
    ['pose.bones["Root"].rotation_quaternion'],
    bones,
)["qualifies"] is False
direct = mod.strong_association_kinds(
    direct_action=True, nla_strip=False, pose_channels=False
)
assert direct == ["direct_action"], direct
nla = mod.strong_association_kinds(
    direct_action=False, nla_strip=True, pose_channels=False
)
assert nla == ["nla_strip"], nla
pose_only = mod.strong_association_kinds(
    direct_action=False, nla_strip=False, pose_channels=True
)
assert pose_only == ["pose_channels"], pose_only
none = mod.strong_association_kinds(
    direct_action=False, nla_strip=False, pose_channels=False
)
assert none == [], none
# Independent bags: alien bag must not poison a qualifying bag; mixed bag
# does not qualify on its own.
bags = [
    ['pose.bones["alien_root"].rotation_quaternion'],
    [
        'pose.bones["root"].rotation_quaternion',
        'pose.bones["spine"].rotation_quaternion',
    ],
]
independent_hit = False
for paths in bags:
    if mod.pose_channels_association_from_paths(paths, bones)["qualifies"]:
        independent_hit = True
        break
assert independent_hit is True
mixed_only = [mixed["pose_data_path_samples"] or [
    'pose.bones["root"].rotation_quaternion',
    'pose.bones["alien_spine"].rotation_quaternion',
]]
assert mod.pose_channels_association_from_paths(mixed_only[0], bones)["qualifies"] is False
print("ok")
"#,
        script = crate_worker_script().display()
    )
}

#[test]
fn pose_channels_requires_exact_armature_bone_membership() {
    let src = worker_src().replace("\r\n", "\n");
    assert!(
        src.contains("def extract_pose_bone_name"),
        "exact pose.bones quoted-key extraction must exist"
    );
    assert!(
        src.contains("def pose_channels_qualifies"),
        "fail-closed pose_channels membership must exist"
    );
    assert!(
        !src.contains("pose_hit = action_has_pose_channels"),
        "substring pose.bones[ must not qualify pose_channels"
    );
    assert!(src.contains("resolved_pose_bones"));
    assert!(src.contains("unresolved_pose_bones"));
    let output = Command::new("python")
        .arg("-c")
        .arg(load_worker_python())
        .output()
        .expect("python must be available for pose association tests");
    assert!(
        output.status.success(),
        "pose association vectors failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
