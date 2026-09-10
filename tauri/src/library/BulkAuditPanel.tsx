import { useCallback, useEffect, useMemo, useState } from "react";

import {
  cancelLibraryBulkAudit,
  getLibraryBulkAuditStatus,
  startLibraryBulkAudit,
  type BulkAuditStatus,
} from "../backend/auditCommands";
import { errorMessage } from "../backend/errors";
import "./bulkAudit.css";

const POLL_INTERVAL_MS = 500;

function isActive(status: BulkAuditStatus | null): boolean {
  return status?.state === "running" || status?.state === "cancelling";
}

export function BulkAuditPanel() {
  const [status, setStatus] = useState<BulkAuditStatus | null>(null);
  const [parallelism, setParallelism] = useState(2);
  const [commandError, setCommandError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const next = await getLibraryBulkAuditStatus();
      setStatus(next);
      setCommandError(null);
    } catch (error: unknown) {
      setCommandError(errorMessage(error));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!isActive(status)) {
      return undefined;
    }
    const timer = window.setInterval(() => {
      void refresh();
    }, POLL_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [refresh, status]);

  const processed = (status?.completed ?? 0) + (status?.failed ?? 0);
  const progressLabel = useMemo(() => {
    if (!status) {
      return "Loading bulk audit status…";
    }
    if (status.state === "running" && status.total === 0) {
      return "Preparing the runnable-machine audit queue…";
    }
    if (status.total === 0) {
      return "No runnable machines are queued.";
    }
    return `${processed.toLocaleString()} of ${status.total.toLocaleString()} processed; ${status.completed.toLocaleString()} persisted, ${status.failed.toLocaleString()} failed.`;
  }, [processed, status]);

  function startAudit() {
    setCommandError(null);
    void startLibraryBulkAudit({ maxParallelism: parallelism })
      .then(setStatus)
      .catch((error: unknown) => setCommandError(errorMessage(error)));
  }

  function cancelAudit() {
    setCommandError(null);
    void cancelLibraryBulkAudit()
      .then(setStatus)
      .catch((error: unknown) => setCommandError(errorMessage(error)));
  }

  const active = isActive(status);
  const canRestart =
    status?.state === "cancelled" || status?.state === "completed" || status?.state === "failed";

  return (
    <section className="bulk-audit-panel" aria-labelledby="bulk-audit-heading">
      <div className="bulk-audit-heading-row">
        <div>
          <p className="eyebrow">Media verification</p>
          <h3 id="bulk-audit-heading">Bulk audit</h3>
        </div>
        {status?.jobId !== null && status?.jobId !== undefined && (
          <span className="bulk-audit-job">Job {status.jobId}</span>
        )}
      </div>

      <p className="detail-note">
        Audit every runnable machine with MAME. Results are committed per machine, so a cancelled or
        interrupted run can be restarted safely without a partial batch transaction.
      </p>

      <div className="bulk-audit-controls">
        <label>
          <span>Concurrent MAME processes</span>
          <select
            value={parallelism}
            disabled={active}
            onChange={(event) => setParallelism(Number(event.target.value))}
          >
            <option value={1}>1</option>
            <option value={2}>2</option>
            <option value={3}>3</option>
            <option value={4}>4</option>
          </select>
        </label>
        <button type="button" disabled={active} onClick={startAudit}>
          {canRestart ? "Restart bulk audit" : "Start bulk audit"}
        </button>
        {active && (
          <button
            type="button"
            className="secondary-button"
            disabled={status?.state === "cancelling"}
            onClick={cancelAudit}
          >
            {status?.state === "cancelling" ? "Cancelling…" : "Cancel"}
          </button>
        )}
      </div>

      {status && status.state !== "idle" && (
        <div className="bulk-audit-progress" aria-live="polite">
          <progress max={Math.max(status.total, 1)} value={Math.min(processed, status.total)} />
          <span>{progressLabel}</span>
          {status.active > 0 && <span>{status.active} audit process(es) currently active.</span>}
          {status.lastMachine && <span>Last machine: {status.lastMachine}</span>}
          {status.state === "cancelling" && (
            <span>
              New work is stopped; already-running audits are allowed to finish or time out.
            </span>
          )}
          {status.lastError && <span className="launch-error">Last error: {status.lastError}</span>}
        </div>
      )}

      {commandError && (
        <p className="launch-error" role="alert">
          {commandError}
        </p>
      )}
    </section>
  );
}
