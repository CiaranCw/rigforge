//! JSON/stdio envelopes. Not Product records.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Envelope {
    pub id: String,
    pub op: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MachineMessage {
    Progress { id: String, phase: String },
    Result { id: String, ok: bool, result: Value, error: Option<String> },
}

pub type MachineEvent = MachineMessage;
pub type MachineResult = MachineMessage;

impl MachineMessage {
    pub fn progress(id: impl Into<String>, phase: impl Into<String>) -> Self {
        Self::Progress {
            id: id.into(),
            phase: phase.into(),
        }
    }

    pub fn ok(id: impl Into<String>, result: Value) -> Self {
        Self::Result {
            id: id.into(),
            ok: true,
            result,
            error: None,
        }
    }

    pub fn err(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self::Result {
            id: id.into(),
            ok: false,
            result: Value::Null,
            error: Some(error.into()),
        }
    }

    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub fn parse_line(line: &str) -> Result<Envelope, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err("empty machine request line".into());
    }
    serde_json::from_str(trimmed).map_err(|err| format!("invalid machine request: {err}"))
}

pub fn param_str(params: &Value, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("missing required string param '{key}'"))
}

pub fn param_str_opt(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn param_bool(params: &Value, key: &str, default: bool) -> bool {
    params
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}
