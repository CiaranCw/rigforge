//! Compact Mapping review. Candidates remain unaccepted until an explicit accept op.

use rigforge_app::normalize_joint_name;
use rigforge_domain::{BoneMappingVersion, UnmappedDisposition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePair {
    pub semantic: String,
    pub source_key: Option<String>,
    pub target_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingEntryView {
    pub source_key: String,
    pub target_key: String,
    pub participation: String,
    pub role_profile: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnmappedView {
    pub side: String,
    pub joint_key: String,
    pub disposition: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingReview {
    pub mapped_count: usize,
    pub unmapped_optional_helper_count: usize,
    pub blocking_unmapped_count: usize,
    pub ambiguity_count: usize,
    pub one_click_eligible: bool,
    pub confirmation_required: bool,
    pub accepted: bool,
    pub core_pairs: Vec<CorePair>,
    pub entries: Vec<MappingEntryView>,
    pub unmapped: Vec<UnmappedView>,
    pub ambiguities: Vec<String>,
}

impl MappingReview {
    pub fn fail_closed_for_one_click(&self) -> bool {
        self.blocking_unmapped_count > 0 || self.ambiguity_count > 0
    }
}

pub fn mapping_review_from_version(version: &BoneMappingVersion) -> MappingReview {
    let mapped_count = version.entries().len();
    let unmapped: Vec<UnmappedView> = version
        .unmapped_source()
        .iter()
        .chain(version.unmapped_target().iter())
        .map(|joint| UnmappedView {
            side: format!("{:?}", joint.skeleton_side()).to_ascii_lowercase(),
            joint_key: joint.joint_key().as_str().to_string(),
            disposition: format!("{:?}", joint.disposition()).to_ascii_lowercase(),
            reason: joint.reason().to_string(),
        })
        .collect();
    let blocking_unmapped_count = version
        .unmapped_source()
        .iter()
        .chain(version.unmapped_target().iter())
        .filter(|joint| joint.disposition() == UnmappedDisposition::Blocking)
        .count();
    let unmapped_optional_helper_count = version
        .unmapped_source()
        .iter()
        .chain(version.unmapped_target().iter())
        .filter(|joint| {
            matches!(
                joint.disposition(),
                UnmappedDisposition::Optional | UnmappedDisposition::Helper
            )
        })
        .count();
    let ambiguities = version.review().ambiguities().to_vec();
    let ambiguity_count = ambiguities.len();
    let accepted = version.review().reviewed();
    let entries: Vec<MappingEntryView> = version
        .entries()
        .iter()
        .map(|entry| MappingEntryView {
            source_key: entry.source().joint_key().as_str().to_string(),
            target_key: entry.target().joint_key().as_str().to_string(),
            participation: format!("{:?}", entry.participation()).to_ascii_lowercase(),
            role_profile: entry.role_profile().map(str::to_string),
        })
        .collect();
    MappingReview {
        one_click_eligible: blocking_unmapped_count == 0 && ambiguity_count == 0,
        mapped_count,
        unmapped_optional_helper_count,
        blocking_unmapped_count,
        ambiguity_count,
        confirmation_required: version.review().confirmation_required() || !accepted,
        accepted,
        core_pairs: core_semantic_pairs(&entries),
        entries,
        unmapped,
        ambiguities,
    }
}

pub fn core_semantic_pairs(entries: &[MappingEntryView]) -> Vec<CorePair> {
    const GROUPS: &[(&str, &[&str])] = &[
        ("Root", &["root", "armature"]),
        ("Pelvis/Hips", &["pelvis", "hips", "body"]),
        ("Spine", &["spine", "spine_01", "spine01", "spine_02", "spine02", "spine_03", "chest", "torso"]),
        ("Clavicle", &["clavicle", "collar"]),
        ("UpperArm", &["upperarm", "upper_arm", "uparm", "shoulder"]),
        ("LowerArm", &["lowerarm", "lower_arm", "forearm", "elbow"]),
        ("Hand", &["hand", "fist", "wrist"]),
        ("UpperLeg", &["thigh", "upperleg", "upper_leg", "upleg"]),
        ("LowerLeg", &["calf", "lowerleg", "lower_leg", "shin"]),
        ("Foot", &["foot", "ankle"]),
    ];
    GROUPS
        .iter()
        .map(|(semantic, stems)| {
            let found = entries.iter().find(|entry| {
                stem_matches(&entry.source_key, stems) || stem_matches(&entry.target_key, stems)
            });
            CorePair {
                semantic: (*semantic).to_string(),
                source_key: found.map(|entry| entry.source_key.clone()),
                target_key: found.map(|entry| entry.target_key.clone()),
            }
        })
        .collect()
}

fn stem_matches(name: &str, stems: &[&str]) -> bool {
    let normalized = normalize_joint_name(name);
    stems.iter().any(|stem| normalized.stem == *stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_click_fails_closed_on_blocking_or_ambiguity() {
        let blocking = MappingReview {
            mapped_count: 1,
            unmapped_optional_helper_count: 0,
            blocking_unmapped_count: 1,
            ambiguity_count: 0,
            one_click_eligible: false,
            confirmation_required: true,
            accepted: false,
            core_pairs: vec![],
            entries: vec![],
            unmapped: vec![],
            ambiguities: vec![],
        };
        assert!(blocking.fail_closed_for_one_click());
        let ambiguous = MappingReview {
            blocking_unmapped_count: 0,
            ambiguity_count: 2,
            ..blocking
        };
        assert!(ambiguous.fail_closed_for_one_click());
        let ok = MappingReview {
            blocking_unmapped_count: 0,
            ambiguity_count: 0,
            one_click_eligible: true,
            ..ambiguous
        };
        assert!(!ok.fail_closed_for_one_click());
    }

    #[test]
    fn core_pairs_use_observed_keys_not_alias_guesses() {
        let entries = vec![
            MappingEntryView {
                source_key: "root".into(),
                target_key: "Root".into(),
                participation: "required".into(),
                role_profile: Some("root".into()),
            },
            MappingEntryView {
                source_key: "pelvis".into(),
                target_key: "Hips".into(),
                participation: "required".into(),
                role_profile: Some("pelvis".into()),
            },
            MappingEntryView {
                source_key: "upperarm_l".into(),
                target_key: "Shoulder_L".into(),
                participation: "required".into(),
                role_profile: Some("upper_arm".into()),
            },
        ];
        let pairs = core_semantic_pairs(&entries);
        let pelvis = pairs.iter().find(|pair| pair.semantic == "Pelvis/Hips").unwrap();
        assert_eq!(pelvis.source_key.as_deref(), Some("pelvis"));
        assert_eq!(pelvis.target_key.as_deref(), Some("Hips"));
        let arm = pairs.iter().find(|pair| pair.semantic == "UpperArm").unwrap();
        assert_eq!(arm.target_key.as_deref(), Some("Shoulder_L"));
    }
}
