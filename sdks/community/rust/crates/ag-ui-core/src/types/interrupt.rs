use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// An interrupt represents a human-in-the-loop pause point.
///
/// When an agent needs user input to proceed (e.g., approval, decision),
/// it emits an interrupt. The frontend collects the user's response
/// and resumes the run with a `ResumeEntry`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interrupt {
    pub id: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(rename = "toolCallId", skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(rename = "responseSchema", skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<JsonValue>,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

impl Interrupt {
    pub fn new(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            reason: reason.into(),
            message: None,
            tool_call_id: None,
            response_schema: None,
            expires_at: None,
            metadata: None,
        }
    }
}

/// Status of a resume entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResumeStatus {
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// A resume entry represents the user's response to an interrupt.
///
/// Sent by the frontend to resume an agent run after a human-in-the-loop
/// interrupt. Maps an `interruptId` to a resolution status and optional payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumeEntry {
    #[serde(rename = "interruptId")]
    pub interrupt_id: String,
    pub status: ResumeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<JsonValue>,
}

impl ResumeEntry {
    pub fn new(interrupt_id: impl Into<String>, status: ResumeStatus) -> Self {
        Self {
            interrupt_id: interrupt_id.into(),
            status,
            payload: None,
        }
    }
}
