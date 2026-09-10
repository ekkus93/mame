import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import { BulkAuditPanel } from "./library/BulkAuditPanel";
import { CollectionManager } from "./library/CollectionManager";
import { LibraryBrowser } from "./library/LibraryBrowser";
import { RecentHistoryPanel } from "./library/RecentHistoryPanel";
import { PathConfigurationPanel } from "./settings/PathConfigurationPanel";
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
        <div>
          <p className="eyebrow">MAME Tauri</p>
          <h1>Machine Library</h1>
          <p className="summary">
            Browse the local MAME catalog while emulation, video, audio, timing, and gameplay input
            remain native.
          </p>
        </div>

        <section className="backend-status" aria-live="polite" aria-label="Backend status">
          {state.status === "idle" && <span>Waiting for backend</span>}
          {state.status === "loading" && <span>Connecting…</span>}
          {state.status === "ready" && (
            <>
              <span className="backend-dot" aria-hidden="true" />
              <span>Rust backend connected</span>
              <code>v{state.info.protocolVersion}</code>
            </>
          )}
          {state.status === "error" && <span className="error-message">{state.message}</span>}
        </section>
      </header>

      {state.status === "ready" ? (
        <>
          <PathConfigurationPanel />
          <BulkAuditPanel />
          <LibraryBrowser />
          <RecentHistoryPanel />
          <CollectionManager />
        </>
      ) : state.status === "error" ? (
        <section className="fatal-error" role="alert">
          <h2>Backend unavailable</h2>
          <p>{state.message}</p>
        </section>
      ) : (
        <section className="status-card" aria-live="polite">
          <h2>Starting application</h2>
          <p>Connecting to the trusted Rust backend…</p>
        </section>
      )}
    </main>
  );
}
