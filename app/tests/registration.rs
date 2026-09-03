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
