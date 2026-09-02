mod common;

use std::path::Path;

use common::sha256_file;
use rigforge_app::{Application, MappingAssistProfile, WorkerCapabilityProfile};
use rigforge_blender_worker::{enforce_pin, BlenderPin, BlenderSkeletonInspector};
use rigforge_domain::*;
use rigforge_workbench::WorkbenchApp;

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const GOBLIN: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Goblin_Male.fbx";
const MANNEQUIN: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Female Mannequin\Unity\Mannequin_F.fbx";
const HORSE: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qanimal_extract\Ultimate Animated Animals - July 2021\FBX\Horse.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";

const KNIGHT_SHA: &str = "fd323fbc4962a9b94ab58d303bbc92ead7228393b735a95d98a8509e33747e3f";
const GOBLIN_SHA: &str = "ce92e034493a2866c2427419cf627b0ac817236077f3c53e671d54162dfafeae";
const MANNEQUIN_SHA: &str = "1a7fb4bb8850860eff36aabd60d65d19273a4a072b3c638001fbb38e42add4de";
const HORSE_SHA: &str = "babb40391b8c3e217be2bb9c1f7c4d21910e943c28840363509dde7e88b54cba";
const UAL2_SHA: &str = "d26d0e9f4a202d473194c056045143095a605a53ba1d823ef24055be4b86851d";
const CLIP: &str = "Armature|Armature|Walk_Carry_Loop";

struct Pair {
    id: &'static str,
    character_name: &'static str,
    character_path: &'static str,
    character_sha: &'static str,
    provenance: &'static str,
    non_humanoid: bool,
}

fn evidence(path: &str, sha: &str, note: &str) -> SourceArtifactEvidence {
    let bytes = std::fs::metadata(path).unwrap().len();
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(sha).unwrap(),
        bytes,
        "application/octet-stream",
        Some(note.into()),
        None,
    )
    .unwrap()
}

fn qualify_pair(pair: &Pair, via_workbench: bool) -> serde_json::Value {
    common::ensure_test_runtime();
    let mut row = serde_json::json!({
        "pair_id": pair.id,
        "character_name": pair.character_name,
        "character_path": pair.character_path,
        "character_sha256": pair.character_sha,
        "motion_path": UAL2,
        "motion_sha256": UAL2_SHA,
        "clip": CLIP,
        "provenance": pair.provenance,
        "non_humanoid": pair.non_humanoid,
        "via_workbench_handlers": via_workbench,
        "transfer_attempted": "NO",
        "classification": "observed",
    });
    if sha256_file(Path::new(pair.character_path)) != pair.character_sha {
        row["outcome"] = serde_json::json!("ASSET_DIGEST_MISMATCH");
        row["transfer_attempted"] = serde_json::json!("NO");
        return row;
    }
    let mut app = Application::open_in_memory().unwrap();
    let mut character = CharacterAsset::new(pair.character_name).unwrap();
    let mut character_version = CharacterAssetVersion::draft(
        character.id(),
        format!("{} V1-8", pair.character_name),
        evidence(pair.character_path, pair.character_sha, pair.provenance),
    )
    .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());
    let source_skeleton = SourceSkeletonReference::new("UAL2_Standard").unwrap();
    let mut motion = MotionAsset::new("UAL2_Standard").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "UAL2 Walk_Carry_Loop",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            CLIP,
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "unmapped target joints remain at target rest",
        )
        .unwrap(),
        evidence(UAL2, UAL2_SHA, "Quaternius Universal Animation Library 2 Standard; local research extract"),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());
    let mut policy = RetargetPolicy::new("rest-relative-proven").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(character).unwrap(),
            &Validated::certify(character_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated(&Validated::certify(source_skeleton.clone()).unwrap())
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(motion).unwrap(),
            &Validated::certify(motion_version.clone()).unwrap(),
        )
        .unwrap();
    app.catalog_mut()
        .put_validated_pair(
            &Validated::certify(policy).unwrap(),
            &Validated::certify(policy_version.clone()).unwrap(),
        )
        .unwrap();

    row["character_version_id"] = serde_json::json!(character_version.id().canonical());
    row["motion_version_id"] = serde_json::json!(motion_version.id().canonical());

    let inspector = match BlenderSkeletonInspector::production() {
        Ok(inspector) => inspector,
        Err(err) => {
            row["outcome"] = serde_json::json!("INSPECTOR_UNAVAILABLE");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };

    if via_workbench {
        let mut shell = WorkbenchApp::empty();
        shell.select_character_version(character_version.id().canonical());
        shell.select_motion_version(motion_version.id().canonical());
        if let Err(err) = shell.propose_mapping_with(&mut app, &inspector) {
            row["outcome"] = serde_json::json!("MAPPING_PROPOSE_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
        if let Err(err) = shell.on_accept_mapping_clicked(&mut app) {
            row["outcome"] = serde_json::json!("MAPPING_ACCEPT_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            row["manual_intervention"] = serde_json::json!("accept failed");
            return row;
        }
        if let Err(err) = shell.on_evaluate_compatibility_clicked(&mut app) {
            row["outcome"] = serde_json::json!("COMPATIBILITY_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
        if shell.transfer_requires_acknowledgement() {
            let _ = shell.on_warnings_checkbox_changed(&mut app, true);
            row["warnings_acknowledged"] = serde_json::json!(true);
        }
        row["mapping_accepted"] = serde_json::json!(shell.mapping_accepted());
        row["accepted_mapping_version_id"] =
            serde_json::json!(shell.accepted_mapping_version_id());
        row["compatibility_summary"] = serde_json::json!(shell.compatibility_summary());
        row["compatibility_id"] = serde_json::json!(shell.compatibility_id());
        row["transfer_available"] = serde_json::json!(shell.transfer_available());
        row["outcome"] = serde_json::json!("HANDLERS_COMPLETED");
        return row;
    }

    let target_summary = match app
        .inspect_and_store_character_summary(&character_version.id().canonical(), &inspector)
    {
        Ok(summary) => summary,
        Err(err) => {
            row["outcome"] = serde_json::json!("CHARACTER_INSPECT_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };
    let source_summary = match app.inspect_and_store_source_summary(
        &source_skeleton.id().canonical(),
        &inspector,
        &motion_version.id().canonical(),
    ) {
        Ok(summary) => summary,
        Err(err) => {
            row["outcome"] = serde_json::json!("SOURCE_INSPECT_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };
    row["source_joint_count"] = serde_json::json!(source_summary.as_record().joints().len());
    row["target_joint_count"] = serde_json::json!(target_summary.as_record().joints().len());
    let proposal = match app.propose_mapping(
        &source_summary.as_record().id().canonical(),
        &target_summary.as_record().id().canonical(),
        MappingAssistProfile::OptionalHumanoid,
    ) {
        Ok(proposal) => proposal,
        Err(err) => {
            row["outcome"] = serde_json::json!("MAPPING_PROPOSE_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };
    row["mapped_entries"] = serde_json::json!(proposal.entries.len());
    row["unmapped_source_count"] = serde_json::json!(proposal.unmapped_source.len());
    row["unmapped_target_count"] = serde_json::json!(proposal.unmapped_target.len());
    row["proposal_confirmation_required"] = serde_json::json!(proposal.confirmation_required);
    let (logical, draft) = match app.store_mapping_draft(
        pair.id,
        &character_version.id().canonical(),
        &source_skeleton.id().canonical(),
        &proposal,
        &source_summary.as_record().id().canonical(),
        &target_summary.as_record().id().canonical(),
    ) {
        Ok(value) => value,
        Err(err) => {
            row["outcome"] = serde_json::json!("MAPPING_STORE_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };
    let published = match app.accept_mapping_version(
        &logical.as_record().id().canonical(),
        &draft.as_record().id().canonical(),
    ) {
        Ok(published) => published,
        Err(err) => {
            row["outcome"] = serde_json::json!("MAPPING_ACCEPT_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            row["manual_intervention"] = serde_json::json!(true);
            return row;
        }
    };
    let result = match app.run_compatibility_preflight(
        &character_version.id().canonical(),
        &motion_version.id().canonical(),
        &published.as_record().id().canonical(),
        &policy_version.id().canonical(),
        &WorkerCapabilityProfile::v1_3_isolated_worker(),
    ) {
        Ok(result) => result,
        Err(err) => {
            row["outcome"] = serde_json::json!("COMPATIBILITY_FAILED");
            row["failure_reason"] = serde_json::json!(err.to_string());
            return row;
        }
    };
    row["mapping_version_id"] = serde_json::json!(published.as_record().id().canonical());
    row["compatibility_summary"] = serde_json::json!(format!("{:?}", result.as_record().summary()));
    row["compatibility_notes"] = serde_json::json!(result.as_record().notes());
    row["manual_intervention"] = serde_json::json!(false);
    row["outcome"] = serde_json::json!("MAPPING_COMPAT_COMPLETED");
    row
}

#[test]
fn v1_8_real_asset_campaign_and_workbench_handlers() {
    common::ensure_test_runtime();
    let pin = BlenderPin::accepted();
    enforce_pin(&pin.executable, &pin).unwrap();
    assert_eq!(sha256_file(Path::new(UAL2)), UAL2_SHA);
    assert_eq!(sha256_file(Path::new(KNIGHT)), KNIGHT_SHA);

    let pairs = [
        Pair {
            id: "knight-ual2-frozen",
            character_name: "Knight_Male",
            character_path: KNIGHT,
            character_sha: KNIGHT_SHA,
            provenance: "Quaternius Ultimate Animated Character Pack Nov 2019; local POC-FBX-01 extract; CC0 pack page SOURCE_CONFIRMED",
            non_humanoid: false,
        },
        Pair {
            id: "goblin-ual2",
            character_name: "Goblin_Male",
            character_path: GOBLIN,
            character_sha: GOBLIN_SHA,
            provenance: "Quaternius Ultimate Animated Character Pack Nov 2019; local extract; CC0 pack page SOURCE_CONFIRMED",
            non_humanoid: false,
        },
        Pair {
            id: "mannequin_f-ual2",
            character_name: "Mannequin_F",
            character_path: MANNEQUIN,
            character_sha: MANNEQUIN_SHA,
            provenance: "Quaternius Universal Animation Library 2 Standard Female Mannequin; local POC-BLENDER-E2E-01 extract; CC0 pack page SOURCE_CONFIRMED",
            non_humanoid: false,
        },
        Pair {
            id: "horse-ual2-nonhumanoid",
            character_name: "Horse",
            character_path: HORSE,
            character_sha: HORSE_SHA,
            provenance: "Quaternius Ultimate Animated Animals July 2021; local POC-FBX-01 extract; CC0 pack page SOURCE_CONFIRMED",
            non_humanoid: true,
        },
    ];

    let mut rows = Vec::new();
    rows.push(qualify_pair(&pairs[0], true));
    for pair in &pairs[1..] {
        rows.push(qualify_pair(pair, false));
    }

    let payload = serde_json::json!({
        "campaign": "V1-8 representative real-asset matrix",
        "motion": {
            "path": UAL2,
            "sha256": UAL2_SHA,
            "clip": CLIP
        },
        "transfer_note": "Transfer/QC/publication for the frozen Knight/UAL2 pair remains the V1-5 production checkpoint; this campaign qualifies Mapping/Compatibility diversity and native Workbench handlers.",
        "rows": rows,
    });
    let tmp = std::env::temp_dir().join("rigforge_v1_8_asset_matrix.json");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
    let evidence = Path::new(r"F:\NewResearch\rigforge\review_evidence");
    if evidence.is_dir() {
        std::fs::write(
            evidence.join("v1_8_asset_matrix.json"),
            serde_json::to_vec_pretty(&payload).unwrap(),
        )
        .unwrap();
    }

    let knight = &rows[0];
    assert_eq!(knight["outcome"], "HANDLERS_COMPLETED");
    assert_eq!(knight["mapping_accepted"], true);
    let summary = knight["compatibility_summary"].as_str().unwrap_or("");
    assert!(
        summary.contains("Ready")
            || summary.contains("ReadyWithWarnings")
            || summary.contains("MappingConfirmationRequired"),
        "frozen pair compatibility was {summary}"
    );

    let horse = rows.iter().find(|r| r["pair_id"] == "horse-ual2-nonhumanoid").unwrap();
    assert!(horse["outcome"].is_string());
    assert_ne!(horse["outcome"], "ASSET_DIGEST_MISMATCH");
}
