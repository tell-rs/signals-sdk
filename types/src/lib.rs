//! Signal types — lightweight extraction boundary for signals.
//!
//! This crate contains only the `Signal` struct and its serde impls.
//! It exists so that external consumers (e.g. `signals-sdk`) can depend on
//! signal types without pulling in the full signal bus machinery.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A typed signal emitted by a pipeline component.
///
/// Signals are lightweight notifications that answer:
/// - **what** happened (`kind` — dot-notation topic)
/// - **who** reported it (`source` — colon-separated producer identity)
/// - **when** it happened (`ts` — unix milliseconds)
/// - **detail** (`payload` — freeform JSON, <4KB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Monotonic signal ID, assigned by the bus on emit.
    /// Used for Last-Event-ID replay on SSE reconnect.
    #[serde(default)]
    pub id: u64,

    /// Workspace this signal belongs to
    pub workspace_id: String,

    /// Semantic topic: what happened (e.g. `ip.banned`, `threshold.crossed`)
    pub kind: String,

    /// Producer identity: who reported it (e.g. `transform:jail:sshd`)
    pub source: String,

    /// Unix milliseconds timestamp
    pub ts: u64,

    /// Structured detail (<4KB)
    pub payload: Value,
}

impl Signal {
    /// Create a new signal with the current timestamp.
    pub fn new(
        workspace_id: impl Into<String>,
        kind: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: 0,
            workspace_id: workspace_id.into(),
            kind: kind.into(),
            source: source.into(),
            ts: now,
            payload: Value::Null,
        }
    }

    /// Attach a JSON payload.
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }
}
