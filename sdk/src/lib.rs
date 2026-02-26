//! Signals SDK — reconnecting SSE client for signal streams.
//!
//! Provides a high-level client that subscribes to a signal bus via
//! Server-Sent Events. The client automatically reconnects with exponential
//! backoff and replays missed signals using `Last-Event-ID`.
//!
//! # Example
//!
//! ```rust,no_run
//! use signals_sdk::{AgentClient, AgentConfig, ReconnectConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AgentConfig {
//!     server_url: "https://tell.example.com".into(),
//!     token: "tell_eyJ...".into(),
//!     workspace_id: "ws_abc123".into(),
//!     kind_filter: Some("ip.*".into()),
//!     source_filter: None,
//!     reconnect: ReconnectConfig::default(),
//! };
//!
//! let client = AgentClient::new(config)?;
//! let mut stream = client.connect();
//!
//! while let Some(signal) = stream.next().await {
//!     println!("[{}] {} from {}", signal.kind, signal.payload, signal.source);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod config;
mod error;
mod sse;

pub use client::{AgentClient, SignalStream};
pub use config::{AgentConfig, ReconnectConfig};
pub use error::AgentError;
pub use signals_types::Signal;
