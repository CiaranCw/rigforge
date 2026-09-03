use std::path::PathBuf;

use rigforge_app::{
    AnimationCandidate, AssociationEvidence, CharacterSourceInspection, InspectedJoint,
    MotionSourceInspection, ObservedTimingContext, SkeletonCandidate,
};
use rigforge_workbench::file_pick::{apply_picked_path, derive_display_name, FilePickState};
use rigforge_workbench::WorkbenchApp;

fn joint() -> InspectedJoint {
    InspectedJoint {
        joint_key: "root".into(),
        display_name: "root".into(),
        parent_key: None,
        is_root: true,
        deform_observation: Some("deforming".into()),
        rest_evidence: Some("rest".into()),
    }
}

#[test]
fn cancel_preserves_prior_file_selection() {
    let mut state = FilePickState::default();
    assert!(apply_picked_path(
        &mut state,
        Some(PathBuf::from(r"C:\assets\Knight_Male.fbx"))
    ));
    let kept = state.clone();
    assert!(!apply_picked_path(&mut state, None));
    assert_eq!(state, kept);
}

#[test]
fn filename_derived_names_and_custom_name_survives_change() {
    assert_eq!(
        derive_display_name(&PathBuf::from("Knight_Male.fbx")),
        "Knight Male"
    );
    assert_eq!(
        derive_display_name(&PathBuf::from("UAL2_Standard.fbx")),
        "UAL2 Standard"
    );
    let mut state = FilePickState::default();
    apply_picked_path(&mut state, Some(PathBuf::from(r"C:\assets\Knight_Male.fbx")));
    assert_eq!(state.display_name, "Knight Male");
    assert_eq!(state.filename, "Knight_Male.fbx");
    apply_picked_path(
        &mut state,
        Some(PathBuf::from(r"C:\assets\Goblin_Male.fbx")),
    );
    assert_eq!(state.display_name, "Goblin Male");
    state.display_name = "My Hero".into();
    apply_picked_path(
        &mut state,
        Some(PathBuf::from(r"C:\assets\UAL2_Standard.fbx")),
    );
    assert_eq!(state.display_name, "My Hero");
    assert_eq!(state.filename, "UAL2_Standard.fbx");
    assert_eq!(
        state.path.as_ref().unwrap().to_string_lossy(),
        r"C:\assets\UAL2_Standard.fbx"
    );
}

#[test]
fn stale_character_inspection_cannot_overwrite_newer_selection() {
    let mut shell = WorkbenchApp::empty();
    assert!(shell.apply_character_selection(Some(PathBuf::from(r"C:\assets\A.fbx"))));
    assert!(shell.apply_character_selection(Some(PathBuf::from(r"C:\assets\B.fbx"))));
    shell.apply_character_inspect_result(
        1,
        Ok(CharacterSourceInspection {
            source_path: PathBuf::from(r"C:\assets\A.fbx"),
            source_digest: "aa".repeat(32),
            size_bytes: 10,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates: vec![SkeletonCandidate {
                source_local_key: "Armature".into(),
                display_name: "Stale".into(),
                joint_count: 1,
                joints: vec![joint()],
            }],
            diagnostics: Vec::new(),
        }),
    );
    assert_eq!(
        shell
            .character_pick()
            .path
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap(),
        "B.fbx"
    );
    assert!(shell.character_inspection().is_none());
}

#[test]
fn motion_multi_clip_requires_explicit_selection() {
    let mut shell = WorkbenchApp::empty();
    shell.apply_motion_selection(Some(PathBuf::from(r"C:\assets\UAL2_Standard.fbx")));
    let request_id = 1;
    let clip = |identity: &str| AnimationCandidate {
        clip_identity: identity.into(),
        display_label: identity.replace('_', " "),
        source_skeleton_local_key: "Armature".into(),
        association_kind: "direct_action".into(),
        association_evidence: AssociationEvidence::default(),
        start_frame: Some(1),
        end_frame: Some(61),
        fps_num: Some(30),
        fps_den: Some(1),
        usable: true,
        unusable_reason: None,
    };
    shell.apply_motion_inspect_result(
        request_id,
        Ok(MotionSourceInspection {
            source_path: PathBuf::from(r"C:\assets\UAL2_Standard.fbx"),
            source_digest: "bb".repeat(32),
            size_bytes: 12,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates: vec![SkeletonCandidate {
                source_local_key: "Armature".into(),
                display_name: "Armature".into(),
                joint_count: 1,
                joints: vec![joint()],
            }],
            animation_candidates: vec![
                clip("Walk_Carry_Loop"),
                clip("Zombie_Walk_Fwd_Loop"),
            ],
            timing_context: Some(ObservedTimingContext {
                fps_num: 30,
                fps_den: 1,
            }),
            diagnostics: Vec::new(),
        }),
    );
    assert!(shell.motion_selected_clip().is_none());
    assert!(!shell.motion_can_add());
}

#[test]
fn motion_single_clip_is_auto_selected() {
    let mut shell = WorkbenchApp::empty();
    shell.apply_motion_selection(Some(PathBuf::from(r"C:\assets\walk.fbx")));
    let clip = AnimationCandidate {
        clip_identity: "Walk_Carry_Loop".into(),
        display_label: "Walk Carry Loop".into(),
        source_skeleton_local_key: "Armature".into(),
        association_kind: "nla_strip".into(),
        association_evidence: AssociationEvidence::default(),
        start_frame: Some(1),
        end_frame: Some(61),
        fps_num: Some(30),
        fps_den: Some(1),
        usable: true,
        unusable_reason: None,
    };
    shell.apply_motion_inspect_result(
        1,
        Ok(MotionSourceInspection {
            source_path: PathBuf::from(r"C:\assets\walk.fbx"),
            source_digest: "cc".repeat(32),
            size_bytes: 8,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates: vec![SkeletonCandidate {
                source_local_key: "Armature".into(),
                display_name: "Armature".into(),
                joint_count: 1,
                joints: vec![joint()],
            }],
            animation_candidates: vec![clip],
            timing_context: Some(ObservedTimingContext {
                fps_num: 30,
                fps_den: 1,
            }),
            diagnostics: Vec::new(),
        }),
    );
    assert_eq!(shell.motion_selected_clip(), Some("Walk_Carry_Loop"));
}

#[test]
fn ingest_ui_does_not_require_typed_path() {
    let src = include_str!("../src/lib.rs");
    assert!(src.contains("pick_fbx_file"));
    assert!(src.contains("Add Character"));
    assert!(src.contains("Add Motion"));
    assert!(!src.contains("ui.text_edit_singleline(&mut self.character_path_input)"));
    assert!(!src.contains("character_path_input"));
    assert!(!src.contains("motion_path_input"));
    assert!(src.contains("Inspecting Character"));
    assert!(src.contains("Inspecting Motion"));
}

#[test]
fn workbench_cancel_preserves_character_selection() {
    let mut shell = WorkbenchApp::empty();
    assert!(shell.apply_character_selection(Some(PathBuf::from(r"C:\assets\Knight_Male.fbx"))));
    assert!(!shell.apply_character_selection(None));
    assert_eq!(
        shell.character_pick().filename,
        "Knight_Male.fbx"
    );
}

#[test]
fn motion_zero_usable_clips_cannot_add() {
    let mut shell = WorkbenchApp::empty();
    shell.apply_motion_selection(Some(PathBuf::from(r"C:\assets\empty.fbx")));
    shell.apply_motion_inspect_result(
        1,
        Ok(MotionSourceInspection {
            source_path: PathBuf::from(r"C:\assets\empty.fbx"),
            source_digest: "dd".repeat(32),
            size_bytes: 4,
            observed_media_type: "application/octet-stream".into(),
            usable_armature_count: 1,
            skeleton_candidates: vec![SkeletonCandidate {
                source_local_key: "Armature".into(),
                display_name: "Armature".into(),
                joint_count: 1,
                joints: vec![joint()],
            }],
            animation_candidates: Vec::new(),
            timing_context: Some(ObservedTimingContext {
                fps_num: 30,
                fps_den: 1,
            }),
            diagnostics: Vec::new(),
        }),
    );
    assert!(shell.motion_selected_clip().is_none());
    assert!(!shell.motion_can_add());
    assert_eq!(
        shell.motion_ingest_status(),
        Some("No usable animation clips were found.")
    );
}
