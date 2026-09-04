//! Deterministic Mapping candidate generation.
//! Candidates are not accepted Product Mapping until an explicit accept operation.

use rigforge_domain::{
    BoneMappingEntry, JointKey, JointObservation, JointParticipation, JointRef, SkeletonSummary,
    UnmappedDisposition, UnmappedJoint,
};

use crate::error::AppError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MappingAssistProfile {
    None,
    OptionalHumanoid,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum CandidateSignal {
    RestEvidence,
    DeformEvidence,
    ParentContext,
    RoleHint,
    RootPosition,
    Alias,
    NormalizedName,
    ExactName,
}

impl CandidateSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactName => "EXACT_NAME",
            Self::NormalizedName => "NORMALIZED_NAME",
            Self::Alias => "ALIAS",
            Self::ParentContext => "PARENT_CONTEXT",
            Self::RoleHint => "ROLE_HINT",
            Self::RootPosition => "ROOT_POSITION",
            Self::DeformEvidence => "DEFORM_EVIDENCE",
            Self::RestEvidence => "REST_EVIDENCE",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Laterality {
    None,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedName {
    pub stem: String,
    pub laterality: Laterality,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposedMappingEntry {
    pub source_key: String,
    pub target_key: String,
    pub participation: JointParticipation,
    pub role_profile: Option<String>,
    pub signals: Vec<CandidateSignal>,
    pub evidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingAmbiguity {
    pub source_key: String,
    pub target_keys: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingProposal {
    pub entries: Vec<ProposedMappingEntry>,
    pub unmapped_source: Vec<UnmappedJoint>,
    pub unmapped_target: Vec<UnmappedJoint>,
    pub ambiguities: Vec<MappingAmbiguity>,
    pub confirmation_required: bool,
}

impl MappingProposal {
    pub fn to_domain_entries(&self) -> Result<Vec<BoneMappingEntry>, AppError> {
        self.entries
            .iter()
            .map(|item| {
                BoneMappingEntry::new(
                    JointRef::source(JointKey::new(&item.source_key)?),
                    JointRef::target(JointKey::new(&item.target_key)?),
                    item.participation,
                    item.role_profile.clone(),
                    item.evidence.clone(),
                )
                .map_err(AppError::from)
            })
            .collect()
    }
}

pub fn normalize_joint_name(name: &str) -> NormalizedName {
    let mut s = name.trim().to_ascii_lowercase();
    if let Some(rest) = s.strip_prefix("mixamorig:") {
        s = rest.to_string();
    }
    if let Some(idx) = s.rfind('|') {
        s = s[idx + 1..].to_string();
    }
    s = s.replace(['.', '-', ' '], "_");
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    let laterality = laterality_of(&s);
    s = strip_laterality(&s);
    if let Some(stripped) = s.strip_suffix("_end") {
        s = stripped.to_string();
    }
    NormalizedName {
        stem: s.trim_matches('_').to_string(),
        laterality,
    }
}

fn laterality_of(s: &str) -> Laterality {
    let tokens: Vec<&str> = s.split('_').collect();
    if tokens.iter().any(|t| *t == "l" || *t == "left") || s.ends_with("_l") {
        return Laterality::Left;
    }
    if tokens.iter().any(|t| *t == "r" || *t == "right") || s.ends_with("_r") {
        return Laterality::Right;
    }
    Laterality::None
}

fn strip_laterality(s: &str) -> String {
    let parts: Vec<&str> = s
        .split('_')
        .filter(|t| !matches!(*t, "l" | "r" | "left" | "right"))
        .collect();
    parts.join("_")
}

fn optional_humanoid_role(stem: &str) -> Option<&'static str> {
    match stem {
        "root" | "armature" | "bone" => Some("root"),
        "pelvis" | "body" => Some("pelvis"),
        "spine" | "spine_01" | "spine01" | "spine1" | "hips" => Some("spine_01"),
        "spine_02" | "spine02" | "spine2" | "abdomen" => Some("spine_02"),
        "spine_03" | "spine03" | "spine3" | "torso" | "chest" | "spine_04" => Some("spine_03"),
        "neck" | "neck_01" | "neck01" => Some("neck"),
        "head" => Some("head"),
        "clavicle" | "shoulder" | "collar" => Some("clavicle"),
        "upperarm" | "upper_arm" | "uparm" => Some("upper_arm"),
        "lowerarm" | "lower_arm" | "forearm" | "elbow" => Some("lower_arm"),
        "hand" | "fist" | "wrist" => Some("hand"),
        "thigh" | "upperleg" | "upper_leg" | "upleg" => Some("thigh"),
        "calf" | "lowerleg" | "lower_leg" | "shin" => Some("shin"),
        "foot" | "ankle" => Some("foot"),
        "ball" | "toe" | "toebase" => Some("ball"),
        _ => None,
    }
}

fn is_optional_extremity(stem: &str, display: &str) -> bool {
    let d = display.to_ascii_lowercase();
    [
        "thumb", "index", "middle", "ring", "pinky", "finger", "leaf", "ball", "toe",
        "eye", "eyebrow", "jaw",
    ]
        .iter()
        .any(|n| stem.contains(n) || d.contains(n))
}

fn participation_for(joint: &JointObservation, norm: &NormalizedName) -> JointParticipation {
    if joint.is_helper_or_control() || is_optional_extremity(&norm.stem, joint.display_name()) {
        JointParticipation::Optional
    } else {
        JointParticipation::Required
    }
}

#[derive(Clone)]
struct Ranked {
    target_idx: usize,
    signals: Vec<CandidateSignal>,
}

fn score(signals: &[CandidateSignal]) -> u8 {
    signals.iter().map(|s| *s as u8).max().unwrap_or(0)
}

fn signals_for(
    source: &JointObservation,
    target: &JointObservation,
    src_n: &NormalizedName,
    dst_n: &NormalizedName,
    profile: MappingAssistProfile,
) -> Vec<CandidateSignal> {
    let mut signals = Vec::new();
    if source.display_name() == target.display_name() {
        signals.push(CandidateSignal::ExactName);
    }
    if src_n.stem == dst_n.stem
        && src_n.laterality == dst_n.laterality
        && !src_n.stem.is_empty()
    {
        signals.push(CandidateSignal::NormalizedName);
    }
    if src_n.laterality != dst_n.laterality
        && src_n.laterality != Laterality::None
        && dst_n.laterality != Laterality::None
    {
        return signals
            .into_iter()
            .filter(|s| matches!(s, CandidateSignal::ExactName))
            .collect();
    }
    if source.is_root() && target.is_root() {
        signals.push(CandidateSignal::RootPosition);
    }
    if profile == MappingAssistProfile::OptionalHumanoid {
        if let (Some(a), Some(b)) = (
            optional_humanoid_role(&src_n.stem),
            optional_humanoid_role(&dst_n.stem),
        ) {
            if a == b && src_n.laterality == dst_n.laterality {
                if src_n.stem != dst_n.stem {
                    signals.push(CandidateSignal::Alias);
                }
                signals.push(CandidateSignal::RoleHint);
            }
        }
    }
    let src_def = source.deform_observation().unwrap_or("");
    let dst_def = target.deform_observation().unwrap_or("");
    if !src_def.is_empty() && src_def == dst_def {
        signals.push(CandidateSignal::DeformEvidence);
    }
    if source.rest_evidence().is_some() && target.rest_evidence().is_some() {
        signals.push(CandidateSignal::RestEvidence);
    }
    signals.sort();
    signals.dedup();
    signals
}

fn laterality_compatible(a: Laterality, b: Laterality) -> bool {
    a == b || a == Laterality::None || b == Laterality::None
}

/// Deterministic joint-to-joint candidate generation. Repeated calls with the
/// same summaries and profile produce the same proposal.
pub fn generate_mapping_proposal(
    source: &SkeletonSummary,
    target: &SkeletonSummary,
    profile: MappingAssistProfile,
) -> Result<MappingProposal, AppError> {
    if source.joints().is_empty() || target.joints().is_empty() {
        return Err(AppError::Catalog(
            "SkeletonSummary must contain joints before mapping candidates can be generated".into(),
        ));
    }

    let src_norms: Vec<NormalizedName> = source
        .joints()
        .iter()
        .map(|j| normalize_joint_name(j.display_name()))
        .collect();
    let dst_norms: Vec<NormalizedName> = target
        .joints()
        .iter()
        .map(|j| normalize_joint_name(j.display_name()))
        .collect();

    let mut ranked: Vec<Vec<Ranked>> = source
        .joints()
        .iter()
        .enumerate()
        .map(|(si, sj)| {
            target
                .joints()
                .iter()
                .enumerate()
                .filter_map(|(ti, tj)| {
                    if tj.is_helper_or_control() {
                        return None;
                    }
                    let signals = signals_for(sj, tj, &src_norms[si], &dst_norms[ti], profile);
                    let meaningful = signals.iter().any(|s| {
                        !matches!(s, CandidateSignal::DeformEvidence | CandidateSignal::RestEvidence)
                    });
                    if meaningful {
                        Some(Ranked {
                            target_idx: ti,
                            signals,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();

    for row in &mut ranked {
        row.sort_by(|a, b| {
            score(&b.signals)
                .cmp(&score(&a.signals))
                .then_with(|| a.target_idx.cmp(&b.target_idx))
        });
    }

    let mut assigned_source: Vec<Option<usize>> = vec![None; source.joints().len()];
    let mut assigned_target: Vec<Option<usize>> = vec![None; target.joints().len()];
    let mut ambiguities = Vec::new();

    let mut order: Vec<usize> = (0..source.joints().len()).collect();
    order.sort_by(|&a, &b| {
        let sa = ranked[a].first().map(|r| score(&r.signals)).unwrap_or(0);
        let sb = ranked[b].first().map(|r| score(&r.signals)).unwrap_or(0);
        sb.cmp(&sa).then_with(|| {
            source.joints()[a]
                .joint_key()
                .as_str()
                .cmp(source.joints()[b].joint_key().as_str())
        })
    });

    for si in order {
        if assigned_source[si].is_some() {
            continue;
        }
        let row = &ranked[si];
        if row.is_empty() {
            continue;
        }
        let best = score(&row[0].signals);
        let tied: Vec<&Ranked> = row
            .iter()
            .filter(|r| score(&r.signals) == best && assigned_target[r.target_idx].is_none())
            .collect();
        if tied.is_empty() {
            continue;
        }
        if tied.len() > 1 {
            let src_part = participation_for(&source.joints()[si], &src_norms[si]);
            if src_part == JointParticipation::Required {
                ambiguities.push(MappingAmbiguity {
                    source_key: source.joints()[si].joint_key().as_str().to_string(),
                    target_keys: tied
                        .iter()
                        .map(|r| target.joints()[r.target_idx].joint_key().as_str().to_string())
                        .collect(),
                    reason: "equal candidate scores; confirmation required".into(),
                });
            }
            continue;
        }
        let choice = tied[0];
        if let Some(prev) = assigned_target[choice.target_idx] {
            let src_part = participation_for(&source.joints()[si], &src_norms[si]);
            let prev_part = participation_for(&source.joints()[prev], &src_norms[prev]);
            if src_part == JointParticipation::Required && prev_part == JointParticipation::Required
            {
                ambiguities.push(MappingAmbiguity {
                    source_key: source.joints()[si].joint_key().as_str().to_string(),
                    target_keys: vec![target.joints()[choice.target_idx]
                        .joint_key()
                        .as_str()
                        .to_string()],
                    reason: "required target collision; confirmation required".into(),
                });
            }
            continue;
        }
        assigned_source[si] = Some(choice.target_idx);
        assigned_target[choice.target_idx] = Some(si);
    }

    parent_context_pass(
        source,
        target,
        &src_norms,
        &dst_norms,
        &mut assigned_source,
        &mut assigned_target,
        &mut ranked,
    );

    let mut entries = Vec::new();
    let mut unmapped_source = Vec::new();
    let mut unmapped_target = Vec::new();

    for (si, sj) in source.joints().iter().enumerate() {
        if let Some(ti) = assigned_source[si] {
            let signals = ranked[si]
                .iter()
                .find(|r| r.target_idx == ti)
                .map(|r| r.signals.clone())
                .unwrap_or_else(|| vec![CandidateSignal::ParentContext]);
            let participation = participation_for(sj, &src_norms[si]);
            let role = match profile {
                MappingAssistProfile::OptionalHumanoid => {
                    optional_humanoid_role(&src_norms[si].stem).map(|s| s.to_string())
                }
                MappingAssistProfile::None => None,
            };
            let evidence = signals
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",");
            entries.push(ProposedMappingEntry {
                source_key: sj.joint_key().as_str().to_string(),
                target_key: target.joints()[ti].joint_key().as_str().to_string(),
                participation,
                role_profile: role,
                signals,
                evidence,
            });
        } else {
            let participation = participation_for(sj, &src_norms[si]);
            let blocking = participation == JointParticipation::Required
                && ambiguities
                    .iter()
                    .all(|a| a.source_key != sj.joint_key().as_str());
            let disposition = if sj.is_helper_or_control() {
                UnmappedDisposition::Helper
            } else if participation == JointParticipation::Optional {
                UnmappedDisposition::Optional
            } else {
                UnmappedDisposition::Blocking
            };
            let reason = if blocking {
                "required source joint has no unique candidate"
            } else if participation == JointParticipation::Optional || sj.is_helper_or_control() {
                "optional or helper joint left unmapped"
            } else {
                "source joint left unmapped pending confirmation"
            };
            unmapped_source.push(UnmappedJoint::new(
                rigforge_domain::SkeletonSide::Source,
                sj.joint_key().clone(),
                reason,
                disposition,
            )?);
        }
    }

    for (ti, tj) in target.joints().iter().enumerate() {
        if assigned_target[ti].is_none() {
            let participation = participation_for(tj, &dst_norms[ti]);
            let helper = tj.is_helper_or_control();
            let disposition = if helper {
                UnmappedDisposition::Helper
            } else if participation == JointParticipation::Optional {
                UnmappedDisposition::Optional
            } else {
                UnmappedDisposition::Blocking
            };
            let reason = if helper || participation == JointParticipation::Optional {
                "optional/helper target joint left unmapped"
            } else {
                "target joint has no unique source candidate"
            };
            unmapped_target.push(UnmappedJoint::new(
                rigforge_domain::SkeletonSide::Target,
                tj.joint_key().clone(),
                reason,
                disposition,
            )?);
        }
    }

    entries.sort_by(|a, b| a.source_key.cmp(&b.source_key));
    ambiguities.sort_by(|a, b| a.source_key.cmp(&b.source_key));
    let confirmation_required = !ambiguities.is_empty()
        || unmapped_source
            .iter()
            .any(|u| u.disposition() == UnmappedDisposition::Blocking);

    Ok(MappingProposal {
        entries,
        unmapped_source,
        unmapped_target,
        ambiguities,
        confirmation_required,
    })
}

fn parent_context_pass(
    source: &SkeletonSummary,
    target: &SkeletonSummary,
    src_norms: &[NormalizedName],
    dst_norms: &[NormalizedName],
    assigned_source: &mut [Option<usize>],
    assigned_target: &mut [Option<usize>],
    ranked: &mut [Vec<Ranked>],
) {
    for si in 0..source.joints().len() {
        if assigned_source[si].is_some() {
            continue;
        }
        let Some(parent_key) = source.joints()[si].parent_key() else {
            continue;
        };
        let Some(parent_si) = source
            .joints()
            .iter()
            .position(|j| j.joint_key() == parent_key)
        else {
            continue;
        };
        let Some(parent_ti) = assigned_source[parent_si] else {
            continue;
        };
        let parent_target_key = target.joints()[parent_ti].joint_key();
        let mut candidates: Vec<usize> = target
            .joints()
            .iter()
            .enumerate()
            .filter(|(ti, tj)| {
                assigned_target[*ti].is_none()
                    && tj.parent_key() == Some(parent_target_key)
                    && laterality_compatible(src_norms[si].laterality, dst_norms[*ti].laterality)
                    && !tj.is_helper_or_control()
            })
            .map(|(ti, _)| ti)
            .collect();
        if src_norms[si].laterality != Laterality::None {
            let lat_match: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|ti| dst_norms[*ti].laterality == src_norms[si].laterality)
                .collect();
            if lat_match.len() == 1 {
                candidates = lat_match;
            }
        }
        if candidates.len() == 1 {
            let ti = candidates[0];
            assigned_source[si] = Some(ti);
            assigned_target[ti] = Some(si);
            ranked[si].push(Ranked {
                target_idx: ti,
                signals: vec![CandidateSignal::ParentContext],
            });
        }
    }
}
