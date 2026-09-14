import { useReducer, useState } from "react";

import type { AppInfoResponse, MameVersionReport } from "../backend/types";
import { MameBrowser } from "../browser/MameBrowser";
import { BulkAuditPanel } from "../library/BulkAuditPanel";
import { CollectionManager } from "../library/CollectionManager";
import { LibraryBrowser } from "../library/LibraryBrowser";
import { RecentHistoryPanel } from "../library/RecentHistoryPanel";
import { SessionControlPanel } from "../session/SessionControlPanel";
import { DiagnosticsPanel } from "../settings/DiagnosticsPanel";
import { GeneralSettingsPanel } from "../settings/GeneralSettingsPanel";
import "./MameShell.css";

type ShellView =
  | "library"
  | "session"
  | "settings"
  | "audit"
  | "history"
  | "collections"
  | "diagnostics"
  | "legacy";

function mameVersionLabel(report: MameVersionReport): string {
  switch (report.status) {
    case "notConfigured":
      return "MAME not configured";
    case "available":
      return report.identity.rawVersionLine;
    case "unavailable":
      return `MAME unavailable: ${report.errorMessage}`;
  }
}

export function MameShell({ appInfo }: { appInfo: AppInfoResponse }) {
  const [view, setView] = useState<ShellView>("library");
  const [availabilityRevision, bumpAvailabilityRevision] = useReducer(
    (value: number) => value + 1,
    0,
  );

  const navigate = (next: ShellView) => () => setView(next);

  return (
    <main className="mame-shell">
      <header className="mame-shell-header">
        <div className="mame-brand" onClick={navigate("library")} role="presentation">
          <strong>MAME</strong>
          <span>Tauri frontend</span>
        </div>
        <nav className="mame-shell-nav" aria-label="Application views">
          <button
            type="button"
            aria-current={view === "library" ? "page" : undefined}
            onClick={navigate("library")}
          >
            Machines
          </button>
          <button
            type="button"
            aria-current={view === "session" ? "page" : undefined}
            onClick={navigate("session")}
          >
            Session
          </button>
          <button
            type="button"
            aria-current={view === "audit" ? "page" : undefined}
            onClick={navigate("audit")}
          >
            Audit
          </button>
          <button
            type="button"
            aria-current={view === "history" ? "page" : undefined}
            onClick={navigate("history")}
          >
            History
          </button>
          <button
            type="button"
            aria-current={view === "collections" ? "page" : undefined}
            onClick={navigate("collections")}
          >
            Collections
          </button>
          <button
            type="button"
            aria-current={view === "settings" ? "page" : undefined}
            onClick={navigate("settings")}
          >
            Settings
          </button>
          <button
            type="button"
            aria-current={view === "diagnostics" ? "page" : undefined}
            onClick={navigate("diagnostics")}
          >
            Diagnostics
          </button>
        </nav>
      </header>

      <section className="mame-shell-workspace">
        {view === "library" && (
          <MameBrowser
            availabilityRevision={availabilityRevision}
            onAuditResultsChanged={bumpAvailabilityRevision}
          />
        )}
        {view === "session" && <SessionControlPanel />}
        {view === "audit" && <BulkAuditPanel onAuditResultsChanged={bumpAvailabilityRevision} />}
        {view === "history" && <RecentHistoryPanel />}
        {view === "collections" && <CollectionManager />}
        {view === "settings" && (
          <GeneralSettingsPanel onContentPathsChanged={bumpAvailabilityRevision} />
        )}
        {view === "diagnostics" && <DiagnosticsPanel />}
        {view === "legacy" && (
          <div className="mame-legacy-stack" aria-label="Legacy dashboard during migration">
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
          </div>
        )}
      </section>

      <footer className="mame-status-bar">
        <span>{mameVersionLabel(appInfo.mame)}</span>
        <span>App {appInfo.appVersion}</span>
        <span>Backend {appInfo.backend}</span>
        <button type="button" className="mame-legacy-link" onClick={navigate("legacy")}>
          Legacy UI
        </button>
      </footer>
    </main>
  );
}
