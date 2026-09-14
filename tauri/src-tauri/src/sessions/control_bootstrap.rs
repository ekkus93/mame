pub(super) struct ControlBootstrap {
    file: NamedTempFile,
    frame_token: String,
}

impl ControlBootstrap {
    pub(super) fn create(session_id: &str) -> AppResult<Self> {
        let frame_token = generate_frame_token()?;
        let ready = ReadyEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "event".to_owned(),
            session_id: session_id.to_owned(),
            event: "ready".to_owned(),
            payload: ReadyPayload {
                commands: MT705_COMMANDS
                    .iter()
                    .map(|command| (*command).to_owned())
                    .collect(),
                max_message_bytes: MAX_DECODED_MESSAGE_BYTES,
            },
        };
        let ready_json = serde_json::to_vec(&ready).map_err(|error| {
            AppError::new(
                "CONTROL_BOOTSTRAP_SERIALIZE_FAILED",
                "The runtime-control ready message could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if ready_json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "CONTROL_BOOTSTRAP_MESSAGE_TOO_LARGE",
                "The runtime-control ready message exceeds the protocol limit.",
            ));
        }
        let encoded_ready = base64url_encode(&ready_json);
        let ready_frame = format!("{FRAME_PREFIX}{frame_token}@@{encoded_ready}");
        if ready_frame.len() + 1 > MAX_ENCODED_LINE_BYTES {
            return Err(AppError::new(
                "CONTROL_BOOTSTRAP_FRAME_TOO_LARGE",
                "The runtime-control ready frame exceeds the protocol line limit.",
            ));
        }

        let script = build_bootstrap_script(session_id, &frame_token, &ready_frame);
        let mut file = Builder::new()
            .prefix(".mame-tauri-control-")
            .suffix(".lua")
            .tempfile()
            .map_err(|error| bootstrap_io_error("create", error))?;
        restrict_bootstrap_permissions(file.path())?;
        file.write_all(script.as_bytes())
            .map_err(|error| bootstrap_io_error("write", error))?;
        file.flush()
            .map_err(|error| bootstrap_io_error("flush", error))?;
        file.as_file()
            .sync_all()
            .map_err(|error| bootstrap_io_error("sync", error))?;
        register_bootstrap(session_id, &frame_token)?;

        Ok(Self { file, frame_token })
    }

    pub(super) fn append_launch_arguments(&self, argv: &mut Vec<OsString>) {
        argv.push(OsString::from("-console"));
        argv.push(OsString::from("-autoboot_script"));
        argv.push(self.file.path().as_os_str().to_owned());
    }

    pub(super) fn frame_token(&self) -> &str {
        &self.frame_token
    }

    #[cfg(test)]
    fn path(&self) -> &Path {
        self.file.path()
    }
}

impl Drop for ControlBootstrap {
    fn drop(&mut self) {
        unregister_bootstrap(&self.frame_token);
    }
}

fn build_bootstrap_script(session_id: &str, frame_token: &str, ready_frame: &str) -> String {
    include_str!("control_pause_resume.lua")
        .replace("__TOKEN__", frame_token)
        .replace("__SESSION__", session_id)
        .replace("__READY_FRAME__", ready_frame)
}
