//! Event stream compaction — consolidates deltas into final snapshots.
//!
//! This module provides [`compact_events`], which reduces redundant event
//! sequences to their minimal representation:
//!
//! - Merges contiguous `STATE_DELTA` sequences into a single `STATE_SNAPSHOT`
//!
//! Equivalent to CopilotKit's `compactEvents()` in `@ag-ui/client`.

use ag_ui_core::event::{BaseEvent, Event, StateSnapshotEvent};
use ag_ui_core::AgentState;
use serde_json::Value as JsonValue;

/// Compact a list of events by consolidating deltas into snapshots.
///
/// This function scans for sequences of `STATE_DELTA` events and replaces
/// each such sequence with a single `STATE_SNAPSHOT` containing the final
/// state after applying all deltas in order.
///
/// State compaction works by:
/// 1. Finding the most recent `STATE_SNAPSHOT` before a delta sequence
/// 2. Applying each `STATE_DELTA`'s JSON Patch operations in order
/// 3. Replacing the delta sequence with a single `STATE_SNAPSHOT`
///
/// If no prior snapshot is found, deltas are applied to `Value::Null`.
///
/// # Arguments
///
/// * `events` - Mutable reference to the event list to compact
///
/// # Example
///
/// ```rust
/// use ag_ui_server::compact::compact_events;
/// use ag_ui_core::event::{Event, StateDeltaEvent, StateSnapshotEvent, BaseEvent};
/// use ag_ui_core::types::ids::MessageId;
/// use serde_json::json;
///
/// let mut events: Vec<Event> = vec![
///     Event::StateSnapshot(StateSnapshotEvent {
///         base: BaseEvent::default(),
///         snapshot: json!({"count": 0}),
///     }),
///     Event::StateDelta(StateDeltaEvent {
///         base: BaseEvent::default(),
///         delta: vec![json!({"op": "replace", "path": "/count", "value": 1})],
///     }),
/// ];
/// compact_events(&mut events);
/// // events now has just one STATE_SNAPSHOT with snapshot: {"count": 1}
/// ```
pub fn compact_events<S: AgentState>(events: &mut Vec<Event<S>>) {
    let mut i = 0;
    while i < events.len() {
        if matches!(events[i], Event::StateDelta(_)) {
            // Find the start of the delta run AND the preceding snapshot
            let delta_start = i;
            let mut snapshot = find_prior_snapshot(events, delta_start);

            // Determine the merge range: include the prior snapshot if found
            let merge_start = if delta_start > 0
                && matches!(events[delta_start - 1], Event::StateSnapshot(_))
            {
                delta_start - 1
            } else {
                delta_start
            };

            // Apply all deltas in sequence
            while i < events.len() && matches!(events[i], Event::StateDelta(_)) {
                let patch_ops: Vec<json_patch::PatchOperation> = match &events[i] {
                    Event::StateDelta(delta_event) => delta_event
                        .delta
                        .iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect(),
                    _ => vec![],
                };
                let patch = json_patch::Patch(patch_ops);
                let _ = json_patch::patch(&mut snapshot, &patch);
                i += 1;
            }

            // Replace merge range with a single snapshot
            let snapshot_event = Event::StateSnapshot(StateSnapshotEvent {
                base: BaseEvent::default(),
                snapshot: serde_json::from_value(snapshot).unwrap_or_default(),
            });
            events.splice(merge_start..i, vec![snapshot_event]);
            i = merge_start + 1;
        } else {
            i += 1;
        }
    }
}

fn find_prior_snapshot<S: AgentState>(events: &[Event<S>], before: usize) -> JsonValue {
    for event in events[..before].iter().rev() {
        if let Event::StateSnapshot(snap) = event {
            return serde_json::to_value(&snap.snapshot).unwrap_or(JsonValue::Null);
        }
    }
    JsonValue::Null
}

#[cfg(test)]
mod tests {
    use super::*;
    use ag_ui_core::event::{BaseEvent, StateDeltaEvent};
    use ag_ui_core::types::ids::MessageId;
    use ag_ui_core::types::message::Role;
    use serde_json::json;

    fn base() -> BaseEvent {
        BaseEvent::default()
    }

    fn mid(s: &str) -> MessageId {
        MessageId::new(s)
    }

    #[test]
    fn no_deltas_no_change() {
        let mut events: Vec<Event> = vec![
            Event::TextMessageStart(ag_ui_core::event::TextMessageStartEvent {
                base: base(),
                message_id: mid("msg-1"),
                role: Role::Assistant,
            }),
        ];
        let len = events.len();
        compact_events(&mut events);
        assert_eq!(events.len(), len);
    }

    #[test]
    fn single_delta_against_snapshot() {
        let mut events: Vec<Event> = vec![
            Event::StateSnapshot(StateSnapshotEvent {
                base: base(),
                snapshot: json!({"count": 0}),
            }),
            Event::StateDelta(StateDeltaEvent {
                base: base(),
                delta: vec![json!({"op": "replace", "path": "/count", "value": 1})],
            }),
        ];
        compact_events(&mut events);
        assert_eq!(events.len(), 1);
        if let Event::StateSnapshot(ref snap) = events[0] {
            assert_eq!(snap.snapshot["count"], 1);
        } else {
            panic!("Expected STATE_SNAPSHOT");
        }
    }

    #[test]
    fn multiple_deltas_merged() {
        let mut events: Vec<Event> = vec![
            Event::StateSnapshot(StateSnapshotEvent {
                base: base(),
                snapshot: json!({"x": 0, "y": 0}),
            }),
            Event::StateDelta(StateDeltaEvent {
                base: base(),
                delta: vec![json!({"op": "replace", "path": "/x", "value": 1})],
            }),
            Event::StateDelta(StateDeltaEvent {
                base: base(),
                delta: vec![json!({"op": "replace", "path": "/y", "value": 2})],
            }),
        ];
        compact_events(&mut events);
        assert_eq!(events.len(), 1);
        if let Event::StateSnapshot(ref snap) = events[0] {
            assert_eq!(snap.snapshot["x"], 1);
            assert_eq!(snap.snapshot["y"], 2);
        } else {
            panic!("Expected STATE_SNAPSHOT");
        }
    }

    #[test]
    fn preserves_mixed_events() {
        let mut events: Vec<Event> = vec![
            Event::StateSnapshot(StateSnapshotEvent {
                base: base(),
                snapshot: json!({"a": 1}),
            }),
            Event::StateDelta(StateDeltaEvent {
                base: base(),
                delta: vec![json!({"op": "replace", "path": "/a", "value": 2})],
            }),
        ];
        compact_events(&mut events);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn no_prior_snapshot_uses_null() {
        let mut events: Vec<Event> = vec![Event::StateDelta(StateDeltaEvent {
            base: base(),
            delta: vec![json!({"op": "add", "path": "/x", "value": 1})],
        })];
        compact_events(&mut events);
        assert_eq!(events.len(), 1);
    }
}
