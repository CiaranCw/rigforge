mod common;

use common::{certify, mutate_json_field, source, unpublished_graph, valid_graph};
use rigforge_app::{
    AppError, CatalogLocationEvidence, SqliteCatalog, DB_SCHEMA_VERSION,
};
use rigforge_domain::{
    to_json, CharacterAsset, CharacterAssetVersion, ErrorCode, Lifecycle, ProductVersionStore,
    RecordType, Validated,
};

fn seed_lineage(catalog: &mut SqliteCatalog) -> common::Graph {
    let g = valid_graph();
    catalog
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated_pair(&certify(g.mapping.clone()), &certify(g.mapping_version.clone()))
        .unwrap();
    catalog
        .put_validated_pair(&certify(g.policy.clone()), &certify(g.policy_version.clone()))
        .unwrap();
    catalog.put_validated(&certify(g.job.clone())).unwrap();
    catalog.put_validated(&certify(g.worker.clone())).unwrap();
    catalog.put_validated(&certify(g.qc.clone())).unwrap();
    catalog
        .put_validated_pair(&certify(g.derived.clone()), &certify(g.derived_version.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.persistence.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.verification.clone()))
        .unwrap();
    catalog.put_validated(&certify(g.preview.clone())).unwrap();
    g
}

#[test]
fn empty_db_migrates_to_current_schema() {
    let catalog = SqliteCatalog::open_in_memory().unwrap();
    assert_eq!(catalog.db_schema_version().unwrap(), DB_SCHEMA_VERSION);
    assert_eq!(catalog.schema_name().unwrap(), "v1_2_catalog");
}

#[test]
fn store_and_load_validated_character_and_motion_versions() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    catalog
        .put_validated(&certify(g.source_skeleton.clone()))
        .unwrap();
    catalog
        .put_validated_pair(
            &certify(g.motion.clone()),
            &certify(g.motion_version.clone()),
        )
        .unwrap();

    let loaded_c = catalog
        .load_character_version(&g.character_version.id().canonical())
        .unwrap();
    let loaded_m = catalog
        .load_motion_version(&g.motion_version.id().canonical())
        .unwrap();
    assert_eq!(loaded_c.as_record(), &g.character_version);
    assert_eq!(loaded_m.as_record(), &g.motion_version);
    assert_eq!(loaded_c.as_record().lifecycle(), Lifecycle::Published);
}

#[test]
fn product_version_store_trait_round_trip() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let version = certify(g.character_version.clone());
    ProductVersionStore::store_immutable_version(&mut catalog, version.clone()).unwrap();
    let loaded: Validated<CharacterAssetVersion> =
        ProductVersionStore::load_exact_version(&catalog, &g.character_version.id().canonical())
            .unwrap();
    assert_eq!(loaded, version);
}

#[test]
fn logical_object_lists_all_versions() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated_pair(
            &certify(g.character.clone()),
            &certify(g.character_version.clone()),
        )
        .unwrap();
    let mut v2 = CharacterAssetVersion::draft(
        g.character.id(),
        "Knight v2",
        source("knight-v2", 5),
    )
    .unwrap();
    v2.publish().unwrap();
    catalog.put_validated(&certify(v2.clone())).unwrap();

    let ids = catalog
        .list_version_ids(&g.character.id().canonical())
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&g.character_version.id().canonical()));
    assert!(ids.contains(&v2.id().canonical()));
}

#[test]
fn published_version_cannot_be_overwritten() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated(&certify(g.character_version.clone()))
        .unwrap();
    let mutated = mutate_json_field(
        &g.character_version,
        "display_name",
        serde_json::json!("mutated published name"),
    );
    let err = catalog.put_validated(&mutated).unwrap_err();
    match err {
        AppError::ImmutablePublished { product_id } => {
            assert_eq!(product_id, g.character_version.id().canonical());
        }
        other => panic!("expected ImmutablePublished, got {other}"),
    }
    let loaded = catalog
        .load_character_version(&g.character_version.id().canonical())
        .unwrap();
    assert_eq!(loaded.as_record().display_name(), "Knight v1");
}

#[test]
fn published_identical_payload_is_idempotent() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let version = certify(g.character_version.clone());
    catalog.put_validated(&version).unwrap();
    catalog.put_validated(&version).unwrap();
    let loaded = catalog
        .load_character_version(&g.character_version.id().canonical())
        .unwrap();
    assert_eq!(loaded, version);
}

#[test]
fn product_ids_are_not_sqlite_rowids() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated(&certify(g.character.clone()))
        .unwrap();
    let product_id = g.character.id().canonical();
    let rowid = catalog.internal_rowid_for_tests(&product_id).unwrap();
    assert_ne!(product_id, rowid.to_string());
    assert!(product_id.contains('-'), "UUIDv7 is hyphenated");
    assert!(!product_id.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn path_overlay_does_not_change_product_identity() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated(&certify(g.character_version.clone()))
        .unwrap();
    let product_id = g.character_version.id().canonical();
    catalog
        .record_payload_location(CatalogLocationEvidence {
            product_id: product_id.clone(),
            instance_id: String::new(),
            location_kind: "filesystem_path_evidence".into(),
            location_value: r"D:\moved\Knight_Male.fbx".into(),
            observed_at: 1,
            note: Some("payload moved; Product identity unchanged".into()),
        })
        .unwrap();
    let loaded = catalog.load_character_version(&product_id).unwrap();
    assert_eq!(loaded.as_record().id(), g.character_version.id());
    let locations = catalog.list_payload_locations(&product_id).unwrap();
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].location_value, r"D:\moved\Knight_Male.fbx");
    assert_ne!(product_id, locations[0].location_value);
}

#[test]
fn artifact_and_verification_metadata_round_trip() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = valid_graph();
    catalog
        .put_validated(&certify(g.persistence.clone()))
        .unwrap();
    catalog
        .put_validated(&certify(g.verification.clone()))
        .unwrap();
    let artifact = catalog
        .load_artifact_metadata(&g.persistence.id().canonical())
        .unwrap();
    let verification = catalog
        .load_persistence_verification(&g.verification.id().canonical())
        .unwrap();
    assert_eq!(artifact.as_record(), &g.persistence);
    assert_eq!(verification.as_record(), &g.verification);
    assert_ne!(
        artifact.as_record().id().canonical(),
        artifact.as_record().digest().sha256()
    );
}

#[test]
fn regenerated_artifact_appends_instance_without_clobber() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = valid_graph();
    catalog
        .put_validated(&certify(g.persistence.clone()))
        .unwrap();
    let regenerated = g
        .persistence
        .regenerate(common::digest(9), 8192)
        .unwrap();
    catalog
        .put_validated(&certify(regenerated.clone()))
        .unwrap();
    let instances = catalog
        .list_artifact_instances(&g.persistence.id().canonical())
        .unwrap();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].as_record().instance_id(), g.persistence.instance_id());
    assert_eq!(instances[1].as_record().instance_id(), regenerated.instance_id());
    assert_ne!(
        instances[0].as_record().instance_id(),
        instances[1].as_record().instance_id()
    );
}

#[test]
fn unsupported_domain_schema_fails_closed_on_load() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    let mut payload: serde_json::Value =
        serde_json::from_str(&to_json(&g.character_version).unwrap()).unwrap();
    payload["schema_version"] = serde_json::json!(99);
    catalog
        .insert_unvalidated_payload_for_tests(
            &g.character_version.id().canonical(),
            "character_asset_version",
            &payload.to_string(),
            99,
        )
        .unwrap();
    let err = catalog
        .load_character_version(&g.character_version.id().canonical())
        .unwrap_err();
    match err {
        AppError::Domain(domain) => {
            assert_eq!(domain.code, ErrorCode::UnsupportedSchemaVersion);
        }
        other => panic!("expected Domain UnsupportedSchemaVersion, got {other}"),
    }
}

#[test]
fn transaction_rolls_back_on_invalid_graph_write() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = unpublished_graph();
    catalog
        .put_validated(&certify(g.character_version.clone()))
        .unwrap();
    let extra = CharacterAsset::new("ShouldRollBack").unwrap();
    let mutated = mutate_json_field(
        &g.character_version,
        "display_name",
        serde_json::json!("illegal in-place published edit"),
    );
    let err = catalog
        .put_validated_pair(&certify(extra.clone()), &mutated)
        .unwrap_err();
    match err {
        AppError::ImmutablePublished { .. } => {}
        other => panic!("expected ImmutablePublished, got {other}"),
    }
    let missing = catalog.load_character_asset(&extra.id().canonical());
    match missing {
        Err(AppError::NotFound { .. }) => {}
        other => panic!("rolled-back logical asset must be absent, got {other:?}"),
    }
    let original = catalog
        .load_character_version(&g.character_version.id().canonical())
        .unwrap();
    assert_eq!(original.as_record().display_name(), "Knight v1");
}

#[test]
fn catalog_lookups_cover_required_product_records() {
    let mut catalog = SqliteCatalog::open_in_memory().unwrap();
    let g = seed_lineage(&mut catalog);
    assert_eq!(
        catalog
            .load_source_skeleton(&g.source_skeleton.id().canonical())
            .unwrap()
            .as_record(),
        &g.source_skeleton
    );
    assert_eq!(
        catalog
            .load_mapping_version(&g.mapping_version.id().canonical())
            .unwrap()
            .as_record(),
        &g.mapping_version
    );
    assert_eq!(
        catalog
            .load_policy_version(&g.policy_version.id().canonical())
            .unwrap()
            .as_record(),
        &g.policy_version
    );
    assert_eq!(
        catalog
            .load_job_spec(&g.job.id().canonical())
            .unwrap()
            .as_record(),
        &g.job
    );
    let characters = catalog.list_assets(RecordType::CharacterAsset).unwrap();
    let motions = catalog.list_assets(RecordType::MotionAsset).unwrap();
    let derived = catalog.list_assets(RecordType::DerivedVariant).unwrap();
    assert_eq!(characters.len(), 1);
    assert_eq!(motions.len(), 1);
    assert_eq!(derived.len(), 1);
    assert_ne!(characters[0].logical_id, motions[0].logical_id);
    assert_ne!(characters[0].logical_id, derived[0].logical_id);
}

#[test]
fn raw_json_is_not_a_store_api() {
    // Compile-time / API check: persistence takes Validated<T>, not (String, String).
    fn _accepts_validated(_: Validated<CharacterAssetVersion>) {}
    let g = unpublished_graph();
    _accepts_validated(certify(g.character_version));
}
