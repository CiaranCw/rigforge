//! Motion time-domain provenance. Frames are not seconds.

use crate::error::{DomainError, ErrorCode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeKind {
    Seconds,
    Frames,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimePoint {
    kind: TimeKind,
    value_num: i64,
    value_den: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fps_num: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fps_den: Option<u32>,
}

impl TimePoint {
    pub fn kind(&self) -> TimeKind {
        self.kind
    }

    pub fn value_num(&self) -> i64 {
        self.value_num
    }

    pub fn value_den(&self) -> u32 {
        self.value_den
    }

    pub fn is_integral_frame(&self) -> bool {
        self.kind == TimeKind::Frames && self.value_den == 1
    }

    pub fn fps_num(&self) -> Option<u32> {
        self.fps_num
    }

    pub fn fps_den(&self) -> Option<u32> {
        self.fps_den
    }

    pub fn seconds(num: i64, den: u32) -> Result<Self, DomainError> {
        let point = Self {
            kind: TimeKind::Seconds,
            value_num: num,
            value_den: den,
            fps_num: None,
            fps_den: None,
        };
        point.validate()?;
        Ok(point)
    }

    pub fn frames(frame: i64, fps_num: u32, fps_den: u32) -> Result<Self, DomainError> {
        Self::frames_rational(frame, 1, fps_num, fps_den)
    }

    /// Exact rational frame provenance. V1-3 Blender execution accepts only
    /// `value_den == 1`; non-integral points remain valid Domain records.
    pub fn frames_rational(
        value_num: i64,
        value_den: u32,
        fps_num: u32,
        fps_den: u32,
    ) -> Result<Self, DomainError> {
        let point = Self {
            kind: TimeKind::Frames,
            value_num,
            value_den,
            fps_num: Some(fps_num),
            fps_den: Some(fps_den),
        };
        point.validate()?;
        Ok(point)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.value_den == 0 {
            return Err(DomainError::new(
                ErrorCode::TimeDomainInvalid,
                "time denominator must be non-zero",
            ));
        }
        match self.kind {
            TimeKind::Seconds => {
                if self.fps_num.is_some() || self.fps_den.is_some() {
                    return Err(DomainError::new(
                        ErrorCode::TimeDomainInvalid,
                        "seconds time points must not carry fps as if they were frames",
                    ));
                }
            }
            TimeKind::Frames => match (self.fps_num, self.fps_den) {
                (Some(n), Some(d)) if n > 0 && d > 0 => {}
                _ => {
                    return Err(DomainError::new(
                        ErrorCode::TimeDomainInvalid,
                        "frame time points require fps_num/fps_den",
                    ));
                }
            },
        }
        Ok(())
    }

    fn as_i128_num_den(&self) -> (i128, i128) {
        (self.value_num as i128, self.value_den as i128)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingInterpretation {
    AuthoredKeys,
    BakedEverySourceFrame,
    MixedDeclared,
    UnknownDeclared,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeDomainProvenance {
    clip_identity_evidence: String,
    start: TimePoint,
    end: TimePoint,
    sampling: SamplingInterpretation,
    missing_channel_semantics: String,
}

impl TimeDomainProvenance {
    pub fn new(
        clip_identity_evidence: impl Into<String>,
        start: TimePoint,
        end: TimePoint,
        sampling: SamplingInterpretation,
        missing_channel_semantics: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let value = Self {
            clip_identity_evidence: clip_identity_evidence.into(),
            start,
            end,
            sampling,
            missing_channel_semantics: missing_channel_semantics.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn clip_identity_evidence(&self) -> &str {
        &self.clip_identity_evidence
    }

    pub fn start(&self) -> &TimePoint {
        &self.start
    }

    pub fn end(&self) -> &TimePoint {
        &self.end
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.clip_identity_evidence.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::MissingRequiredField,
                "clip_identity_evidence is required",
            ));
        }
        if self.missing_channel_semantics.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::MissingRequiredField,
                "missing_channel_semantics is required",
            ));
        }
        if self.start.kind != self.end.kind {
            return Err(DomainError::new(
                ErrorCode::TimeDomainInvalid,
                "start and end must share an explicit time kind; frames are not seconds",
            ));
        }
        self.start.validate()?;
        self.end.validate()?;
        match self.start.kind {
            TimeKind::Frames => {
                if self.start.fps_num != self.end.fps_num || self.start.fps_den != self.end.fps_den {
                    return Err(DomainError::new(
                        ErrorCode::TimeDomainInvalid,
                        "frame-based start and end must share the same fps rational",
                    ));
                }
            }
            TimeKind::Seconds => {}
        }
        if Self::rational_greater(&self.start, &self.end) {
            return Err(DomainError::new(
                ErrorCode::TimeDomainInvalid,
                "time-domain start must be <= end",
            ));
        }
        Ok(())
    }

    fn rational_greater(left: &TimePoint, right: &TimePoint) -> bool {
        let (ln, ld) = left.as_i128_num_den();
        let (rn, rd) = right.as_i128_num_den();
        ln * rd > rn * ld
    }
}
