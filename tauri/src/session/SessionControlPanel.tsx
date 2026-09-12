import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { SessionLifecycleEventV1, SessionSnapshot } from "../backend/types";

function isActiveSession(session: SessionSnapshot | null): session is SessionSnapshot {
  return (
    session !== null &&
    (session.state === "created" ||
      session.state === "starting" ||
      session.state === "running" ||
      session.state === "stopping")
  );
}

export function SessionControlPanel() {
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [stopping, setStopping] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    let unlistenFns: UnlistenFn[] = [];

    function acceptLifecycleEvent(event: SessionLifecycleEventV1) {
      if (disposed) {
        return;
      }
      setSession(isActiveSession(event.session) ? event.session : null);
      setStopping(false);
      if (event.session.state === "running") {
        setMessage(null);
      }
    }

    void (async () => {
      try {
        const listeners = await Promise.all([
          listen<SessionLifecycleEventV1>("session.started", ({ payload }) =>
            acceptLifecycleEvent(payload),
          ),
          listen<SessionLifecycleEventV1>("session.exited", ({ payload }) =>
            acceptLifecycleEvent(payload),
          ),
          listen<SessionLifecycleEventV1>("session.crashed", ({ payload }) =>
            acceptLifecycleEvent(payload),
          ),
          listen<SessionLifecycleEventV1>("session.failed", ({ payload }) =>
            acceptLifecycleEvent(payload),
          ),
        ]);

        if (disposed) {
          listeners.forEach((unlisten) => unlisten());
          return;
        }
        unlistenFns = listeners;

        const current = await getMameSession();
        if (!disposed) {
          setSession(isActiveSession(current) ? current : null);
          setLoading(false);
        }
      } catch (error: unknown) {
        if (!disposed) {
          setLoading(false);
          setMessage(errorMessage(error));
        }
      }
    })();

    return () => {
      disposed = true;
      unlistenFns.forEach((unlisten) => unlisten());
    };
  }, []);

  function requestStop() {
    if (!session || stopping) {
      return;
    }

    setStopping(true);
    setMessage(null);
    void stopMame({ sessionId: session.sessionId })
      .then((result) => {
        setSession(isActiveSession(result.session) ? result.session : null);
        setStopping(false);
        setMessage(
          result.forcedTermination
            ? "MAME did not exit during the graceful shutdown window and required forced termination."
            : null,
        );
      })
      .catch((error: unknown) => {
        setStopping(false);
        setMessage(errorMessage(error));
      });
  }

  if (loading) {
    return (
      <section className="status-card" aria-live="polite" aria-label="MAME session control">
        <h2>Session</h2>
        <p>Checking for an active MAME session…</p>
      </section>
    );
  }

  return (
    <section className="status-card" aria-live="polite" aria-label="MAME session control">
      <h2>Session</h2>
      {session ? (
        <>
          <p>
            MAME is {session.state} for <strong>{session.machine}</strong>
            {session.software ? ` · ${session.software}` : ""}
            {session.pid ? ` · PID ${session.pid}` : ""}.
          </p>
          <button
            type="button"
            disabled={stopping || session.state !== "running"}
            onClick={requestStop}
          >
            {stopping || session.state === "stopping" ? "Stopping MAME…" : "Stop MAME"}
          </button>
        </>
      ) : (
        <p>No active MAME session.</p>
      )}
      {message && (
        <p className="launch-error" role="alert">
          {message}
        </p>
      )}
    </section>
  );
}
