use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rigforge_app::rigforge_domain::*;
use rigforge_app::{
    persist_unpublished_without_authority, sha256_bytes, sha256_file, synthetic_preview_glb,
    unpublished_graph, Application, MemoryPreviewGenerator, PreviewGenerationRequest,
    PreviewSubject, PREVIEW_MEDIA_TYPE,
};
use rigforge_workbench::preview_host::PreviewHost;
use rigforge_workbench::WorkbenchApp;

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rf-v16-wb-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

fn source_from_file(path: &Path) -> SourceArtifactEvidence {
    let bytes = fs::read(path).unwrap();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path.display().to_string()).unwrap(),
        ContentDigest::parse(&sha256_bytes(&bytes)).unwrap(),
        bytes.len() as u64,
        "application/octet-stream",
        Some("v1-6-workbench".into()),
        None,
    )
    .unwrap()
}

fn seeded_app() -> (Application, String, String) {
    let character_src = temp_file("character.bin", b"character-source-bytes");
    let motion_src = temp_file("motion.bin", b"motion-source-bytes-xxxxxxxx");
    let mut app = Application::open_in_memory().unwrap();
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        "Knight v1",
        source_from_file(&character_src),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            "clip:walk-carry",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        source_from_file(&motion_src),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character.clone()).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion.clone()).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    (
        app,
        character_version.id().canonical(),
        motion_version.id().canonical(),
    )
}

fn measure(url: &str, kind: &str) -> serde_json::Value {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("preview-viewer")
        .join("measure_controls.py");
    let output = Command::new("python")
        .args([script.to_str().unwrap(), url, kind])
        .output()
        .expect("python measure_controls.py");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "viewer measure failed kind={kind} stdout={stdout} stderr={stderr}"
    );
    serde_json::from_str(&stdout).unwrap_or_else(|_| serde_json::json!({ "raw": stdout }))
}

#[test]
fn click_to_preview_character_and_motion_without_blender_generation() {
    let (mut app, character_id, motion_id) = seeded_app();
    app.generate_preview_with(
        &MemoryPreviewGenerator::default(),
        PreviewGenerationRequest::character(&character_id),
    )
    .unwrap();
    app.generate_preview_with(
        &MemoryPreviewGenerator::default(),
        PreviewGenerationRequest::motion(&motion_id),
    )
    .unwrap();
    let mut shell = WorkbenchApp::from_application(&app).unwrap();
    let session = shell
        .request_preview(
            &mut app,
            PreviewSubject::Character {
                version_id: character_id.clone(),
            },
            false,
            false,
        )
        .unwrap();
    assert!(shell.preview_valid());
    assert!(session.document.valid);
    assert!(session.root.join("payload.glb").is_file());
    let host = PreviewHost::serve(&session).unwrap();
    let measured = measure(&host.url, "character");
    assert_eq!(measured["loaded"], true);
    assert_eq!(measured["offline_local"], true);
    assert_eq!(measured["camera"]["orbit_changed"], true);
    assert_eq!(measured["camera"]["zoom_changed"], true);

    let motion_session = shell
        .request_preview(
            &mut app,
            PreviewSubject::Motion {
                version_id: motion_id,
            },
            false,
            false,
        )
        .unwrap();
    assert!(shell.preview_valid());
    let motion_host = PreviewHost::serve(&motion_session).unwrap();
    let motion_m = measure(&motion_host.url, "motion");
    assert_eq!(motion_m["transport"]["play_advanced"], true);
    assert_eq!(motion_m["transport"]["pause_held"], true);
    assert_eq!(motion_m["transport"]["seek_ok"], true);
    assert_eq!(motion_m["transport"]["restart_near_start"], true);
}

#[test]
fn selection_change_clears_previous_valid_preview() {
    let (mut app, character_id, _motion_id) = seeded_app();
    app.generate_preview_with(
        &MemoryPreviewGenerator::default(),
        PreviewGenerationRequest::character(&character_id),
    )
    .unwrap();
    let mut shell = WorkbenchApp::from_application(&app).unwrap();
    shell
        .request_preview(
            &mut app,
            PreviewSubject::Character {
                version_id: character_id,
            },
            false,
            false,
        )
        .unwrap();
    assert!(shell.preview_valid());
    shell.select_character_version("not-the-previous-version");
    assert!(!shell.preview_valid());
    assert!(shell
        .preview_status()
        .contains("selection changed"));
}

#[test]
fn invalid_session_does_not_write_or_serve_payload() {
    let (mut app, character_id, _) = seeded_app();
    let mut shell = WorkbenchApp::from_application(&app).unwrap();
    let session = shell
        .request_preview(
            &mut app,
            PreviewSubject::Character {
                version_id: character_id,
            },
            false,
            false,
        )
        .unwrap();
    assert!(!session.document.valid);
    assert!(!session.root.join("payload.glb").is_file());
    let host = PreviewHost::serve(&session).unwrap();
    let measured = measure(&host.url, "fail");
    assert_eq!(measured["fail_closed"], true);
    assert_eq!(measured["valid"], false);
}

#[test]
fn payload_digest_is_application_sha256() {
    let (mut app, character_id, _) = seeded_app();
    app.generate_preview_with(
        &MemoryPreviewGenerator::default(),
        PreviewGenerationRequest::character(&character_id),
    )
    .unwrap();
    let session = app
        .materialize_preview_session(PreviewSubject::Character {
            version_id: character_id,
        })
        .unwrap();
    let path = session.root.join("payload.glb");
    assert_eq!(
        sha256_file(&path).unwrap(),
        session.document.payload_sha256.as_deref().unwrap()
    );
}

fn persist_preview_producer(app: &mut Application) -> BackendExecutionContextId {
    let ctx = BackendExecutionContext::new(
        "preview",
        "test",
        "none",
        "workbench-test",
        "v1-6-preview-1",
    )
    .unwrap();
    let id = ctx.id();
    app.catalog_mut()
        .put_validated(&Validated::certify(ctx).unwrap())
        .unwrap();
    id
}

fn write_session_descriptor(payload: &Path, has_animation: bool, names: &[&str]) {
    let desc = serde_json::json!({
        "has_animation": has_animation,
        "animation_names": names,
        "default_animation": names.first().copied(),
        "generation_recipe": "v1-6-preview-1",
        "generator_id": "workbench-preview-test",
    });
    fs::write(
        payload.with_file_name("descriptor.json"),
        serde_json::to_vec_pretty(&desc).unwrap(),
    )
    .unwrap();
}

fn persist_located_preview(app: &mut Application, mut artifact: PreviewArtifact, path: &Path) {
    artifact
        .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
        .unwrap();
    app.catalog_mut()
        .persist_preview_artifact(&artifact, path)
        .unwrap();
}

fn persist_glb_preview(
    app: &mut Application,
    kind: ProductKind,
    version_id: &str,
    producer: BackendExecutionContextId,
    animated: bool,
    descriptor_claims_animation: bool,
) {
    let bytes = synthetic_preview_glb(animated);
    let path = temp_file("preview.glb", &bytes);
    let names: Vec<&str> = if descriptor_claims_animation {
        if animated {
            vec!["proxy_motion"]
        } else {
            vec!["claimed_absent"]
        }
    } else {
        Vec::new()
    };
    write_session_descriptor(&path, descriptor_claims_animation, &names);
    let digest = ContentDigest::parse(&sha256_bytes(&bytes)).unwrap();
    let size = bytes.len() as u64;
    let artifact = match kind {
        ProductKind::MotionAssetVersion => PreviewArtifact::for_motion(
            MotionAssetVersionId::parse(version_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            producer,
        )
        .unwrap(),
        ProductKind::DerivedVariantVersion => PreviewArtifact::for_derived_variant(
            DerivedVariantVersionId::parse(version_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            producer,
        )
        .unwrap(),
        ProductKind::CharacterAssetVersion => panic!("character helper unused"),
    };
    persist_located_preview(app, artifact, &path);
}

#[test]
fn motion_viewer_rejects_glb_without_actual_animation() {
    let (mut app, _, motion_id) = seeded_app();
    let producer = persist_preview_producer(&mut app);
    persist_glb_preview(
        &mut app,
        ProductKind::MotionAssetVersion,
        &motion_id,
        producer,
        false,
        true,
    );
    let session = app
        .materialize_preview_session(PreviewSubject::Motion {
            version_id: motion_id,
        })
        .unwrap();
    assert!(
        session.document.valid,
        "Application must not treat descriptor.has_animation as loaded-payload proof"
    );
    assert_eq!(session.document.has_animation, Some(true));
    assert!(session.root.join("payload.glb").is_file());
    let host = PreviewHost::serve(&session).unwrap();
    let measured = measure(&host.url, "fail");
    assert_eq!(measured["fail_closed"], true);
    assert_eq!(measured["valid"], false);
    let err = measured["error"].as_str().unwrap_or_default();
    assert!(
        err.to_lowercase().contains("animation"),
        "{measured}"
    );
}

#[test]
fn derived_viewer_rejects_glb_without_actual_animation() {
    let mut app = Application::open_in_memory().unwrap();
    let g = unpublished_graph();
    persist_unpublished_without_authority(app.catalog_mut(), &g).unwrap();
    let derived_id = g.derived_version.id().canonical();
    persist_glb_preview(
        &mut app,
        ProductKind::DerivedVariantVersion,
        &derived_id,
        g.backend.id(),
        false,
        true,
    );
    let session = app
        .materialize_preview_session(PreviewSubject::DerivedVariant {
            version_id: derived_id,
        })
        .unwrap();
    assert!(session.document.valid);
    assert_eq!(session.document.has_animation, Some(true));
    assert!(session.root.join("payload.glb").is_file());
    let host = PreviewHost::serve(&session).unwrap();
    let measured = measure(&host.url, "fail");
    assert_eq!(measured["fail_closed"], true);
    assert_eq!(measured["valid"], false);
    let err = measured["error"].as_str().unwrap_or_default();
    assert!(
        err.to_lowercase().contains("animation"),
        "{measured}"
    );
}

#[test]
fn derived_viewer_plays_valid_animated_glb() {
    let mut app = Application::open_in_memory().unwrap();
    let g = unpublished_graph();
    persist_unpublished_without_authority(app.catalog_mut(), &g).unwrap();
    let derived_id = g.derived_version.id().canonical();
    persist_glb_preview(
        &mut app,
        ProductKind::DerivedVariantVersion,
        &derived_id,
        g.backend.id(),
        true,
        true,
    );
    let session = app
        .materialize_preview_session(PreviewSubject::DerivedVariant {
            version_id: derived_id,
        })
        .unwrap();
    assert!(session.document.valid);
    let host = PreviewHost::serve(&session).unwrap();
    let measured = measure(&host.url, "derived");
    assert_eq!(measured["loaded"], true);
    assert_eq!(measured["transport"]["play_advanced"], true);
    assert_eq!(measured["transport"]["pause_held"], true);
    assert_eq!(measured["transport"]["seek_ok"], true);
    assert_eq!(measured["transport"]["restart_near_start"], true);
}
