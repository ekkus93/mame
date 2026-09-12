import { useCallback, useEffect, useState } from "react";

import { errorMessage } from "../backend/errors";
import {
  deleteSaveStateRecord,
  listSaveStateRecords,
  loadKnownSaveState,
  saveKnownState,
  type StoredSaveStateRecord,
} from "../backend/saveStateRecords";
import type { SessionSnapshot } from "../backend/types";
import { saveStateCompatibility } from "./saveStateCompatibility";
import "./SaveStateBrowser.css";

const PAGE_LIMIT = 100;

export function SaveStateBrowser({
  session,
}: {
  session: SessionSnapshot | null;
}) {
  const [items, setItems] = useState<StoredSaveStateRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [slot, setSlot] = useState("quick");
  const [loading, setLoading] = useState(true);
  const [operation, setOperation] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    const page = await listSaveStateRecords({ limit: PAGE_LIMIT, offset: 0 });
    setItems(page.items);
    setTotal(page.total);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    void refresh()
      .catch((error: unknown) => {
        if (!cancelled) setFailure(errorMessage(error));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [refresh, session?.sessionId]);

  const run = async (key: string, action: () => Promise<string>) => {
    setOperation(key);
    setFailure(null);
    setNotice(null);
    try {
      setNotice(await action());
      await refresh();
    } catch (error: unknown) {
      setFailure(errorMessage(error));
    } finally {
      setOperation(null);
    }
  };

  const save = () => {
    if (!session || session.state !== "running") return;
    void run("save", async () => {
      const saved = await saveKnownState({
        sessionId: session.sessionId,
        slot,
      });
      return `Saved ${saved.record.machine} to slot ${saved.record.slot}.`;
    });
  };

  const load = (item: StoredSaveStateRecord) => {
    if (!session || !saveStateCompatibility(item, session).loadable) return;
    void run(`load-${item.id}`, async () => {
      await loadKnownSaveState({
        sessionId: session.sessionId,
        recordId: item.id,
      });
      return `Loaded slot ${item.record.slot} after runtime compatibility validation.`;
    });
  };

  const confirmDelete = (item: StoredSaveStateRecord) => {
    void run(`delete-${item.id}`, async () => {
      const result = await deleteSaveStateRecord({ recordId: item.id });
      setConfirmDeleteId(null);
      return result.fileDeleted
        ? `Deleted slot ${item.record.slot} and its state file.`
        : `Removed the stale record for slot ${item.record.slot}; its file was already missing.`;
    });
  };

  const busy = operation !== null;

  return (
    <section
      className="status-card save-state-browser"
      aria-labelledby="save-state-browser-heading"
    >
      <p className="eyebrow">Save states</p>
      <h2 id="save-state-browser-heading">State browser</h2>
      <p className="save-state-policy">
        A matching MAME version is not treated as proof of compatibility. Every
        load performs the runtime structural compatibility check before
        restoration.
      </p>

      <div className="save-state-create-row">
        <label>
          Slot
          <input
            value={slot}
            maxLength={32}
            onChange={(event) => setSlot(event.target.value)}
            disabled={busy}
            aria-label="Save-state slot"
          />
        </label>
        <button
          type="button"
          onClick={save}
          disabled={!session || session.state !== "running" || busy}
        >
          {operation === "save" ? "Saving…" : "Save current state"}
        </button>
      </div>

      {failure && (
        <p className="error-message" role="alert">
          {failure}
        </p>
      )}
      {notice && (
        <p className="save-state-notice" aria-live="polite">
          {notice}
        </p>
      )}

      {loading ? (
        <p>Loading known save states…</p>
      ) : items.length === 0 ? (
        <p>No application-managed save states yet.</p>
      ) : (
        <>
          <p className="save-state-count">
            Showing {items.length} of {total} known state
            {total === 1 ? "" : "s"}.
          </p>
          <ul className="save-state-list">
            {items.map((item) => {
              const compatibility = saveStateCompatibility(item, session);
              const deleting = confirmDeleteId === item.id;
              const itemBusy =
                operation === `load-${item.id}` ||
                operation === `delete-${item.id}`;
              return (
                <li key={item.id} className="save-state-item">
                  <div>
                    <strong>{item.record.machine}</strong>
                    {item.record.software ? ` · ${item.record.software}` : ""}
                    <span> · slot {item.record.slot}</span>
                  </div>
                  <div className="save-state-meta">
                    {new Date(item.record.savedAtEpochMs).toLocaleString()} ·{" "}
                    {formatBytes(item.record.bytes)} · MAME {item.record.mame.version}
                    {item.record.mame.build
                      ? ` (${item.record.mame.build})`
                      : ""}
                  </div>
                  <p
                    className={`save-state-compatibility ${compatibility.tone}`}
                  >
                    {compatibility.message}
                  </p>
                  <div className="save-state-actions">
                    <button
                      type="button"
                      onClick={() => load(item)}
                      disabled={!compatibility.loadable || busy}
                    >
                      {operation === `load-${item.id}` ? "Loading…" : "Load"}
                    </button>
                    {deleting ? (
                      <>
                        <span className="save-state-delete-warning">
                          Delete this state file permanently?
                        </span>
                        <button
                          type="button"
                          onClick={() => confirmDelete(item)}
                          disabled={busy}
                        >
                          {operation === `delete-${item.id}`
                            ? "Deleting…"
                            : "Confirm delete"}
                        </button>
                        <button
                          type="button"
                          onClick={() => setConfirmDeleteId(null)}
                          disabled={busy}
                        >
                          Cancel
                        </button>
                      </>
                    ) : (
                      <button
                        type="button"
                        onClick={() => setConfirmDeleteId(item.id)}
                        disabled={busy || itemBusy}
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </li>
              );
            })}
          </ul>
        </>
      )}
    </section>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}
