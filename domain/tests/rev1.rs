mod common;

use common::*;
use rigforge_domain::*;

fn mismatched_job(g: &Graph, character: CharacterAssetVersionId) -> JobSpec {
    JobSpec::new(
        character,
        g.motion_version.id(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap()
}

#[test]
fn published_version_has_no_public_field_mutation() {
    let mut g = valid_graph();
    assert_eq!(g.character_version.display_name(), "Knight v1");
    let err = g
        .character_version
        .try_set_display_name("mutated")
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
    assert_eq!(g.character_version.display_name(), "Knight v1");
    let err = g.mapping_version.bind_source_motion_version(g.motion_version.id());
    assert_eq!(err.unwrap_err().code, ErrorCode::ImmutableVersion);
    let err = g
        .derived_version
        .bind_persistence_artifact(g.persistence.id())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
    let err = g
        .derived_version
        .bind_persistence_verification(g.verification.id())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ImmutableVersion);
}

#[test]
fn published_lifecycle_cannot_transition_to_draft_or_ready() {
    assert_eq!(
        assert_lifecycle_transition(Lifecycle::Published, Lifecycle::Draft)
            .unwrap_err()
            .code,
        ErrorCode::InvalidLifecycle
    );
    assert_eq!(
        assert_lifecycle_transition(Lifecycle::Published, Lifecycle::Ready)
            .unwrap_err()
            .code,
        ErrorCode::InvalidLifecycle
    );
}

#[test]
fn invalidated_version_cannot_be_job_input() {
    let mut g = unpublished_graph();
    g.character_version.invalidate().unwrap();
    let err = validate_job_inputs(
        &g.character_version,
        &g.motion_version,
        &g.source_skeleton,
        &g.mapping_version,
        &g.policy_version,
        &g.job,
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidatedInput);
}

#[test]
fn source_skeleton_historical_reference_is_immutable() {
    let g = valid_graph();
    assert_eq!(g.motion_version.source_skeleton_ref_id(), g.source_skeleton.id());
    assert_eq!(g.source_skeleton.display_name(), "UAL2");
    let rebound = SourceSkeletonReference::new("UAL2-mutated-name").unwrap();
    assert_ne!(rebound.id(), g.source_skeleton.id());
    assert_eq!(g.motion_version.source_skeleton_ref_id(), g.source_skeleton.id());
}

#[test]
fn worker_result_job_spec_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.worker = WorkerResult::new(
        JobSpecId::generate(),
        g.backend.clone(),
        true,
        "completed",
        fixture_correlation(),
    )
        .unwrap()
        .with_staged_artifact_digests(vec![g.persistence.digest().clone()])
        .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn job_spec_character_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.job = mismatched_job(&g, CharacterAssetVersionId::generate());
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn job_spec_motion_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.job = JobSpec::new(
        g.character_version.id(),
        MotionAssetVersionId::generate(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn job_spec_source_skeleton_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.job = JobSpec::new(
        g.character_version.id(),
        g.motion_version.id(),
        SourceSkeletonReferenceId::generate(),
        g.mapping_version.id(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn job_spec_mapping_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.job = JobSpec::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.source_skeleton.id(),
        BoneMappingVersionId::generate(),
        g.policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn job_spec_policy_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.job = JobSpec::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.source_skeleton.id(),
        g.mapping_version.id(),
        RetargetPolicyVersionId::generate(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn qc_worker_result_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.qc = QcReport::for_derived_variant(
        g.derived_version.id(),
        g.policy_version.id(),
        WorkerResultId::generate(),
        QcVerdict::Pass,
    )
    .unwrap();
    g.derived_version.bind_qc_report(g.qc.id()).unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn qc_policy_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    g.qc = QcReport::for_derived_variant(
        g.derived_version.id(),
        RetargetPolicyVersionId::generate(),
        g.worker.id(),
        QcVerdict::Pass,
    )
    .unwrap();
    g.derived_version.bind_qc_report(g.qc.id()).unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn backend_execution_context_mismatch_fails_publication() {
    let mut g = unpublished_graph();
    let other = backend();
    g.worker = WorkerResult::new(g.job.id(), other, true, "completed", fixture_correlation())
        .unwrap()
        .with_staged_artifact_digests(vec![g.persistence.digest().clone()])
        .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn missing_persistence_artifact_fails_publication() {
    let mut g = unpublished_graph();
    let err = g.try_publish_missing_persistence().unwrap_err();
    assert_eq!(err.code, ErrorCode::PublicationEvidenceMissing);
    assert_eq!(g.derived_version.lifecycle(), Lifecycle::Draft);
}

#[test]
fn wrong_persistence_artifact_instance_fails_publication() {
    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        PersistenceArtifactInstanceId::generate(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn wrong_persistence_digest_fails_publication() {
    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        g.persistence.instance_id(),
        digest(9),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn missing_fresh_reopen_verification_fails_publication() {
    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        g.persistence.instance_id(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Missing,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::PublicationEvidenceMissing);
}

#[test]
fn failed_reopen_verification_fails_publication() {
    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        g.persistence.instance_id(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Fail,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::VerificationFailed);
}

#[test]
fn failed_structural_verification_fails_publication() {
    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        g.persistence.instance_id(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Fail,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::VerificationFailed);
}

#[test]
fn missing_verification_record_fails_publication() {
    let mut g = unpublished_graph();
    let err = g.try_publish_missing_verification().unwrap_err();
    assert_eq!(err.code, ErrorCode::PublicationEvidenceMissing);
}

#[test]
fn wrong_record_type_json_fails_ingress() {
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.job).unwrap(),
        "record_type",
        serde_json::json!("character_asset_version"),
    );
    let err = from_json_validated::<JobSpec>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRecordType);
}

#[test]
fn semantic_invalid_time_domain_json_fails_ingress() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.motion_version).unwrap();
    value["time"]["end"] = serde_json::json!({
        "kind": "seconds",
        "value_num": 2,
        "value_den": 1
    });
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<MotionAssetVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn time_start_after_end_json_fails_ingress() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.motion_version).unwrap();
    value["time"]["start"]["value_num"] = serde_json::json!(90);
    value["time"]["end"]["value_num"] = serde_json::json!(1);
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<MotionAssetVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn time_frame_fps_mismatch_json_fails_ingress() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.motion_version).unwrap();
    value["time"]["end"]["fps_num"] = serde_json::json!(24);
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<MotionAssetVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn invalid_compatibility_summary_json_fails_ingress() {
    let g = valid_graph();
    let result = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Unknown,
        CompatibilitySummary::Ready,
        vec![],
    )
    .unwrap();
    let mut value = serde_json::to_value(&result).unwrap();
    value["mapping_completeness"] = serde_json::json!("fail");
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<CompatibilityResult>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn ready_summary_cannot_coexist_with_fail_dimension() {
    let g = valid_graph();
    let err = CompatibilityResult::new(
        g.character_version.id(),
        g.motion_version.id(),
        g.mapping_version.id(),
        g.policy_version.id(),
        Judgment::Fail,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        Judgment::Pass,
        CompatibilitySummary::Ready,
        vec![],
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::CompatibilityContradiction);
}

#[test]
fn skeleton_subject_union_json_fails_ingress() {
    let g = valid_graph();
    let summary = SkeletonSummary::new(
        SkeletonSubjectKind::CharacterAssetVersion,
        Some(g.character_version.id()),
        None,
        g.backend.clone(),
        vec![JointObservation::new(
            JointKey::new("Bone").unwrap(),
            "Bone",
            None,
            true,
            None,
            None,
        )
        .unwrap()],
        vec![],
    )
    .unwrap();
    let mut value = serde_json::to_value(&summary).unwrap();
    value["subject_source_skeleton_ref_id"] = serde_json::json!(g.source_skeleton.id().canonical());
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<SkeletonSummary>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::SubjectUnion);
}

#[test]
fn qc_subject_union_json_fails_ingress() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.qc).unwrap();
    value["subject_persistence_artifact_id"] = serde_json::json!(g.persistence.id().canonical());
    let text = serde_json::to_string(&value).unwrap();
    let err = from_json_validated::<QcReport>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::SubjectUnion);
}

#[test]
fn invalid_exact_provenance_json_fails_ingress() {
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.job).unwrap(),
        "character_version_id",
        serde_json::json!("C:\\assets\\hero.fbx"),
    );
    let err = from_json_validated::<JobSpec>(&text).unwrap_err();
    assert!(matches!(
        err.code,
        ErrorCode::InvalidJson | ErrorCode::PathUsedAsId
    ));
}

#[test]
fn uuid_v4_rejected_on_product_id_deserialize() {
    let err = CharacterAssetId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap_err();
    assert_eq!(err.code, ErrorCode::UuidVersion);
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.character_version).unwrap(),
        "id",
        serde_json::json!("550e8400-e29b-41d4-a716-446655440000"),
    );
    let err = from_json_validated::<CharacterAssetVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::UuidVersion);
}

#[test]
fn structured_policy_round_trip_and_known_modes() {
    let g = valid_graph();
    assert_eq!(
        g.policy_version.channel_policy(),
        ChannelPolicy::RotationOnlyMappedNonRoot
    );
    assert_eq!(
        g.policy_version.quaternion_normalization_policy(),
        QuaternionNormalizationPolicy::NormalizeBeforeKey
    );
    assert_eq!(
        g.policy_version.quaternion_continuity_policy(),
        QuaternionContinuityPolicy::ConsecutiveHemisphere
    );
    assert_eq!(g.policy_version.root_policy(), RootPolicy::CopyWorldTranslationDelta);
    assert_eq!(g.policy_version.ik_policy(), IkPolicy::ExplicitNoIk);
    assert!(g.policy_version.contract_status().recognized());
    assert!(g.policy_version.contract_status().supported());
    assert!(!g.policy_version.contract_status().executed());
    assert!(g.policy_version.contract_status().audited());
    let json = to_json(&g.policy_version).unwrap();
    let lower = json.to_ascii_lowercase();
    assert!(!lower.contains("blender"));
    assert!(!lower.contains("bpy"));
    assert!(!lower.contains("posebone"));
    let back: RetargetPolicyVersion = from_json_validated(&json).unwrap();
    assert_eq!(g.policy_version, back);
}

#[test]
fn unknown_serialized_policy_mode_rejected() {
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.policy_version).unwrap(),
        "root_policy",
        serde_json::json!("invented_root_mode"),
    );
    let err = from_json_validated::<RetargetPolicyVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnknownField);
}

#[test]
fn worker_diagnostics_may_mention_backend_observations() {
    let g = unpublished_graph();
    let worker = WorkerResult::new(g.job.id(), g.backend.clone(), true, "completed", fixture_correlation())
        .unwrap()
        .with_diagnostics(vec![
            "Blender bpy error on object Knight_Armature".to_string(),
        ])
        .unwrap();
    worker.validate().unwrap();
    assert!(worker.diagnostics()[0].contains("Blender"));
}

fn memory_store() -> impl ProductVersionStore {
    struct MemoryStore;
    impl ProductVersionStore for MemoryStore {
        fn store_immutable_version<T: DomainRecord>(
            &mut self,
            record: Validated<T>,
        ) -> Result<(), DomainError> {
            let _ = to_json(record.as_record())?;
            Ok(())
        }

        fn load_exact_version<T: DomainRecord>(
            &self,
            _version_id: &str,
        ) -> Result<Validated<T>, DomainError> {
            Err(DomainError::new(
                ErrorCode::StoreContract,
                "V1-2 owns durable storage",
            ))
        }

        fn resolve_logical_object_versions(
            &self,
            _logical_id: &str,
        ) -> Result<Vec<String>, DomainError> {
            Err(DomainError::new(
                ErrorCode::StoreContract,
                "V1-2 owns durable storage",
            ))
        }

        fn store_artifact_metadata<T: DomainRecord>(
            &mut self,
            record: Validated<T>,
        ) -> Result<(), DomainError> {
            let _ = to_json(record.as_record())?;
            Ok(())
        }
    }
    MemoryStore
}

#[test]
fn storage_handoff_requires_typed_validated_records() {
    let mut store = memory_store();
    let g = valid_graph();
    store
        .store_immutable_version(Validated::certify(g.character_version.clone()).unwrap())
        .unwrap();
    let err = store
        .load_exact_version::<CharacterAssetVersion>(&g.character_version.id().canonical())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreContract);
}

#[test]
fn old_verification_does_not_apply_to_regenerated_bytes() {
    let g = valid_graph();
    let regenerated = g.persistence.regenerate(digest(9), 8192).unwrap();
    assert_eq!(regenerated.id(), g.persistence.id());
    assert_ne!(regenerated.instance_id(), g.persistence.instance_id());
    assert_ne!(regenerated.digest(), g.persistence.digest());
    let mut unpublished = unpublished_graph();
    unpublished.verification = PersistenceVerification::new(
        regenerated.id(),
        unpublished.persistence.instance_id(),
        regenerated.digest().clone(),
        unpublished.derived_version.id(),
        unpublished.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = unpublished.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
}

#[test]
fn zero_fps_is_rejected() {
    let err = TimePoint::frames(1, 0, 1).unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn validated_json_accepts_rational_frame_provenance() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.motion_version).unwrap();
    value["time"]["start"]["value_num"] = serde_json::json!(3);
    value["time"]["start"]["value_den"] = serde_json::json!(2);
    value["time"]["end"]["value_num"] = serde_json::json!(4);
    value["time"]["end"]["value_den"] = serde_json::json!(1);
    let loaded = from_json_validated::<MotionAssetVersion>(&value.to_string()).unwrap();
    let start = loaded.time().start();
    assert_eq!(start.value_num(), 3);
    assert_eq!(start.value_den(), 2);
    assert!(!start.is_integral_frame());
    assert_eq!(loaded.time().end().value_den(), 1);
}

#[test]
fn rational_frame_start_end_order_is_exact() {
    let start = TimePoint::frames_rational(3, 2, 30, 1).unwrap();
    let end = TimePoint::frames_rational(2, 1, 30, 1).unwrap();
    TimeDomainProvenance::new(
        "clip:rational-order",
        start.clone(),
        end.clone(),
        SamplingInterpretation::BakedEverySourceFrame,
        "declared",
    )
    .unwrap();
    let err = TimeDomainProvenance::new(
        "clip:rational-order",
        end,
        start,
        SamplingInterpretation::BakedEverySourceFrame,
        "declared",
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::TimeDomainInvalid);
}

#[test]
fn worker_result_json_requires_execution_correlation() {
    let g = unpublished_graph();
    let mut value = serde_json::to_value(&g.worker).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("execution_correlation");
    let err = from_json_validated::<WorkerResult>(&value.to_string()).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidJson);
}

#[test]
fn published_derived_records_exact_persistence_verification() {
    let g = valid_graph();
    assert_eq!(g.derived_version.lifecycle(), Lifecycle::Published);
    assert_eq!(
        g.derived_version.persistence_verification_id(),
        Some(g.verification.id())
    );
    assert_eq!(
        g.verification.persistence_artifact_id(),
        g.persistence.id()
    );
    assert_eq!(
        g.verification.persistence_artifact_instance_id(),
        g.persistence.instance_id()
    );
    assert_eq!(g.verification.payload_digest(), g.persistence.digest());
    assert_eq!(
        g.verification.subject_derived_variant_version_id(),
        g.derived_version.id()
    );
    assert_eq!(g.verification.fresh_reopen(), VerificationOutcome::Pass);
    assert_eq!(
        g.verification.structural_verification(),
        VerificationOutcome::Pass
    );
}

#[test]
fn regenerated_artifact_does_not_change_published_verification_reference() {
    let g = valid_graph();
    let bound = g.derived_version.persistence_verification_id();
    assert_eq!(bound, Some(g.verification.id()));
    let regenerated = g.persistence.regenerate(digest(9), 8192).unwrap();
    assert_eq!(regenerated.id(), g.persistence.id());
    assert_ne!(regenerated.instance_id(), g.persistence.instance_id());
    assert_ne!(regenerated.digest(), g.persistence.digest());
    assert_eq!(g.derived_version.persistence_verification_id(), bound);
    assert_eq!(
        g.verification.persistence_artifact_instance_id(),
        g.persistence.instance_id()
    );
    assert_eq!(g.verification.payload_digest(), g.persistence.digest());
    assert_ne!(
        g.verification.persistence_artifact_instance_id(),
        regenerated.instance_id()
    );
}

#[test]
fn wrong_verification_id_or_instance_cannot_publish() {
    let mut g = unpublished_graph();
    g.derived_version
        .bind_persistence_verification(PersistenceVerificationId::generate())
        .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
    assert_eq!(g.derived_version.lifecycle(), Lifecycle::Draft);

    let mut g = unpublished_graph();
    g.verification = PersistenceVerification::new(
        g.persistence.id(),
        PersistenceArtifactInstanceId::generate(),
        g.persistence.digest().clone(),
        g.derived_version.id(),
        g.backend.id(),
        VerificationOutcome::Pass,
        VerificationOutcome::Pass,
    )
    .unwrap();
    let err = g.try_publish().unwrap_err();
    assert_eq!(err.code, ErrorCode::GraphMismatch);
    assert!(g.derived_version.persistence_verification_id().is_none());
}

#[test]
fn raw_deserialized_record_cannot_be_stored_through_normal_store_contract() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.character_version).unwrap();
    value["display_name"] = serde_json::json!("");
    let text = serde_json::to_string(&value).unwrap();
    let raw: CharacterAssetVersion = serde_json::from_str(&text).unwrap();
    assert!(raw.validate().is_err());
    let err = Validated::certify(raw).unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingRequiredField);
}

#[test]
fn validated_ingress_can_be_stored() {
    let mut store = memory_store();
    let g = valid_graph();
    let json = to_json(&g.character_version).unwrap();
    let validated = ingest_validated::<CharacterAssetVersion>(&json).unwrap();
    store.store_immutable_version(validated).unwrap();
}

#[test]
fn semantic_invalid_json_cannot_produce_validated_record() {
    let g = valid_graph();
    let mut value = serde_json::to_value(&g.character_version).unwrap();
    value["display_name"] = serde_json::json!("");
    let text = serde_json::to_string(&value).unwrap();
    let err = ingest_validated::<CharacterAssetVersion>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingRequiredField);
    assert!(from_json_validated::<CharacterAssetVersion>(&text).is_err());
}

#[test]
fn unsupported_schema_cannot_produce_validated_record() {
    let g = valid_graph();
    let text = json_with(
        serde_json::to_value(&g.job).unwrap(),
        "schema_version",
        serde_json::json!(99),
    );
    let err = ingest_validated::<JobSpec>(&text).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnsupportedSchemaVersion);
    assert!(from_json_validated::<JobSpec>(&text).is_err());
}
