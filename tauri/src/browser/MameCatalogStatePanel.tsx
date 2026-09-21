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
  let title: string;
  let detail: string;
  let showConfigure = false;
  let showImport = false;
  let importLabel = "Import Metadata";
  let alert = false;

  switch (state.status) {
    case "notConfigured":
      title = "MAME is not configured";
      detail =
        "Choose a MAME executable in Configure Options before importing the machine catalog.";
      showConfigure = true;
      break;
    case "executableUnavailable":
      title = "Configured MAME is unavailable";
      detail = state.message;
      showConfigure = true;
      alert = true;
      break;
    case "checking":
      title = "Checking MAME metadata…";
      detail = "Reading the active catalog generation for the configured MAME executable.";
      break;
    case "importNeeded":
      title =
        state.freshness === "stale" ? "MAME metadata needs refresh" : "MAME metadata needs import";
      if (state.freshness === "stale") {
        detail =
          state.previousMachineCount === null
            ? "The active catalog belongs to a different MAME executable or version. Refresh it before browsing or launching."
            : `The active catalog belongs to a different MAME executable or version and contains ${state.previousMachineCount.toLocaleString()} machines. Refresh it before browsing or launching.`;
      } else {
        detail =
          "No successfully imported MAME machine catalog is active. Import metadata to populate the machine list.";
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
