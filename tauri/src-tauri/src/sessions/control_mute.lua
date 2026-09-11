local function has_only_set_mute_params(value)
    if type(value) ~= "table" or type(value.muted) ~= "boolean" then
        return false
    end
    for key, _ in pairs(value) do
        if key ~= "muted" then
            return false
        end
    end
    return true
end

local function emit_mt709_completed(request_id, result)
    emit_message({
        version = 1,
        type = "response",
        sessionId = session_id,
        requestId = request_id,
        status = "completed",
        ok = true,
        result = result
    })
end

local function handle_set_mute(request, request_id)
    local ok, result = pcall(function ()
        manager.machine.sound.ui_mute = request.params.muted
        return {
            uiMuted = manager.machine.sound.ui_mute,
            effectiveMuted = manager.machine.sound.muted
        }
    end)
    if not ok then
        emit_rejected(
            request_id,
            "CONTROL_OPERATION_FAILED",
            "MAME rejected the native UI-mute request: " .. tostring(result),
            {},
            false
        )
        return
    end
    emit_mt709_completed(request_id, result)
end
