#![doc = include_str!("../README.md")]

pub mod capabilities;
pub mod error;
pub mod event;
mod state;
pub mod types;

pub use error::{AGUIError, AGUIConnectNotImplementedError, AgUiError, Result};
pub use state::{AgentState, FwdProps};

/// Re-export to ensure the same type is used
pub use serde_json::Value as JsonValue;
