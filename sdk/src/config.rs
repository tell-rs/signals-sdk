//! Configuration for the Signals SDK client.

use std::time::Duration;

/// Top-level configuration for connecting to a signal bus.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Server URL (e.g. `https://tell.example.com`)
    pub server_url: String,

    /// Authentication token
    pub token: String,

    /// Workspace to subscribe to
    pub workspace_id: String,

    /// Optional signal kind filter (e.g. `ip.*`)
    pub kind_filter: Option<String>,

    /// Optional signal source filter (e.g. `transform:jail:*`)
    pub source_filter: Option<String>,

    /// Reconnection behavior
    pub reconnect: ReconnectConfig,
}

/// Controls exponential backoff on SSE reconnection.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Delay before the first reconnect attempt.
    pub initial_delay: Duration,

    /// Upper bound on reconnect delay.
    pub max_delay: Duration,

    /// Multiplier applied after each failed attempt.
    pub backoff_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2.0,
        }
    }
}
