#[cfg(test)]
mod mt711_adversarial_tests {
    use std::{
        fs,
        sync::{mpsc, Arc, Mutex},
        time::Duration,
    };

    use serde_json::Value;

    use super::{
        base64url_encode, begin_request, notify_channel_closed, recv_signal_until, CommandName,
        CommandSignal, ControlBootstrap, ControlChannelState, ControlRuntime, ControlStdoutParser,
        ParserEvent, ReceiveDeadlineError, FRAME_PREFIX, MAX_DECODED_MESSAGE_BYTES,
        MAX_ENCODED_LINE_BYTES,
    };

    fn protocol_frame(token: &str, message: &Value) -> String {
        let encoded = base64url_encode(&serde_json::to_vec(message).expect("serialize frame"));
        format!("{FRAME_PREFIX}{token}@@{encoded}\n")
    }

    fn parser_with_runtime(token: &str) -> ControlStdoutParser {
        ControlStdoutParser::new_for_test(
            "mame-live-1".to_owned(),
            token.to_owned(),
            Arc::new(ControlRuntime::default()),
        )
    }

    #[test]
    fn authenticated_malformed_json_fails_closed() {
        let token = "M".repeat(43);
        let encoded = base64url_encode(b"{not-json");
        let input = format!("{FRAME_PREFIX}{token}@@{encoded}\n");
        let mut parser = parser_with_runtime(&token);
        let batch = parser.feed(input.as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
        assert!(batch.diagnostics.is_empty());
    }

    #[test]
    fn decoded_oversized_authenticated_payload_fails_closed() {
        let token = "O".repeat(43);
        let payload = vec![b'x'; MAX_DECODED_MESSAGE_BYTES + 1];
        let encoded = base64url_encode(&payload);
        assert!(encoded.len() + FRAME_PREFIX.len() + token.len() + 3 <= MAX_ENCODED_LINE_BYTES);
        let input = format!("{FRAME_PREFIX}{token}@@{encoded}\n");
        let mut parser = parser_with_runtime(&token);
        let batch = parser.feed(input.as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    #[test]
    fn encoded_oversized_authenticated_line_fails_closed_before_parse() {
        let token = "L".repeat(43);
        let fixed = FRAME_PREFIX.len() + token.len() + 3;
        let encoded = "A".repeat(MAX_ENCODED_LINE_BYTES - fixed + 1);
        let input = format!("{FRAME_PREFIX}{token}@@{encoded}\n");
        let mut parser = parser_with_runtime(&token);
        let batch = parser.feed(input.as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    #[test]
    fn wrong_protocol_version_ready_frame_is_rejected() {
        let token = "V".repeat(43);
        let ready = serde_json::json!({
            "version": 2,
            "type": "event",
            "sessionId": "mame-live-1",
            "event": "ready",
            "payload": {"commands": ["pause"], "maxMessageBytes": 16384}
        });
        let mut parser = parser_with_runtime(&token);
        let batch = parser.feed(protocol_frame(&token, &ready).as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    #[test]
    fn stale_session_ready_frame_is_rejected() {
        let token = "S".repeat(43);
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-stale-0",
            "event": "ready",
            "payload": {"commands": ["pause"], "maxMessageBytes": 16384}
        });
        let mut parser = parser_with_runtime(&token);
        let batch = parser.feed(protocol_frame(&token, &ready).as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    #[test]
    fn unknown_peer_command_completion_is_rejected() {
        let token = "U".repeat(43);
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-live-1",
            "event": "ready",
            "payload": {"commands": ["pause"], "maxMessageBytes": 16384}
        });
        let unknown = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-live-1",
            "event": "command_completed",
            "requestId": "req-unknown",
            "payload": {"command": "not_a_protocol_command", "ok": true, "result": {}}
        });
        let mut parser = parser_with_runtime(&token);
        let ready_batch = parser.feed(protocol_frame(&token, &ready).as_bytes());
        assert_eq!(ready_batch.events, vec![ParserEvent::Ready]);
        let batch = parser.feed(protocol_frame(&token, &unknown).as_bytes());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    #[test]
    fn generated_lua_rejects_unknown_version_stale_session_and_oversize_requests() {
        let bootstrap = ControlBootstrap::create("mame-711-1").expect("bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        for code in [
            "PROTOCOL_UNKNOWN_COMMAND",
            "PROTOCOL_UNSUPPORTED_VERSION",
            "PROTOCOL_SESSION_MISMATCH",
            "PROTOCOL_MESSAGE_TOO_LARGE",
        ] {
            assert!(script.contains(code), "generated shim must contain {code}");
        }
        assert!(script.contains("not known_commands[request.command]"));
        assert!(script.contains("request.version ~= 1"));
        assert!(script.contains("request.sessionId ~= session_id"));
    }

    #[test]
    fn connection_drop_mid_command_resolves_waiter_without_deadline_wait() {
        let runtime = Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-drop".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        notify_channel_closed(&runtime);
        assert!(matches!(
            receiver.recv().expect("closed signal"),
            CommandSignal::Closed
        ));
    }

    #[test]
    fn response_deadline_reports_timeout_when_peer_stays_silent() {
        let (_sender, receiver) = mpsc::channel::<CommandSignal>();
        let state = Arc::new(Mutex::new(ControlChannelState::Ready));
        let outcome = recv_signal_until(&receiver, &state, Duration::from_millis(5));
        assert!(matches!(outcome, Err(ReceiveDeadlineError::Timeout)));
    }
}
