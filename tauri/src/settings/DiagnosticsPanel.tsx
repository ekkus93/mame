import { useState } from "react";

import {
  exportDiagnosticsBundle,
  getDiagnostics,
  type DiagnosticsSnapshot,
} from "../backend/diagnostics";
import { errorMessage } from "../backend/errors";

function platformPathLabel(path: string | { encoding: string; data: string }): string {
  return typeof path === "string" ? path : `<${path.encoding}:${path.data}>`;
}

function runtimeLabel(snapshot: DiagnosticsSnapshot): string {
  const runtime = snapshot.runtime;
  if (runtime.status === "available") {
    return `${runtime.activeSource ?? "unknown"} · ${runtime.trust ?? "unknown trust"}`;
  }
  return [runtime.status, runtime.failureDomain, runtime.errorCode].filter(Boolean).join(" · ");
}

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
              <dt>Runtime</dt>
              <dd>{runtimeLabel(snapshot)}</dd>
            </div>
            <div>
              <dt>Gameplay transport</dt>
              <dd>{snapshot.gameplay.videoTransport}</dd>
            </div>
            <div>
              <dt>Gameplay session</dt>
              <dd>
                {snapshot.gameplay.session
                  ? `${snapshot.gameplay.session.machine} · ${snapshot.gameplay.session.state} · ${snapshot.gameplay.session.sessionId}`
                  : "No recorded session"}
              </dd>
            </div>
            <div>
              <dt>Video</dt>
              <dd>
                {snapshot.gameplay.video
                  ? `recv ${snapshot.gameplay.video.received} · drop ${snapshot.gameplay.video.dropped} · delivered ${snapshot.gameplay.video.delivered} · presented ${snapshot.gameplay.video.presented} · age ${snapshot.gameplay.video.latestAgeMs ?? "—"}ms`
                  : "No frame metrics"}
              </dd>
            </div>
            <div>
              <dt>Input</dt>
              <dd>
                {snapshot.gameplay.input.state} · {snapshot.gameplay.input.bridge} · max{" "}
                {snapshot.gameplay.input.maxUpdatesPerBatch} updates/batch
              </dd>
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
              <dt>Content paths</dt>
              <dd>
                {snapshot.contentPaths.total} effective · {snapshot.contentPaths.accessible}{" "}
                accessible · {snapshot.contentPaths.missing} missing ·{" "}
                {snapshot.contentPaths.notDirectory +
                  snapshot.contentPaths.permissionDenied +
                  snapshot.contentPaths.unreadable}{" "}
                invalid/inaccessible · {snapshot.contentPaths.resolutionPolicy}
                {snapshot.contentPaths.entries.length > 0 && (
                  <ul>
                    {snapshot.contentPaths.entries.map((entry, index) => (
                      <li key={`${entry.kind}-${entry.source}-${index}`}>
                        {entry.kind} · {entry.source} · {entry.validation.status} ·{" "}
                        <code>{platformPathLabel(entry.validation.path)}</code>
                      </li>
                    ))}
                  </ul>
                )}
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
