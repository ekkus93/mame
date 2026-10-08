import { useEffect, useRef } from "react";

import type { MameCatalogState } from "./mameCatalogState";

export function MameCatalogStatePanel({
  state,
  onConfigureOptions,
  onImportMetadata,
}: {
  state: Exclude<MameCatalogState, { status: "ready" }>;
  onConfigureOptions: () => void;
  onImportMetadata: () => void;
}) {
  const autoImportKeyRef = useRef<string | null>(null);
  const autoImportKey =
    state.status === "importNeeded"
      ? `${state.freshness}:${state.previousMachineCount ?? "none"}`
      : null;

  useEffect(() => {
    if (state.status !== "importNeeded" || autoImportKey === null) return;
    if (autoImportKeyRef.current === autoImportKey) return;
    autoImportKeyRef.current = autoImportKey;
    onImportMetadata();
  }, [autoImportKey, onImportMetadata, state.status]);

  let title: string;
  let detail: string;
  let showConfigure = false;
  let showImport = false;
  let importLabel = "Import Metadata";
  let alert = false;

  switch (state.status) {
    case "notConfigured":
      title = "MAME runtime is not available";
      detail =
        "The backend did not report an active MAME runtime. Check the installed package diagnostics or use the advanced runtime override.";
      showConfigure = true;
      break;
    case "executableUnavailable":
      title = "MAME runtime is unavailable";
      detail = state.message;
      showConfigure = true;
      alert = true;
      break;
    case "checking":
      title = "Checking MAME metadata…";
      detail = "Reading the active catalog generation for the effective MAME runtime.";
      break;
    case "importNeeded":
      title =
        state.freshness === "stale" ? "MAME metadata needs refresh" : "MAME metadata needs import";
      if (state.freshness === "stale") {
        detail =
          state.previousMachineCount === null
            ? "The active catalog belongs to a different MAME executable or version. Refresh is starting automatically before browsing or launching."
            : `The active catalog belongs to a different MAME executable or version and contains ${state.previousMachineCount.toLocaleString()} machines. Refresh is starting automatically before browsing or launching.`;
      } else {
        detail =
          "No successfully imported MAME machine catalog is active. Metadata import is starting automatically to populate the machine list. This imports machine metadata only; it does not download or install ROM, CHD, BIOS, or software content.";
      }
      showConfigure = true;
      showImport = true;
      importLabel = state.freshness === "stale" ? "Refresh Metadata" : "Import Metadata";
      break;
    case "importing":
      title = "Importing MAME metadata…";
      detail =
        "MAME is generating the machine catalog. The machine list will populate when the import completes.";
      break;
    case "importFailed":
      title = "MAME metadata import failed";
      detail = state.message;
      showConfigure = true;
      showImport = true;
      importLabel = "Retry Metadata Import";
      alert = true;
      break;
  }

  return (
    <div
      className={`mame-catalog-state is-${state.status}`}
      role={alert ? "alert" : "status"}
      aria-live="polite"
    >
      <strong>{title}</strong>
      <span>{detail}</span>
      {(showConfigure || showImport) && (
        <div className="mame-catalog-actions">
          {showImport && (
            <button type="button" className="mame-start-button" onClick={onImportMetadata}>
              {importLabel}
            </button>
          )}
          {showConfigure && (
            <button type="button" className="secondary-button" onClick={onConfigureOptions}>
              Configure Options
            </button>
          )}
        </div>
      )}
    </div>
  );
}
