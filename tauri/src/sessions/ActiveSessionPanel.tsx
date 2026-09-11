import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import { getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { SessionLifecycleEventV1, SessionSnapshot, SessionState } from "../backend/types";

type SessionPanelState =
  | { status: "loading" }
  | { status: "idle" }
  | { status: "active"; session: SessionSnapshot; stopError: string | null }
  | { status: "stopping"; session: SessionSnapshot }
  | { status: "error"; message: string };

const TERMINAL_EVENTS = ["session.exited", "session.crashed", "session.failed"] as const;

function isActiveSessionState(state: SessionState): boolean {
  return state === "created" || state === "starting" || state === "running" || state === "stopping";
}

export function ActiveSessionPanel({ onSessionEnded }: { onSessionEnded: () => void }) {
  const [state, setState] = useState<SessionPanelState>({ status: "loading" });
  const activeSessionIdRef = useRef<string | null>(null);
  const endedSessionIdRef = useRef<string | null>(null);

  const showActiveSession = useCallback((session: SessionSnapshot) => {
    activeSessionIdRef.current = session.sessionId;
    endedSessionIdRef.current = null;
    setState({ status: "active", session, stopError: null });
  }, []);

  const finishSession = useCallback(
    (sessionId: string) => {
      if (
        activeSessionIdRef.current !== null &&
        activeSessionIdRef.current !== sessionId
      ) {
        return;
      }
      if (endedSessionIdRef.current === sessionId) {
        return;
      }

      activeSessionIdRef.current = null;
      endedSessionIdRef.current = sessionId;
      setState({ status: "idle" });
      onSessionEnded();
    },
    [onSessionEnded],
  );

  useEffect(() => {
    let disposed = false;
    const unlistenFns: UnlistenFn[] = [];

    async function registerEventListeners() {
      const startedUnlisten = await listen<SessionLifecycleEventV1>(
        "session.started",
        ({ payload }) => {
          if (!disposed) {
            showActiveSession(payload.session);
          }
        },
      );
      if (disposed) {
        startedUnlisten();
        return;
      }
      unlistenFns.push(startedUnlisten);

      for (const eventName of TERMINAL_EVENTS) {
        const terminalUnlisten = await listen<SessionLifecycleEventV1>(eventName, ({ payload }) => {
          if (!disposed) {
            finishSession(payload.session.sessionId);
          }
        });
        if (disposed) {
          terminalUnlisten();
          return;
        }
        unlistenFns.push(terminalUnlisten);
      }

      const current = await getMameSession();
      if (disposed) {
        return;
      }
      if (current && isActiveSessionState(current.state)) {
        showActiveSession(current);
      } else {
        activeSessionIdRef.current = null;
        setState({ status: "idle" });
      }
    }

    void registerEventListeners().catch((error: unknown) => {
      for (const unlisten of unlistenFns.splice(0)) {
        unlisten();
      }
      if (!disposed) {
        activeSessionIdRef.current = null;
        setState({ status: "error", message: errorMessage(error) });
      }
    });

    return () => {
      disposed = true;
      for (const unlisten of unlistenFns) {
        unlisten();
      }
    };
  }, [finishSession, showActiveSession]);

  async function requestStop() {
    if (state.status !== "active" || state.session.state !== "running") {
      return;
    }

    const session = state.session;
    setState({ status: "stopping", session });

    try {
      const result = await stopMame({ sessionId: session.sessionId });
      if (result.session.sessionId !== session.sessionId) {
        throw new Error("The stop result did not match the requested MAME session.");
      }
      if (result.session.state !== "exited") {
        throw new Error(`MAME stop completed with unexpected state ${result.session.state}.`);
      }
      finishSession(session.sessionId);
    } catch (error: unknown) {
      activeSessionIdRef.current = session.sessionId;
      endedSessionIdRef.current = null;
      setState({
        status: "active",
        session,
        stopError: errorMessage(error),
      });
    }
  }

  if (state.status === "loading") {
    return (
      <section className="status-card" aria-live="polite" aria-label="MAME session">
        <strong>Checking MAME session…</strong>
      </section>
    );
  }

  if (state.status === "error") {
    return (
      <section className="status-card error-state" role="alert" aria-label="MAME session">
        <strong>Session status unavailable</strong>
        <span>{state.message}</span>
      </section>
    );
  }

  if (state.status === "idle") {
    return (
      <section className="status-card" aria-live="polite" aria-label="MAME session">
        <strong>No active MAME session</strong>
        <span>Launch a machine or software item from the library.</span>
      </section>
    );
  }

  const session = state.session;
  const stopping = state.status === "stopping" || session.state === "stopping";
  const canStop = state.status === "active" && session.state === "running";

  return (
    <section className="status-card" aria-live="polite" aria-label="MAME session">
      <div>
        <strong>{stopping ? "Stopping MAME…" : "MAME session running"}</strong>
        <span>
          {session.machine}
          {session.software ? ` · ${session.software}` : ""}
          {session.pid ? ` · PID ${session.pid}` : ""}
        </span>
      </div>
      <button type="button" disabled={!canStop} onClick={() => void requestStop()}>
        {stopping ? "Stopping…" : "Stop MAME"}
      </button>
      {state.status === "active" && state.stopError && (
        <p className="error-message" role="alert">
          Stop failed: {state.stopError}
        </p>
      )}
    </section>
  );
}
