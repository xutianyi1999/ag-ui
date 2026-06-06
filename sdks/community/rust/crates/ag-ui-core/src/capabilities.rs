use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Information about a sub-agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubAgentInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Identity capabilities — who or what the agent is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IdentityCapabilities {
    /// Human-readable agent name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Framework/platform type (e.g. "langgraph", "mastra").
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// What the agent does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Semantic version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Organization or team.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Link to documentation.
    #[serde(rename = "documentationUrl", skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    /// Arbitrary key-value metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

/// Transport capabilities — how the agent communicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TransportCapabilities {
    /// SSE streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    /// WebSocket support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websocket: Option<bool>,
    /// Binary protobuf over HTTP.
    #[serde(rename = "httpBinary", skip_serializing_if = "Option::is_none")]
    pub http_binary: Option<bool>,
    /// Webhook push notifications.
    #[serde(rename = "pushNotifications", skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    /// Resumable via sequence numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumable: Option<bool>,
}

/// Tools capabilities — what tools the agent supports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ToolsCapabilities {
    /// Agent can make tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    /// Tool definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<super::types::tool::Tool>>,
    /// Concurrent tool invocation.
    #[serde(rename = "parallelCalls", skip_serializing_if = "Option::is_none")]
    pub parallel_calls: Option<bool>,
    /// Accepts runtime client tools.
    #[serde(rename = "clientProvided", skip_serializing_if = "Option::is_none")]
    pub client_provided: Option<bool>,
}

/// Output capabilities — what outputs the agent produces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OutputCapabilities {
    /// Structured JSON output.
    #[serde(rename = "structuredOutput", skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<bool>,
    /// MIME types the agent can produce.
    #[serde(rename = "supportedMimeTypes", skip_serializing_if = "Option::is_none")]
    pub supported_mime_types: Option<Vec<String>>,
}

/// State capabilities — how the agent manages state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StateCapabilities {
    /// STATE_SNAPSHOT events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<bool>,
    /// STATE_DELTA events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deltas: Option<bool>,
    /// Long-term memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<bool>,
    /// State preserved across runs.
    #[serde(rename = "persistentState", skip_serializing_if = "Option::is_none")]
    pub persistent_state: Option<bool>,
}

/// Multi-agent capabilities — delegation and handoffs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MultiAgentCapabilities {
    /// Multi-agent support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    /// Delegate subtasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation: Option<bool>,
    /// Transfer conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handoffs: Option<bool>,
    /// Sub-agent definitions.
    #[serde(rename = "subAgents", skip_serializing_if = "Option::is_none")]
    pub sub_agents: Option<Vec<SubAgentInfo>>,
}

/// Reasoning capabilities — chain-of-thought support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReasoningCapabilities {
    /// Produces reasoning tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    /// Incremental streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    /// Encrypted reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<bool>,
}

/// Multimodal input capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MultimodalInputCapabilities {
    /// Image input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    /// Audio input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    /// Video input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<bool>,
    /// PDF input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf: Option<bool>,
    /// File input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<bool>,
}

/// Multimodal output capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MultimodalOutputCapabilities {
    /// Image output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    /// Audio output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
}

/// Multimodal capabilities — input + output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MultimodalCapabilities {
    /// Input capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<MultimodalInputCapabilities>,
    /// Output capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<MultimodalOutputCapabilities>,
}

/// Execution capabilities — code execution limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExecutionCapabilities {
    /// Code execution support.
    #[serde(rename = "codeExecution", skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<bool>,
    /// Sandboxed execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandboxed: Option<bool>,
    /// Maximum iterations.
    #[serde(rename = "maxIterations", skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u64>,
    /// Maximum execution time (milliseconds).
    #[serde(rename = "maxExecutionTime", skip_serializing_if = "Option::is_none")]
    pub max_execution_time: Option<u64>,
}

/// Human-in-the-loop capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HumanInTheLoopCapabilities {
    /// HITL support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    /// Approval before sensitive actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvals: Option<bool>,
    /// Modify plan mid-execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interventions: Option<bool>,
    /// User feedback incorporation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<bool>,
    /// Interrupt protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupts: Option<bool>,
    /// Approve with edits (editedArgs in resume).
    #[serde(rename = "approveWithEdits", skip_serializing_if = "Option::is_none")]
    pub approve_with_edits: Option<bool>,
}

/// Top-level agent capabilities declaration.
///
/// All fields are optional. Omitted fields are interpreted as "unknown / not applicable".
/// Equivalent to `AgentCapabilitiesSchema` in `@ag-ui/core`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentCapabilities {
    /// Identity information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityCapabilities>,
    /// Transport capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportCapabilities>,
    /// Tools capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapabilities>,
    /// Output capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputCapabilities>,
    /// State capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<StateCapabilities>,
    /// Multi-agent capabilities.
    #[serde(rename = "multiAgent", skip_serializing_if = "Option::is_none")]
    pub multi_agent: Option<MultiAgentCapabilities>,
    /// Reasoning capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningCapabilities>,
    /// Multimodal capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multimodal: Option<MultimodalCapabilities>,
    /// Execution capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionCapabilities>,
    /// Human-in-the-loop capabilities.
    #[serde(rename = "humanInTheLoop", skip_serializing_if = "Option::is_none")]
    pub human_in_the_loop: Option<HumanInTheLoopCapabilities>,
    /// Custom escape hatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<JsonValue>,
}
