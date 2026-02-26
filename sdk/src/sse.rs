//! Minimal SSE line parser for signal events.
//!
//! Handles the subset of the Server-Sent Events spec needed for Tell:
//! `event:`, `data:`, `id:` fields plus empty-line event boundaries
//! and `:` comment keep-alives.

/// A single parsed SSE event.
pub(crate) struct SseEvent {
    pub event_type: Option<String>,
    pub data: Option<String>,
    pub id: Option<String>,
}

impl SseEvent {
    fn new() -> Self {
        Self {
            event_type: None,
            data: None,
            id: None,
        }
    }
}

/// Incremental SSE parser that accumulates fields until an empty-line boundary.
pub(crate) struct SseParser {
    current: SseEvent,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            current: SseEvent::new(),
        }
    }

    /// Feed a single line (without trailing newline).
    ///
    /// Returns `Some(event)` when an empty line signals event completion.
    /// Comment lines (`:` prefix) and unknown fields are silently ignored.
    pub fn feed_line(&mut self, line: &str) -> Option<SseEvent> {
        let trimmed = line.trim_end_matches('\r');

        // Empty line = event boundary
        if trimmed.is_empty() {
            // Only yield if we accumulated at least a data field
            if self.current.data.is_some() {
                let event = std::mem::replace(&mut self.current, SseEvent::new());
                return Some(event);
            }
            return None;
        }

        // Comment / keep-alive
        if trimmed.starts_with(':') {
            return None;
        }

        // Split on first ':'
        let (field, value) = match trimmed.find(':') {
            Some(pos) => {
                let value = trimmed[pos + 1..].trim_start();
                (&trimmed[..pos], value)
            }
            None => (trimmed, ""),
        };

        match field {
            "event" => self.current.event_type = Some(value.to_owned()),
            "data" => self.current.data = Some(value.to_owned()),
            "id" => self.current.id = Some(value.to_owned()),
            _ => {} // retry, etc. — ignored
        }

        None
    }
}

#[cfg(test)]
#[path = "sse_test.rs"]
mod tests;
