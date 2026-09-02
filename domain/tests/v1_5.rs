mod common;

use common::*;
use rigforge_domain::*;

#[test]
fn derived_variant_draft_can_exist_before_qc() {
    let g = unpublished_graph();
    let draft = DerivedVariantVersion::draft(
        g.derived.id(),
        g.character_version.id(),
        g.motion_version.id(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        g.job.id(),
        g.backend.id(),
        g.worker.id(),
    )
    .unwrap();
    assert_eq!(draft.lifecycle(), Lifecycle::Draft);
    assert!(draft.qc_report_id().is_none());
    draft.validate().unwrap();
}

#[test]
fn published_requires_qc_report() {
    let mut g = unpublished_graph();
    let mut value = serde_json::to_value(&g.derived_version).unwrap();
    value["qc_report_id"] = serde_json::Value::Null;
    g.derived_version = from_json_validated(&value.to_string()).unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingProvenance);
}

#[test]
fn qc_binding_is_write_once() {
    let mut g = unpublished_graph();
    let err = g
        .derived_version
        .bind_qc_report(QcReportId::generate())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WriteOnceBinding);
    g.derived_version.bind_qc_report(g.qc.id()).unwrap();
}

#[test]
fn persistence_artifact_binding_is_write_once() {
    let mut g = unpublished_graph();
    let err = g
        .derived_version
        .bind_persistence_artifact(PersistenceArtifactId::generate())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WriteOnceBinding);
}

#[test]
fn persistence_verification_binding_is_write_once() {
    let mut g = unpublished_graph();
    g.derived_version
        .bind_persistence_verification(g.verification.id())
        .unwrap();
    let err = g
        .derived_version
        .bind_persistence_verification(PersistenceVerificationId::generate())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WriteOnceBinding);
}

#[test]
fn published_derived_variant_is_immutable() {
    let mut g = valid_graph();
    let err = g
        .derived_version
        .bind_qc_report(g.qc.id())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
}

#[test]
fn qc_verdict_is_derived_from_typed_checks() {
    let g = unpublished_graph();
    assert_eq!(g.qc.verdict(), QcVerdict::Pass);
    assert_eq!(g.qc.rule_set_id(), Some(V1_STRUCTURAL_QC_RULE_SET));
    assert_eq!(g.qc.checks().len(), 7);
    let mut checks = passing_structural_qc_checks();
    checks[3] = QcCheck::new(QcCheckName::TimeRangeDurationSane, QcCheckOutcome::Fail);
    let qc = QcReport::for_derived_variant(
        g.derived_version.id(),
        g.policy_version.id(),
        g.worker.id(),
        g.persistence.id(),
        g.persistence.instance_id(),
        g.persistence.digest().clone(),
        checks,
    )
    .unwrap();
    assert_eq!(qc.verdict(), QcVerdict::Fail);
}

#[test]
fn missing_required_qc_check_fails_closed() {
    let g = unpublished_graph();
    let err = derive_qc_verdict(&[]).unwrap_err();
    assert_eq!(err.code, ErrorCode::QcChecksIncomplete);
    let mut value = serde_json::to_value(&g.qc).unwrap();
    value["checks"] = serde_json::json!([]);
    value["verdict"] = serde_json::json!("pass");
    let err = from_json_validated::<QcReport>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::QcChecksIncomplete);
}

#[test]
fn untrusted_json_cannot_claim_pass_with_failing_check() {
    let g = unpublished_graph();
    let mut value = serde_json::to_value(&g.qc).unwrap();
    value["checks"][0]["outcome"] = serde_json::json!("fail");
    value["verdict"] = serde_json::json!("pass");
    let err = from_json_validated::<QcReport>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn qc_artifact_exact_binding_is_required() {
    let g = unpublished_graph();
    assert_eq!(
        g.qc.evaluated_persistence_artifact_id(),
        Some(g.persistence.id())
    );
    assert_eq!(
        g.qc.evaluated_persistence_artifact_instance_id(),
        Some(g.persistence.instance_id())
    );
    assert_eq!(
        g.qc.evaluated_payload_digest(),
        Some(g.persistence.digest())
    );
}

#[test]
fn publication_requires_compatibility_authorization() {
    let mut g = unpublished_graph();
    let mut value = serde_json::to_value(&g.job).unwrap();
    value.as_object_mut().unwrap().remove("compatibility_result_id");
    value["compatibility_warnings_acknowledged"] = serde_json::json!(false);
    g.job = from_json_validated(&value.to_string()).unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn ready_with_warnings_publication_requires_acknowledgement() {
    let mut g = unpublished_graph();
    g.compatibility = CompatibilityResult::from_preflight(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::PassWithWarnings,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        vec!["helpers unmapped".into()],
    )
    .unwrap();
    let mut value = serde_json::to_value(&g.job).unwrap();
    value["compatibility_result_id"] = serde_json::json!(g.compatibility.id().canonical());
    value["compatibility_warnings_acknowledged"] = serde_json::json!(false);
    g.job = from_json_validated(&value.to_string()).unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
    value["compatibility_warnings_acknowledged"] = serde_json::json!(true);
    g.job = from_json_validated(&value.to_string()).unwrap();
    g.try_publish().unwrap();
    assert_eq!(g.derived_version.lifecycle(), Lifecycle::Published);
}
