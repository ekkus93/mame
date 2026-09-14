import {
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";

import {
  getMameMachineDetail,
  getMameSession,
  launchLibraryMachine,
  queryMameLibrary,
} from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
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
import { buildMameBrowserRequest, nextBrowserIndex, type MameBrowserFilter } from "./model";

type LoadState =
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

export function MameBrowser({
  availabilityRevision,
  onAuditResultsChanged,
}: {
  availabilityRevision: number;
  onAuditResultsChanged: () => void;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const machineRowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const querySequence = useRef(0);
  const detailSequence = useRef(0);
  const [filter, setFilter] = useState<MameBrowserFilter>("all");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [selected, setSelected] = useState<MachineListItem | null>(null);
  const [detailState, setDetailState] = useState<DetailState>({ status: "idle" });
  const [rightView, setRightView] = useState<MachineRightView>("images");
  const [pendingLaunchOverrides, setPendingLaunchOverrides] = useState<LaunchPreferences | null>(
    null,
  );
  const [launchState, setLaunchState] = useState<LaunchState>({ status: "idle" });
  const [favoriteRevision, bumpFavoriteRevision] = useReducer((value: number) => value + 1, 0);
  const [gameplayInputOwned, setGameplayInputOwned] = useState(true);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => setOffset(0), [filter, debouncedSearch]);

  const request = useMemo(
    () => buildMameBrowserRequest(filter, debouncedSearch, offset),
    [filter, debouncedSearch, offset],
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
    const sequence = ++querySequence.current;
    setLoadState({ status: "loading" });
    void queryMameLibrary(request)
      .then((page) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "ready", page });
        setSelected((current) => {
          if (current && page.items.some((item) => item.shortName === current.shortName)) {
            return current;
          }
          return page.items[0] ?? null;
        });
      })
      .catch((reason: unknown) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "error", message: errorMessage(reason) });
        setSelected(null);
      });
  }, [availabilityRevision, request]);

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
        if (detailSequence.current === sequence) {
          setDetailState({ status: "ready", detail });
        }
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
  }, [detailState, gameplayInputOwned, launchDetail, search]);

  const detail = detailState.status === "ready" ? detailState.detail : null;

  return (
    <section className="mame-browser" aria-label="MAME machine selection">
      <div className="mame-browser-toolbar">
        <label className="mame-search">
          <span className="visually-hidden">Search machines</span>
          <input
            ref={searchInputRef}
            type="search"
            value={search}
            aria-keyshortcuts="/"
            placeholder="Search systems..."
            autoComplete="off"
            onChange={(event) => setSearch(event.target.value)}
          />
        </label>
        <span className="mame-browser-range" aria-live="polite">
          {range}
        </span>
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
              onClick={() => setRightView("audit")}
            >
              Audit
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => setRightView("settings")}
            >
              Configure
            </button>
            {detail.softwareLists.length > 0 && (
              <button
                type="button"
                className="secondary-button"
                onClick={() => setRightView("software")}
              >
                Software
              </button>
            )}
          </div>
        )}
      </div>

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

      <div className="mame-browser-grid">
        <MachineFilterPanel active={filter} onChange={setFilter} />

        <section
          className="mame-list-region"
          aria-label="Machine list"
          aria-busy={loadState.status === "loading"}
        >
          <div className="mame-region-heading">Machines</div>
          {loadState.status === "loading" && (
            <div className="mame-panel-state">Loading catalog…</div>
          )}
          {loadState.status === "error" && (
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
                onClick={() => setOffset(Math.max(0, page.offset - page.limit))}
              >
                Previous
              </button>
              <span>{range}</span>
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

        {detail ? (
          <MachineRightPanel
            detail={detail}
            view={rightView}
            onViewChange={setRightView}
            pendingLaunchOverrides={pendingLaunchOverrides}
            onPendingLaunchOverridesChanged={setPendingLaunchOverrides}
            onAuditResultChanged={onAuditResultsChanged}
            onSoftwareSessionStarted={(session) => {
              setPendingLaunchOverrides(null);
              setGameplayInputOwned(isGameplaySessionState(session.state));
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
