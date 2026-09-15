import { useCallback, useEffect, useState } from "react";

import { getMameUiBootstrapStatus, refreshConfiguredMameMetadata } from "../backend/mameBootstrap";
import { errorMessage } from "../backend/errors";
import {
  beginMetadataImport,
  bootstrapFromBackend,
  catalogIsReady,
  failMetadataImport,
  type MameBrowserBootstrapState,
} from "./bootstrapModel";
import { MameBrowser } from "./MameBrowser";

export function MameBrowserWorkspace({
  availabilityRevision,
  onAuditResultsChanged,
  onConfigureMame,
  onDiagnostics,
}: {
  availabilityRevision: number;
  onAuditResultsChanged: () => void;
  onConfigureMame: () => void;
  onDiagnostics: () => void;
}) {
  const [bootstrap, setBootstrap] = useState<MameBrowserBootstrapState>({ status: "checking" });

  const refreshBootstrap = useCallback(async () => {
    setBootstrap({ status: "checking" });
    try {
      setBootstrap(bootstrapFromBackend(await getMameUiBootstrapStatus()));
    } catch (reason: unknown) {
      setBootstrap({ status: "checkFailed", message: errorMessage(reason) });
    }
  }, []);

  useEffect(() => {
    void refreshBootstrap();
  }, [refreshBootstrap]);

  const importMetadata = useCallback(async () => {
    let importStarted = false;
    setBootstrap((current) => {
      const next = beginMetadataImport(current);
      importStarted = next.status === "importing";
      return next;
    });
    if (!importStarted) return;

    try {
      await refreshConfiguredMameMetadata();
      setBootstrap(bootstrapFromBackend(await getMameUiBootstrapStatus()));
    } catch (reason: unknown) {
      const message = errorMessage(reason);
      setBootstrap((current) => failMetadataImport(current, message));
    }
  }, []);

  if (catalogIsReady(bootstrap)) {
    return (
      <MameBrowser
        availabilityRevision={availabilityRevision}
        onAuditResultsChanged={onAuditResultsChanged}
      />
    );
  }

  const importSource =
    bootstrap.status === "importing" || bootstrap.status === "importFailed"
      ? bootstrap.previous
      : bootstrap.status === "metadataMissing" || bootstrap.status === "metadataStale"
        ? bootstrap
        : null;

  return (
    <section
      className="mame-browser mame-bootstrap-browser"
      aria-label="MAME setup and catalog status"
    >
      <div className={`mame-bootstrap-state is-${bootstrap.status}`}>
        {bootstrap.status === "checking" && (
          <>
            <p className="mame-bootstrap-kicker">Machine catalog</p>
            <h1>Checking MAME setup…</h1>
            <p>Verifying the configured MAME executable and active machine metadata.</p>
          </>
        )}

        {bootstrap.status === "checkFailed" && (
          <>
            <p className="mame-bootstrap-kicker">Machine catalog</p>
            <h1>Setup status unavailable</h1>
            <p>{bootstrap.message}</p>
            <div className="mame-bootstrap-actions">
              <button type="button" autoFocus onClick={() => void refreshBootstrap()}>
                Retry status check
              </button>
              <button type="button" className="secondary-button" onClick={onDiagnostics}>
                Diagnostics
              </button>
            </div>
          </>
        )}

        {bootstrap.status === "notConfigured" && (
          <>
            <p className="mame-bootstrap-kicker">Setup required</p>
            <h1>MAME is not configured</h1>
            <p>
              Choose the MAME executable before importing its machine list. This is a setup state,
              not an empty machine catalog.
            </p>
            <div className="mame-bootstrap-actions">
              <button type="button" autoFocus onClick={onConfigureMame}>
                Configure MAME
              </button>
              <button type="button" className="secondary-button" onClick={onDiagnostics}>
                Diagnostics
              </button>
            </div>
          </>
        )}

        {bootstrap.status === "executableUnavailable" && (
          <>
            <p className="mame-bootstrap-kicker">Configuration problem</p>
            <h1>Configured MAME is unavailable</h1>
            <p>{bootstrap.errorMessage}</p>
            <p className="mame-bootstrap-code">{bootstrap.errorCode}</p>
            <div className="mame-bootstrap-actions">
              <button type="button" autoFocus onClick={onConfigureMame}>
                Configure MAME
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => void refreshBootstrap()}
              >
                Retry status check
              </button>
              <button type="button" className="secondary-button" onClick={onDiagnostics}>
                Diagnostics
              </button>
            </div>
          </>
        )}

        {(bootstrap.status === "metadataMissing" || bootstrap.status === "metadataStale") && (
          <>
            <p className="mame-bootstrap-kicker">Machine metadata required</p>
            <h1>
              {bootstrap.status === "metadataMissing"
                ? "Import the MAME machine list"
                : "Refresh the MAME machine list"}
            </h1>
            <p>
              MAME {bootstrap.mame.rawVersionLine} is configured, but the machine browser does not
              have a matching active metadata generation.
            </p>
            {bootstrap.status === "metadataStale" && (
              <p>
                The previous catalog contains{" "}
                {bootstrap.activeGeneration.machineCount.toLocaleString()} machines from MAME{" "}
                {bootstrap.activeGeneration.mameVersion}; it will not be treated as current.
              </p>
            )}
            <div className="mame-bootstrap-actions">
              <button type="button" autoFocus onClick={() => void importMetadata()}>
                Import MAME metadata
              </button>
              <button type="button" className="secondary-button" onClick={onConfigureMame}>
                Configure MAME
              </button>
              <button type="button" className="secondary-button" onClick={onDiagnostics}>
                Diagnostics
              </button>
            </div>
          </>
        )}

        {bootstrap.status === "importing" && importSource && (
          <>
            <p className="mame-bootstrap-kicker">Importing metadata</p>
            <h1>Building the machine catalog…</h1>
            <p>
              Running MAME {importSource.mame.rawVersionLine} with <code>-listxml</code> and
              importing the result. The existing active catalog is left intact until the import
              completes.
            </p>
            <div className="mame-bootstrap-progress" role="status" aria-live="polite">
              Import in progress
            </div>
          </>
        )}

        {bootstrap.status === "importFailed" && importSource && (
          <>
            <p className="mame-bootstrap-kicker">Import failed</p>
            <h1>MAME metadata could not be imported</h1>
            <p>{bootstrap.message}</p>
            <p>MAME {importSource.mame.rawVersionLine} remains configured.</p>
            <div className="mame-bootstrap-actions">
              <button type="button" autoFocus onClick={() => void importMetadata()}>
                Retry import
              </button>
              <button type="button" className="secondary-button" onClick={onConfigureMame}>
                Configure MAME
              </button>
              <button type="button" className="secondary-button" onClick={onDiagnostics}>
                Diagnostics
              </button>
            </div>
          </>
        )}
      </div>
    </section>
  );
}
