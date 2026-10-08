import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useReducer, useState } from "react";

import { getAppInfo, getMameSession, stopMame } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import {
  SESSION_CRASHED_EVENT,
  SESSION_EXITED_EVENT,
  SESSION_FAILED_EVENT,
  SESSION_STARTED_EVENT,
} from "../backend/events";
import type { AppInfoResponse, SessionLifecycleEventV1, SessionSnapshot } from "../backend/types";
import { MameBrowser } from "../browser/MameBrowser";
import { BulkAuditPanel } from "../library/BulkAuditPanel";
import { CollectionManager } from "../library/CollectionManager";
import { isGameplaySessionState } from "../library/keyboardNavigation";
import { RecentHistoryPanel } from "../library/RecentHistoryPanel";
import { SessionControlPanel } from "../session/SessionControlPanel";
import { libraryReturnAction } from "./sessionNavigation";
import { DiagnosticsPanel } from "../settings/DiagnosticsPanel";
import { GeneralSettingsPanel } from "../settings/GeneralSettingsPanel";
import "./MameShell.css";
import "./ContextualSurfaces.css";

type ShellView =
  "library" | "session" | "settings" | "audit" | "history" | "collections" | "diagnostics";

function activeSession(session: SessionSnapshot | null): SessionSnapshot | null {
  return session && isGameplaySessionState(session.state) ? session : null;
}

export function MameShell({ appInfo }: { appInfo: AppInfoResponse }) {
  const [view, setView] = useState<ShellView>("library");
  const [mameReport, setMameReport] = useState(appInfo.mame);
  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [gameplayInputOwned, setGameplayInputOwned] = useState(true);
  const [returningToLibrary, setReturningToLibrary] = useState(false);
  const [returnError, setReturnError] = useState<string | null>(null);
  const [availabilityRevision, bumpAvailabilityRevision] = useReducer(
    (value: number) => value + 1,
    0,
  );

  const observeSession = (next: SessionSnapshot | null) => {
    const active = activeSession(next);
    setSession(active);
    setGameplayInputOwned(isGameplaySessionState(next?.state));
    if (active) setView("session");
  };

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    void getMameSession()
      .then((current) => {
        if (!disposed) observeSession(current);
      })
      .catch(() => {
        if (!disposed) {
          setSession(null);
          setGameplayInputOwned(true);
        }
      });

    const bind = async () => {
      unlisteners.push(
        await listen<SessionLifecycleEventV1>(SESSION_STARTED_EVENT, (event) => {
          if (!disposed) observeSession(event.payload.session);
        }),
      );
      for (const eventName of [
        SESSION_EXITED_EVENT,
        SESSION_CRASHED_EVENT,
        SESSION_FAILED_EVENT,
      ] as const) {
        unlisteners.push(
          await listen<SessionLifecycleEventV1>(eventName, () => {
            if (!disposed) {
              setSession(null);
              setGameplayInputOwned(false);
              setView("library");
            }
          }),
        );
      }
      if (disposed) unlisteners.splice(0).forEach((unlisten) => unlisten());
    };

    void bind().catch(() => {
      if (!disposed) setGameplayInputOwned(true);
    });

    return () => {
      disposed = true;
      unlisteners.splice(0).forEach((unlisten) => unlisten());
    };
  }, []);

  const returnToLibrary = () => {
    if (returningToLibrary) return;
    const action = libraryReturnAction(view, session?.state ?? null);
    if (action === "wait") {
      setReturnError("Wait for the MAME session to finish starting or stopping before returning.");
      return;
    }
    if (action === "stop" && session) {
      const sessionId = session.sessionId;
      setReturningToLibrary(true);
      setReturnError(null);
      void stopMame({ sessionId })
        .then(() => {
          setSession(null);
          setGameplayInputOwned(false);
          setView("library");
          void getAppInfo()
            .then((info) => setMameReport(info.mame))
            .catch(() => undefined);
        })
        .catch((error: unknown) => setReturnError(errorMessage(error)))
        .finally(() => setReturningToLibrary(false));
      return;
    }
    setReturnError(null);
    setView("library");
    void getAppInfo()
      .then((info) => setMameReport(info.mame))
      .catch(() => undefined);
  };

  const activeSessionSoftware = session?.software ? ` · ${session.software}` : "";
  const activeSessionLabel = session
    ? `Session: ${session.machine}${activeSessionSoftware} · ${session.state}`
    : "Session";

  return (
    <main className="mame-shell">
      <section className="mame-shell-workspace" aria-label="MAME machine browser">
        {view === "library" && (
          <MameBrowser
            availabilityRevision={availabilityRevision}
            gameplayInputOwned={gameplayInputOwned}
            mame={mameReport}
            onAuditResultsChanged={bumpAvailabilityRevision}
            onConfigureOptions={() => setView("settings")}
            onOpenAudit={() => setView("audit")}
            onOpenCollections={() => setView("collections")}
            onOpenDiagnostics={() => setView("diagnostics")}
            onOpenHistory={() => setView("history")}
            onOpenSession={() => setView("session")}
            sessionLabel={activeSessionLabel}
            onSessionStarted={observeSession}
          />
        )}
        {view !== "library" && (
          <section className="mame-secondary-surface" aria-label="Secondary MAME tool surface">
            <button
              type="button"
              className="mame-secondary-back"
              disabled={returningToLibrary}
              onClick={returnToLibrary}
            >
              {returningToLibrary ? "Stopping MAME…" : "← Machine Selection"}
            </button>
            {returnError && (
              <p className="error-message" role="alert">
                {returnError}
              </p>
            )}
            {view === "session" && <SessionControlPanel />}
            {view === "settings" && (
              <GeneralSettingsPanel
                mame={mameReport}
                onContentPathsChanged={bumpAvailabilityRevision}
              />
            )}
            {view === "audit" && (
              <BulkAuditPanel onAuditResultsChanged={bumpAvailabilityRevision} />
            )}
            {view === "history" && <RecentHistoryPanel />}
            {view === "collections" && <CollectionManager />}
            {view === "diagnostics" && <DiagnosticsPanel />}
          </section>
        )}
      </section>
    </main>
  );
}
