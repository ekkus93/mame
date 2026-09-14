import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

import {
  getMameSession,
  pauseMame,
  queryMameRuntimeState,
  resetMame,
  resumeMame,
  setMameMute,
  stopMame,
} from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type {
  QueryMameRuntimeStateResult,
  SessionLifecycleEventV1,
  SessionPauseEventV1,
  SessionSnapshot,
  SetMameMuteResult,
} from "../backend/types";
import { SaveStateBrowser } from "./SaveStateBrowser";

type RuntimeOperation = "pause" | "resume" | "reset" | "mute" | "unmute" | "refresh" | null;

const TERMINAL_SESSION_STATES = new Set(["exited", "failed", "crashed"]);

function isActiveSession(session: SessionSnapshot | null): session is SessionSnapshot {
  return session !== null && ["created", "starting", "running", "stopping"].includes(session.state);
}

function isTerminalSession(session: SessionSnapshot): boolean {
  return TERMINAL_SESSION_STATES.has(session.state);
}

function isRunningSession(session: SessionSnapshot | null): session is SessionSnapshot {
  return session !== null && session.state === "running";
}

export function SessionControlPanel() {
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [runtimeState, setRuntimeState] = useState<QueryMameRuntimeStateResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [stopping, setStopping] = useState(false);
  const [operation, setOperation] = useState<RuntimeOperation>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refreshRuntimeState = useCallback(async (target: SessionSnapshot | null) => {
    if (!isRunningSession(target)) {
      setRuntimeState(null);
      return;
    }

    const state = await queryMameRuntimeState({ sessionId: target.sessionId });
    setRuntimeState(state);
  }, []);

  const acceptSession = useCallback(
    (next: SessionSnapshot | null) => {
      if (!next || !isActiveSession(next) || isTerminalSession(next)) {
        setSession(null);
        setRuntimeState(null);
        return;
      }
      setSession(next);
      if (next.state === "running") {
        void refreshRuntimeState(next).catch((error: unknown) => {
          setFailure(errorMessage(error));
          setRuntimeState(null);
        });
      } else {
        setRuntimeState(null);
      }
    },
    [refreshRuntimeState],
  );

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    void getMameSession()
      .then((current) => {
        if (!disposed) {
          acceptSession(current);
          setLoading(false);
        }
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setFailure(errorMessage(error));
          setLoading(false);
        }
      });

    const bind = async () => {
      const started = await listen<SessionLifecycleEventV1>("session.started", (event) => {
        if (!disposed) {
          acceptSession(event.payload.session);
          setFailure(null);
          setNotice(null);
        }
      });
      unlisteners.push(started);

      const exited = await listen<SessionLifecycleEventV1>("session.exited", () => {
        if (!disposed) {
          setSession(null);
          setRuntimeState(null);
          setStopping(false);
          setOperation(null);
          setFailure(null);
        }
      });
      unlisteners.push(exited);

      for (const eventName of ["session.crashed", "session.failed"] as const) {
        const unlisten = await listen<SessionLifecycleEventV1>(eventName, (event) => {
          if (!disposed) {
            setSession(null);
            setRuntimeState(null);
            setStopping(false);
            setOperation(null);
            setFailure(
              `MAME session ${event.payload.session.sessionId} ended unexpectedly (${event.payload.session.state}).`,
            );
          }
        });
        unlisteners.push(unlisten);
      }

      const paused = await listen<SessionPauseEventV1>("session.paused", (event) => {
        if (!disposed) {
          setRuntimeState((current) =>
            current && current.sessionId === event.payload.sessionId
              ? { ...current, paused: true }
              : current,
          );
        }
      });
      unlisteners.push(paused);

      const resumed = await listen<SessionPauseEventV1>("session.resumed", (event) => {
        if (!disposed) {
          setRuntimeState((current) =>
            current && current.sessionId === event.payload.sessionId
              ? { ...current, paused: false }
              : current,
          );
        }
      });
      unlisteners.push(resumed);

      if (disposed) {
        unlisteners.splice(0).forEach((unlisten) => unlisten());
      }
    };

    void bind().catch((error: unknown) => {
      if (!disposed) {
        setFailure(errorMessage(error));
      }
    });

    return () => {
      disposed = true;
      unlisteners.splice(0).forEach((unlisten) => unlisten());
    };
  }, [acceptSession]);

  const requestStop = () => {
    if (!isRunningSession(session) || stopping) {
      return;
    }
    setStopping(true);
    setFailure(null);
    setNotice(null);
    void stopMame({ sessionId: session.sessionId })
      .then((result) => {
        if (isTerminalSession(result.session)) {
          setSession(null);
          setRuntimeState(null);
        } else {
          acceptSession(result.session);
        }
        setStopping(false);
      })
      .catch((error: unknown) => {
        setStopping(false);
        setFailure(errorMessage(error));
      });
  };

  const runRuntimeOperation = (
    name: Exclude<RuntimeOperation, null>,
    action: (runningSession: SessionSnapshot) => Promise<string>,
  ) => {
    if (!isRunningSession(session) || operation || stopping) {
      return;
    }
    const target = session;
    setOperation(name);
    setFailure(null);
    setNotice(null);
    void action(target)
      .then((message) => {
        setNotice(message);
        return refreshRuntimeState(target);
      })
      .catch((error: unknown) => {
        setFailure(errorMessage(error));
      })
      .finally(() => setOperation(null));
  };

  const requestPause = () => {
    runRuntimeOperation("pause", async (target) => {
      await pauseMame({ sessionId: target.sessionId });
      return "Pause requested through the authenticated runtime-control channel.";
    });
  };

  const requestResume = () => {
    runRuntimeOperation("resume", async (target) => {
      await resumeMame({ sessionId: target.sessionId });
      return "Resume requested through the authenticated runtime-control channel.";
    });
  };

  const requestReset = () => {
    runRuntimeOperation("reset", async (target) => {
      await resetMame({ sessionId: target.sessionId });
      return "Soft reset requested through the authenticated runtime-control channel.";
    });
  };

  const requestMute = (muted: boolean) => {
    runRuntimeOperation(muted ? "mute" : "unmute", async (target) => {
      const result: SetMameMuteResult = await setMameMute({
        sessionId: target.sessionId,
        muted,
      });
      setRuntimeState((current) =>
        current && current.sessionId === result.sessionId
          ? {
              ...current,
              uiMuted: result.uiMuted,
              effectiveMuted: result.effectiveMuted,
            }
          : current,
      );
      return result.effectiveMuted
        ? "MAME native audio is muted."
        : "MAME native audio is unmuted.";
    });
  };

  const requestRuntimeRefresh = () => {
    runRuntimeOperation("refresh", async (target) => {
      await refreshRuntimeState(target);
      return "Runtime state refreshed from the authenticated MAME control channel.";
    });
  };

  const canCommand = isRunningSession(session) && !operation && !stopping;
  const paused = runtimeState?.paused ?? false;
  const muted = runtimeState?.uiMuted ?? runtimeState?.effectiveMuted ?? false;

  return (
    <>
      <section className="status-card" aria-labelledby="session-control-heading" aria-live="polite">
        <p className="eyebrow">MAME session</p>
        <h2 id="session-control-heading">Session control</h2>
        {loading ? (
          <p>Checking supervised MAME session…</p>
        ) : session ? (
          <>
            <p>
              <strong>{session.machine}</strong>
              {session.software ? ` · ${session.software}` : ""} · {session.state}
              {session.pid ? ` · PID ${session.pid}` : ""}
            </p>
            {runtimeState && (
              <p>
                Runtime: {runtimeState.running ? "running" : "not running"} ·{" "}
                {runtimeState.paused ? "paused" : "not paused"} ·{" "}
                {runtimeState.effectiveMuted ? "muted" : "not muted"}
              </p>
            )}
            <div
              className="session-control-actions"
              role="group"
              aria-label="MAME runtime controls"
            >
              <button type="button" disabled={!canCommand || paused} onClick={requestPause}>
                {operation === "pause" ? "Pausing…" : "Pause"}
              </button>
              <button type="button" disabled={!canCommand || !paused} onClick={requestResume}>
                {operation === "resume" ? "Resuming…" : "Resume"}
              </button>
              <button type="button" disabled={!canCommand} onClick={requestReset}>
                {operation === "reset" ? "Resetting…" : "Soft reset"}
              </button>
              <button type="button" disabled={!canCommand} onClick={() => requestMute(!muted)}>
                {operation === "mute" || operation === "unmute"
                  ? "Updating mute…"
                  : muted
                    ? "Unmute"
                    : "Mute"}
              </button>
              <button type="button" disabled={!canCommand} onClick={requestRuntimeRefresh}>
                {operation === "refresh" ? "Refreshing…" : "Refresh runtime state"}
              </button>
              <button type="button" disabled={!canCommand} onClick={requestStop}>
                {stopping ? "Stopping MAME…" : "Stop MAME"}
              </button>
            </div>
          </>
        ) : (
          <p>No active MAME session.</p>
        )}
        {notice && (
          <p className="save-state-notice" aria-live="polite">
            {notice}
          </p>
        )}
        {failure && (
          <p className="error-message" role="alert">
            {failure}
          </p>
        )}
      </section>
      <SaveStateBrowser session={session} />
    </>
  );
}
