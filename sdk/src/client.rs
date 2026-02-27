//! Reconnecting SSE client for signal streams.

use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Url;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::sse::SseParser;
use signals_types::Signal;

/// Channel capacity for the internal signal buffer.
const CHANNEL_CAPACITY: usize = 256;

/// SSE client that maintains a persistent connection to a signal bus.
///
/// On disconnect, automatically reconnects with exponential backoff and
/// replays missed signals via `Last-Event-ID`.
pub struct AgentClient {
    config: Arc<AgentConfig>,
    http: reqwest::Client,
}

/// Receiver end of the signal stream.
///
/// Yields `Signal` values as they arrive from the server.
/// Returns `None` when the background connection task has terminated.
pub struct SignalStream {
    rx: mpsc::Receiver<Signal>,
}

impl AgentClient {
    /// Create a new agent client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns `AgentError::Config` if the server URL or token is empty.
    pub fn new(config: AgentConfig) -> Result<Self, AgentError> {
        validate_config(&config)?;

        // SSE connections are long-lived — no timeout
        let http = reqwest::Client::builder()
            .build()
            .map_err(AgentError::Http)?;

        Ok(Self {
            config: Arc::new(config),
            http,
        })
    }

    /// Connect to the signal bus and return a stream of signals.
    ///
    /// Spawns a background tokio task that handles reconnection with
    /// exponential backoff. On disconnect, the task reconnects with
    /// `Last-Event-ID` to replay missed signals.
    pub fn connect(&self) -> SignalStream {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        let config = Arc::clone(&self.config);
        let http = self.http.clone();

        tokio::spawn(connection_loop(config, http, tx));

        SignalStream { rx }
    }
}

impl SignalStream {
    /// Receive the next signal from the stream.
    ///
    /// Returns `None` when the background task has stopped (server
    /// permanently unreachable or task cancelled).
    pub async fn next(&mut self) -> Option<Signal> {
        self.rx.recv().await
    }
}

/// Build the SSE endpoint URL with optional query filters.
fn build_url(config: &AgentConfig) -> String {
    let base = config.server_url.trim_end_matches('/');
    let endpoint = format!("{base}/api/v1/signals");

    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref kind) = config.kind_filter {
        params.push(("kind", kind));
    }
    if let Some(ref source) = config.source_filter {
        params.push(("source", source));
    }

    if params.is_empty() {
        return endpoint;
    }

    // Use Url to properly percent-encode query parameters
    let mut url = Url::parse(&endpoint).expect("build_url: server_url produced an invalid URL");
    {
        let mut query = url.query_pairs_mut();
        for (key, value) in &params {
            query.append_pair(key, value);
        }
    }
    url.to_string()
}

/// Validate that required fields are present.
fn validate_config(config: &AgentConfig) -> Result<(), AgentError> {
    if config.server_url.is_empty() {
        return Err(AgentError::Config("server_url must not be empty".into()));
    }
    if config.token.is_empty() {
        return Err(AgentError::Config("token must not be empty".into()));
    }
    if config.workspace_id.is_empty() {
        return Err(AgentError::Config("workspace_id must not be empty".into()));
    }
    Ok(())
}

/// Background task: connect, stream, reconnect on failure.
async fn connection_loop(
    config: Arc<AgentConfig>,
    http: reqwest::Client,
    tx: mpsc::Sender<Signal>,
) {
    let url = build_url(&config);
    let mut last_event_id: Option<String> = None;
    let mut delay = config.reconnect.initial_delay;

    loop {
        match stream_signals(&http, &url, &config, &last_event_id, &tx).await {
            Ok(latest_id) => {
                if let Some(id) = latest_id {
                    last_event_id = Some(id);
                }
                // Clean disconnect — reset backoff
                delay = config.reconnect.initial_delay;
                info!("SSE stream ended, reconnecting");
            }
            Err(AgentError::AuthFailed { status }) => {
                warn!("authentication failed ({status}), stopping reconnect loop");
                return;
            }
            Err(e) => {
                warn!("SSE connection error: {e}, retrying in {delay:?}");
            }
        }

        // Check if the receiver has been dropped
        if tx.is_closed() {
            return;
        }

        tokio::time::sleep(delay).await;
        delay = next_delay(delay, &config.reconnect);
    }
}

/// Open a single SSE connection and stream signals until it ends or errors.
///
/// Returns the last seen event ID on clean stream termination.
async fn stream_signals(
    http: &reqwest::Client,
    url: &str,
    config: &AgentConfig,
    last_event_id: &Option<String>,
    tx: &mpsc::Sender<Signal>,
) -> Result<Option<String>, AgentError> {
    let mut request = http
        .get(url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("X-Workspace-ID", &config.workspace_id)
        .header("Accept", "text/event-stream");

    if let Some(id) = last_event_id {
        request = request.header("Last-Event-ID", id);
    }

    let response = request.send().await?;

    let status = response.status().as_u16();
    if status == 401 || status == 403 {
        return Err(AgentError::AuthFailed { status });
    }
    if !response.status().is_success() {
        return Err(AgentError::ServerError { status });
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut parser = SseParser::new();
    let mut latest_id: Option<String> = None;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer.drain(..=pos);

            if let Some(event) = parser.feed_line(&line) {
                if let Some(id) = event.id {
                    latest_id = Some(id);
                }
                if let Some(ref data) = event.data {
                    match serde_json::from_str::<Signal>(data) {
                        Ok(signal) => {
                            // If the receiver is gone, stop streaming
                            if tx.send(signal).await.is_err() {
                                return Ok(latest_id);
                            }
                        }
                        Err(e) => {
                            warn!("failed to deserialize signal: {e}, data: {data}");
                        }
                    }
                }
            }
        }
    }

    Ok(latest_id)
}

/// Compute the next backoff delay, capped at `max_delay`.
fn next_delay(current: Duration, reconnect: &crate::config::ReconnectConfig) -> Duration {
    let next = current.mul_f64(reconnect.backoff_factor);
    next.min(reconnect.max_delay)
}
