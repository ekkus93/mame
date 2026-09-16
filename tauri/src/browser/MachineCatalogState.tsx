import type { MameVersionReport, MachinePage } from "../backend/types";
import type { MameBrowserFilter } from "./model";

export type MachineCatalogLoadState =
  | { status: "awaitingFilterValue" }
  | { status: "loading" }
  | { status: "ready"; page: MachinePage }
  | { status: "error"; message: string };

type MachineCatalogStateModel = {
  kind: "empty" | "error" | "loading" | "needed" | "valueRequired";
  heading: string;
  body: string;
  showConfigureOptions: boolean;
};

function hasActiveBrowserQuery(
  filter: MameBrowserFilter,
  search: string,
  filterValue: string,
): boolean {
  return filter !== "all" || search.trim() !== "" || filterValue.trim() !== "";
}

function catalogStateFor({
  mameReport,
  uiStateHydrated,
  loadState,
  page,
  search,
  filter,
  filterValue,
}: {
  mameReport: MameVersionReport;
  uiStateHydrated: boolean;
  loadState: MachineCatalogLoadState;
  page: MachinePage | null;
  search: string;
  filter: MameBrowserFilter;
  filterValue: string;
}): MachineCatalogStateModel | null {
  if (mameReport.status === "notConfigured") {
    return {
      kind: "needed",
      heading: "MAME executable is not configured",
      body: "Open Configure Options, choose the MAME executable, then refresh metadata before browsing machines.",
      showConfigureOptions: true,
    };
  }

  if (mameReport.status === "unavailable") {
    return {
      kind: "error",
      heading: "MAME executable is unavailable",
      body: mameReport.errorMessage,
      showConfigureOptions: true,
    };
  }

  if (!uiStateHydrated || loadState.status === "loading") {
    return {
      kind: "loading",
      heading: "MAME metadata import is in progress",
      body: "The catalog is being restored inside the MAME-style browser layout.",
      showConfigureOptions: false,
    };
  }

  if (loadState.status === "error") {
    return {
      kind: "error",
      heading: "MAME metadata import failed",
      body: loadState.message,
      showConfigureOptions: true,
    };
  }

  if (loadState.status === "awaitingFilterValue") {
    return {
      kind: "valueRequired",
      heading: "Filter value required",
      body: "Enter a value for the selected filter or choose another category from the left list.",
      showConfigureOptions: false,
    };
  }

  if (page && page.items.length === 0) {
    if (hasActiveBrowserQuery(filter, search, filterValue)) {
      return {
        kind: "empty",
        heading: "No machines match this filter",
        body: "Clear Search or select another filter category to return to the imported MAME catalog.",
        showConfigureOptions: false,
      };
    }

    return {
      kind: "needed",
      heading: "MAME metadata is not imported",
      body: "Use Configure Options to select a MAME executable and import metadata so the machine list can populate.",
      showConfigureOptions: true,
    };
  }

  return null;
}

function availabilitySummary(page: MachinePage): string {
  const statuses = page.items.map(
    (machine) => page.availabilityByShortName[machine.shortName] ?? "unknown",
  );
  const available = statuses.filter((status) => status === "available").length;
  const missing = statuses.filter((status) => status === "missing").length;
  const unknown = statuses.filter((status) => status === "unknown").length;

  if (unknown > 0) {
    return `ROM availability unknown for ${unknown.toLocaleString()} displayed system${
      unknown === 1 ? "" : "s"
    }`;
  }

  if (missing === statuses.length) {
    return "No ROMs available for displayed systems";
  }

  return `Available ROMs: ${available.toLocaleString()} · Missing ROMs: ${missing.toLocaleString()}`;
}

export function MachineCatalogState({
  mameReport,
  uiStateHydrated,
  loadState,
  page,
  search,
  filter,
  filterValue,
  onConfigureOptions,
}: {
  mameReport: MameVersionReport;
  uiStateHydrated: boolean;
  loadState: MachineCatalogLoadState;
  page: MachinePage | null;
  search: string;
  filter: MameBrowserFilter;
  filterValue: string;
  onConfigureOptions: () => void;
}) {
  const state = catalogStateFor({
    mameReport,
    uiStateHydrated,
    loadState,
    page,
    search,
    filter,
    filterValue,
  });

  if (!state) return null;

  return (
    <div
      className={`mame-catalog-state is-${state.kind}`}
      role={state.kind === "error" ? "alert" : "status"}
      aria-live="polite"
    >
      <div>
        <h2>{state.heading}</h2>
        <p>{state.body}</p>
        {state.showConfigureOptions && (
          <div className="mame-catalog-state__actions">
            <button type="button" onClick={onConfigureOptions}>
              Configure Options
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export function MachineCatalogSummary({ page }: { page: MachinePage }) {
  if (page.items.length === 0) return null;

  return (
    <p className="mame-catalog-summary" aria-live="polite">
      MAME metadata loaded · Showing {page.items.length.toLocaleString()} of{" "}
      {page.total.toLocaleString()} machines · {availabilitySummary(page)}
    </p>
  );
}
