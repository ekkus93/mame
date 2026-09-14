impl ControlRequestHandle {
    fn wait_for_command(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        command: CommandName,
    ) -> AppResult<bool> {
        let first = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not acknowledge the request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the runtime-control request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": command.wire_name(),
                    "timeoutMs": CONTROL_RESPONSE_TIMEOUT.as_millis()
                })));
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                return Err(channel_state_error(state));
            }
            Err(ReceiveDeadlineError::Disconnected) => return Err(channel_closed_error()),
        };

        match first {
            CommandSignal::Response(response) => match response.status.as_str() {
                "completed" => extract_paused_result(
                    response.result.as_ref(),
                    command
                        .expected_paused()
                        .expect("pause/resume command has an expected pause state"),
                    request_id,
                ),
                "accepted" => {
                    if response
                        .result
                        .as_ref()
                        .is_some_and(|result| !result.is_empty())
                    {
                        return Err(protocol_output_error(
                            "An accepted pause/resume response contained an unexpected result payload.",
                        ));
                    }
                    self.wait_for_completion(receiver, request_id, command)
                }
                "rejected" => {
                    Err(response
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A rejected response omitted its error.")
                        }))
                }
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "A command completion arrived before its request response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }

    fn wait_for_completion(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        command: CommandName,
    ) -> AppResult<bool> {
        match recv_signal_until(&receiver, &self.state, PAUSE_RESUME_COMPLETION_TIMEOUT) {
            Ok(CommandSignal::Completion(completion)) => {
                if completion.ok {
                    extract_paused_result(
                        completion.result.as_ref(),
                        command
                            .expected_paused()
                            .expect("pause/resume command has an expected pause state"),
                        request_id,
                    )
                } else {
                    Err(completion
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A failed command completion omitted its error.")
                        }))
                }
            }
            Ok(CommandSignal::Response(_)) => Err(protocol_output_error(
                "The runtime-control peer emitted more than one response for a request.",
            )),
            Ok(CommandSignal::Closed) => Err(channel_closed_error()),
            Ok(CommandSignal::Failed(message)) => {
                Err(AppError::new("CONTROL_CHANNEL_FAILED", message))
            }
            Err(ReceiveDeadlineError::Timeout) => {
                abandon_pending(&self.runtime, request_id, command);
                Err(AppError::new(
                    "CONTROL_COMPLETION_TIMEOUT",
                    "MAME accepted the pause/resume command but did not confirm the state transition before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": command.wire_name(),
                    "timeoutMs": PAUSE_RESUME_COMPLETION_TIMEOUT.as_millis()
                })))
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                Err(channel_state_error(state))
            }
            Err(ReceiveDeadlineError::Disconnected) => Err(channel_closed_error()),
        }
    }

    fn wait_for_reset(&self, receiver: Receiver<CommandSignal>, request_id: &str) -> AppResult<()> {
        let command = CommandName::Reset;
        let first = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not acknowledge the reset request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the reset request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": command.wire_name(),
                    "timeoutMs": CONTROL_RESPONSE_TIMEOUT.as_millis()
                })));
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                return Err(channel_state_error(state));
            }
            Err(ReceiveDeadlineError::Disconnected) => return Err(channel_closed_error()),
        };

        match first {
            CommandSignal::Response(response) => match response.status.as_str() {
                "accepted" => {
                    if response
                        .result
                        .as_ref()
                        .is_some_and(|result| !result.is_empty())
                    {
                        return Err(protocol_output_error(
                            "An accepted reset response contained an unexpected result payload.",
                        ));
                    }
                    self.wait_for_reset_completion(receiver, request_id)
                }
                "completed" => Err(protocol_output_error(
                    "Soft reset cannot complete synchronously; notifier-backed completion is required.",
                )),
                "rejected" => Err(response
                    .error
                    .map(wire_error_to_app_error)
                    .unwrap_or_else(|| protocol_output_error("A rejected response omitted its error."))),
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "A reset completion arrived before its request response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }

    fn wait_for_reset_completion(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
    ) -> AppResult<()> {
        match recv_signal_until(&receiver, &self.state, RESET_COMPLETION_TIMEOUT) {
            Ok(CommandSignal::Completion(completion)) => {
                if completion.ok {
                    extract_reset_result(completion.result.as_ref(), request_id)
                } else {
                    Err(completion
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A failed reset completion omitted its error.")
                        }))
                }
            }
            Ok(CommandSignal::Response(_)) => Err(protocol_output_error(
                "The runtime-control peer emitted more than one response for a reset request.",
            )),
            Ok(CommandSignal::Closed) => Err(channel_closed_error()),
            Ok(CommandSignal::Failed(message)) => {
                Err(AppError::new("CONTROL_CHANNEL_FAILED", message))
            }
            Err(ReceiveDeadlineError::Timeout) => {
                abandon_pending(&self.runtime, request_id, CommandName::Reset);
                Err(AppError::new(
                    "CONTROL_COMPLETION_TIMEOUT",
                    "MAME accepted the soft reset but did not confirm reset completion before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "reset",
                    "kind": "soft",
                    "timeoutMs": RESET_COMPLETION_TIMEOUT.as_millis()
                })))
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                Err(channel_state_error(state))
            }
            Err(ReceiveDeadlineError::Disconnected) => Err(channel_closed_error()),
        }
    }
}
