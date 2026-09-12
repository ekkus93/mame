import { useCallback, useEffect, useState } from "react";

import { getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { SessionSnapshot } from "../backend/types";
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

export function SessionControlPanel({ onStopped }: { onStopped: () => void }) {
  const [state, setState] = useState<SessionPanelState>({ status: "loading" });

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
          const session =
            current.status === "active" || current.status === "error" ? current.session : null;
          return { status: "error", message: errorMessage(error), session };
        });
      });
  }, []);

  useEffect(() => {
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  function requestStop(session: SessionSnapshot) {
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
        <p className={state.warning ? "error-message" : undefined} role={state.warning ? "alert" : "status"}>
          {state.message}
        </p>
      </section>
    );
  }

  const session = state.status === "error" ? state.session : state.session;
  if (!session) {
    return (
      <section className="status-card" role="alert" aria-label="MAME session status">
        <strong>Session status unavailable</strong>
        <p className="error-message">{state.status === "error" ? state.message : "Unknown error"}</p>
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
