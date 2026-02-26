use super::*;

#[test]
fn parses_complete_event() {
    let mut parser = SseParser::new();
    assert!(parser.feed_line("event: signal").is_none());
    assert!(parser.feed_line("id: 42").is_none());
    assert!(parser.feed_line("data: {\"kind\":\"ip.banned\"}").is_none());

    let event = parser.feed_line("").expect("empty line should yield event");
    assert_eq!(event.event_type.as_deref(), Some("signal"));
    assert_eq!(event.id.as_deref(), Some("42"));
    assert_eq!(event.data.as_deref(), Some("{\"kind\":\"ip.banned\"}"));
}

#[test]
fn ignores_comments() {
    let mut parser = SseParser::new();
    assert!(parser.feed_line(": keep-alive").is_none());
    assert!(parser.feed_line("data: hello").is_none());
    let event = parser.feed_line("").expect("should yield event");
    assert_eq!(event.data.as_deref(), Some("hello"));
}

#[test]
fn empty_line_without_data_yields_nothing() {
    let mut parser = SseParser::new();
    assert!(parser.feed_line("event: signal").is_none());
    // Empty line but no data field accumulated
    assert!(parser.feed_line("").is_none());
}

#[test]
fn handles_carriage_return() {
    let mut parser = SseParser::new();
    assert!(parser.feed_line("data: payload\r").is_none());
    let event = parser.feed_line("").expect("should yield event");
    assert_eq!(event.data.as_deref(), Some("payload"));
}

#[test]
fn consecutive_events() {
    let mut parser = SseParser::new();

    parser.feed_line("data: first");
    let e1 = parser.feed_line("").expect("first event");
    assert_eq!(e1.data.as_deref(), Some("first"));

    parser.feed_line("data: second");
    let e2 = parser.feed_line("").expect("second event");
    assert_eq!(e2.data.as_deref(), Some("second"));
    // First event state should not leak into second
    assert!(e2.event_type.is_none());
}

#[test]
fn field_without_colon() {
    let mut parser = SseParser::new();
    // A line with no colon — treated as a field with empty value (spec-compliant)
    assert!(parser.feed_line("data").is_none());
    // Parser stores empty string as data
    let event = parser.feed_line("").expect("should yield event");
    assert_eq!(event.data.as_deref(), Some(""));
}
