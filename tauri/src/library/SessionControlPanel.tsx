import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import { getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { SessionLifecycleEventV1, SessionSnapshot } from "../backend/types";
import { isGameplaySessionState } from "./keyboardNavigation";

type SessionPanelState =
  | { status: "loading" }
  | { status: "idle"; message?: string; warning?: boolean }
  | { status: "active"; session: SessionSnapshot }
  | { status: "stopping"; session: SessionSnapshot }
  | { status: "error"; message: string; session: SessionSnapshot | null };

function activeSession(session: SessionSnapshot | null): SessionSnapshot | null {
  return session && isGameplaySessionState(session.state) ? session : null;
}

function retainedSession(state: SessionPanelState): SessionSnapshot | null {
  return state.status === "active" || state.status === "stopping" || state.status === "error"
    ? state.session
    : null;
}

export function SessionControlPanel({ onStopped }: { onStopped: () => void }) {
  const [state, setState] = useState<SessionPanelState>({ status: "loading" });
  const stoppingSessionId = useRef<string | null>(null);

  const refresh = useCallback(() => {
    void getMameSession()
      .then((session) => {
        setState((current) => {
          if (current.status === "stopping") {
            return current;
          }
          const active = activeSession(session);
          return active ? { status: "active", session: active } : { status: "idle" };
        });
      })
      .catch((error: unknown) => {
        setState((current) => {
          if (current.status === "stopping") {
            return current;
          }
          return {
            status: "error",
            message: errorMessage(error),
            session: retainedSession(current),
          };
        });
      });
  }, []);

  useEffect(() => {
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: Array<() => void> = [];
    const eventNames = [
      "session.started",
      "session.exited",
      "session.crashed",
      "session.failed",
    ] as const;

    void Promise.all(
      eventNames.map((eventName) =>
        listen<SessionLifecycleEventV1>(eventName, (event) => {
          if (cancelled) {
            return;
          }
          const session = event.payload.session;
          if (stoppingSessionId.current === session.sessionId) {
            return;
          }
          if (isGameplaySessionState(session.state)) {
            setState({ status: "active", session });
            return;
          }

          onStopped();
          setState({
            status: "idle",
            message:
              session.state === "crashed"
                ? "MAME session ended after an abnormal exit."
                : session.state === "failed"
                  ? "MAME session supervision failed."
                  : "MAME session exited.",
            warning: session.state === "crashed" || session.state === "failed",
          });
        }),
      ),
    )
      .then((registered) => {
        if (cancelled) {
          registered.forEach((dispose) => dispose());
        } else {
          unlisten = registered;
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState((current) => ({
            status: "error",
            message: `MAME session event subscription failed: ${errorMessage(error)}`,
            session: retainedSession(current),
          }));
        }
      });

    return () => {
      cancelled = true;
      unlisten.forEach((dispose) => dispose());
    };
  }, [onStopped]);

  function requestStop(session: SessionSnapshot) {
    stoppingSessionId.current = session.sessionId;
    setState({ status: "stopping", session });
    void stopMame({ sessionId: session.sessionId })
      .then((result) => {
        if (isGameplaySessionState(result.session.state)) {
          setState({
            status: "error",
            message: "MAME stop returned without reaching a terminal session state.",
            session: result.session,
          });
          return;
        }

        onStopped();
        setState({
          status: "idle",
          message: result.forcedTermination
            ? "MAME stopped after forced termination was required."
            : "MAME session stopped cleanly.",
          warning: result.forcedTermination,
        });
      })
      .catch((error: unknown) => {
        setState({ status: "error", message: errorMessage(error), session });
      })
      .finally(() => {
        stoppingSessionId.current = null;
      });
  }

  if (state.status === "loading") {
    return (
      <section className="status-card" aria-live="polite" aria-label="MAME session status">
        <strong>Checking MAME session…</strong>
      </section>
    );
  }

  if (state.status === "idle") {
    if (!state.message) {
      return null;
    }
    return (
      <section className="status-card" aria-live="polite" aria-label="MAME session status">
        <strong>Session idle</strong>
        <p
          className={state.warning ? "error-message" : undefined}
          role={state.warning ? "alert" : "status"}
        >
          {state.message}
        </p>
      </section>
    );
  }

  const session = state.session;
  if (!session) {
    return (
      <section className="status-card" role="alert" aria-label="MAME session status">
        <strong>Session status unavailable</strong>
        <p className="error-message">
          {state.status === "error" ? state.message : "Unknown error"}
        </p>
        <button type="button" className="secondary-button" onClick={refresh}>
          Retry
        </button>
      </section>
    );
  }

  const stopping = state.status === "stopping";
  return (
    <section className="status-card" aria-live="polite" aria-label="MAME session controls">
      <div>
        <p className="eyebrow">Active MAME session</p>
        <strong>{session.machine}</strong>
        {session.software && <span> · {session.software}</span>}
        <p>
          <code>{session.sessionId}</code>
          {session.pid ? ` · PID ${session.pid}` : ""}
        </p>
      </div>
      <button type="button" disabled={stopping} onClick={() => requestStop(session)}>
        {stopping ? "Stopping MAME…" : state.status === "error" ? "Retry stop" : "Stop MAME"}
      </button>
      {state.status === "error" && (
        <p className="error-message" role="alert">
          {state.message}
        </p>
      )}
    </section>
  );
}
