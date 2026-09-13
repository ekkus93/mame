import { useState } from "react";

import {
  exportDiagnosticsBundle,
  getDiagnostics,
  type DiagnosticsSnapshot,
} from "../backend/diagnostics";
import { errorMessage } from "../backend/errors";

function mameLabel(snapshot: DiagnosticsSnapshot): string {
  switch (snapshot.app.mame.status) {
    case "notConfigured":
      return "Not configured";
    case "available":
      return `${snapshot.app.mame.identity.rawVersionLine} · ${snapshot.app.mame.identity.path}`;
    case "unavailable":
      return `Unavailable (${snapshot.app.mame.errorCode})`;
  }
}

export function DiagnosticsPanel() {
  const [snapshot, setSnapshot] = useState<DiagnosticsSnapshot | null>(null);
  const [status, setStatus] = useState<string>("");
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setBusy(true);
    setStatus("");
    try {
      setSnapshot(await getDiagnostics());
    } catch (error: unknown) {
      setStatus(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function copyBundle() {
    setBusy(true);
    setStatus("");
    try {
      const value = snapshot ?? (await getDiagnostics());
      await navigator.clipboard.writeText(JSON.stringify(value, null, 2));
      setSnapshot(value);
      setStatus("Diagnostics copied to the clipboard.");
    } catch (error: unknown) {
      setStatus(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function saveBundle() {
    setBusy(true);
    setStatus("");
    try {
      const result = await exportDiagnosticsBundle();
      setStatus(`Saved diagnostics bundle to ${result.path}`);
      setSnapshot(await getDiagnostics());
    } catch (error: unknown) {
      setStatus(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <details className="version-report">
      <summary>Diagnostics and support</summary>
      <p className="summary">
        Review the current runtime identity and bounded, sanitized backend log tail before exporting
        a support bundle.
      </p>
      <div className="toolbar" role="group" aria-label="Diagnostics actions">
        <button type="button" onClick={() => void refresh()} disabled={busy}>
          Refresh
        </button>
        <button type="button" onClick={() => void copyBundle()} disabled={busy}>
          Copy bundle
        </button>
        <button type="button" onClick={() => void saveBundle()} disabled={busy}>
          Save bundle
        </button>
      </div>
      {status && <p aria-live="polite">{status}</p>}
      {snapshot && (
        <>
          <dl className="version-grid">
            <div>
              <dt>Application</dt>
              <dd>v{snapshot.app.appVersion}</dd>
            </div>
            <div>
              <dt>Platform</dt>
              <dd>
                {snapshot.platform} · {snapshot.architecture}
              </dd>
            </div>
            <div>
              <dt>MAME</dt>
              <dd>{mameLabel(snapshot)}</dd>
            </div>
            <div>
              <dt>Settings</dt>
              <dd>
                <code>{snapshot.settingsPath}</code>
              </dd>
            </div>
            <div>
              <dt>Catalog</dt>
              <dd>
                <code>{snapshot.catalogPath}</code>
              </dd>
            </div>
            <div>
              <dt>Log</dt>
              <dd>
                <code>{snapshot.logPath}</code>
              </dd>
            </div>
          </dl>
          <p>
            {snapshot.recentLogs.length} recent sanitized log entries retained in this snapshot.
          </p>
        </>
      )}
    </details>
  );
}
