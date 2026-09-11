local expected_token = "__TOKEN__"
local session_id = "__SESSION__"
local ready_frame = "__READY_FRAME__"
local json = require("json")
local frame_prefix = "@@MAME_TAURI_CONTROL_V1@@"
local max_message_bytes = 16384
local max_encoded_payload_bytes = 24576
local known_commands = {
    pause = true,
    resume = true,
    reset = true,
    exit = true,
    save_state = true,
    load_state = true,
    set_mute = true,
    set_volume = true,
    query_state = true
}
local supported_commands = { pause = true, resume = true, reset = true, exit = true }
local control_state = {
    seen_request_ids = {},
    seen_request_order = {},
    max_seen_request_ids = 4096,
    pending_pause = nil,
    pending_resume = nil,
    pending_reset = nil,
    subscriptions = {}
}
local base64url_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
local base64url_decode_map = {}
for index = 1, #base64url_alphabet do
    base64url_decode_map[base64url_alphabet:sub(index, index)] = index - 1
end

local function base64url_encode(input)
    local output = {}
    local index = 1
    while index + 2 <= #input do
        local a, b, c = input:byte(index, index + 2)
        local value = (a << 16) | (b << 8) | c
        output[#output + 1] = base64url_alphabet:sub(((value >> 18) & 0x3f) + 1, ((value >> 18) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub(((value >> 12) & 0x3f) + 1, ((value >> 12) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub(((value >> 6) & 0x3f) + 1, ((value >> 6) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub((value & 0x3f) + 1, (value & 0x3f) + 1)
        index = index + 3
    end
    local remaining = #input - index + 1
    if remaining == 1 then
        local a = input:byte(index)
        local value = a << 16
        output[#output + 1] = base64url_alphabet:sub(((value >> 18) & 0x3f) + 1, ((value >> 18) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub(((value >> 12) & 0x3f) + 1, ((value >> 12) & 0x3f) + 1)
    elseif remaining == 2 then
        local a, b = input:byte(index, index + 1)
        local value = (a << 16) | (b << 8)
        output[#output + 1] = base64url_alphabet:sub(((value >> 18) & 0x3f) + 1, ((value >> 18) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub(((value >> 12) & 0x3f) + 1, ((value >> 12) & 0x3f) + 1)
        output[#output + 1] = base64url_alphabet:sub(((value >> 6) & 0x3f) + 1, ((value >> 6) & 0x3f) + 1)
    end
    return table.concat(output)
end

local function base64url_decode(input)
    if type(input) ~= "string" or (#input % 4) == 1 then
        return nil
    end
    local output = {}
    local index = 1
    while index + 3 <= #input do
        local a = base64url_decode_map[input:sub(index, index)]
        local b = base64url_decode_map[input:sub(index + 1, index + 1)]
        local c = base64url_decode_map[input:sub(index + 2, index + 2)]
        local d = base64url_decode_map[input:sub(index + 3, index + 3)]
        if a == nil or b == nil or c == nil or d == nil then
            return nil
        end
        local value = (a << 18) | (b << 12) | (c << 6) | d
        output[#output + 1] = string.char((value >> 16) & 0xff, (value >> 8) & 0xff, value & 0xff)
        index = index + 4
    end
    local remaining = #input - index + 1
    if remaining == 2 then
        local a = base64url_decode_map[input:sub(index, index)]
        local b = base64url_decode_map[input:sub(index + 1, index + 1)]
        if a == nil or b == nil or (b & 0x0f) ~= 0 then
            return nil
        end
        output[#output + 1] = string.char((a << 2) | (b >> 4))
    elseif remaining == 3 then
        local a = base64url_decode_map[input:sub(index, index)]
        local b = base64url_decode_map[input:sub(index + 1, index + 1)]
        local c = base64url_decode_map[input:sub(index + 2, index + 2)]
        if a == nil or b == nil or c == nil or (c & 0x03) ~= 0 then
            return nil
        end
        output[#output + 1] = string.char((a << 2) | (b >> 4), ((b << 4) | (c >> 2)) & 0xff)
    elseif remaining ~= 0 then
        return nil
    end
    return table.concat(output)
end

local function wire_error(code, message, details, retryable)
    return {
        code = code,
        message = message,
        details = details or {},
        retryable = retryable == true
    }
end

local function emit_message(message)
    local ok, encoded_json = pcall(json.stringify, message)
    if not ok or type(encoded_json) ~= "string" or #encoded_json > max_message_bytes then
        return false
    end
    local encoded = base64url_encode(encoded_json)
    if #encoded + #frame_prefix + #expected_token + 3 > max_encoded_payload_bytes then
        return false
    end
    io.write("\n", frame_prefix, expected_token, "@@", encoded, "\n")
    io.flush()
    return true
end

local function emit_ready()
    emit_message({
        version = 1,
        type = "event",
        sessionId = session_id,
        event = "ready",
        payload = {
            commands = { "pause", "resume", "reset", "exit" },
            maxMessageBytes = max_message_bytes
        }
    })
end

local function emit_rejected(request_id, code, message, details, retryable)
    emit_message({
        version = 1,
        type = "response",
        sessionId = session_id,
        requestId = request_id,
        status = "rejected",
        ok = false,
        error = wire_error(code, message, details, retryable)
    })
end

local function emit_protocol_error(code, message)
    emit_message({
        version = 1,
        type = "event",
        sessionId = session_id,
        event = "protocol_error",
        payload = {
            fatal = true,
            error = wire_error(code, message, {}, false)
        }
    })
end

local function emit_accepted(request_id)
    emit_message({
        version = 1,
        type = "response",
        sessionId = session_id,
        requestId = request_id,
        status = "accepted",
        ok = true,
        result = {}
    })
end

local function emit_completed(request_id, paused)
    emit_message({
        version = 1,
        type = "response",
        sessionId = session_id,
        requestId = request_id,
        status = "completed",
        ok = true,
        result = { paused = paused }
    })
end

local function emit_state(paused, request_id)
    local message = {
        version = 1,
        type = "event",
        sessionId = session_id,
        event = paused and "paused" or "resumed",
        payload = { paused = paused }
    }
    if request_id ~= nil then
        message.requestId = request_id
    end
    emit_message(message)
end

local function emit_command_completed(request_id, command, result)
    emit_message({
        version = 1,
        type = "event",
        sessionId = session_id,
        event = "command_completed",
        requestId = request_id,
        payload = {
            command = command,
            ok = true,
            result = result
        }
    })
end

local function emit_command_failed(request_id, command, message)
    emit_message({
        version = 1,
        type = "event",
        sessionId = session_id,
        event = "command_completed",
        requestId = request_id,
        payload = {
            command = command,
            ok = false,
            error = wire_error("CONTROL_OPERATION_FAILED", message, {}, false)
        }
    })
end

control_state.subscriptions.pause = emu.add_machine_pause_notifier(function ()
    local request_id = control_state.pending_pause
    control_state.pending_pause = nil
    emit_state(true, request_id)
    if request_id ~= nil then
        emit_command_completed(request_id, "pause", { paused = true })
    end
end)

control_state.subscriptions.resume = emu.add_machine_resume_notifier(function ()
    local request_id = control_state.pending_resume
    control_state.pending_resume = nil
    emit_state(false, request_id)
    if request_id ~= nil then
        emit_command_completed(request_id, "resume", { paused = false })
    end
end)

local function emit_reset(request_id)
    emit_message({
        version = 1,
        type = "event",
        sessionId = session_id,
        event = "reset",
        requestId = request_id,
        payload = { kind = "soft" }
    })
end

control_state.subscriptions.reset = emu.add_machine_reset_notifier(function ()
    local request_id = control_state.pending_reset
    control_state.pending_reset = nil
    if request_id ~= nil then
        emit_reset(request_id)
        emit_command_completed(request_id, "reset", { kind = "soft" })
    end
end)

local function has_only_request_fields(request)
    local allowed = {
        version = true,
        type = true,
        sessionId = true,
        requestId = true,
        command = true,
        params = true
    }
    for key, _ in pairs(request) do
        if not allowed[key] then
            return false
        end
    end
    return true
end

local function empty_table(value)
    if type(value) ~= "table" then
        return false
    end
    return next(value) == nil
end

local function has_only_soft_reset_params(value)
    if type(value) ~= "table" or value.kind ~= "soft" then
        return false
    end
    for key, _ in pairs(value) do
        if key ~= "kind" then
            return false
        end
    end
    return true
end

function mame_tauri_control_v1(token, payload)
    if token ~= expected_token then
        return
    end
    if type(payload) ~= "string" or #payload > max_encoded_payload_bytes then
        emit_protocol_error("PROTOCOL_MESSAGE_TOO_LARGE", "The encoded request payload is invalid or too large.")
        return
    end
    local decoded = base64url_decode(payload)
    if decoded == nil or #decoded > max_message_bytes then
        emit_protocol_error("PROTOCOL_INVALID_REQUEST", "The request payload is not valid bounded base64url JSON.")
        return
    end
    local parsed_ok, request = pcall(json.parse, decoded)
    if not parsed_ok or type(request) ~= "table" then
        emit_protocol_error("PROTOCOL_INVALID_REQUEST", "The request JSON envelope is malformed.")
        return
    end

    local request_id = request.requestId
    if type(request_id) ~= "string" or #request_id < 1 or #request_id > 64 or request_id:match("^[A-Za-z0-9_-]+$") == nil then
        emit_protocol_error("PROTOCOL_INVALID_REQUEST_ID", "The request identifier is invalid.")
        return
    end
    if control_state.seen_request_ids[request_id] then
        emit_rejected(request_id, "PROTOCOL_DUPLICATE_REQUEST_ID", "The request identifier has already been used.", {}, false)
        return
    end
    control_state.seen_request_ids[request_id] = true
    control_state.seen_request_order[#control_state.seen_request_order + 1] = request_id
    if #control_state.seen_request_order > control_state.max_seen_request_ids then
        local oldest = table.remove(control_state.seen_request_order, 1)
        control_state.seen_request_ids[oldest] = nil
    end

    if not has_only_request_fields(request) or request.type ~= "request" then
        emit_rejected(request_id, "PROTOCOL_INVALID_REQUEST", "The request envelope is malformed.", {}, false)
        return
    end
    if request.version ~= 1 then
        emit_rejected(request_id, "PROTOCOL_UNSUPPORTED_VERSION", "The protocol version is not supported.", {}, false)
        return
    end
    if request.sessionId ~= session_id then
        emit_rejected(request_id, "PROTOCOL_SESSION_MISMATCH", "The request belongs to a different MAME session.", {}, false)
        emit_protocol_error("PROTOCOL_SESSION_MISMATCH", "A cross-session request was received on this control channel.")
        return
    end
    if type(request.command) ~= "string" or not known_commands[request.command] then
        emit_rejected(request_id, "PROTOCOL_UNKNOWN_COMMAND", "The command is not part of protocol v1.", {}, false)
        return
    end
    if not supported_commands[request.command] then
        emit_rejected(request_id, "CONTROL_UNSUPPORTED", "The command is not implemented by this runtime-control shim.", {}, false)
        return
    end
    if request.command == "reset" then
        if type(request.params) ~= "table" then
            emit_rejected(request_id, "PROTOCOL_INVALID_PARAMS", "Reset requires an object parameter payload.", {}, false)
            return
        end
        if request.params.kind ~= "soft" then
            emit_rejected(request_id, "CONTROL_UNSUPPORTED_RESET_KIND", "Protocol v1 supports soft reset only.", { requestedKind = request.params.kind }, false)
            return
        end
        if not has_only_soft_reset_params(request.params) then
            emit_rejected(request_id, "PROTOCOL_INVALID_PARAMS", "Soft reset requires exactly { kind = \"soft\" }.", {}, false)
            return
        end
    elseif not empty_table(request.params) then
        emit_rejected(request_id, "PROTOCOL_INVALID_PARAMS", "Pause, resume, and exit require an empty parameter object.", {}, false)
        return
    end

    if request.command == "pause" then
        if manager.machine.paused then
            emit_completed(request_id, true)
            return
        end
        control_state.pending_pause = request_id
        emit_accepted(request_id)
        local ok, err = pcall(emu.pause)
        if not ok then
            control_state.pending_pause = nil
            emit_command_failed(request_id, "pause", tostring(err))
        end
        return
    end

    if request.command == "resume" then
        if not manager.machine.paused then
            emit_completed(request_id, false)
            return
        end
        control_state.pending_resume = request_id
        emit_accepted(request_id)
        local ok, err = pcall(emu.unpause)
        if not ok then
            control_state.pending_resume = nil
            emit_command_failed(request_id, "resume", tostring(err))
        end
        return
    end

    if request.command == "exit" then
        emit_accepted(request_id)
        local ok, err = pcall(function ()
            manager.machine:exit()
        end)
        if not ok then
            emit_protocol_error("CONTROL_OPERATION_FAILED", "MAME rejected the native clean-exit request: " .. tostring(err))
        end
        return
    end

    control_state.pending_reset = request_id
    emit_accepted(request_id)
    local ok, err = pcall(function ()
        manager.machine:soft_reset()
    end)
    if not ok then
        control_state.pending_reset = nil
        emit_command_failed(request_id, "reset", tostring(err))
    end
end

-- Keep the generated ready_frame literal for backwards-compatible bootstrap
-- inspection tests; production emits the current capability set dynamically.
emit_ready()
