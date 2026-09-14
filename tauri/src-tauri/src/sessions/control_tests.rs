#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{mpsc, Arc, Mutex},
    };

    use serde_json::Value;

    use super::{
        abandon_pending, base64url_decode, base64url_encode, begin_request, deliver_completion,
        deliver_response, generate_frame_token, notify_channel_closed, record_pause_state,
        record_reset, CommandCompletion, CommandName, CommandSignal, ControlBootstrap,
        ControlResponse, ControlRuntime, ControlStdoutParser, ParserEvent, FRAME_PREFIX,
        MAX_DECODED_MESSAGE_BYTES, TOKEN_REDACTION,
    };

    #[test]
    fn generated_tokens_are_32_random_bytes_in_unpadded_base64url() {
        let first = generate_frame_token().expect("first token");
        let second = generate_frame_token().expect("second token");
        assert_eq!(first.len(), 43);
        assert_eq!(base64url_decode(&first).expect("decode first").len(), 32);
        assert!(first
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'));
        assert_ne!(first, second);
    }

    #[test]
    fn base64url_round_trips_protocol_payloads_without_padding() {
        for payload in [b"".as_slice(), b"a", b"ab", b"abc", b"protocol-v1"] {
            let encoded = base64url_encode(payload);
            assert!(!encoded.contains('='));
            assert_eq!(base64url_decode(&encoded).expect("decode"), payload);
        }
    }

    #[test]
    fn bootstrap_is_private_and_advertises_pause_resume_and_soft_reset() {
        let bootstrap = ControlBootstrap::create("mame-123-1").expect("bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("emu.add_machine_pause_notifier"));
        assert!(script.contains("emu.add_machine_resume_notifier"));
        assert!(script.contains("pcall(emu.pause)"));
        assert!(script.contains("pcall(emu.unpause)"));
        assert!(script.contains("emu.add_machine_reset_notifier"));
        assert!(script.contains("manager.machine:soft_reset()"));
        assert!(!script.contains("hard_reset()"));
        assert!(script.contains("CONTROL_UNSUPPORTED_RESET_KIND"));
        let ready_frame = script
            .lines()
            .find_map(|line| {
                line.strip_prefix("local ready_frame = \"")?
                    .strip_suffix('"')
            })
            .expect("ready frame literal");
        let expected_prefix = format!("{FRAME_PREFIX}{}@@", bootstrap.frame_token());
        let encoded = ready_frame
            .strip_prefix(&expected_prefix)
            .expect("session token prefix");
        let decoded = base64url_decode(encoded).expect("ready base64url");
        let ready: Value = serde_json::from_slice(&decoded).expect("ready JSON");
        assert_eq!(ready["version"], 1);
        assert_eq!(ready["type"], "event");
        assert_eq!(ready["sessionId"], "mame-123-1");
        assert_eq!(ready["event"], "ready");
        assert_eq!(
            ready["payload"]["commands"],
            serde_json::json!(["pause", "resume", "reset"])
        );
        assert_eq!(
            ready["payload"]["maxMessageBytes"],
            MAX_DECODED_MESSAGE_BYTES
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(bootstrap.path())
                .expect("bootstrap metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0);
        }
    }

    #[test]
    fn parser_accepts_ready_response_state_and_completion_frames() {
        let token = "A".repeat(43);
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "ready",
            "payload": {"commands": ["pause", "resume"], "maxMessageBytes": 16384}
        });
        let accepted = serde_json::json!({
            "version": 1,
            "type": "response",
            "sessionId": "mame-1-1",
            "requestId": "req-1",
            "status": "accepted",
            "ok": true,
            "result": {}
        });
        let paused = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "paused",
            "requestId": "req-1",
            "payload": {"paused": true}
        });
        let completed = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "command_completed",
            "requestId": "req-1",
            "payload": {"command": "pause", "ok": true, "result": {"paused": true}}
        });
        let stream = [ready, accepted, paused, completed]
            .into_iter()
            .map(|message| protocol_frame(&token, &message))
            .collect::<String>();
        let split = stream.len() / 2;
        let mut parser =
            ControlStdoutParser::new_for_test("mame-1-1".to_owned(), token, runtime.clone());
        let first = parser.feed(&stream.as_bytes()[..split]);
        let second = parser.feed(&stream.as_bytes()[split..]);
        let events: Vec<_> = first.events.into_iter().chain(second.events).collect();
        assert_eq!(events, vec![ParserEvent::Ready]);
        assert!(matches!(
            receiver.recv().expect("accepted signal"),
            CommandSignal::Response(_)
        ));
        assert!(matches!(
            receiver.recv().expect("completion signal"),
            CommandSignal::Completion(_)
        ));
    }

    #[test]
    fn parser_accepts_notifier_backed_soft_reset_completion() {
        let token = "R".repeat(43);
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-reset".to_owned(), CommandName::Reset, sender)
            .expect("begin reset request");
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "ready",
            "payload": {"commands": ["pause", "resume", "reset"], "maxMessageBytes": 16384}
        });
        let accepted = serde_json::json!({
            "version": 1,
            "type": "response",
            "sessionId": "mame-1-1",
            "requestId": "req-reset",
            "status": "accepted",
            "ok": true,
            "result": {}
        });
        let reset = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "reset",
            "requestId": "req-reset",
            "payload": {"kind": "soft"}
        });
        let completed = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "command_completed",
            "requestId": "req-reset",
            "payload": {"command": "reset", "ok": true, "result": {"kind": "soft"}}
        });
        let stream = [ready, accepted, reset, completed]
            .into_iter()
            .map(|message| protocol_frame(&token, &message))
            .collect::<String>();
        let mut parser =
            ControlStdoutParser::new_for_test("mame-1-1".to_owned(), token, runtime.clone());
        let batch = parser.feed(stream.as_bytes());
        assert_eq!(batch.events, vec![ParserEvent::Ready]);
        assert!(matches!(
            receiver.recv().expect("accepted reset signal"),
            CommandSignal::Response(_)
        ));
        assert!(matches!(
            receiver.recv().expect("completed reset signal"),
            CommandSignal::Completion(_)
        ));
    }

    #[test]
    fn reset_completion_requires_reset_notifier_evidence() {
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-reset".to_owned(), CommandName::Reset, sender)
            .expect("begin reset request");
        deliver_response(
            &runtime,
            ControlResponse {
                request_id: "req-reset".to_owned(),
                status: "accepted".to_owned(),
                result: Some(serde_json::Map::new()),
                error: None,
            },
        )
        .expect("accepted reset response");
        assert!(matches!(
            receiver.recv().expect("accepted signal"),
            CommandSignal::Response(_)
        ));
        let completion = CommandCompletion {
            request_id: "req-reset".to_owned(),
            command: "reset".to_owned(),
            ok: true,
            result: Some(serde_json::Map::from_iter([(
                "kind".to_owned(),
                Value::String("soft".to_owned()),
            )])),
            error: None,
        };
        assert!(deliver_completion(&runtime, completion.clone()).is_err());
        record_reset(&runtime, "req-reset").expect("reset notifier evidence");
        deliver_completion(&runtime, completion).expect("completion after notifier");
        assert!(matches!(
            receiver.recv().expect("completion signal"),
            CommandSignal::Completion(_)
        ));
    }

    #[test]
    fn command_correlation_accepts_late_completion_after_local_timeout() {
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        let response = ControlResponse {
            request_id: "req-1".to_owned(),
            status: "accepted".to_owned(),
            result: Some(serde_json::Map::new()),
            error: None,
        };
        deliver_response(&runtime, response).expect("accepted response");
        assert!(matches!(
            receiver.recv().expect("accepted signal"),
            CommandSignal::Response(_)
        ));
        abandon_pending(&runtime, "req-1", CommandName::Pause);
        drop(receiver);
        record_pause_state(&runtime, true, Some("req-1")).expect("late pause state");
        let completion = CommandCompletion {
            request_id: "req-1".to_owned(),
            command: "pause".to_owned(),
            ok: true,
            result: Some(serde_json::Map::from_iter([(
                "paused".to_owned(),
                Value::Bool(true),
            )])),
            error: None,
        };
        deliver_completion(&runtime, completion).expect("late completion is correlation-safe");
    }

    #[test]
    fn pause_state_notifier_drives_registered_state_sink_until_disconnect() {
        let runtime = Arc::new(ControlRuntime::default());
        let observed = Arc::new(Mutex::new(Vec::new()));
        let observed_for_sink = observed.clone();
        *super::recover_lock(&runtime.pause_state_sink) = Some(Arc::new(move |paused| {
            super::recover_lock(&observed_for_sink).push(paused);
        }));

        record_pause_state(&runtime, true, None).expect("independent pause state");
        assert_eq!(*super::recover_lock(&observed), vec![true]);

        notify_channel_closed(&runtime);
        record_pause_state(&runtime, false, None).expect("late state remains parseable");
        assert_eq!(
            *super::recover_lock(&observed),
            vec![true],
            "disconnect must detach the application state-event sink"
        );
    }

    #[test]
    fn channel_close_resolves_outstanding_command_immediately() {
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        notify_channel_closed(&runtime);
        assert!(matches!(
            receiver.recv().expect("closed signal"),
            CommandSignal::Closed
        ));
    }

    #[test]
    fn wrong_token_is_diagnostic_but_current_token_is_redacted_from_console_echo() {
        let token = "B".repeat(43);
        let wrong = "C".repeat(43);
        let input = format!(
            "{FRAME_PREFIX}{wrong}@@garbage\nmame_tauri_control_v1(\"{token}\",\"payload\")\n"
        );
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token.clone());
        let mut batch = parser.feed(input.as_bytes());
        let finished = parser.finish();
        batch.diagnostics.extend(finished.diagnostics);
        let text = String::from_utf8(batch.diagnostics).expect("diagnostics utf8");
        assert!(text.contains(&wrong));
        assert!(!text.contains(&token));
        assert!(text.contains(std::str::from_utf8(TOKEN_REDACTION).expect("redaction utf8")));
        assert!(batch.events.is_empty());
    }

    #[test]
    fn authenticated_malformed_frame_fails_closed() {
        let token = "D".repeat(43);
        let input = format!("{FRAME_PREFIX}{token}@@not+base64\n");
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token);
        let batch = parser.feed(input.as_bytes());
        assert!(batch.diagnostics.is_empty());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    fn protocol_frame(token: &str, message: &Value) -> String {
        let encoded = base64url_encode(&serde_json::to_vec(message).expect("serialize frame"));
        format!("{FRAME_PREFIX}{token}@@{encoded}\n")
    }
}
