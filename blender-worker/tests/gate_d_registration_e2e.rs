mod common;

use std::path::{Path, PathBuf};

use common::{sha256_file, temp_dir};
use rigforge_app::{
    Application, PreviewGenerationRequest, PreviewSubject, TransferOutcomeKind,
};
use rigforge_blender_worker::{enforce_pin, BlenderPin, BlenderSkeletonInspector};
use rigforge_domain::*;
use rigforge_workbench::WorkbenchApp;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";
const KNIGHT_SHA: &str = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";

fn write_evidence(value: &serde_json::Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("review_evidence")
        .join("gate_d_registration_e2e.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serde_json::to_vec_pretty(value).unwrap());
}

#[test]
fn fresh_catalog_registration_to_preview_frozen_pair() {
    common::ensure_test_runtime();
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(KNIGHT)), KNIGHT_SHA);
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);

    let dir = temp_dir();
    let catalog_path = dir.join("catalog.sqlite");
    let mut app = Application::open(&catalog_path).unwrap();
    assert!(app.list_characters().unwrap().is_empty());
    assert!(app.list_motions().unwrap().is_empty());

    let inspector = BlenderSkeletonInspector::production().unwrap();
    let character = app
        .register_local_character("Knight_Male", KNIGHT, &inspector)
        .unwrap();
    let motion = app
        .register_local_motion(
            "UAL2 Walk_Carry_Loop",
            UAL2,
            "UAL2_Standard",
            CLIP,
            1,
            61,
            30,
            1,
            &inspector,
        )
        .unwrap();

    let characters = app.list_characters().unwrap();
    let motions = app.list_motions().unwrap();
    assert_eq!(characters.len(), 1);
    assert_eq!(motions.len(), 1);
    assert_eq!(
        characters[0].published_version_id.as_deref(),
        Some(character.version_id.as_str())
    );
    assert_eq!(
        motions[0].published_version_id.as_deref(),
        Some(motion.version_id.as_str())
    );
    let character_version = app
        .resolve_exact_character_version(&character.version_id)
        .unwrap();
    assert_eq!(character_version.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(
        character_version.as_record().source().digest().sha256(),
        KNIGHT_SHA
    );
    assert_ne!(character.asset_id, KNIGHT);
    assert_ne!(character.version_id, KNIGHT_SHA);
    let motion_version = app
        .resolve_exact_motion_version(&motion.version_id)
        .unwrap();
    assert_eq!(motion_version.as_record().lifecycle(), Lifecycle::Published);
    assert_eq!(motion_version.as_record().source().digest().sha256(), UAL2_SHA);
    assert_ne!(motion.asset_id, UAL2_SHA);

    let mut shell = WorkbenchApp::empty();
    shell.reload_from_application(&app).unwrap();
    shell.select_character_version(&character.version_id);
    shell.select_motion_version(&motion.version_id);
    shell
        .propose_mapping_with(&mut app, &inspector)
        .unwrap();
    shell.on_accept_mapping_clicked(&mut app).unwrap();
    assert!(shell.mapping_accepted());
    shell
        .on_evaluate_compatibility_clicked(&mut app)
        .unwrap();
    if shell.transfer_requires_acknowledgement() {
        shell
            .on_warnings_checkbox_changed(&mut app, true)
            .unwrap();
    }
    assert!(shell.transfer_available());
    shell.on_transfer_action(&mut app).unwrap();
    assert!(
        shell.mutation_locked(),
        "Transfer start must return before Blender-length waits"
    );
    assert_ne!(shell.publication_state(), Some("Published"));
    shell.drive_transfer_to_terminal(&mut app).unwrap();
    assert_eq!(shell.publication_state(), Some("Published"));
    let derived_version_id = shell
        .selected_derived_variant_version()
        .expect("Published Derived must become the current Derived selection")
        .to_string();
    assert_eq!(
        shell.derived_variant_version_id(),
        Some(derived_version_id.as_str())
    );

    let preview = app
        .generate_preview(PreviewGenerationRequest::derived_variant(
            &derived_version_id,
        ))
        .unwrap();
    let session = shell
        .request_preview(
            &mut app,
            PreviewSubject::DerivedVariant {
                version_id: derived_version_id.clone(),
            },
            true,
            false,
        )
        .unwrap();
    assert!(session.document.valid);
    assert_eq!(
        session.document.selected_product_version_id,
        derived_version_id
    );
    assert_eq!(
        preview.as_record().bound_product_version_id().as_deref(),
        Some(derived_version_id.as_str())
    );

    let mapping_version_id = shell.accepted_mapping_version_id().unwrap().to_string();
    let compatibility_id = shell.compatibility_id().unwrap().to_string();
    let derived_variant_id = shell.derived_variant_id().unwrap().to_string();
    let outcome_qc = shell.qc_verdict().unwrap().to_string();
    let persistence_verification_id = shell
        .persistence_verification_id()
        .expect("PersistenceVerification")
        .to_string();

    let jobs = app.catalog().list_job_runs().unwrap();
    assert!(!jobs.is_empty());
    let run = jobs.last().expect("Transfer JobRun");
    let spec = app.load_job_spec(&run.job_spec_id).unwrap();
    let derived = app
        .catalog()
        .load_derived_variant_version(&derived_version_id)
        .unwrap();
    let qc_id = derived
        .as_record()
        .qc_report_id()
        .expect("Published Derived requires QC")
        .canonical();
    let artifact_id = derived
        .as_record()
        .persistence_artifact_id()
        .expect("Published Derived requires PersistenceArtifact")
        .canonical();

    drop(app);
    let reopened = Application::open(&catalog_path).unwrap();
    let verification = reopened
        .catalog()
        .load_persistence_verification(&persistence_verification_id)
        .unwrap();
    let reopened_derived = reopened
        .catalog()
        .load_derived_variant_version(&derived_version_id)
        .unwrap();
    assert_eq!(
        reopened_derived.as_record().lifecycle(),
        Lifecycle::Published
    );
    assert_eq!(
        reopened.list_characters().unwrap()[0]
            .published_version_id
            .as_deref(),
        Some(character.version_id.as_str())
    );

    let evidence = serde_json::json!({
        "character_asset_id": character.asset_id,
        "character_asset_version_id": character.version_id,
        "motion_asset_id": motion.asset_id,
        "motion_asset_version_id": motion.version_id,
        "source_skeleton_reference_id": motion.source_skeleton_id,
        "mapping_version_id": mapping_version_id,
        "compatibility_result_id": compatibility_id,
        "job_spec_id": spec.as_record().id().canonical(),
        "job_run_id": run.run_id,
        "attempt_id": run.attempt_id,
        "worker_execution_ref": run.worker_execution_ref,
        "qc_report_id": qc_id,
        "qc_verdict": outcome_qc,
        "persistence_artifact_id": artifact_id,
        "persistence_verification_id": persistence_verification_id,
        "verification_fresh_reopen": format!("{:?}", verification.as_record().fresh_reopen()),
        "derived_variant_id": derived_variant_id,
        "derived_variant_version_id": derived_version_id,
        "selected_derived_preview_version_id": session.document.selected_product_version_id,
        "preview_artifact_id": preview.as_record().id().canonical(),
        "publication_state": "Published",
        "result": "PASS",
        "seeded_character_or_motion": false,
    });
    write_evidence(&evidence);
    assert_eq!(shell.publication_state(), Some("Published"));
    let _ = TransferOutcomeKind::Published;
}
