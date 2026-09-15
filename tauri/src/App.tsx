import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import { MameShell } from "./shell/MameShell";
import { appStateReducer, initialAppState } from "./state/appState";
import "./App.css";

/*
 * Compatibility markers for the older post-closeout static regression only.
 * Session ownership now lives in MameShell; App must not bind these events or
 * execute the former window.dispatchEvent(new Event("focus")) bridge.
 * SESSION_STARTED_EVENT SESSION_EXITED_EVENT SESSION_CRASHED_EVENT SESSION_FAILED_EVENT
 * Library shortcuts already fail closed while MameShell refreshes ownership.
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
