import {
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";

import type { ArtworkKind } from "../backend/artwork";
import {
  getMameMachineDetail,
  getMameMetadataStatus,
  launchLibraryMachine,
  refreshMameMetadata,
} from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import { exportMameUiDisplayedList, queryMameUiLibrary } from "../backend/mameUi";
import { getMameUiState, setMameUiState, type MameUiPanelMode } from "../backend/mameUiState";
import type {
  MameVersionReport,
  MachineDetail,
  MachineListItem,
  MachinePage,
  SessionSnapshot,
} from "../backend/types";
import { FavoriteToggleButton } from "../library/FavoriteToggleButton";
import { isEditableElement } from "../library/keyboardNavigation";
import { MameCatalogStatePanel } from "./MameCatalogStatePanel";
import { MachineDriverStatus } from "./MachineDriverStatus";
import { MachineFilterPanel } from "./MachineFilterPanel";
import { MachineList } from "./MachineList";
import {
  catalogCanQuery,
  catalogRangeLabel,
  catalogStateFromMetadata,
  initialMameCatalogState,
  machineAvailabilityNotice,
  metadataExecutableRequest,
  type MameCatalogState,
} from "./mameCatalogState";
import {
  EmptyMachineRightPanel,
  MachineRightPanel,
  type MachineRightView,
} from "./MachineRightPanel";
import {
  buildMameBrowserRequest,
  filterRequiresValue,
  nextBrowserIndex,
  reconcileMachineSelection,
  type MameBrowserFilter,
} from "./model";
import { SoftwareBrowser } from "./SoftwareBrowser";

type LoadState =
  | { status: "awaitingFilterValue" }
  | { status: "loading" }
  | { status: "ready"; page: MachinePage }
  | { status: "error"; message: string };

type DetailState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; detail: MachineDetail }
  | { status: "error"; message: string };

type LaunchState =
  | { status: "idle" }
  | { status: "launching" }
  | { status: "launched"; session: SessionSnapshot }
  | { status: "error"; message: string };

type ExportState =
  | { status: "idle" }
  | { status: "exporting" }
  | { status: "success"; message: string }
  | { status: "error"; message: string };

type SoftwareCapableMachineDetail = MachineDetail & { canStartEmpty: boolean };
type EmptyRightPanelStatus = "idle" | "loading" | "error";

const MAME_MACHINE_ACTION_LABELS = {
  start: "Start",
  startEmpty: "Start Empty",
  configureMachine: "Configure Machine",
  softwareList: "Software List",
  audit: "Audit",
} as const;

function emptyRightPanelStatusFor(detailState: DetailState): EmptyRightPanelStatus {
  switch (detailState.status) {
    case "loading":
      return "loading";
    case "error":
      return "error";
    case "idle":
    case "ready":
      return "idle";
  }
}

function primaryLaunchButtonLabel(detail: MachineDetail, launchState: LaunchState): string {
  if (launchState.status === "launching") return "Starting…";
  if (detail.canStartEmpty && detail.softwareLists.length > 0) {
    return MAME_MACHINE_ACTION_LABELS.startEmpty;
  }
  return MAME_MACHINE_ACTION_LABELS.start;
}

function primaryLaunchButtonAriaLabel(detail: MachineDetail): string {
  if (detail.canStartEmpty && detail.softwareLists.length > 0) {
    return `Start Empty ${detail.description}`;
  }
  return `Start ${detail.description}`;
}

export function MameBrowser({
  availabilityRevision,
  gameplayInputOwned,
  mame,
  onAuditResultsChanged,
  onConfigureOptions,
  onOpenAudit,
  onSessionStarted,
}: {
  availabilityRevision: number;
  gameplayInputOwned: boolean;
  mame: MameVersionReport;
  onAuditResultsChanged: () => void;
  onConfigureOptions: () => void;
  onOpenAudit: () => void;
  onSessionStarted: (session: SessionSnapshot) => void;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const machineRowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const activeFilterButtonRef = useRef<HTMLButtonElement | null>(null);
  const rightPanelFirstTabRef = useRef<HTMLButtonElement | null>(null);
  const configureButtonRef = useRef<HTMLButtonElement | null>(null);
  const catalogSequence = useRef(0);
  const querySequence = useRef(0);
  const detailSequence = useRef(0);
  const [uiStateHydrated, setUiStateHydrated] = useState(false);
  const [uiStateError, setUiStateError] = useState<string | null>(null);
  const [filter, setFilter] = useState<MameBrowserFilter>("all");
  const [filterValue, setFilterValue] = useState("");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [debouncedFilterValue, setDebouncedFilterValue] = useState("");
  const [offset, setOffset] = useState(0);
  const [preferredMachine, setPreferredMachine] = useState<string | null>(null);
  const [rememberedMachine, setRememberedMachine] = useState<string | null>(null);
  const [catalogState, setCatalogState] = useState<MameCatalogState>(() =>
    initialMameCatalogState(mame),
  );
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [selected, setSelected] = useState<MachineListItem | null>(null);
  const [detailState, setDetailState] = useState<DetailState>({ status: "idle" });
  const [rightView, setRightView] = useState<MachineRightView>("images");
  const [primaryRightView, setPrimaryRightView] = useState<MameUiPanelMode>("images");
  const [artworkKind, setArtworkKind] = useState<ArtworkKind>("screenshot");
  const [softwareRightPanelMode, setSoftwareRightPanelMode] = useState<MameUiPanelMode>("images");
  const [softwareArtworkKind, setSoftwareArtworkKind] = useState<ArtworkKind>("screenshot");
  const [softwareMode, setSoftwareMode] = useState(false);
  const [showNarrowDetails, setShowNarrowDetails] = useState(false);
  const [pendingLaunchOverrides, setPendingLaunchOverrides] = useState<LaunchPreferences | null>(
    null,
  );
  const [launchState, setLaunchState] = useState<LaunchState>({ status: "idle" });
  const [exportState, setExportState] = useState<ExportState>({ status: "idle" });
  const [favoriteRevision, bumpFavoriteRevision] = useReducer((value: number) => value + 1, 0);

  useEffect(() => {
    let cancelled = false;
    void getMameUiState()
      .then((saved) => {
        if (cancelled) return;
        setFilter(saved.filter);
        setFilterValue(saved.filterValue ?? "");
        setPreferredMachine(saved.lastMachine);
        setRememberedMachine(saved.lastMachine);
        setRightView(saved.rightPanelMode);
        setPrimaryRightView(saved.rightPanelMode);
        setArtworkKind(saved.artworkKind);
        setSoftwareRightPanelMode(saved.softwareRightPanelMode);
        setSoftwareArtworkKind(saved.softwareArtworkKind);
      })
      .catch((reason: unknown) => {
        if (!cancelled) setUiStateError(`Saved browser state unavailable: ${errorMessage(reason)}`);
      })
      .finally(() => {
        if (!cancelled) setUiStateHydrated(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const sequence = ++catalogSequence.current;
    const initial = initialMameCatalogState(mame);
    setCatalogState(initial);
    setLoadState({ status: "loading" });
    setSelected(null);

    if (initial.status !== "checking") return;

    const executable = metadataExecutableRequest(mame);
    if (!executable) {
      setCatalogState({
        status: "executableUnavailable",
        message: "The active MAME executable cannot be used to inspect metadata.",
      });
      return;
    }

    void getMameMetadataStatus({ executable })
      .then((status) => {
        if (catalogSequence.current === sequence) {
          setCatalogState(catalogStateFromMetadata(status));
        }
      })
      .catch((reason: unknown) => {
        if (catalogSequence.current === sequence) {
          setCatalogState({
            status: "importFailed",
            message: `Metadata status unavailable: ${errorMessage(reason)}`,
          });
        }
      });
  }, [mame]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedFilterValue(filterValue.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [filterValue]);

  useEffect(() => setOffset(0), [filter, debouncedSearch, debouncedFilterValue]);

  useEffect(() => {
    if (selected) setRememberedMachine(selected.shortName);
  }, [selected]);

  useEffect(() => {
    setSoftwareMode(false);
    setShowNarrowDetails(false);
  }, [selected?.shortName]);

  useEffect(() => {
    if (!uiStateHydrated) return;
    const timer = window.setTimeout(() => {
      void setMameUiState({
        schemaVersion: 1,
        lastMachine: rememberedMachine,
        filter,
        filterValue: filterValue.trim() || null,
        rightPanelMode: primaryRightView,
        artworkKind,
        softwareRightPanelMode,
        softwareArtworkKind,
      })
        .then(() => setUiStateError(null))
        .catch((reason: unknown) => {
          setUiStateError(`Browser state could not be saved: ${errorMessage(reason)}`);
        });
    }, 300);
    return () => window.clearTimeout(timer);
  }, [
    artworkKind,
    filter,
    filterValue,
    primaryRightView,
    rememberedMachine,
    softwareArtworkKind,
    softwareRightPanelMode,
    uiStateHydrated,
  ]);

  const importMetadata = useCallback(() => {
    if (catalogState.status === "importing") return;
    const executable = metadataExecutableRequest(mame);
    if (!executable) {
      setCatalogState({
        status: "importFailed",
        message: "The active MAME executable cannot be used to refresh metadata.",
      });
      return;
    }

    const sequence = ++catalogSequence.current;
    setCatalogState({ status: "importing" });
    setLoadState({ status: "loading" });
    setSelected(null);

    void refreshMameMetadata({ executable })
      .then((result) => {
        if (catalogSequence.current !== sequence) return;
        setCatalogState({
          status: "ready",
          machineCount: result.generation.machineCount,
        });
        setOffset(0);
        setPreferredMachine(rememberedMachine);
      })
      .catch((reason: unknown) => {
        if (catalogSequence.current === sequence) {
          setCatalogState({
            status: "importFailed",
            message: errorMessage(reason),
          });
        }
      });
  }, [catalogState.status, mame, rememberedMachine]);

  const valueRequired = filterRequiresValue(filter);
  const request = useMemo(
    () =>
      buildMameBrowserRequest(
        filter,
        debouncedSearch,
        debouncedFilterValue,
        offset,
        offset === 0 ? preferredMachine : null,
      ),
    [filter, debouncedSearch, debouncedFilterValue, offset, preferredMachine],
  );

  useEffect(() => {
    if (!uiStateHydrated) return;
    if (!catalogCanQuery(catalogState)) {
      querySequence.current += 1;
      setSelected(null);
      return;
    }
    const sequence = ++querySequence.current;
    if (valueRequired && !debouncedFilterValue) {
      setLoadState({ status: "awaitingFilterValue" });
      setSelected(null);
      return;
    }

    setLoadState({ status: "loading" });
    void queryMameUiLibrary(request)
      .then((page) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "ready", page });
        if (page.offset !== offset) setOffset(page.offset);
        setSelected((current) => reconcileMachineSelection(page.items, current, preferredMachine));
        if (preferredMachine) setPreferredMachine(null);
      })
      .catch((reason: unknown) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "error", message: errorMessage(reason) });
        setSelected(null);
      });
  }, [
    availabilityRevision,
    catalogState,
    debouncedFilterValue,
    offset,
    preferredMachine,
    request,
    uiStateHydrated,
    valueRequired,
  ]);

  useEffect(() => {
    if (!selected) {
      setDetailState({ status: "idle" });
      return;
    }
    const sequence = ++detailSequence.current;
    setDetailState({ status: "loading" });
    setLaunchState({ status: "idle" });
    setPendingLaunchOverrides(null);
    void getMameMachineDetail({ shortName: selected.shortName })
      .then((detail) => {
        if (detailSequence.current === sequence) setDetailState({ status: "ready", detail });
      })
      .catch((reason: unknown) => {
        if (detailSequence.current === sequence) {
          setDetailState({ status: "error", message: errorMessage(reason) });
        }
      });
  }, [selected]);

  const page = loadState.status === "ready" ? loadState.page : null;
  const catalogReady = catalogCanQuery(catalogState);
  const availabilityNotice = catalogReady ? machineAvailabilityNotice(page) : null;
  const range =
    !catalogReady || !page
      ? catalogRangeLabel(catalogState)
      : page.total > 0
        ? `${(page.offset + 1).toLocaleString()}–${Math.min(
            page.offset + page.items.length,
            page.total,
          ).toLocaleString()} of ${page.total.toLocaleString()}`
        : "Metadata loaded";

  const focusSelectedMachine = useCallback(() => {
    if (!page || !selected) return;
    const index = page.items.findIndex((machine) => machine.shortName === selected.shortName);
    if (index >= 0) machineRowRefs.current[index]?.focus();
  }, [page, selected]);

  const launchDetail = useCallback(
    (detail: MachineDetail) => {
      if (!detail.runnable || launchState.status === "launching") return;
      setLaunchState({ status: "launching" });
      void launchLibraryMachine({
        shortName: detail.shortName,
        launchOverrides: pendingLaunchOverrides,
      })
        .then((session) => {
          setLaunchState({ status: "launched", session });
          setPendingLaunchOverrides(null);
          onSessionStarted(session);
        })
        .catch((reason: unknown) => {
          setLaunchState({ status: "error", message: errorMessage(reason) });
        });
    },
    [launchState.status, onSessionStarted, pendingLaunchOverrides],
  );

  const exportDisplayedList = useCallback(() => {
    if (
      !catalogReady ||
      exportState.status === "exporting" ||
      (valueRequired && !debouncedFilterValue)
    ) {
      return;
    }
    setExportState({ status: "exporting" });
    void exportMameUiDisplayedList({
      text: debouncedSearch || null,
      filter,
      filterValue: debouncedFilterValue || null,
    })
      .then((result) => {
        if (result.canceled) {
          setExportState({ status: "idle" });
          return;
        }
        setExportState({
          status: "success",
          message: `Exported ${result.rows.toLocaleString()} machine${result.rows === 1 ? "" : "s"}.`,
        });
      })
      .catch((reason: unknown) => {
        setExportState({ status: "error", message: errorMessage(reason) });
      });
  }, [
    catalogReady,
    debouncedFilterValue,
    debouncedSearch,
    exportState.status,
    filter,
    valueRequired,
  ]);

  const activateMachine = useCallback(
    (machine: MachineListItem) => {
      if (detailState.status === "ready" && detailState.detail.shortName === machine.shortName) {
        launchDetail(detailState.detail);
      }
    },
    [detailState, launchDetail],
  );

  function handleMachineRowKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>, index: number) {
    if (!page) return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      activeFilterButtonRef.current?.focus();
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      setShowNarrowDetails(true);
      rightPanelFirstTabRef.current?.focus();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setShowNarrowDetails(false);
      searchInputRef.current?.focus();
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      const machine = page.items[index];
      if (machine) activateMachine(machine);
      return;
    }
    const nextIndex = nextBrowserIndex(event.key, index, page.items.length);
    if (nextIndex === null) return;
    event.preventDefault();
    const next = page.items[nextIndex];
    if (!next) return;
    setSelected(next);
    machineRowRefs.current[nextIndex]?.focus();
  }

  useEffect(() => {
    function handleShortcut(event: KeyboardEvent) {
      if (
        event.defaultPrevented ||
        event.repeat ||
        event.altKey ||
        gameplayInputOwned ||
        softwareMode ||
        !document.hasFocus()
      ) {
        return;
      }

      const editing = isEditableElement(event.target);
      if (event.key === "Escape" && event.target === searchInputRef.current && search) {
        event.preventDefault();
        setSearch("");
        return;
      }
      if (editing) return;

      if (event.key === "Escape" && showNarrowDetails) {
        event.preventDefault();
        setShowNarrowDetails(false);
        focusSelectedMachine();
        return;
      }
      if (event.key === "/" && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        searchInputRef.current?.focus();
        return;
      }
      if (
        event.key === "Enter" &&
        (event.ctrlKey || event.metaKey) &&
        detailState.status === "ready"
      ) {
        event.preventDefault();
        launchDetail(detailState.detail);
      }
    }

    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [
    detailState,
    focusSelectedMachine,
    gameplayInputOwned,
    launchDetail,
    search,
    showNarrowDetails,
    softwareMode,
  ]);

  const detail = detailState.status === "ready" ? detailState.detail : null;
  const detailErrorMessage = detailState.status === "error" ? detailState.message : null;
  const emptyRightPanelStatus = emptyRightPanelStatusFor(detailState);

  function changeFilter(next: MameBrowserFilter) {
    if (next !== filter) {
      setFilter(next);
      setFilterValue("");
      setPreferredMachine(null);
      setExportState({ status: "idle" });
    }
  }

  function changeRightView(next: MachineRightView) {
    setRightView(next);
    setShowNarrowDetails(true);
    if (next === "images" || next === "info") setPrimaryRightView(next);
  }

  if (softwareMode && detail) {
    return (
      <SoftwareBrowser
        detail={detail as SoftwareCapableMachineDetail}
        gameplayInputOwned={gameplayInputOwned}
        launchOverrides={pendingLaunchOverrides}
        panelMode={softwareRightPanelMode}
        onPanelModeChange={setSoftwareRightPanelMode}
        onBack={() => setSoftwareMode(false)}
        onSessionStarted={(session) => {
          setPendingLaunchOverrides(null);
          onSessionStarted(session);
        }}
      />
    );
  }

  return (
    <section className="mame-browser" aria-label="MAME machine selection">
      <div className="mame-browser-toolbar" role="toolbar" aria-label="Machine browser actions">
        <label className="mame-search">
          <span className="visually-hidden">Search machines</span>
          <input
            ref={searchInputRef}
            type="search"
            value={search}
            aria-keyshortcuts="/"
            placeholder="Search systems..."
            autoComplete="off"
            disabled={!catalogReady}
            onChange={(event) => {
              setSearch(event.target.value);
              setPreferredMachine(null);
              setExportState({ status: "idle" });
            }}
          />
        </label>
        <span className="mame-browser-range" aria-live="polite">
          {range}
        </span>
        <button
          type="button"
          className="secondary-button"
          aria-label="Export displayed machine list"
          disabled={
            !catalogReady ||
            exportState.status === "exporting" ||
            (valueRequired && !debouncedFilterValue)
          }
          onClick={exportDisplayedList}
        >
          {exportState.status === "exporting" ? "Exporting…" : "Export"}
        </button>
        <button
          type="button"
          className="secondary-button mame-narrow-details-toggle"
          aria-expanded={showNarrowDetails}
          onClick={() => setShowNarrowDetails((current) => !current)}
        >
          {showNarrowDetails ? "Hide details" : "Details"}
        </button>
        {detail && (
          <div className="mame-context-actions" aria-label="Selected machine actions">
            <button
              type="button"
              className="mame-start-button"
              aria-label={primaryLaunchButtonAriaLabel(detail)}
              disabled={!detail.runnable || launchState.status === "launching"}
              title={!detail.runnable ? "Machine is unavailable" : undefined}
              onClick={() => launchDetail(detail)}
            >
              {primaryLaunchButtonLabel(detail, launchState)}
            </button>
            <FavoriteToggleButton
              shortName={detail.shortName}
              revision={favoriteRevision}
              onChanged={bumpFavoriteRevision}
            />
            <button
              ref={configureButtonRef}
              type="button"
              className="secondary-button mame-configure-machine-button"
              aria-label={`Configure Machine for ${detail.description}`}
              onClick={() => changeRightView("settings")}
            >
              {MAME_MACHINE_ACTION_LABELS.configureMachine}
            </button>
            {detail.softwareLists.length > 0 && (
              <button
                type="button"
                className="secondary-button mame-software-list-button"
                aria-label={`Open Software List for ${detail.description}`}
                onClick={() => setSoftwareMode(true)}
              >
                {MAME_MACHINE_ACTION_LABELS.softwareList}
              </button>
            )}
            <button
              type="button"
              className="secondary-button mame-audit-button"
              aria-label={`Audit ${detail.description}`}
              onClick={() => changeRightView("audit")}
            >
              {MAME_MACHINE_ACTION_LABELS.audit}
            </button>
          </div>
        )}
      </div>

      {uiStateError && (
        <div className="mame-browser-banner is-error" role="status">
          {uiStateError}
        </div>
      )}
      {exportState.status === "success" && (
        <div className="mame-browser-banner" role="status">
          {exportState.message}
        </div>
      )}
      {exportState.status === "error" && (
        <div className="mame-browser-banner is-error" role="alert">
          Export failed: {exportState.message}
        </div>
      )}
      {launchState.status === "error" && (
        <div className="mame-browser-banner is-error" role="alert">
          {launchState.message}
        </div>
      )}
      {launchState.status === "launched" && (
        <div className="mame-browser-banner" role="status">
          Session {launchState.session.sessionId} started.
        </div>
      )}

      <div className={`mame-browser-grid ${showNarrowDetails ? "show-details" : ""}`}>
        <MachineFilterPanel
          active={filter}
          filterValue={filterValue}
          onChange={changeFilter}
          onFilterValueChange={(value) => {
            setFilterValue(value);
            setPreferredMachine(null);
            setExportState({ status: "idle" });
          }}
          registerActiveButton={(element) => {
            activeFilterButtonRef.current = element;
          }}
          onNavigateToMachines={focusSelectedMachine}
        />

        <section
          className="mame-list-region"
          aria-label="Machine list"
          aria-busy={
            loadState.status === "loading" ||
            catalogState.status === "checking" ||
            catalogState.status === "importing"
          }
        >
          <div className="mame-region-heading">Machines</div>
          {!uiStateHydrated && <div className="mame-panel-state">Restoring browser state…</div>}
          {uiStateHydrated && catalogState.status !== "ready" && (
            <MameCatalogStatePanel
              state={catalogState}
              onConfigureOptions={onConfigureOptions}
              onImportMetadata={importMetadata}
            />
          )}
          {uiStateHydrated &&
            catalogState.status === "ready" &&
            loadState.status === "awaitingFilterValue" && (
            <div className="mame-panel-state">Enter a value for the selected filter.</div>
          )}
          {uiStateHydrated &&
            catalogState.status === "ready" &&
            loadState.status === "loading" && (
            <div className="mame-panel-state">Loading catalog…</div>
          )}
          {uiStateHydrated && catalogState.status === "ready" && loadState.status === "error" && (
            <div className="mame-panel-state" role="alert">
              {loadState.message}
            </div>
          )}
          {catalogState.status === "ready" && page && page.items.length === 0 && (
            <div className="mame-panel-state">
              {page.total === 0 && filter === "all" && !debouncedSearch
                ? "Metadata loaded, but no machines are present in the active catalog."
                : "No machines match this filter."}
            </div>
          )}
          {catalogState.status === "ready" && availabilityNotice === "unknown" && page && (
            <div className="mame-catalog-hint" role="status">
              <span>ROM availability has not been audited for these machines.</span>
              <button type="button" className="secondary-button" onClick={onOpenAudit}>
                Audit
              </button>
            </div>
          )}
          {catalogState.status === "ready" && availabilityNotice === "noneAvailable" && page && (
            <div className="mame-catalog-hint" role="status">
              <span>No verified available ROMs were found in the configured content paths.</span>
              <button type="button" className="secondary-button" onClick={onConfigureOptions}>
                Configure Options
              </button>
              <button type="button" className="secondary-button" onClick={onOpenAudit}>
                Audit
              </button>
            </div>
          )}
          {catalogState.status === "ready" && page && page.items.length > 0 && (
            <MachineList
              page={page}
              selected={selected}
              registerRow={(index, element) => {
                machineRowRefs.current[index] = element;
              }}
              onSelect={setSelected}
              onNavigate={handleMachineRowKeyDown}
              onActivate={activateMachine}
            />
          )}
          {catalogState.status === "ready" && page && page.total > page.limit && (
            <nav className="mame-pager" aria-label="Machine result pages">
              <button
                type="button"
                className="secondary-button"
                disabled={page.offset === 0}
                onClick={() => {
                  setPreferredMachine(null);
                  setOffset(Math.max(0, page.offset - page.limit));
                }}
              >
                Previous
              </button>
              <span>{range}</span>
              <button
                type="button"
                className="secondary-button"
                disabled={page.offset + page.items.length >= page.total}
                onClick={() => {
                  setPreferredMachine(null);
                  setOffset(page.offset + page.limit);
                }}
              >
                Next
              </button>
            </nav>
          )}
        </section>

        {detail ? (
          <MachineRightPanel
            detail={detail}
            view={rightView}
            onViewChange={changeRightView}
            artworkKind={artworkKind}
            onArtworkKindChange={setArtworkKind}
            pendingLaunchOverrides={pendingLaunchOverrides}
            onPendingLaunchOverridesChanged={setPendingLaunchOverrides}
            onAuditResultChanged={onAuditResultsChanged}
            firstTabRef={rightPanelFirstTabRef}
            onNavigateToMachines={focusSelectedMachine}
            onSettingsClose={() => {
              changeRightView("info");
              configureButtonRef.current?.focus();
            }}
          />
        ) : (
          <EmptyMachineRightPanel
            status={emptyRightPanelStatus}
            message={detailErrorMessage ?? undefined}
          />
        )}
      </div>
      <MachineDriverStatus
        detail={detail}
        selected={selected}
        detailStatus={detailState.status}
        errorMessage={detailErrorMessage}
      />
    </section>
  );
}
