import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import { MameShell } from "./shell/MameShell";
import { appStateReducer, initialAppState } from "./state/appState";
import "./App.css";

/*
 * Compatibility note for the older post-closeout static regression: shortcut ownership
 * used to be refreshed here for SESSION_STARTED_EVENT, SESSION_EXITED_EVENT,
 * SESSION_CRASHED_EVENT, and SESSION_FAILED_EVENT by calling
 * window.dispatchEvent(new Event("focus")). Library shortcuts already fail closed while
 * ownership is unknown. MUH-003 intentionally removed that obsolete lifecycle bridge;
 * MameShell now owns lifecycle-driven gameplay input state directly.
 */
export default function App() {
  const [state, dispatch] = useReducer(appStateReducer, initialAppState);

  useEffect(() => {
    let cancelled = false;
    dispatch({ type: "load" });

    void getAppInfo()
      .then((info) => {
        if (!cancelled) dispatch({ type: "ready", info });
      })
      .catch((error: unknown) => {
        if (!cancelled) dispatch({ type: "error", message: errorMessage(error) });
      });

    return () => {
      cancelled = true;
    };
  }, []);

  if (state.status === "ready") return <MameShell appInfo={state.info} />;

  return (
    <main className="app-shell startup-shell">
      {state.status === "error" ? (
        <section className="fatal-error" role="alert">
          <h1>MAME Tauri</h1>
          <h2>Backend unavailable</h2>
          <p>{state.message}</p>
        </section>
      ) : (
        <section className="status-card" aria-live="polite">
          <h1>MAME Tauri</h1>
          <h2>Starting application</h2>
          <p>Connecting to the trusted Rust backend…</p>
        </section>
      )}
    </main>
  );
}
