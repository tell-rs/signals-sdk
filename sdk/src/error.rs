//! Error types for the Signals SDK.

use thiserror::Error;

/// Errors returned by the agent client.
#[derive(Debug, Error)]
pub enum AgentError {
    /// HTTP transport error (connection refused, DNS failure, etc.)
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// SSE stream ended without a reconnectable error.
    #[error("SSE stream ended unexpectedly")]
    StreamEnded,

    /// Server rejected the authentication credentials.
    #[error("authentication failed: {status}")]
    AuthFailed { status: u16 },

    /// Configuration is invalid (empty URL, missing token, etc.)
    #[error("invalid configuration: {0}")]
    Config(String),
}
