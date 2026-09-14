import { useState } from "react";

import type { AppInfoResponse, MameVersionReport } from "../backend/types";
import { MachineBrowser } from "../browser/MachineBrowser";
import { LegacyDashboard } from "./LegacyDashboard";
import "./MameShell.css";

export type ShellWorkspace = "machines" | "legacy";

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

export function MameShell({
  info,
  availabilityRevision,
  onAvailabilityChanged,
}: {
  info: AppInfoResponse;
  availabilityRevision: number;
  onAvailabilityChanged: () => void;
}) {
  const [workspace, setWorkspace] = useState<ShellWorkspace>("machines");

  return (
    <main className="mame-shell">
      <header className="mame-shell__topbar">
        <div className="mame-shell__brand" aria-label="MAME Tauri frontend">
          <strong>MAME</strong>
          <span>Tauri frontend</span>
        </div>
        <nav className="mame-shell__nav" aria-label="Application views">
          <button
            type="button"
            className={workspace === "machines" ? "is-active" : undefined}
            aria-current={workspace === "machines" ? "page" : undefined}
            onClick={() => setWorkspace("machines")}
          >
            Machines
          </button>
          <button
            type="button"
            className={workspace === "legacy" ? "is-active" : undefined}
            aria-current={workspace === "legacy" ? "page" : undefined}
            onClick={() => setWorkspace("legacy")}
          >
            Legacy tools
          </button>
        </nav>
        <div className="mame-shell__identity" title={mameVersionLabel(info.mame)}>
          <span>{mameVersionLabel(info.mame)}</span>
          <code>app {info.appVersion}</code>
        </div>
      </header>

      <section className="mame-shell__workspace">
        {workspace === "machines" ? (
          <MachineBrowser availabilityRevision={availabilityRevision} />
        ) : (
          <LegacyDashboard
            availabilityRevision={availabilityRevision}
            onAvailabilityChanged={onAvailabilityChanged}
          />
        )}
      </section>

      <footer className="mame-shell__statusbar" aria-label="Application status">
        <span>Rust backend connected</span>
        <span>Catalog schema {info.databaseSchemaVersion}</span>
        <span>Runtime protocol {info.runtimeProtocolVersion}</span>
      </footer>
    </main>
  );
}
