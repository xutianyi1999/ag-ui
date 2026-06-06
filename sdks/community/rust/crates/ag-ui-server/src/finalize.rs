//! Event stream finalization — closes unpaired events and enforces terminal events.
//!
//! This module provides [`finalize_run_events`], which ensures an event stream
//! is valid per the AG-UI protocol by:
//!
//! - Closing unpaired `TEXT_MESSAGE_START` events with `TEXT_MESSAGE_END`
//! - Closing unpaired `TOOL_CALL_START` events with `TOOL_CALL_END` and stub `TOOL_CALL_RESULT`
//! - Adding `RUN_FINISHED` if the stream has no terminal event
//!
//! Equivalent to CopilotKit's `finalizeRunEvents()` in `@copilotkit/shared`.

use ag_ui_core::event::{
    BaseEvent, Event, RunFinishedEvent, TextMessageEndEvent, ToolCallEndEvent,
    ToolCallResultEvent,
};
use ag_ui_core::types::ids::{MessageId, RunId, ThreadId, ToolCallId};
use ag_ui_core::types::message::Role;
use ag_ui_core::AgentState;
use std::collections::HashSet;

/// Finalize a list of events to ensure protocol compliance.
///
/// Scans the event list and:
/// 1. Finds unclosed `TEXT_MESSAGE_START` events → appends `TEXT_MESSAGE_END`
/// 2. Finds unclosed `TOOL_CALL_START` events → appends `TOOL_CALL_END` + `TOOL_CALL_RESULT`
/// 3. If no `RUN_FINISHED` or `RUN_ERROR` is present → appends `RUN_FINISHED`
///
/// # Arguments
///
/// * `events` - Mutable reference to the event list to finalize
/// * `thread_id` - Thread ID for the generated terminal events
/// * `run_id` - Run ID for the generated terminal events
///
/// # Example
///
/// ```rust
/// use ag_ui_server::finalize::finalize_run_events;
/// use ag_ui_core::types::{RunId, ThreadId};
///
/// let mut events = vec![];
/// finalize_run_events(&mut events, &ThreadId::new("t1"), &RunId::new("r1"));
/// // events now has RUN_FINISHED appended
/// ```
pub fn finalize_run_events<S: AgentState>(
    events: &mut Vec<Event<S>>,
    thread_id: &ThreadId,
    run_id: &RunId,
) {
    let mut active_messages: HashSet<MessageId> = HashSet::new();
    let mut active_tool_calls: HashSet<ToolCallId> = HashSet::new();
    let mut has_terminal = false;

    // Scan existing events to track state
    for event in events.iter() {
        match event {
            Event::TextMessageStart(e) => {
                active_messages.insert(e.message_id.clone());
            }
            Event::TextMessageEnd(e) => {
                active_messages.remove(&e.message_id);
            }
            Event::ToolCallStart(e) => {
                active_tool_calls.insert(e.tool_call_id.clone());
            }
            Event::ToolCallEnd(e) => {
                active_tool_calls.remove(&e.tool_call_id);
            }
            Event::ToolCallResult(e) => {
                active_tool_calls.remove(&e.tool_call_id);
            }
            Event::RunFinished(_) | Event::RunError(_) => {
                has_terminal = true;
            }
            _ => {}
        }
    }

    // Close unpaired TEXT_MESSAGE_START events
    for mid in active_messages {
        events.push(Event::TextMessageEnd(TextMessageEndEvent {
            base: BaseEvent::default(),
            message_id: mid,
        }));
    }

    // Close unpaired TOOL_CALL_START events
    for tcid in active_tool_calls {
        events.push(Event::ToolCallEnd(ToolCallEndEvent {
            base: BaseEvent::default(),
            tool_call_id: tcid.clone(),
        }));
        let result_msg_id = MessageId::random();
        events.push(Event::ToolCallResult(ToolCallResultEvent {
            base: BaseEvent::default(),
            message_id: result_msg_id,
            tool_call_id: tcid,
            content: String::new(),
            role: Role::Tool,
        }));
    }

    // Add RUN_FINISHED if no terminal event found
    if !has_terminal {
        events.push(Event::RunFinished(RunFinishedEvent {
            base: BaseEvent::default(),
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            result: None,
            outcome: None,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ag_ui_core::event::{
        BaseEvent, RunErrorEvent, TextMessageEndEvent, TextMessageStartEvent, ToolCallStartEvent,
    };
    use ag_ui_core::types::{MessageId, ToolCallId};
    use serde_json::Value as JsonValue;

    fn base() -> BaseEvent {
        BaseEvent {
            timestamp: None,
            raw_event: None,
        }
    }

    #[test]
    fn adds_run_finished_to_empty_stream() {
        let mut events: Vec<Event<JsonValue>> = vec![];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], Event::RunFinished(_)));
    }

    #[test]
    fn closes_unpaired_text_message() {
        let mid = MessageId::new("msg-1");
        let mut events: Vec<Event<JsonValue>> = vec![Event::TextMessageStart(
            TextMessageStartEvent {
                base: base(),
                message_id: mid.clone(),
                role: Role::Assistant,
            name: None,
            },
        )];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        // Should have: TEXT_MESSAGE_START, TEXT_MESSAGE_END, RUN_FINISHED
        assert_eq!(events.len(), 3);
        assert!(matches!(events[1], Event::TextMessageEnd(_)));
        assert!(matches!(events[2], Event::RunFinished(_)));
    }

    #[test]
    fn closes_unpaired_tool_call() {
        let tcid = ToolCallId::new("tc-1");
        let mut events: Vec<Event<JsonValue>> = vec![Event::ToolCallStart(
            ToolCallStartEvent {
                base: base(),
                tool_call_id: tcid.clone(),
                tool_call_name: "test_tool".to_string(),
                parent_message_id: None,
            },
        )];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        // Should have: TOOL_CALL_START, TOOL_CALL_END, TOOL_CALL_RESULT, RUN_FINISHED
        assert_eq!(events.len(), 4);
        assert!(matches!(events[1], Event::ToolCallEnd(_)));
        assert!(matches!(events[2], Event::ToolCallResult(_)));
        assert!(matches!(events[3], Event::RunFinished(_)));
    }

    #[test]
    fn preserves_existing_run_finished() {
        let mut events: Vec<Event<JsonValue>> = vec![Event::RunFinished(RunFinishedEvent {
            base: base(),
            thread_id: ThreadId::new("t1"),
            run_id: RunId::new("r1"),
            result: None,
            outcome: None,
        })];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn preserves_existing_run_error() {
        let mut events: Vec<Event<JsonValue>> = vec![Event::RunError(RunErrorEvent {
            base: base(),
            message: "oops".to_string(),
            code: None,
        })];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn closes_paired_message_and_skips() {
        let mid = MessageId::new("msg-1");
        let mut events: Vec<Event<JsonValue>> = vec![
            Event::TextMessageStart(TextMessageStartEvent {
                base: base(),
                message_id: mid.clone(),
                role: Role::Assistant,
            name: None,
            }),
            Event::TextMessageEnd(TextMessageEndEvent {
                base: base(),
                message_id: mid,
            }),
        ];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        assert_eq!(events.len(), 3);
        assert!(matches!(events[2], Event::RunFinished(_)));
    }

    #[test]
    fn mixed_unpaired_events() {
        let mid = MessageId::new("msg-1");
        let tcid = ToolCallId::new("tc-1");
        let mut events: Vec<Event<JsonValue>> = vec![
            Event::TextMessageStart(TextMessageStartEvent {
                base: base(),
                message_id: mid.clone(),
                role: Role::Assistant,
            name: None,
            }),
            Event::ToolCallStart(ToolCallStartEvent {
                base: base(),
                tool_call_id: tcid.clone(),
                tool_call_name: "tool_a".to_string(),
                parent_message_id: None,
            }),
        ];
        let tid = ThreadId::new("t1");
        let rid = RunId::new("r1");
        finalize_run_events(&mut events, &tid, &rid);
        // START, START, TEXT_END, TOOL_END, TOOL_RESULT, RUN_FINISHED
        assert_eq!(events.len(), 6);
    }
}
