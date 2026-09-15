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
import { getMameMachineDetail, getMameSession, launchLibraryMachine } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import { exportMameUiDisplayedList, queryMameUiLibrary } from "../backend/mameUi";
import { getMameUiState, setMameUiState, type MameUiPanelMode } from "../backend/mameUiState";
import type {
  MachineDetail,
  MachineListItem,
  MachinePage,
  SessionSnapshot,
} from "../backend/types";
import { FavoriteToggleButton } from "../library/FavoriteToggleButton";
import { isEditableElement, isGameplaySessionState } from "../library/keyboardNavigation";
import { MachineFilterPanel } from "./MachineFilterPanel";
import { MachineList } from "./MachineList";
import { MachineRightPanel, type MachineRightView } from "./MachineRightPanel";
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

export function MameBrowser({
  availabilityRevision,
  onAuditResultsChanged,
}: {
  availabilityRevision: number;
  onAuditResultsChanged: () => void;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const machineRowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const activeFilterButtonRef = useRef<HTMLButtonElement | null>(null);
  const rightPanelFirstTabRef = useRef<HTMLButtonElement | null>(null);
  const configureButtonRef = useRef<HTMLButtonElement | null>(null);
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
  const [gameplayInputOwned, setGameplayInputOwned] = useState(true);

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

  const refreshGameplayOwnership = useCallback(() => {
    void getMameSession()
      .then((session) => setGameplayInputOwned(isGameplaySessionState(session?.state)))
      .catch(() => setGameplayInputOwned(true));
  }, []);

  useEffect(() => {
    refreshGameplayOwnership();
    window.addEventListener("focus", refreshGameplayOwnership);
    return () => window.removeEventListener("focus", refreshGameplayOwnership);
  }, [refreshGameplayOwnership]);

  useEffect(() => {
    if (!uiStateHydrated) return;
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
  const range =
    page && page.total > 0
      ? `${(page.offset + 1).toLocaleString()}–${Math.min(
          page.offset + page.items.length,
          page.total,
        ).toLocaleString()} of ${page.total.toLocaleString()}`
      : "0 machines";

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
          setGameplayInputOwned(isGameplaySessionState(session.state));
        })
        .catch((reason: unknown) => {
          setLaunchState({ status: "error", message: errorMessage(reason) });
          refreshGameplayOwnership();
        });
    },
    [launchState.status, pendingLaunchOverrides, refreshGameplayOwnership],
  );

  const exportDisplayedList = useCallback(() => {
    if (exportState.status === "exporting" || (valueRequired && !debouncedFilterValue)) return;
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
  }, [debouncedFilterValue, debouncedSearch, exportState.status, filter, valueRequired]);

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
  }, [detailState, gameplayInputOwned, launchDetail, search, softwareMode]);

  const detail = detailState.status === "ready" ? detailState.detail : null;

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
        launchOverrides={pendingLaunchOverrides}
        panelMode={softwareRightPanelMode}
        onPanelModeChange={setSoftwareRightPanelMode}
        onBack={() => setSoftwareMode(false)}
        onSessionStarted={(session) => {
          setPendingLaunchOverrides(null);
          setGameplayInputOwned(isGameplaySessionState(session.state));
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
          disabled={exportState.status === "exporting" || (valueRequired && !debouncedFilterValue)}
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
              disabled={!detail.runnable || launchState.status === "launching"}
              onClick={() => launchDetail(detail)}
            >
              {launchState.status === "launching" ? "Starting…" : "Start"}
            </button>
            <FavoriteToggleButton
              shortName={detail.shortName}
              revision={favoriteRevision}
              onChanged={bumpFavoriteRevision}
            />
            <button
              type="button"
              className="secondary-button"
              onClick={() => changeRightView("audit")}
            >
              Audit
            </button>
            <button
              ref={configureButtonRef}
              type="button"
              className="secondary-button"
              onClick={() => changeRightView("settings")}
            >
              Configure
            </button>
            {detail.softwareLists.length > 0 && (
              <button
                type="button"
                className="secondary-button"
                onClick={() => setSoftwareMode(true)}
              >
                Software
              </button>
            )}
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
          aria-busy={loadState.status === "loading"}
        >
          <div className="mame-region-heading">Machines</div>
          {!uiStateHydrated && <div className="mame-panel-state">Restoring browser state…</div>}
          {uiStateHydrated && loadState.status === "awaitingFilterValue" && (
            <div className="mame-panel-state">Enter a value for the selected filter.</div>
          )}
          {uiStateHydrated && loadState.status === "loading" && (
            <div className="mame-panel-state">Loading catalog…</div>
          )}
          {uiStateHydrated && loadState.status === "error" && (
            <div className="mame-panel-state" role="alert">
              {loadState.message}
            </div>
          )}
          {page && page.items.length === 0 && (
            <div className="mame-panel-state">No machines match this filter.</div>
          )}
          {page && page.items.length > 0 && (
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
          {page && page.total > page.limit && (
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
          <aside className="mame-right-panel">
            <div className="mame-panel-state">
              {detailState.status === "loading"
                ? "Loading machine details…"
                : detailState.status === "error"
                  ? detailState.message
                  : "Select a machine."}
            </div>
          </aside>
        )}
      </div>
    </section>
  );
}
