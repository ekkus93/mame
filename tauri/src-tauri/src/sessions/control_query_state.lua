local function emit_mt710_completed(request_id, result)
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

local function handle_query_state(request_id)
    local ok, result = pcall(function ()
        return {
            running = true,
            paused = manager.machine.paused,
            machine = manager.machine.system.name,
            uiMuted = manager.machine.sound.ui_mute,
            effectiveMuted = manager.machine.sound.muted
        }
    end)
    if not ok then
        emit_rejected(
            request_id,
            "CONTROL_OPERATION_FAILED",
            "MAME could not provide bounded runtime state: " .. tostring(result),
            {},
            false
        )
        return
    end
    emit_mt710_completed(request_id, result)
end
