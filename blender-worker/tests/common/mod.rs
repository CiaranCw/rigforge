#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rigforge_app::SqliteCatalog;
use rigforge_domain::*;
use sha2::{Digest, Sha256};

pub fn digest(n: u8) -> ContentDigest {
    ContentDigest::parse(&format!("{n:x}").repeat(64)).unwrap()
}

pub fn source_at(path: &str, n: u8) -> SourceArtifactEvidence {
    SourceArtifactEvidence::new(
        LocationEvidence::filesystem_path(path).unwrap(),
        digest(n),
        1024,
        "application/octet-stream",
        Some("2026-09-01T00:00:00Z".to_string()),
        None,
    )
    .unwrap()
}

pub fn time_domain(clip: &str) -> TimeDomainProvenance {
    time_domain_points(
        clip,
        TimePoint::frames(1, 30, 1).unwrap(),
        TimePoint::frames(61, 30, 1).unwrap(),
    )
}

pub fn time_domain_points(clip: &str, start: TimePoint, end: TimePoint) -> TimeDomainProvenance {
    TimeDomainProvenance::new(
        clip,
        start,
        end,
        SamplingInterpretation::BakedEverySourceFrame,
        "unmapped target joints remain at target rest",
    )
    .unwrap()
}

pub fn mapping_entry(src: &str, dst: &str, role: &str, required: bool) -> BoneMappingEntry {
    BoneMappingEntry::new(
        JointRef::source(JointKey::new(src).unwrap()),
        JointRef::target(JointKey::new(dst).unwrap()),
        if required {
            JointParticipation::Required
        } else {
            JointParticipation::Optional
        },
        Some(role.to_string()),
        "hierarchy plus review",
    )
    .unwrap()
}

pub struct Graph {
    pub character_version: CharacterAssetVersion,
    pub motion_version: MotionAssetVersion,
    pub source_skeleton: SourceSkeletonReference,
    pub mapping_version: BoneMappingVersion,
    pub policy_version: RetargetPolicyVersion,
    pub job: JobSpec,
}

pub fn graph_with_sources(character_path: &str, motion_path: &str, clip: &str) -> Graph {
    graph_with_sources_time(character_path, motion_path, time_domain(clip))
}

pub fn graph_with_sources_time(
    character_path: &str,
    motion_path: &str,
    time: TimeDomainProvenance,
) -> Graph {
    let mut character = CharacterAsset::new("Knight").unwrap();
    let mut character_version =
        CharacterAssetVersion::draft(character.id(), "Knight v1", source_at(character_path, 1))
            .unwrap();
    character_version.publish().unwrap();
    character.bind_published(character_version.id());

    let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
    let mut motion = MotionAsset::new("Walk Carry").unwrap();
    let mut motion_version = MotionAssetVersion::draft(
        motion.id(),
        "Walk Carry v1",
        source_skeleton.id(),
        time,
        source_at(motion_path, 2),
    )
    .unwrap();
    motion_version.publish().unwrap();
    motion.bind_published(motion_version.id());

    let mut mapping = BoneMapping::new("knight-ual2").unwrap();
    let mut mapping_version = BoneMappingVersion::draft(
        mapping.id(),
        character_version.id(),
        source_skeleton.id(),
        vec![
            mapping_entry("root", "Bone", "root", true),
            mapping_entry("pelvis", "Body", "pelvis_central", true),
        ],
        MappingReviewProvenance::new(
            true,
            Some("manual review".to_string()),
            vec!["foot parent chain".to_string()],
        )
        .unwrap(),
    )
    .unwrap();
    mapping_version
        .bind_source_motion_version(motion_version.id())
        .unwrap();
    mapping_version.publish().unwrap();
    mapping.bind_published(mapping_version.id());

    let mut policy = RetargetPolicy::new("rest-relative").unwrap();
    let mut policy_version = RetargetPolicyVersion::proven_draft(policy.id()).unwrap();
    policy_version.publish().unwrap();
    policy.bind_published(policy_version.id());

    let job = JobSpec::new(
        character_version.id(),
        motion_version.id(),
        source_skeleton.id(),
        mapping_version.id(),
        policy_version.id(),
        "semantic-consistency; not byte-identical",
        "one isolated process; staged output unpublished on error",
    )
    .unwrap();

    Graph {
        character_version,
        motion_version,
        source_skeleton,
        mapping_version,
        policy_version,
        job,
    }
}

pub fn seed(catalog: &mut SqliteCatalog, g: &Graph) -> Validated<JobSpec> {
    catalog
        .put_validated(&Validated::certify(g.character_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(g.source_skeleton.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(g.motion_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(g.mapping_version.clone()).unwrap())
        .unwrap();
    catalog
        .put_validated(&Validated::certify(g.policy_version.clone()).unwrap())
        .unwrap();
    Validated::certify(g.job.clone()).unwrap()
}

pub fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!("rf-v13-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

pub fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn fake_blender() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rigforge_fake_blender"))
}
