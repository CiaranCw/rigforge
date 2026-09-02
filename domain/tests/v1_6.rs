mod common;

use common::*;
use rigforge_domain::*;

#[test]
fn character_preview_exact_union_valid() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    preview.validate().unwrap();
    preview
        .assert_bound_character(g.character_version.id())
        .unwrap();
    assert_eq!(preview.bound_product_kind(), ProductKind::CharacterAssetVersion);
    assert!(preview.bound_motion_version_id().is_none());
    assert!(preview.bound_derived_variant_version_id().is_none());
    assert!(preview.derived() && preview.rebuildable() && !preview.authoritative());
}

#[test]
fn motion_preview_exact_union_valid() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_motion(
        g.motion_version.id(),
        digest(6),
        2048,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    preview
        .assert_bound_motion(g.motion_version.id())
        .unwrap();
    assert!(preview.bound_character_version_id().is_none());
    assert!(preview.bound_derived_variant_version_id().is_none());
}

#[test]
fn derived_preview_exact_union_valid() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_derived_variant(
        g.derived_version.id(),
        digest(7),
        4096,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    preview
        .assert_bound_derived_variant(g.derived_version.id())
        .unwrap();
}

#[test]
fn multi_product_binding_rejected() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["bound_motion_version_id"] = serde_json::json!(g.motion_version.id().canonical());
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn no_product_binding_rejected() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["bound_character_version_id"] = serde_json::Value::Null;
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn authoritative_true_rejected() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_motion(
        g.motion_version.id(),
        digest(6),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["authoritative"] = serde_json::json!(true);
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn derived_false_rejected() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_derived_variant(
        g.derived_version.id(),
        digest(7),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["derived"] = serde_json::json!(false);
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn rebuildable_false_rejected() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["rebuildable"] = serde_json::json!(false);
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn preview_id_is_not_payload_digest() {
    let g = unpublished_graph();
    let digest = digest(8);
    let preview = PreviewArtifact::for_motion(
        g.motion_version.id(),
        digest.clone(),
        512,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    assert_ne!(preview.id().canonical(), digest.sha256());
    let mut value = serde_json::to_value(&preview).unwrap();
    value["id"] = serde_json::json!(digest.sha256());
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert!(
        err.code == ErrorCode::DigestUsedAsId
            || err.code == ErrorCode::ArtifactProductConfusion
            || err.code == ErrorCode::UuidVersion
            || err.code == ErrorCode::InvalidId
    );
}

#[test]
fn untrusted_json_cannot_create_authoritative_preview() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let mut value = serde_json::to_value(&preview).unwrap();
    value["authoritative"] = serde_json::json!(true);
    value["derived"] = serde_json::json!(false);
    let err = from_json_validated::<PreviewArtifact>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}

#[test]
fn character_preview_wrong_version_fails_assert() {
    let g = unpublished_graph();
    let preview = PreviewArtifact::for_character(
        g.character_version.id(),
        digest(5),
        1024,
        "model/gltf-binary",
        g.backend.id(),
    )
    .unwrap();
    let err = preview
        .assert_bound_character(CharacterAssetVersionId::generate())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::PreviewBinding);
}
