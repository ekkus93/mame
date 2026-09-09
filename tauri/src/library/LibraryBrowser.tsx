import {
  type FormEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
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
import type {
  MachineDetail,
  MachineListItem,
  MachinePage,
  MachineSearchRequest,
  SessionSnapshot,
} from "../backend/types";
import {
  buildMachineSearchRequest,
  DEFAULT_LIBRARY_FILTERS,
  type LibraryFilters,
  machineStatusLabel,
} from "./libraryQuery";
import { FavoriteShelf } from "./FavoriteShelf";
import { FavoriteToggleButton } from "./FavoriteToggleButton";
import {
  isEditableElement,
  isGameplaySessionState,
  nextMachineIndex,
  resolveLibraryShortcut,
} from "./keyboardNavigation";
import { SoftwareListBrowser } from "./SoftwareListBrowser";
import "./keyboardNavigation.css";

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

export function LibraryBrowser() {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const machineRowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const [filters, setFilters] = useState<LibraryFilters>(DEFAULT_LIBRARY_FILTERS);
  const [request, setRequest] = useState<MachineSearchRequest>(() =>
    buildMachineSearchRequest(DEFAULT_LIBRARY_FILTERS),
  );
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [selected, setSelected] = useState<MachineListItem | null>(null);
  const [detailState, setDetailState] = useState<DetailState>({ status: "idle" });
  const [launchState, setLaunchState] = useState<LaunchState>({ status: "idle" });
  const [favoritesRevision, setFavoritesRevision] = useState(0);
  // Fail closed until Rust confirms that the Tauri window owns application shortcuts.
  const [gameplayInputOwned, setGameplayInputOwned] = useState(true);

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
    let cancelled = false;
    setLoadState({ status: "loading" });

    void queryMameLibrary(request)
      .then((page) => {
        if (!cancelled) {
          setLoadState({ status: "ready", page });
          setSelected((current) => {
            if (current && page.items.some((item) => item.shortName === current.shortName)) {
              return current;
            }
            return page.items[0] ?? null;
          });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setLoadState({ status: "error", message: errorMessage(error) });
          setSelected(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [request]);

  useEffect(() => {
    if (!selected) {
      setDetailState({ status: "idle" });
      setLaunchState({ status: "idle" });
      return;
    }

    let cancelled = false;
    setDetailState({ status: "loading" });
    setLaunchState({ status: "idle" });
    void getMameMachineDetail({ shortName: selected.shortName })
      .then((detail) => {
        if (!cancelled) {
          setDetailState({ status: "ready", detail });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setDetailState({ status: "error", message: errorMessage(error) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selected]);

  const page = loadState.status === "ready" ? loadState.page : null;
  const range = useMemo(() => {
    if (!page || page.total === 0) {
      return null;
    }
    const first = page.offset + 1;
    const last = Math.min(page.offset + page.items.length, page.total);
    return `${first.toLocaleString()}–${last.toLocaleString()} of ${page.total.toLocaleString()}`;
  }, [page]);

  function submitFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setRequest(buildMachineSearchRequest(filters));
  }

  function resetFilters() {
    setFilters(DEFAULT_LIBRARY_FILTERS);
    setRequest(buildMachineSearchRequest(DEFAULT_LIBRARY_FILTERS));
  }

  function movePage(offset: number) {
    setRequest(buildMachineSearchRequest(filters, offset));
  }

  const launchSelected = useCallback(
    (detail: MachineDetail) => {
      setLaunchState({ status: "launching" });
      void launchLibraryMachine({ shortName: detail.shortName })
        .then((session) => {
          setLaunchState({ status: "launched", session });
          setGameplayInputOwned(isGameplaySessionState(session.state));
        })
        .catch((error: unknown) => {
          setLaunchState({ status: "error", message: errorMessage(error) });
          refreshGameplayOwnership();
        });
    },
    [refreshGameplayOwnership],
  );

  const selectedRowIndex = useMemo(() => {
    if (!page || page.items.length === 0) {
      return -1;
    }
    const selectedIndex = page.items.findIndex(
      (machine) => machine.shortName === selected?.shortName,
    );
    return selectedIndex >= 0 ? selectedIndex : 0;
  }, [page, selected]);

  function handleMachineRowKeyDown(
    event: ReactKeyboardEvent<HTMLButtonElement>,
    index: number,
    machines: MachineListItem[],
  ) {
    const nextIndex = nextMachineIndex(event.key, index, machines.length);
    if (nextIndex === null) {
      return;
    }
    event.preventDefault();
    const nextMachine = machines[nextIndex];
    if (!nextMachine) {
      return;
    }
    setSelected(nextMachine);
    machineRowRefs.current[nextIndex]?.focus();
  }

  useEffect(() => {
    function handleShortcut(event: KeyboardEvent) {
      const action = resolveLibraryShortcut({
        key: event.key,
        ctrlKey: event.ctrlKey,
        metaKey: event.metaKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        repeat: event.repeat,
        defaultPrevented: event.defaultPrevented,
        editing: isEditableElement(event.target),
        gameplayActive: gameplayInputOwned || !document.hasFocus(),
      });

      if (action === "focusSearch") {
        event.preventDefault();
        searchInputRef.current?.focus();
        return;
      }

      if (
        action === "launchSelected" &&
        detailState.status === "ready" &&
        detailState.detail.runnable &&
        launchState.status !== "launching"
      ) {
        event.preventDefault();
        launchSelected(detailState.detail);
      }
    }

    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [detailState, gameplayInputOwned, launchSelected, launchState.status]);

  return (
    <section className="library-workspace" aria-labelledby="library-heading">
      <div className="library-heading-row">
        <div>
          <p className="eyebrow">Machine catalog</p>
          <h2 id="library-heading">Library</h2>
        </div>
        {range && <p className="library-range">{range}</p>}
      </div>
      <p className="library-keyboard-help">
        Keyboard: <kbd>/</kbd> search · <kbd>↑</kbd>/<kbd>↓</kbd>/<kbd>Home</kbd>/<kbd>End</kbd>{" "}
        browse · <kbd>Ctrl/⌘+Enter</kbd> launch. Shortcuts pause while MAME owns gameplay input.
      </p>

      <form className="library-filters" onSubmit={submitFilters}>
        <label className="search-field">
          <span>Search</span>
          <input
            ref={searchInputRef}
            type="search"
            aria-keyshortcuts="/"
            value={filters.text}
            onChange={(event) => setFilters({ ...filters, text: event.target.value })}
            placeholder="Description or short name"
            autoComplete="off"
          />
        </label>
        <label>
          <span>Manufacturer</span>
          <input
            value={filters.manufacturer}
            onChange={(event) => setFilters({ ...filters, manufacturer: event.target.value })}
            placeholder="Namco"
            autoComplete="off"
          />
        </label>
        <label>
          <span>Year</span>
          <input
            value={filters.year}
            onChange={(event) => setFilters({ ...filters, year: event.target.value })}
            placeholder="1980"
            inputMode="numeric"
            autoComplete="off"
          />
        </label>
        <label>
          <span>Status</span>
          <select
            value={filters.driverStatus}
            onChange={(event) =>
              setFilters({
                ...filters,
                driverStatus: event.target.value as LibraryFilters["driverStatus"],
              })
            }
          >
            <option value="">Any status</option>
            <option value="good">Working</option>
            <option value="imperfect">Imperfect</option>
            <option value="preliminary">Preliminary</option>
          </select>
        </label>
        <label>
          <span>Relationship</span>
          <select
            value={filters.cloneFilter}
            onChange={(event) =>
              setFilters({
                ...filters,
                cloneFilter: event.target.value as LibraryFilters["cloneFilter"],
              })
            }
          >
            <option value="all">Parents and clones</option>
            <option value="parentsOnly">Parents only</option>
            <option value="clonesOnly">Clones only</option>
          </select>
        </label>
        <label>
          <span>Sort</span>
          <select
            value={filters.sort}
            onChange={(event) =>
              setFilters({ ...filters, sort: event.target.value as LibraryFilters["sort"] })
            }
          >
            <option value="descriptionAsc">Description A–Z</option>
            <option value="descriptionDesc">Description Z–A</option>
            <option value="shortNameAsc">Short name A–Z</option>
            <option value="yearAsc">Year oldest first</option>
            <option value="yearDesc">Year newest first</option>
            <option value="manufacturerAsc">Manufacturer A–Z</option>
            <option value="manufacturerDesc">Manufacturer Z–A</option>
          </select>
        </label>
        <div className="filter-actions">
          <button type="submit">Apply</button>
          <button type="button" className="secondary-button" onClick={resetFilters}>
            Reset
          </button>
        </div>
      </form>

      <FavoriteShelf
        revision={favoritesRevision}
        onSelect={setSelected}
        onChanged={() => setFavoritesRevision((current) => current + 1)}
      />

      <div className="library-content" aria-busy={loadState.status === "loading"}>
        <div className="machine-list-panel">
          {loadState.status === "loading" && (
            <div className="library-state">
              <strong>Loading catalog…</strong>
              <span>Querying the local MAME metadata index.</span>
            </div>
          )}

          {loadState.status === "error" && (
            <div className="library-state error-state" role="alert">
              <strong>Catalog unavailable</strong>
              <span>{loadState.message}</span>
              <button type="button" onClick={() => setRequest({ ...request })}>
                Retry
              </button>
            </div>
          )}

          {page && page.items.length === 0 && (
            <div className="library-state">
              <strong>No machines found</strong>
              <span>Try a broader search or clear the active filters.</span>
              <button type="button" className="secondary-button" onClick={resetFilters}>
                Clear filters
              </button>
            </div>
          )}

          {page && page.items.length > 0 && (
            <ul className="machine-list" aria-label="MAME machines">
              {page.items.map((machine, index) => (
                <li key={machine.shortName}>
                  <button
                    ref={(element) => {
                      machineRowRefs.current[index] = element;
                    }}
                    type="button"
                    tabIndex={index === selectedRowIndex ? 0 : -1}
                    aria-current={selected?.shortName === machine.shortName ? "true" : undefined}
                    className={
                      selected?.shortName === machine.shortName
                        ? "machine-row machine-row-selected"
                        : "machine-row"
                    }
                    onFocus={() => setSelected(machine)}
                    onClick={() => setSelected(machine)}
                    onKeyDown={(event) => handleMachineRowKeyDown(event, index, page.items)}
                  >
                    <span className="machine-primary">
                      <strong>{machine.description}</strong>
                      <span>{machine.shortName}</span>
                    </span>
                    <span className="machine-meta">
                      <span>{machine.year ?? "Year unknown"}</span>
                      <span>{machine.manufacturer ?? "Manufacturer unknown"}</span>
                    </span>
                    <span className={`status-pill status-${machine.driverStatus ?? "unknown"}`}>
                      {machineStatusLabel(machine)}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}

          {page && page.total > page.limit && (
            <nav className="pagination" aria-label="Library pages">
              <button
                type="button"
                className="secondary-button"
                disabled={page.offset === 0}
                onClick={() => movePage(Math.max(0, page.offset - page.limit))}
              >
                Previous
              </button>
              <button
                type="button"
                className="secondary-button"
                disabled={page.offset + page.items.length >= page.total}
                onClick={() => movePage(page.offset + page.limit)}
              >
                Next
              </button>
            </nav>
          )}
        </div>

        <aside className="machine-detail-panel" aria-label="Selected machine details">
          {detailState.status === "idle" && (
            <div className="library-state">
              <strong>No machine selected</strong>
              <span>Select a catalog row to inspect its indexed metadata.</span>
            </div>
          )}
          {detailState.status === "loading" && (
            <div className="library-state">
              <strong>Loading machine detail…</strong>
              <span>Reading the selected machine from the active metadata generation.</span>
            </div>
          )}
          {detailState.status === "error" && (
            <div className="library-state error-state" role="alert">
              <strong>Machine detail unavailable</strong>
              <span>{detailState.message}</span>
            </div>
          )}
          {detailState.status === "ready" && (
            <MachineDetailPanel
              detail={detailState.detail}
              launchState={launchState}
              favoriteRevision={favoritesRevision}
              onFavoriteChanged={() => setFavoritesRevision((current) => current + 1)}
              onLaunch={() => launchSelected(detailState.detail)}
              onSoftwareSessionStarted={(session) =>
                setGameplayInputOwned(isGameplaySessionState(session.state))
              }
            />
          )}
        </aside>
      </div>
    </section>
  );
}

function MachineDetailPanel({
  detail,
  launchState,
  favoriteRevision,
  onFavoriteChanged,
  onLaunch,
  onSoftwareSessionStarted,
}: {
  detail: MachineDetail;
  launchState: LaunchState;
  favoriteRevision: number;
  onFavoriteChanged: () => void;
  onLaunch: () => void;
  onSoftwareSessionStarted: (session: SessionSnapshot) => void;
}) {
  return (
    <>
      <p className="eyebrow">Machine detail</p>
      <h3>{detail.description}</h3>
      <p className="machine-short-name">{detail.shortName}</p>

      <div className="detail-launch-row">
        <button
          type="button"
          aria-keyshortcuts="Control+Enter Meta+Enter"
          disabled={!detail.runnable || launchState.status === "launching"}
          onClick={onLaunch}
        >
          {launchState.status === "launching" ? "Launching…" : "Launch in MAME"}
        </button>
        <FavoriteToggleButton
          shortName={detail.shortName}
          revision={favoriteRevision}
          onChanged={onFavoriteChanged}
        />
        {!detail.runnable && <span>This catalog entry is not runnable.</span>}
      </div>
      {launchState.status === "launched" && (
        <p className="launch-result" role="status">
          Session {launchState.session.sessionId} started
          {launchState.session.pid ? ` as PID ${launchState.session.pid}` : ""}.
        </p>
      )}
      {launchState.status === "error" && (
        <p className="launch-error" role="alert">
          {launchState.message}
        </p>
      )}

      <dl className="machine-facts">
        <div>
          <dt>Manufacturer</dt>
          <dd>{detail.manufacturer ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Year</dt>
          <dd>{detail.year ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{machineStatusLabel(detail)}</dd>
        </div>
        <div>
          <dt>Emulation</dt>
          <dd>{detail.driverEmulation ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Save state</dt>
          <dd>{detail.driverSavestate ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Relationship</dt>
          <dd>
            {detail.cloneOf
              ? `Clone of ${detail.parentDescription ?? detail.cloneOf} (${detail.cloneOf})`
              : "Parent"}
          </dd>
        </div>
        <div>
          <dt>ROM relation</dt>
          <dd>{detail.romOf ?? "Independent"}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>{detail.sourceFile ?? "Unknown"}</dd>
        </div>
      </dl>

      <section className="display-detail" aria-labelledby={`displays-${detail.shortName}`}>
        <h4 id={`displays-${detail.shortName}`}>Displays</h4>
        {detail.displays.length === 0 ? (
          <p className="detail-note">No display metadata is recorded for this machine.</p>
        ) : (
          <ul className="display-list">
            {detail.displays.map((display, index) => (
              <li key={`${display.tag ?? "display"}-${index}`}>
                <strong>{display.displayType}</strong>
                <span>
                  {display.width && display.height
                    ? `${display.width}×${display.height}`
                    : "Resolution unknown"}
                </span>
                <span>{display.refreshHz.toFixed(3)} Hz</span>
                <span>{display.rotate === null ? "Rotation unknown" : `${display.rotate}°`}</span>
                {display.tag && <code>{display.tag}</code>}
              </li>
            ))}
          </ul>
        )}
      </section>

      <SoftwareListBrowser detail={detail} onSessionStarted={onSoftwareSessionStarted} />

      {(detail.driverRequiresArtwork ||
        detail.driverUnofficial ||
        detail.driverNoSoundHardware ||
        detail.driverIncomplete) && (
        <section className="driver-flags" aria-label="Driver notes">
          {detail.driverRequiresArtwork && <span>Requires artwork</span>}
          {detail.driverUnofficial && <span>Unofficial</span>}
          {detail.driverNoSoundHardware && <span>No sound hardware</span>}
          {detail.driverIncomplete && <span>Incomplete</span>}
        </section>
      )}
    </>
  );
}
