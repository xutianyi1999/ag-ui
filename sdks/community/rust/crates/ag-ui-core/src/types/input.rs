use crate::JsonValue;
use crate::types::context::Context;
use crate::types::ids::{RunId, ThreadId};
use crate::types::interrupt::ResumeEntry;
use crate::types::message::Message;
use crate::types::tool::Tool;
use serde::{Deserialize, Serialize};

/// Input for running an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunAgentInput<StateT = JsonValue, FwdPropsT = JsonValue> {
    #[serde(rename = "threadId")]
    pub thread_id: ThreadId,
    #[serde(rename = "runId")]
    pub run_id: RunId,
    #[serde(rename = "parentRunId", skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    pub state: StateT,
    pub messages: Vec<Message>,
    pub tools: Vec<Tool>,
    pub context: Vec<Context>,
    #[serde(rename = "forwardedProps")]
    pub forwarded_props: FwdPropsT,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<Vec<ResumeEntry>>,
}

impl<StateT, FwdPropsT> RunAgentInput<StateT, FwdPropsT> {
    pub fn new(
        thread_id: impl Into<ThreadId>,
        run_id: impl Into<RunId>,
        state: StateT,
        messages: Vec<Message>,
        tools: Vec<Tool>,
        context: Vec<Context>,
        forwarded_props: FwdPropsT,
    ) -> Self {
        Self {
            thread_id: thread_id.into(),
            run_id: run_id.into(),
            parent_run_id: None,
            state,
            messages,
            tools,
            context,
            forwarded_props,
            resume: None,
        }
    }
}
