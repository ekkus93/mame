import { useEffect, useState } from "react";

import {
  getLibraryMachineAudit,
  runLibraryMachineAudit,
  type MachineAuditResponse,
  type MameAuditClassification,
  type MameAuditFacts,
} from "../backend/auditCommands";
import { errorMessage } from "../backend/errors";
import "./machineAudit.css";

type AuditState =
  | { status: "loading"; audit: null }
  | { status: "ready"; audit: MachineAuditResponse | null }
  | { status: "auditing"; audit: MachineAuditResponse | null }
  | { status: "error"; audit: MachineAuditResponse | null; message: string };

const CLASSIFICATION_LABELS: Record<MameAuditClassification, string> = {
  complete: "Complete",
  bestAvailable: "Best available",
  missingRequired: "Missing required content",
  incorrect: "Incorrect content",
  mixedFailure: "Missing and incorrect content",
  unknown: "Unknown / audit error",
};

const FACT_LABELS: Array<[keyof MameAuditFacts, string]> = [
  ["optionalMissing", "Optional content missing"],
  ["noGoodDumpKnown", "No good dump known"],
  ["needsRedump", "Content needs redump"],
  ["missingRequired", "Required content missing"],
  ["incorrectChecksum", "Incorrect checksum"],
  ["incorrectLength", "Incorrect length"],
  ["targetNotFound", "MAME target not found"],
];

export function MachineAuditPanel({
  shortName,
  onAuditResultChanged,
}: {
  shortName: string;
  onAuditResultChanged?: () => void;
}) {
  const [state, setState] = useState<AuditState>({ status: "loading", audit: null });

  useEffect(() => {
    let cancelled = false;
    setState({ status: "loading", audit: null });
    void getLibraryMachineAudit({ shortName })
      .then((audit) => {
        if (!cancelled) {
          setState({ status: "ready", audit });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({ status: "error", audit: null, message: errorMessage(error) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [shortName]);

  function runAudit() {
    if (state.status === "auditing" || state.status === "loading") {
      return;
    }
    const previous = state.audit;
    setState({ status: "auditing", audit: previous });
    void runLibraryMachineAudit({ shortName })
      .then((audit) => {
        setState({ status: "ready", audit });
        onAuditResultChanged?.();
      })
      .catch((error: unknown) =>
        setState({ status: "error", audit: previous, message: errorMessage(error) }),
      );
  }

  const factLabels = state.audit ? activeFactLabels(state.audit.result.facts) : [];
  const showDiagnostics =
    state.audit !== null &&
    state.audit.result.classification !== "complete" &&
    state.audit.result.rawExcerpt.trim().length > 0;

  return (
    <section className="machine-audit-panel" aria-labelledby={`audit-${shortName}`}>
      <div className="machine-audit-heading">
        <div>
          <p className="eyebrow">Media verification</p>
          <h4 id={`audit-${shortName}`}>MAME audit</h4>
        </div>
        <button
          type="button"
          className="secondary-button"
          disabled={state.status === "loading" || state.status === "auditing"}
          onClick={runAudit}
        >
          {state.status === "auditing" ? "Auditing…" : state.audit ? "Audit again" : "Audit media"}
        </button>
      </div>

      {state.status === "loading" && (
        <p className="detail-note">Checking for a current persisted audit result…</p>
      )}
      {state.status === "auditing" && (
        <p className="audit-progress" role="status">
          Running MAME media verification for {shortName}…
        </p>
      )}
      {state.status === "error" && (
        <p className="launch-error" role="alert">
          {state.message}
        </p>
      )}
      {state.status === "ready" && state.audit === null && (
        <p className="detail-note">
          This machine has not been audited for the current MAME and path configuration.
        </p>
      )}

      {state.audit && (
        <div className="audit-result" aria-live="polite">
          <div className="audit-result-summary">
            <strong>{CLASSIFICATION_LABELS[state.audit.result.classification]}</strong>
            <span>
              Audited {new Date(state.audit.auditedAtEpochMs).toLocaleString()}
              {state.audit.result.exitCode === null
                ? " · no process exit code"
                : ` · MAME exit ${state.audit.result.exitCode}`}
            </span>
          </div>

          {factLabels.length > 0 && (
            <ul className="audit-facts" aria-label="Audit details">
              {factLabels.map((label) => (
                <li key={label}>{label}</li>
              ))}
            </ul>
          )}

          {showDiagnostics && (
            <details className="audit-diagnostics">
              <summary>Raw MAME diagnostic excerpt</summary>
              <pre>{state.audit.result.rawExcerpt}</pre>
              {state.audit.result.rawTruncated && (
                <p className="detail-note">Diagnostic output was truncated to a bounded excerpt.</p>
              )}
            </details>
          )}
        </div>
      )}
    </section>
  );
}

function activeFactLabels(facts: MameAuditFacts): string[] {
  return FACT_LABELS.filter(([key]) => facts[key]).map(([, label]) => label);
}
