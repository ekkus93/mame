import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import { appStateReducer, initialAppState } from "./state/appState";
import "./App.css";

export default function App() {
  const [state, dispatch] = useReducer(appStateReducer, initialAppState);

  useEffect(() => {
    let cancelled = false;
    dispatch({ type: "load" });

    void getAppInfo()
      .then((info) => {
        if (!cancelled) {
          dispatch({ type: "ready", info });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          dispatch({ type: "error", message: errorMessage(error) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="app-shell">
      <header className="app-header">
        <p className="eyebrow">MAME Tauri modernization</p>
        <h1>Preserve the emulator. Modernize the desktop experience.</h1>
        <p className="summary">
          This shell will manage catalog, configuration, and supervised MAME sessions while video,
          audio, timing, and gameplay input stay native.
        </p>
      </header>

      <section className="status-card" aria-live="polite">
        <h2>Backend status</h2>
        {state.status === "idle" && <p>Waiting to connect…</p>}
        {state.status === "loading" && <p>Connecting to the Rust backend…</p>}
        {state.status === "ready" && (
          <dl>
            <div>
              <dt>Application</dt>
              <dd>{state.info.appVersion}</dd>
            </div>
            <div>
              <dt>Protocol</dt>
              <dd>v{state.info.protocolVersion}</dd>
            </div>
            <div>
              <dt>Backend</dt>
              <dd>{state.info.backend}</dd>
            </div>
          </dl>
        )}
        {state.status === "error" && <p className="error-message">{state.message}</p>}
      </section>

      <section className="next-card">
        <h2>Next vertical slice</h2>
        <p>Configure MAME → index a bounded catalog → launch → supervise → stop.</p>
      </section>
    </main>
  );
}
