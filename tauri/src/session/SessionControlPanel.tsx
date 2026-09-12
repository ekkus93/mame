import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { SessionLifecycleEventV1, SessionSnapshot } from "../backend/types";
import { SaveStateBrowser } from "./SaveStateBrowser";

function isActiveSession(session: SessionSnapshot | null): session is SessionSnapshot {
  return session !== null && ["created", "starting", "running", "stopping"].includes(session.state);
}

export function SessionControlPanel() {
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [stopping, setStopping] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    void getMameSession()
      .then((current) => {
        if (!disposed) {
          setSession(isActiveSession(current) ? current : null);
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
          setSession(event.payload.session);
          setFailure(null);
        }
      });
      unlisteners.push(started);

      const exited = await listen<SessionLifecycleEventV1>("session.exited", () => {
        if (!disposed) {
          setSession(null);
          setStopping(false);
          setFailure(null);
        }
      });
      unlisteners.push(exited);

      for (const eventName of ["session.crashed", "session.failed"] as const) {
        const unlisten = await listen<SessionLifecycleEventV1>(eventName, (event) => {
          if (!disposed) {
            setSession(null);
            setStopping(false);
            setFailure(
              `MAME session ${event.payload.session.sessionId} ended unexpectedly (${event.payload.session.state}).`,
            );
          }
        });
        unlisteners.push(unlisten);
      }

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
  }, []);

  const requestStop = () => {
    if (!session || session.state !== "running" || stopping) {
      return;
    }
    setStopping(true);
    setFailure(null);
    void stopMame({ sessionId: session.sessionId })
      .then((result) => {
        if (["exited", "failed", "crashed"].includes(result.session.state)) {
          setSession(null);
        } else {
          setSession(result.session);
        }
        setStopping(false);
      })
      .catch((error: unknown) => {
        setStopping(false);
        setFailure(errorMessage(error));
      });
  };

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
            <button
              type="button"
              disabled={session.state !== "running" || stopping}
              onClick={requestStop}
            >
              {stopping ? "Stopping MAME…" : "Stop MAME"}
            </button>
          </>
        ) : (
          <p>No active MAME session.</p>
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
