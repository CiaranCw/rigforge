mod common;

use std::path::Path;

use common::certify;
use rigforge_app::SqliteCatalog;
use rigforge_domain::{
    CharacterAsset, CharacterAssetVersion, ContentDigest, LocationEvidence, MotionAsset,
    MotionAssetVersion, SourceArtifactEvidence, SourceSkeletonReference, TimeDomainProvenance,
    TimePoint, SamplingInterpretation,
};
use sha2::{Digest, Sha256};

const KNIGHT: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx";
const UAL2: &str = r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract\Universal Animation Library 2[Standard]\Unity\UAL2_Standard.fbx";

fn sha256_file(path: &Path) -> (String, u64) {
    let bytes = std::fs::read(path).unwrap();
    let size = bytes.len() as u64;
    let digest = Sha256::digest(&bytes);
    let hex = digest.iter().map(|b| format!("{b:02x}")).collect();
    (hex, size)
}

fn evidence(path: &str) -> SourceArtifactEvidence {
    let (hex, size) = sha256_file(Path::new(path));
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        ContentDigest::parse(&hex).unwrap(),
        size,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".into()),
        Some("V1-2 catalog metadata only; FBX bytes were not parsed".into()),
    )
    .unwrap()
}

/// Catalog handles realistic provenance for the accepted Knight/UAL2 lineage.
/// This does **not** validate animation semantics and does not parse FBX.
#[test]
fn catalog_stores_real_lineage_path_size_hash_metadata() {
    if !Path::new(KNIGHT).is_file() || !Path::new(UAL2).is_file() {
        eprintln!(
            "SKIP real lineage: frozen Knight_Male / UAL2_Standard files are not on this machine"
        );
        return;
    }

    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let mut character = CharacterAsset::new("Knight_Male").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight_Male frozen FBX", evidence(KNIGHT))
            .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2_Standard").unwrap();
    let mut motion = MotionAsset::new("UAL2_Standard").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "UAL2_Standard frozen FBX",
        source_skeleton.id(),
        TimeDomainProvenance::new(
            "clip:ual2-standard-walk-carry-loop",
            TimePoint::frames(1, 30, 1).unwrap(),
            TimePoint::frames(61, 30, 1).unwrap(),
            SamplingInterpretation::BakedEverySourceFrame,
            "time provenance is catalog metadata; clip semantics were not re-validated",
        )
        .unwrap(),
        evidence(UAL2),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());

    catalog
        .put_validated_pair(&certify(character.clone()), &certify(character_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(&certify(motion.clone()), &certify(motion_version.clone()))
        .unwrap();

    let loaded = catalog
        .load_character_version(&character_version.id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().id(), character_version.id());
    assert_ne!(loaded.as_record().id().canonical(), KNIGHT);
    assert!(!loaded.as_record().id().canonical().contains('\\'));
    let loaded_motion = catalog
        .load_motion_version(&motion_version.id().canonical())
        .unwrap();
    assert_ne!(loaded_motion.as_record().id().canonical(), UAL2);
    assert_eq!(
        catalog
            .load_source_skeleton(&source_skeleton.id().canonical())
            .unwrap()
            .as_record()
            .display_name(),
        "UAL2_Standard"
    );
}
