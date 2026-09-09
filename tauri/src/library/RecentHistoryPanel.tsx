import { useCallback, useEffect, useMemo, useState } from "react";

import { errorMessage } from "../backend/errors";
import { queryLibraryHistory, type RecentHistoryPage } from "../backend/history";
import "./recentHistory.css";

const HISTORY_PAGE_SIZE = 50;

type LoadState =
  | { status: "loading" }
  | { status: "ready"; page: RecentHistoryPage }
  | { status: "error"; message: string };

export function RecentHistoryPanel() {
  const [offset, setOffset] = useState(0);
  const [revision, setRevision] = useState(0);
  const [state, setState] = useState<LoadState>({ status: "loading" });

  const load = useCallback(() => {
    let cancelled = false;
    setState({ status: "loading" });
    void queryLibraryHistory(HISTORY_PAGE_SIZE, offset)
      .then((page) => {
        if (!cancelled) {
          if (offset > 0 && page.items.length === 0 && page.total > 0) {
            setOffset(
              Math.max(0, page.total - (page.total % HISTORY_PAGE_SIZE || HISTORY_PAGE_SIZE)),
            );
            return;
          }
          setState({ status: "ready", page });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({ status: "error", message: errorMessage(error) });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [offset]);

  useEffect(() => load(), [load, revision]);

  const page = state.status === "ready" ? state.page : null;
  const range = useMemo(() => {
    if (!page || page.total === 0) {
      return null;
    }
    const first = page.offset + 1;
    const last = Math.min(page.offset + page.items.length, page.total);
    return `${first.toLocaleString()}–${last.toLocaleString()} of ${page.total.toLocaleString()}`;
  }, [page]);

  return (
    <section className="history-panel" aria-labelledby="history-heading">
      <div className="history-heading-row">
        <div>
          <p className="eyebrow">Launch activity</p>
          <h2 id="history-heading">Recents</h2>
          <p className="history-summary">
            The newest {page?.retentionLimit ?? 500} launch attempts are kept locally.
          </p>
        </div>
        <div className="history-heading-actions">
          {range && <span>{range}</span>}
          <button
            type="button"
            className="secondary-button"
            onClick={() => setRevision((current) => current + 1)}
          >
            Refresh
          </button>
        </div>
      </div>

      {state.status === "loading" && (
        <div className="history-state" aria-live="polite">
          <strong>Loading recent launches…</strong>
          <span>Reading the local play-history index.</span>
        </div>
      )}

      {state.status === "error" && (
        <div className="history-state history-error" role="alert">
          <strong>Recent history unavailable</strong>
          <span>{state.message}</span>
          <button type="button" onClick={() => setRevision((current) => current + 1)}>
            Retry
          </button>
        </div>
      )}

      {page && page.items.length === 0 && (
        <div className="history-state">
          <strong>No launch history yet</strong>
          <span>Launch a machine and its attempt will appear here.</span>
        </div>
      )}

      {page && page.items.length > 0 && (
        <ul className="history-list" aria-label="Recent MAME launch attempts">
          {page.items.map((entry) => (
            <li key={entry.id} className="history-row">
              <div className="history-primary">
                <strong>{entry.machine?.description ?? entry.machineShortName}</strong>
                <code>{entry.machineShortName}</code>
                {entry.softwareItem && <span>Software: {entry.softwareItem}</span>}
                {!entry.machine && <span className="history-stale">Not in active catalog</span>}
              </div>
              <div className="history-meta">
                <time dateTime={new Date(entry.launchedAtEpochMs).toISOString()}>
                  {formatTimestamp(entry.launchedAtEpochMs)}
                </time>
                <span className={`history-outcome ${outcomeClass(entry.succeeded)}`}>
                  {outcomeLabel(entry.succeeded)}
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}

      {page && page.total > page.limit && (
        <nav className="history-pagination" aria-label="Recent history pages">
          <button
            type="button"
            className="secondary-button"
            disabled={page.offset === 0}
            onClick={() => setOffset(Math.max(0, page.offset - page.limit))}
          >
            Previous
          </button>
          <button
            type="button"
            className="secondary-button"
            disabled={page.offset + page.items.length >= page.total}
            onClick={() => setOffset(page.offset + page.limit)}
          >
            Next
          </button>
        </nav>
      )}
    </section>
  );
}

function outcomeLabel(succeeded: boolean | null): string {
  if (succeeded === true) {
    return "Started";
  }
  if (succeeded === false) {
    return "Failed to start";
  }
  return "Outcome pending";
}

function outcomeClass(succeeded: boolean | null): string {
  if (succeeded === true) {
    return "history-outcome-success";
  }
  if (succeeded === false) {
    return "history-outcome-failure";
  }
  return "history-outcome-pending";
}

function formatTimestamp(epochMs: number): string {
  const timestamp = new Date(epochMs);
  if (Number.isNaN(timestamp.getTime())) {
    return `Timestamp ${epochMs}`;
  }
  return timestamp.toLocaleString();
}
