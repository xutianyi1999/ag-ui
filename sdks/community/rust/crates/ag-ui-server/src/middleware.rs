//! Composable event stream transformation middleware.
//!
//! This module provides the [`StreamTransformer`] trait for building reusable
//! event stream transformers, [`TransformerChain`] for composing multiple
//! transformers, and [`EventStreamExt`] for fluent chaining.
//!
//! # Example
//!
//! ```rust
//! use ag_ui_server::middleware::{StreamTransformer, TransformerChain, EventStreamExt};
//! use ag_ui_core::event::Event;
//! use ag_ui_server::AgentResult;
//! use futures::{stream, StreamExt};
//!
//! // A simple logging transformer
//! struct LoggingTransformer;
//!
//! impl StreamTransformer for LoggingTransformer {
//!     fn transform<'a>(
//!         &'a self,
//!         stream: BoxStream<'a, AgentResult<Event>>,
//!     ) -> BoxStream<'a, AgentResult<Event>> {
//!         Box::pin(stream.inspect(|result| {
//!             if let Ok(event) = result {
//!                 tracing::debug!("event: {:?}", event.event_type());
//!             }
//!         }))
//!     }
//! }
//! ```

use crate::error::AgentResult;
use ag_ui_core::event::Event;
use ag_ui_core::AgentState;
use futures::stream::BoxStream;
use futures::StreamExt;

/// A stream transformer that can modify, filter, or augment event streams.
///
/// Unlike a full middleware (which has agent-level knowledge), a
/// `StreamTransformer` operates purely on event streams, making it
/// simpler to compose and reuse.
pub trait StreamTransformer<StateT: AgentState = serde_json::Value>: Send + Sync {
    /// Transform an event stream into another event stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The input event stream to transform
    ///
    /// # Returns
    ///
    /// A transformed event stream.
    fn transform<'a>(
        &'a self,
        stream: BoxStream<'a, AgentResult<Event<StateT>>>,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>>;
}

/// Extension trait for chaining stream transformers on event streams.
pub trait EventStreamExt<'a, StateT: AgentState>: Sized {
    /// Apply a stream transformer to this stream.
    fn with_transformer<T: StreamTransformer<StateT>>(
        self,
        transformer: &'a T,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>>;
}

impl<'a, StateT: AgentState + 'a> EventStreamExt<'a, StateT>
    for BoxStream<'a, AgentResult<Event<StateT>>>
{
    fn with_transformer<T: StreamTransformer<StateT>>(
        self,
        transformer: &'a T,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>> {
        transformer.transform(self)
    }
}

/// A chain of stream transformers applied in sequence.
pub struct TransformerChain<StateT: AgentState = serde_json::Value> {
    transformers: Vec<Box<dyn StreamTransformer<StateT>>>,
}

impl<StateT: AgentState> Default for TransformerChain<StateT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<StateT: AgentState + 'static> TransformerChain<StateT> {
    /// Create a new empty transformer chain.
    pub fn new() -> Self {
        Self {
            transformers: Vec::new(),
        }
    }

    /// Add a transformer to the chain.
    ///
    /// Transformers are applied in the order they are added.
    pub fn push<T: StreamTransformer<StateT> + 'static>(mut self, transformer: T) -> Self {
        self.transformers.push(Box::new(transformer));
        self
    }

    /// Apply all transformers in the chain to an event stream (borrowed).
    pub fn apply<'a>(
        &'a self,
        mut stream: BoxStream<'a, AgentResult<Event<StateT>>>,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>> {
        for transformer in &self.transformers {
            stream = transformer.transform(stream);
        }
        stream
    }

    /// Apply all transformers in the chain to a 'static event stream by buffering.
    ///
    /// Collects the stream into a Vec, applies all transformers, and returns a
    /// new stream. This avoids lifetime issues with borrowed transformers at
    /// the cost of buffering all events in memory.
    pub async fn into_apply_buffered(
        self,
        stream: BoxStream<'static, AgentResult<Event<StateT>>>,
    ) -> Vec<AgentResult<Event<StateT>>> {
        let mut events: Vec<AgentResult<Event<StateT>>> = stream.collect().await;
        for transformer in &self.transformers {
            let buf: Vec<AgentResult<Event<StateT>>> = std::mem::take(&mut events);
            let stream: BoxStream<'static, AgentResult<Event<StateT>>> =
                Box::pin(futures::stream::iter(buf));
            events = transformer.transform(stream).collect().await;
        }
        events
    }
}

/// Middleware that logs every event passing through the stream.
///
/// Useful for debugging agent event flows.
pub struct LoggingTransformer {
    label: String,
}

impl LoggingTransformer {
    /// Create a new logging transformer with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl<StateT: AgentState> StreamTransformer<StateT> for LoggingTransformer {
    fn transform<'a>(
        &'a self,
        stream: BoxStream<'a, AgentResult<Event<StateT>>>,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>> {
        let label = self.label.clone();
        Box::pin(stream.inspect(move |result| {
            match result {
                Ok(event) => tracing::debug!("[{}] event: {:?}", label, event.event_type()),
                Err(e) => tracing::warn!("[{}] error: {}", label, e),
            }
        }))
    }
}

/// Middleware that filters tool calls based on allow/block lists.
///
/// This middleware intercepts tool call events and filters out calls to
/// tools that are not allowed (or are explicitly blocked). When a tool call
/// is filtered, all related events (ARGS, END, RESULT) are also filtered.
pub struct FilterToolCallsMiddleware {
    allowed: Option<std::collections::HashSet<String>>,
    blocked: std::collections::HashSet<String>,
    blocked_ids: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl FilterToolCallsMiddleware {
    /// Only allow the specified tools (all others are blocked).
    pub fn allow<I, S>(tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: Some(tools.into_iter().map(Into::into).collect()),
            blocked: std::collections::HashSet::new(),
            blocked_ids: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// Block the specified tools (all others are allowed).
    pub fn block<I, S>(tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: None,
            blocked: tools.into_iter().map(Into::into).collect(),
            blocked_ids: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    fn should_filter(&self, tool_name: &str) -> bool {
        match &self.allowed {
            Some(allowed) => !allowed.contains(tool_name),
            None => self.blocked.contains(tool_name),
        }
    }
}

impl<StateT: AgentState> StreamTransformer<StateT> for FilterToolCallsMiddleware {
    fn transform<'a>(
        &'a self,
        stream: BoxStream<'a, AgentResult<Event<StateT>>>,
    ) -> BoxStream<'a, AgentResult<Event<StateT>>> {
        Box::pin(stream.filter_map(move |result| {
            let blocked_ids = self.blocked_ids.lock().expect("mutex poisoned");
            let should_filter = match &result {
                Ok(Event::ToolCallStart(e)) => {
                    let filtered = self.should_filter(&e.tool_call_name);
                    if filtered {
                        drop(blocked_ids);
                        self.blocked_ids
                            .lock()
                            .expect("mutex poisoned")
                            .insert(e.tool_call_id.to_string());
                    }
                    filtered
                }
                Ok(Event::ToolCallArgs(e)) => blocked_ids.contains(&e.tool_call_id.to_string()),
                Ok(Event::ToolCallEnd(e)) => blocked_ids.contains(&e.tool_call_id.to_string()),
                Ok(Event::ToolCallResult(e)) => {
                    let was = blocked_ids.contains(&e.tool_call_id.to_string());
                    drop(blocked_ids);
                    if was {
                        self.blocked_ids
                            .lock()
                            .expect("mutex poisoned")
                            .remove(&e.tool_call_id.to_string());
                    }
                    was
                }
                Ok(Event::ToolCallChunk(e)) => e
                    .tool_call_id
                    .as_ref()
                    .map_or(false, |id| blocked_ids.contains(&id.to_string())),
                _ => false,
            };
            if should_filter {
                futures::future::ready(None)
            } else {
                futures::future::ready(Some(result))
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ag_ui_core::event::{
        BaseEvent, RunFinishedEvent, RunStartedEvent, TextMessageContentEvent,
        TextMessageEndEvent, TextMessageStartEvent, ToolCallArgsEvent, ToolCallEndEvent,
        ToolCallResultEvent, ToolCallStartEvent,
    };
    use ag_ui_core::types::{MessageId, Role, RunId, ThreadId, ToolCallId};
    use futures::stream::{self, StreamExt};

    fn base() -> BaseEvent {
        BaseEvent::default()
    }

    fn run_started() -> Event {
        Event::RunStarted(RunStartedEvent {
            base: base(),
            thread_id: ThreadId::new("test"),
            run_id: RunId::new("test"),
            parent_run_id: None,
            input: None,
        })
    }

    fn run_finished() -> Event {
        Event::RunFinished(RunFinishedEvent {
            base: base(),
            thread_id: ThreadId::new("test"),
            run_id: RunId::new("test"),
            result: None,
            outcome: None,
        })
    }

    fn tool_call_start(id: &str, name: &str) -> Event {
        Event::ToolCallStart(ToolCallStartEvent {
            base: base(),
            tool_call_id: ToolCallId::new(id),
            tool_call_name: name.to_string(),
            parent_message_id: None,
        })
    }

    fn tool_call_args(id: &str) -> Event {
        Event::ToolCallArgs(ToolCallArgsEvent {
            base: base(),
            tool_call_id: ToolCallId::new(id),
            delta: "{}".to_string(),
        })
    }

    fn tool_call_end(id: &str) -> Event {
        Event::ToolCallEnd(ToolCallEndEvent {
            base: base(),
            tool_call_id: ToolCallId::new(id),
        })
    }

    fn tool_call_result(id: &str) -> Event {
        Event::ToolCallResult(ToolCallResultEvent {
            base: base(),
            message_id: MessageId::new("msg1"),
            tool_call_id: ToolCallId::new(id),
            content: "result".to_string(),
            role: Role::Tool,
        })
    }

    #[tokio::test]
    async fn allowlist_filters_non_allowed_tools() {
        let middleware = FilterToolCallsMiddleware::allow(["allowed_tool"]);

        let events: Vec<AgentResult<Event>> = vec![
            Ok(run_started()),
            Ok(tool_call_start("tc1", "blocked_tool")),
            Ok(tool_call_args("tc1")),
            Ok(tool_call_end("tc1")),
            Ok(tool_call_result("tc1")),
            Ok(run_finished()),
        ];

        let stream = Box::pin(stream::iter(events));
        let transformed = middleware.transform(stream);
        let results: Vec<_> = transformed.collect().await;

        assert_eq!(results.len(), 2); // run_started + run_finished
    }

    #[tokio::test]
    async fn blocklist_filters_blocked_tools() {
        let middleware = FilterToolCallsMiddleware::block(["dangerous_tool"]);

        let events: Vec<AgentResult<Event>> = vec![
            Ok(run_started()),
            Ok(tool_call_start("tc1", "dangerous_tool")),
            Ok(tool_call_args("tc1")),
            Ok(tool_call_end("tc1")),
            Ok(run_finished()),
        ];

        let stream = Box::pin(stream::iter(events));
        let transformed = middleware.transform(stream);
        let results: Vec<_> = transformed.collect().await;

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn non_tool_events_pass_through() {
        let middleware = FilterToolCallsMiddleware::allow(["search"]);

        let events: Vec<AgentResult<Event>> = vec![
            Ok(run_started()),
            Ok(Event::TextMessageStart(TextMessageStartEvent {
                base: base(),
                message_id: MessageId::new("msg1"),
                role: Role::Assistant,
            name: None,
            })),
            Ok(Event::TextMessageContent(TextMessageContentEvent {
                base: base(),
                message_id: MessageId::new("msg1"),
                delta: "Hello".to_string(),
            })),
            Ok(Event::TextMessageEnd(TextMessageEndEvent {
                base: base(),
                message_id: MessageId::new("msg1"),
            })),
            Ok(run_finished()),
        ];

        let stream = Box::pin(stream::iter(events));
        let transformed = middleware.transform(stream);
        let results: Vec<_> = transformed.collect().await;

        assert_eq!(results.len(), 5);
    }

    #[test]
    fn transformer_chain_apply() {
        let chain = TransformerChain::new()
            .push(LoggingTransformer::new("test"))
            .push(FilterToolCallsMiddleware::allow(["allowed_tool"]));

        let events: Vec<AgentResult<Event>> = vec![Ok(run_started()), Ok(run_finished())];
        let stream = Box::pin(stream::iter(events));
        let _transformed = chain.apply(stream);
        // Just verifying it compiles and doesn't panic
    }
}
