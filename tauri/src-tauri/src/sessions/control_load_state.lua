local function write_load_completion_marker(pending, status, message)
    local marker = {
        token = pending.completionToken,
        requestId = pending.requestId,
        status = status
    }
    if status == "error" then
        marker.error = wire_error(
            "CONTROL_OPERATION_FAILED",
            message or "MAME rejected the native load-state request.",
            {},
            false
        )
    end

    local encoded_ok, encoded = pcall(json.stringify, marker)
    if not encoded_ok or type(encoded) ~= "string" or #encoded > 4096 then
        return false
    end
    local file = io.open(pending.completionPath, "wb")
    if file == nil then
        return false
    end
    local write_ok = file:write(encoded)
    local close_ok = file:close()
    return write_ok ~= nil and close_ok == true
end

control_state.subscriptions.load = emu.add_machine_post_load_notifier(function ()
    local pending = control_state.pending_load
    control_state.pending_load = nil
    if pending ~= nil and not write_load_completion_marker(pending, "ok", nil) then
        emit_protocol_error(
            "CONTROL_COMPLETION_MARKER_FAILED",
            "MAME restored the state, but the private load-completion marker could not be written."
        )
    end
end)

local function has_only_load_state_params(value)
    if type(value) ~= "table"
        or type(value.path) ~= "string"
        or #value.path < 1
        or #value.path > 4096
        or type(value.slot) ~= "string"
        or #value.slot < 1
        or #value.slot > 32
        or value.slot:match("^[A-Za-z0-9][A-Za-z0-9_-]*$") == nil
        or type(value.completionPath) ~= "string"
        or #value.completionPath < 1
        or #value.completionPath > 4096
        or type(value.completionToken) ~= "string"
        or #value.completionToken ~= 43
        or value.completionToken:match("^[A-Za-z0-9_-]+$") == nil then
        return false
    end
    for key, _ in pairs(value) do
        if key ~= "path"
            and key ~= "slot"
            and key ~= "completionPath"
            and key ~= "completionToken" then
            return false
        end
    end
    return true
end

local function handle_load_state(request, request_id)
    if control_state.pending_load ~= nil then
        emit_rejected(
            request_id,
            "CONTROL_OPERATION_IN_PROGRESS",
            "A prior load-state request is still awaiting completion.",
            {},
            false
        )
        return
    end

    local pending = {
        requestId = request_id,
        completionPath = request.params.completionPath,
        completionToken = request.params.completionToken
    }
    control_state.pending_load = pending
    emit_accepted(request_id)
    local ok, err = pcall(function ()
        manager.machine:load(request.params.path)
    end)
    if not ok then
        control_state.pending_load = nil
        if not write_load_completion_marker(pending, "error", tostring(err)) then
            emit_protocol_error(
                "CONTROL_COMPLETION_MARKER_FAILED",
                "MAME rejected the native load-state request and its private failure marker could not be written."
            )
        end
    end
end
