import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import { MameShell } from "./shell/MameShell";
import { appStateReducer, initialAppState } from "./state/appState";
import "./App.css";

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
