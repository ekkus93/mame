import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useReducer } from "react";

import { getAppInfo } from "./backend/commands";
import { errorMessage } from "./backend/errors";
import type { MameVersionReport } from "./backend/types";
import { BulkAuditPanel } from "./library/BulkAuditPanel";
import { CollectionManager } from "./library/CollectionManager";
import { LibraryBrowser } from "./library/LibraryBrowser";
import { RecentHistoryPanel } from "./library/RecentHistoryPanel";
import { SessionControlPanel } from "./session/SessionControlPanel";
import { DiagnosticsPanel } from "./settings/DiagnosticsPanel";
import { GeneralSettingsPanel } from "./settings/GeneralSettingsPanel";
import { appStateReducer, initialAppState } from "./state/appState";
import "./App.css";

function mameVersionLabel(report: MameVersionReport): string {
  switch (report.status) {
    case "notConfigured":
      return "Not configured";
    case "available":
      return report.identity.rawVersionLine;
    case "unavailable":
      return `Unavailable (${report.errorCode}): ${report.errorMessage}`;
  }
}

export default function App() {
  const [state, dispatch] = useReducer(appStateReducer, initialAppState);
  const [availabilityRevision, bumpAvailabilityRevision] = useReducer(
    (value: number) => value + 1,
    0,
  );

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

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    const refreshShortcutOwnership = () => {
      if (!disposed) {
        window.dispatchEvent(new Event("focus"));
      }
    };

    const bind = async () => {
      for (const eventName of [
        "session.started",
        "session.exited",
        "session.crashed",
        "session.failed",
      ] as const) {
        unlisteners.push(await listen(eventName, refreshShortcutOwnership));
      }

      if (disposed) {
        unlisteners.splice(0).forEach((unlisten) => unlisten());
      }
    };

    void bind().catch(() => {
      // Library shortcuts already fail closed while ownership cannot be refreshed.
    });

    return () => {
      disposed = true;
      unlisteners.splice(0).forEach((unlisten) => unlisten());
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
              <code>
                app v{state.info.appVersion} · api v{state.info.protocolVersion}
              </code>
            </>
          )}
          {state.status === "error" && <span className="error-message">{state.message}</span>}
        </section>
      </header>

      {state.status === "ready" && (
        <details className="version-report">
          <summary>Version information</summary>
          <dl className="version-grid">
            <div>
              <dt>Application</dt>
              <dd>v{state.info.appVersion}</dd>
            </div>
            <div>
              <dt>Build</dt>
              <dd>
                <code>{state.info.build.gitSha ?? "git unavailable"}</code>
                {" · "}
                {state.info.build.profile}
                {" · "}
                {state.info.build.target}
              </dd>
            </div>
            <div>
              <dt>MAME</dt>
              <dd>{mameVersionLabel(state.info.mame)}</dd>
            </div>
            <div>
              <dt>Catalog schema</dt>
              <dd>v{state.info.databaseSchemaVersion}</dd>
            </div>
            <div>
              <dt>Settings schema</dt>
              <dd>v{state.info.settingsSchemaVersion}</dd>
            </div>
            <div>
              <dt>Runtime protocol</dt>
              <dd>v{state.info.runtimeProtocolVersion}</dd>
            </div>
          </dl>
        </details>
      )}

      {state.status === "ready" ? (
        <>
          <GeneralSettingsPanel onContentPathsChanged={bumpAvailabilityRevision} />
          <DiagnosticsPanel />
          <SessionControlPanel />
          <BulkAuditPanel onAuditResultsChanged={bumpAvailabilityRevision} />
          <LibraryBrowser
            availabilityRevision={availabilityRevision}
            onAuditResultsChanged={bumpAvailabilityRevision}
          />
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
