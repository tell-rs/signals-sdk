//! Error types for the Signals SDK.

use thiserror::Error;

/// Errors returned by the agent client.
#[derive(Debug, Error)]
pub enum AgentError {
    /// HTTP transport error (connection refused, DNS failure, etc.)
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Server returned a non-2xx, non-auth error (500, 502, 503, 429, etc.)
    #[error("server error: HTTP {status}")]
    ServerError { status: u16 },

    /// Server rejected the authentication credentials (401 or 403).
    #[error("authentication failed: {status}")]
    AuthFailed { status: u16 },

    /// Configuration is invalid (empty URL, missing token, etc.)
    #[error("invalid configuration: {0}")]
    Config(String),
}
