use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rigforge_app::{Application, FixtureSkeletonInspector};
use rigforge_domain::*;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_named(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rf-reg-it-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        uuid::Uuid::now_v7()
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

fn sha256_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn fresh_catalog_register_character_lists_and_resolves_published_version() {
    let bytes = b"character-registration-bytes-v1";
    let path = temp_named("knight.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let mut app = Application::open_in_memory().unwrap();
    assert!(app.list_characters().unwrap().is_empty());
    let registered = app
        .register_local_character("Knight", &path, &FixtureSkeletonInspector::usable())
        .unwrap();
    let listed = app.list_characters().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].logical_id, registered.asset_id);
    assert_eq!(
        listed[0].published_version_id.as_deref(),
        Some(registered.version_id.as_str())
    );
    let version = app
        .resolve_exact_character_version(&registered.version_id)
        .unwrap();
    let record = version.as_record();
    assert_eq!(record.lifecycle(), Lifecycle::Published);
    assert_eq!(
        record.source().location().value(),
        path.to_string_lossy().as_ref()
    );
    assert_eq!(record.source().digest().sha256(), digest);
    assert_eq!(record.source().size_bytes(), bytes.len() as u64);
    assert_ne!(registered.asset_id, path.to_string_lossy().as_ref());
    assert_ne!(registered.version_id, path.to_string_lossy().as_ref());
    assert_ne!(registered.asset_id, digest);
    assert_ne!(registered.version_id, digest);
}

#[test]
fn fresh_catalog_register_motion_persists_source_skeleton_and_published_version() {
    let bytes = b"motion-registration-bytes-v1";
    let path = temp_named("ual2.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let mut app = Application::open_in_memory().unwrap();
    assert!(app.list_motions().unwrap().is_empty());
    let registered = app
        .register_local_motion(
            "Walk Carry",
            &path,
            "UAL2_Standard",
            "Armature|Armature|Walk_Carry_Loop",
            1,
            61,
            30,
            1,
            &FixtureSkeletonInspector::usable(),
        )
        .unwrap();
    let listed = app.list_motions().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].logical_id, registered.asset_id);
    assert_eq!(
        listed[0].published_version_id.as_deref(),
        Some(registered.version_id.as_str())
    );
    let skeleton = app
        .lookup_source_skeleton(&registered.source_skeleton_id)
        .unwrap();
    assert_eq!(skeleton.as_record().display_name(), "UAL2_Standard");
    let version = app
        .resolve_exact_motion_version(&registered.version_id)
        .unwrap();
    let record = version.as_record();
    assert_eq!(record.lifecycle(), Lifecycle::Published);
    assert_eq!(
        record.source_skeleton_ref_id().canonical(),
        registered.source_skeleton_id
    );
    assert_eq!(
        record.source().location().value(),
        path.to_string_lossy().as_ref()
    );
    assert_eq!(record.source().digest().sha256(), digest);
    assert_eq!(record.source().size_bytes(), bytes.len() as u64);
    assert_eq!(
        record.time().clip_identity_evidence(),
        "Armature|Armature|Walk_Carry_Loop"
    );
    assert_ne!(registered.asset_id, path.to_string_lossy().as_ref());
    assert_ne!(registered.version_id, digest);
}

#[test]
fn same_source_bytes_registered_twice_receive_distinct_product_identities() {
    let bytes = b"same-bytes-twice";
    let path = temp_named("shared.fbx", bytes);
    let mut app = Application::open_in_memory().unwrap();
    let inspector = FixtureSkeletonInspector::usable();
    let first = app
        .register_local_character("First", &path, &inspector)
        .unwrap();
    let second = app
        .register_local_character("Second", &path, &inspector)
        .unwrap();
    assert_ne!(first.asset_id, second.asset_id);
    assert_ne!(first.version_id, second.version_id);
    let first_v = app
        .resolve_exact_character_version(&first.version_id)
        .unwrap();
    let second_v = app
        .resolve_exact_character_version(&second.version_id)
        .unwrap();
    assert_eq!(
        first_v.as_record().source().digest().sha256(),
        second_v.as_record().source().digest().sha256()
    );
}

#[test]
fn registration_rejects_missing_directory_unsupported_and_uninspectable_sources() {
    let mut app = Application::open_in_memory().unwrap();
    let inspector = FixtureSkeletonInspector::usable();
    let missing = std::env::temp_dir().join("rf-missing-no-such-file.fbx");
    let missing_err = app
        .register_local_character("Missing", &missing, &inspector)
        .unwrap_err()
        .to_string();
    assert!(missing_err.contains("missing"), "{missing_err}");

    let dir = std::env::temp_dir().join(format!(
        "rf-reg-dir-{}",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let dir_err = app
        .register_local_character("Dir", &dir, &inspector)
        .unwrap_err()
        .to_string();
    assert!(dir_err.contains("directory"), "{dir_err}");

    let obj = temp_named("mesh.obj", b"not-fbx");
    let fmt_err = app
        .register_local_character("Obj", &obj, &inspector)
        .unwrap_err()
        .to_string();
    assert!(fmt_err.contains("unsupported source format"), "{fmt_err}");

    let empty_name = app
        .register_local_character("  ", &obj, &inspector)
        .unwrap_err()
        .to_string();
    assert!(empty_name.contains("display name"), "{empty_name}");

    let fbx = temp_named("empty.fbx", b"no-joints");
    let empty = app
        .register_local_character("Empty", &fbx, &FixtureSkeletonInspector::empty_joints())
        .unwrap_err()
        .to_string();
    assert!(empty.contains("no usable Skeleton evidence"), "{empty}");
    assert!(app.list_characters().unwrap().is_empty());

    let fail = app
        .register_local_character("Fail", &fbx, &FixtureSkeletonInspector::failing())
        .unwrap_err()
        .to_string();
    assert!(fail.contains("inspection failed"), "{fail}");
    assert!(app.list_characters().unwrap().is_empty());
}

#[test]
fn motion_registration_rejects_invalid_time_and_skeleton_context() {
    let fbx = temp_named("clip.fbx", b"motion");
    let mut app = Application::open_in_memory().unwrap();
    let inspector = FixtureSkeletonInspector::usable();
    let clip_err = app
        .register_local_motion("Walk", &fbx, "UAL2", "  ", 1, 10, 30, 1, &inspector)
        .unwrap_err()
        .to_string();
    assert!(clip_err.contains("clip identifier"), "{clip_err}");
    let skel_err = app
        .register_local_motion("Walk", &fbx, "  ", "clip", 1, 10, 30, 1, &inspector)
        .unwrap_err()
        .to_string();
    assert!(skel_err.contains("Source Skeleton"), "{skel_err}");
    let fps_err = app
        .register_local_motion("Walk", &fbx, "UAL2", "clip", 1, 10, 30, 0, &inspector)
        .unwrap_err()
        .to_string();
    assert!(fps_err.contains("FPS"), "{fps_err}");
    let order_err = app
        .register_local_motion("Walk", &fbx, "UAL2", "clip", 10, 1, 30, 1, &inspector)
        .unwrap_err()
        .to_string();
    assert!(
        order_err.contains("start must be <=") || order_err.contains("time-domain"),
        "{order_err}"
    );
    assert!(app.list_motions().unwrap().is_empty());
    assert!(app
        .catalog()
        .list_assets(RecordType::SourceSkeletonReference)
        .unwrap()
        .is_empty());
}

fn joint() -> rigforge_app::InspectedJoint {
    rigforge_app::InspectedJoint {
        joint_key: "root".into(),
        display_name: "root".into(),
        parent_key: None,
        is_root: true,
        deform_observation: Some("deforming".into()),
        rest_evidence: Some("rest".into()),
    }
}

fn character_inspection(
    path: PathBuf,
    digest: String,
    size: u64,
    armature_count: usize,
) -> rigforge_app::CharacterSourceInspection {
    let candidates = if armature_count == 1 {
        vec![rigforge_app::SkeletonCandidate {
            source_local_key: "Armature".into(),
            display_name: "Armature".into(),
            joint_count: 1,
            joints: vec![joint()],
        }]
    } else {
        Vec::new()
    };
    rigforge_app::CharacterSourceInspection {
        source_path: path,
        source_digest: digest,
        size_bytes: size,
        observed_media_type: "application/octet-stream".into(),
        usable_armature_count: armature_count,
        skeleton_candidates: candidates,
        diagnostics: Vec::new(),
    }
}

fn motion_inspection(
    path: PathBuf,
    digest: String,
    size: u64,
    clips: Vec<rigforge_app::AnimationCandidate>,
) -> rigforge_app::MotionSourceInspection {
    rigforge_app::MotionSourceInspection {
        source_path: path,
        source_digest: digest,
        size_bytes: size,
        observed_media_type: "application/octet-stream".into(),
        usable_armature_count: 1,
        skeleton_candidates: vec![rigforge_app::SkeletonCandidate {
            source_local_key: "Armature".into(),
            display_name: "UAL2_Armature".into(),
            joint_count: 1,
            joints: vec![joint()],
        }],
        animation_candidates: clips,
        timing_context: Some(rigforge_app::ObservedTimingContext {
            fps_num: 30,
            fps_den: 1,
        }),
        diagnostics: Vec::new(),
    }
}

fn usable_clip(identity: &str, kind: &str) -> rigforge_app::AnimationCandidate {
    rigforge_app::AnimationCandidate {
        clip_identity: identity.into(),
        display_label: identity.replace('_', " "),
        source_skeleton_local_key: "Armature".into(),
        association_kind: kind.into(),
        association_evidence: rigforge_app::AssociationEvidence::default(),
        start_frame: Some(1),
        end_frame: Some(61),
        fps_num: Some(30),
        fps_den: Some(1),
        usable: true,
        unusable_reason: None,
    }
}

#[test]
fn character_from_inspection_rejects_zero_and_multiple_armatures() {
    let bytes = b"character-inspection-bytes";
    let path = temp_named("hero.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let zero = character_inspection(path.clone(), digest.clone(), size, 0);
    let err = app
        .register_character_from_inspection("Hero", &zero)
        .unwrap_err()
        .to_string();
    assert!(err.contains("No skeleton"), "{err}");
    let many = character_inspection(path.clone(), digest.clone(), size, 2);
    let err = app
        .register_character_from_inspection("Hero", &many)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Multiple usable skeletons"), "{err}");
}

#[test]
fn character_from_inspection_accepts_unique_armature_and_toctou() {
    let bytes = b"character-inspection-ok";
    let path = temp_named("knight.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let inspection = character_inspection(path.clone(), digest.clone(), size, 1);
    let registered = app
        .register_character_from_inspection("Knight Male", &inspection)
        .unwrap();
    assert_ne!(registered.asset_id, digest);
    fs::write(&path, b"changed-after-inspect").unwrap();
    let stale = character_inspection(path, digest, size, 1);
    let err = app
        .register_character_from_inspection("Knight Male", &stale)
        .unwrap_err()
        .to_string();
    assert!(err.contains("changed after it was inspected"), "{err}");
}

#[test]
fn motion_from_inspection_uses_exact_clip_identity_and_rejects_empty_clips() {
    let bytes = b"motion-inspection-bytes";
    let path = temp_named("walk.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let empty = motion_inspection(path.clone(), digest.clone(), size, Vec::new());
    let err = app
        .register_motion_from_inspection("Walk", &empty, "clip")
        .unwrap_err()
        .to_string();
    assert!(err.contains("No usable animation clips"), "{err}");
    let identity = "Armature|Armature|Walk_Carry_Loop";
    let inspection = motion_inspection(
        path,
        digest,
        size,
        vec![usable_clip(identity, "direct_action")],
    );
    let registered = app
        .register_motion_from_inspection("Walk Carry", &inspection, identity)
        .unwrap();
    let version = app
        .resolve_exact_motion_version(&registered.version_id)
        .unwrap();
    assert_eq!(
        version.as_record().time().clip_identity_evidence(),
        identity
    );
}

#[test]
fn motion_slot_suitable_alone_is_not_usable() {
    let bytes = b"motion-slot-only";
    let path = temp_named("slots.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let mut clip = usable_clip("CameraAction", "slot_suitable");
    clip.usable = true;
    let inspection = motion_inspection(path, digest, size, vec![clip]);
    let err = app
        .register_motion_from_inspection("Cam", &inspection, "CameraAction")
        .unwrap_err()
        .to_string();
    assert!(err.contains("No usable animation clips"), "{err}");
}

#[test]
fn motion_from_inspection_accepts_nla_and_pose_channel_clips() {
    let bytes = b"motion-association-kinds";
    let path = temp_named("assoc.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    for kind in ["nla_strip", "pose_channels"] {
        let inspection = motion_inspection(
            path.clone(),
            digest.clone(),
            size,
            vec![usable_clip("Walk_Carry_Loop", kind)],
        );
        app.register_motion_from_inspection("Walk Carry", &inspection, "Walk_Carry_Loop")
            .unwrap();
    }
}

#[test]
fn motion_from_inspection_rejects_unrelated_and_fractional_and_stores_rational_fps() {
    let bytes = b"motion-timing-bytes";
    let path = temp_named("timing.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let unrelated = usable_clip("CameraShake", "object");
    let err = app
        .register_motion_from_inspection(
            "Cam",
            &motion_inspection(path.clone(), digest.clone(), size, vec![unrelated]),
            "CameraShake",
        )
        .unwrap_err()
        .to_string();
    assert!(err.contains("No usable animation clips"), "{err}");

    let mut fractional = usable_clip("Walk_Carry_Loop", "direct_action");
    fractional.usable = false;
    fractional.start_frame = None;
    fractional.end_frame = None;
    fractional.unusable_reason = Some("fractional_frames".into());
    let err = app
        .register_motion_from_inspection(
            "Walk",
            &motion_inspection(path.clone(), digest.clone(), size, vec![fractional]),
            "Walk_Carry_Loop",
        )
        .unwrap_err()
        .to_string();
    assert!(err.contains("fractional frame"), "{err}");

    let mut clip = usable_clip("Walk_Carry_Loop", "direct_action");
    clip.fps_num = Some(30000);
    clip.fps_den = Some(1001);
    let registered = app
        .register_motion_from_inspection(
            "Walk NTSC",
            &motion_inspection(path.clone(), digest.clone(), size, vec![clip]),
            "Walk_Carry_Loop",
        )
        .unwrap();
    let version = app
        .resolve_exact_motion_version(&registered.version_id)
        .unwrap();
    assert_eq!(version.as_record().time().start().fps_num(), Some(30000));
    assert_eq!(version.as_record().time().start().fps_den(), Some(1001));

    let mut ntsc24 = usable_clip("Walk_Carry_Loop", "direct_action");
    ntsc24.fps_num = Some(24000);
    ntsc24.fps_den = Some(1001);
    let registered = app
        .register_motion_from_inspection(
            "Walk 24",
            &motion_inspection(path, digest, size, vec![ntsc24]),
            "Walk_Carry_Loop",
        )
        .unwrap();
    let version = app
        .resolve_exact_motion_version(&registered.version_id)
        .unwrap();
    assert_eq!(version.as_record().time().start().fps_num(), Some(24000));
    assert_eq!(version.as_record().time().start().fps_den(), Some(1001));
}

#[test]
fn motion_from_inspection_rejects_multiple_armatures_and_size_mismatch() {
    let bytes = b"motion-toctou-bytes";
    let path = temp_named("multi.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let mut many = motion_inspection(
        path.clone(),
        digest.clone(),
        size,
        vec![usable_clip("Walk_Carry_Loop", "direct_action")],
    );
    many.usable_armature_count = 2;
    let err = app
        .register_motion_from_inspection("Walk", &many, "Walk_Carry_Loop")
        .unwrap_err()
        .to_string();
    assert!(err.contains("Multiple usable skeletons"), "{err}");

    let wrong_size = motion_inspection(
        path,
        digest,
        size + 1,
        vec![usable_clip("Walk_Carry_Loop", "direct_action")],
    );
    let err = app
        .register_motion_from_inspection("Walk", &wrong_size, "Walk_Carry_Loop")
        .unwrap_err()
        .to_string();
    assert!(err.contains("changed after it was inspected"), "{err}");
}

#[test]
fn motion_from_inspection_keeps_selected_clip_when_several_are_usable() {
    let bytes = b"motion-multi-clip-bytes";
    let path = temp_named("clips.fbx", bytes);
    let digest = sha256_bytes(bytes);
    let size = bytes.len() as u64;
    let mut app = Application::open_in_memory().unwrap();
    let inspection = motion_inspection(
        path,
        digest,
        size,
        vec![
            usable_clip("Walk_Carry_Loop", "direct_action"),
            usable_clip("Zombie_Walk_Fwd_Loop", "nla_strip"),
        ],
    );
    let registered = app
        .register_motion_from_inspection("Zombie Walk", &inspection, "Zombie_Walk_Fwd_Loop")
        .unwrap();
    let version = app
        .resolve_exact_motion_version(&registered.version_id)
        .unwrap();
    assert_eq!(
        version.as_record().time().clip_identity_evidence(),
        "Zombie_Walk_Fwd_Loop"
    );
}
