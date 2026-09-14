import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useReducer, useState } from "react";

import { getMameSession } from "../backend/commands";
import {
  SESSION_CRASHED_EVENT,
  SESSION_EXITED_EVENT,
  SESSION_FAILED_EVENT,
  SESSION_STARTED_EVENT,
} from "../backend/events";
import type {
  AppInfoResponse,
  MameVersionReport,
  SessionLifecycleEventV1,
  SessionSnapshot,
} from "../backend/types";
import { MameBrowser } from "../browser/MameBrowser";
import { BulkAuditPanel } from "../library/BulkAuditPanel";
import { CollectionManager } from "../library/CollectionManager";
import { RecentHistoryPanel } from "../library/RecentHistoryPanel";
import { SessionControlPanel } from "../session/SessionControlPanel";
import { DiagnosticsPanel } from "../settings/DiagnosticsPanel";
import { GeneralSettingsPanel } from "../settings/GeneralSettingsPanel";
import "./MameShell.css";
import "./ContextualSurfaces.css";

type ShellView =
  "library" | "session" | "settings" | "audit" | "history" | "collections" | "diagnostics";

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

function activeSession(session: SessionSnapshot | null): SessionSnapshot | null {
  return session && ["created", "starting", "running", "stopping"].includes(session.state)
    ? session
    : null;
}

export function MameShell({ appInfo }: { appInfo: AppInfoResponse }) {
  const [view, setView] = useState<ShellView>("library");
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [availabilityRevision, bumpAvailabilityRevision] = useReducer(
    (value: number) => value + 1,
    0,
  );

  const navigate = (next: ShellView) => () => setView(next);

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    void getMameSession()
      .then((current) => {
        if (!disposed) setSession(activeSession(current));
      })
      .catch(() => {
        if (!disposed) setSession(null);
      });

    const bind = async () => {
      unlisteners.push(
        await listen<SessionLifecycleEventV1>(SESSION_STARTED_EVENT, (event) => {
          if (!disposed) setSession(activeSession(event.payload.session));
        }),
      );
      for (const eventName of [
        SESSION_EXITED_EVENT,
        SESSION_CRASHED_EVENT,
        SESSION_FAILED_EVENT,
      ] as const) {
        unlisteners.push(
          await listen<SessionLifecycleEventV1>(eventName, () => {
            if (!disposed) setSession(null);
          }),
        );
      }
      if (disposed) unlisteners.splice(0).forEach((unlisten) => unlisten());
    };

    void bind().catch(() => {
      // The browser already fails closed for gameplay-input ownership if session state is unknown.
    });

    return () => {
      disposed = true;
      unlisteners.splice(0).forEach((unlisten) => unlisten());
    };
  }, []);

  return (
    <main className="mame-shell">
      <header className="mame-shell-header">
        <button type="button" className="mame-brand" onClick={navigate("library")}>
          <strong>MAME</strong>
          <span>Tauri frontend</span>
        </button>
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
      </section>

      <footer className="mame-status-bar">
        <button type="button" className="mame-session-status" onClick={navigate("session")}>
          {session
            ? `Session: ${session.machine}${session.software ? ` · ${session.software}` : ""} · ${session.state}`
            : "No active MAME session"}
        </button>
        <span>{mameVersionLabel(appInfo.mame)}</span>
        <span>App {appInfo.appVersion}</span>
        <span>Backend {appInfo.backend}</span>
      </footer>
    </main>
  );
}
