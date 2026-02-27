# Signals SDK

Reconnecting SSE client for [Tell](https://github.com/tell-rs) signal streams.

## Crates

| Crate | Description |
|-------|-------------|
| [`signals-types`](./types) | Signal types — lightweight, serde-only extraction boundary |
| [`signals-sdk`](./sdk) | SSE client with auto-reconnect, backoff, and `Last-Event-ID` replay |

## Quick start

```rust
use signals_sdk::{AgentClient, AgentConfig, ReconnectConfig};

let config = AgentConfig {
    server_url: "https://tell.example.com".into(),
    token: "tell_eyJ...".into(),
    workspace_id: "ws_abc123".into(),
    kind_filter: Some("ip.*".into()),
    source_filter: None,
    reconnect: ReconnectConfig::default(),
};

let client = AgentClient::new(config)?;
let mut stream = client.connect();

while let Some(signal) = stream.next().await {
    println!("[{}] {} from {}", signal.kind, signal.payload, signal.source);
}
```

## License

MIT
