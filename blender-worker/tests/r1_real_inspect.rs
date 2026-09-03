mod common;

use std::path::Path;

use common::ensure_test_runtime;
use rigforge_app::SourceInspectionProvider;
use rigforge_blender_worker::BlenderSourceInspector;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";

#[test]
fn inspect_frozen_knight_and_ual2_when_present() {
    if !Path::new(KNIGHT).is_file() || !Path::new(UAL2).is_file() {
        eprintln!("R1-1 real inspection: NOT REPRODUCED");
        return;
    }
    ensure_test_runtime();
    let inspector = BlenderSourceInspector::production().unwrap();
    let character = inspector.inspect_character_source(Path::new(KNIGHT)).unwrap();
    println!(
        "R1-1 Character Knight usable_armature_count={}",
        character.usable_armature_count
    );
    if let Some(skeleton) = character.skeleton_candidates.first() {
        println!(
            "R1-1 Character skeleton={} joints={}",
            skeleton.display_name, skeleton.joint_count
        );
    }
    let motion = inspector.inspect_motion_source(Path::new(UAL2)).unwrap();
    println!(
        "R1-1 Motion UAL2 usable_armature_count={}",
        motion.usable_armature_count
    );
    if let Some(skeleton) = motion.skeleton_candidates.first() {
        println!(
            "R1-1 Motion skeleton={} joints={}",
            skeleton.display_name, skeleton.joint_count
        );
    }
    if let Some(timing) = &motion.timing_context {
        println!("R1-1 Motion fps={}/{}", timing.fps_num, timing.fps_den);
    }
    for clip in &motion.animation_candidates {
        println!(
            "R1-1 Motion clip identity={} label={} kind={} usable={} frames={:?}-{:?} fps={:?}/{:?}",
            clip.clip_identity,
            clip.display_label,
            clip.association_kind,
            clip.usable,
            clip.start_frame,
            clip.end_frame,
            clip.fps_num,
            clip.fps_den
        );
    }
    assert_eq!(character.usable_armature_count, 1);
    assert_eq!(motion.usable_armature_count, 1);
    assert!(
        !motion.usable_clips().is_empty(),
        "UAL2 should expose at least one strongly associated clip"
    );
}
