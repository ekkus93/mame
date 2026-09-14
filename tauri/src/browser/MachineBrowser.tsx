import {
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
import type { MachineDetail, MachineListItem, MachinePage, SessionSnapshot } from "../backend/types";
import { FavoriteToggleButton } from "../library/FavoriteToggleButton";
import { MachineArtworkPanel } from "../library/MachineArtworkPanel";
import {
  isEditableElement,
  isGameplaySessionState,
  nextMachineIndex,
  resolveLibraryShortcut,
} from "../library/keyboardNavigation";
import { machineAvailabilityLabel, machineStatusLabel } from "../library/libraryQuery";
import {
  buildMachineBrowserRequest,
  MAME_MACHINE_FILTERS,
  machinePageRange,
  type MameMachineFilterId,
} from "./machineBrowserModel";
import "./MachineBrowser.css";

const SEARCH_DEBOUNCE_MS = 160;
const PAGE_NAVIGATION_ROWS = 12;

type LoadState =
  | { status: "loading"; previous: MachinePage | null }
  | { status: "ready"; page: MachinePage }
  | { status: "error"; message: string; previous: MachinePage | null };

type DetailState =
  | { status: "idle" }
  | { status: "loading"; shortName: string }
  | { status: "ready"; detail: MachineDetail }
  | { status: "error"; shortName: string; message: string };

type LaunchState =
  | { status: "idle" }
  | { status: "launching"; shortName: string }
  | { status: "launched"; session: SessionSnapshot }
  | { status: "error"; shortName: string; message: string };

type RightPanelMode = "images" | "info";

export function MachineBrowser({
  availabilityRevision,
  onAvailabilityChanged,
}: {
  availabilityRevision: number;
  onAvailabilityChanged: () => void;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const rowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const querySequence = useRef(0);
  const detailSequence = useRef(0);

  const [filter, setFilter] = useState<MameMachineFilterId>("all");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading", previous: null });
  const [selected, setSelected] = useState<MachineListItem | null>(null);
  const [detailState, setDetailState] = useState<DetailState>({ status: "idle" });
  const [launchState, setLaunchState] = useState<LaunchState>({ status: "idle" });
  const [rightPanelMode, setRightPanelMode] = useState<RightPanelMode>("images");
  const [favoritesRevision, setFavoritesRevision] = useState(0);
  // Fail closed until Rust confirms the WebView owns application shortcuts.
  const [gameplayInputOwned, setGameplayInputOwned] = useState(true);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search), SEARCH_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    setOffset(0);
  }, [filter, debouncedSearch]);

  const currentPage =
    loadState.status === "ready"
      ? loadState.page
      : loadState.status === "loading" || loadState.status === "error"
        ? loadState.previous
        : null;

  useEffect(() => {
    const sequence = ++querySequence.current;
    const previous = currentPage;
    setLoadState({ status: "loading", previous });
    const request = buildMachineBrowserRequest(filter, debouncedSearch, offset);

    void queryMameLibrary(request)
      .then((page) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "ready", page });
        setSelected((current) => {
          if (current && page.items.some((item) => item.shortName === current.shortName)) {
            return page.items.find((item) => item.shortName === current.shortName) ?? current;
          }
          return page.items[0] ?? null;
        });
      })
      .catch((reason: unknown) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "error", message: errorMessage(reason), previous });
        if (!previous) setSelected(null);
      });
  }, [availabilityRevision, debouncedSearch, filter, offset]);

  useEffect(() => {
    if (!selected) {
      setDetailState({ status: "idle" });
      return;
    }
    const sequence = ++detailSequence.current;
    const shortName = selected.shortName;
    setDetailState({ status: "loading", shortName });
    void getMameMachineDetail({ shortName })
      .then((detail) => {
        if (detailSequence.current === sequence) setDetailState({ status: "ready", detail });
      })
      .catch((reason: unknown) => {
        if (detailSequence.current === sequence) {
          setDetailState({ status: "error", shortName, message: errorMessage(reason) });
        }
      });
  }, [selected]);

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

  const launchMachine = useCallback(
    (machine: MachineListItem) => {
      if (!machine.runnable || gameplayInputOwned || launchState.status === "launching") return;
      setLaunchState({ status: "launching", shortName: machine.shortName });
      void launchLibraryMachine({ shortName: machine.shortName })
        .then((session) => {
          setLaunchState({ status: "launched", session });
          setGameplayInputOwned(isGameplaySessionState(session.state));
        })
        .catch((reason: unknown) => {
          setLaunchState({
            status: "error",
            shortName: machine.shortName,
            message: errorMessage(reason),
          });
          refreshGameplayOwnership();
        });
    },
    [gameplayInputOwned, launchState.status, refreshGameplayOwnership],
  );

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if (event.key === "Escape" && search.length > 0 && !gameplayInputOwned) {
        event.preventDefault();
        setSearch("");
        searchInputRef.current?.focus();
        return;
      }

      const shortcut = resolveLibraryShortcut({
        key: event.key,
        ctrlKey: event.ctrlKey,
        metaKey: event.metaKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        repeat: event.repeat,
        defaultPrevented: event.defaultPrevented,
        editing: isEditableElement(event.target),
        gameplayActive: gameplayInputOwned,
      });
      if (shortcut === "focusSearch") {
        event.preventDefault();
        searchInputRef.current?.focus();
        return;
      }
      if (shortcut === "launchSelected" && selected) {
        event.preventDefault();
        launchMachine(selected);
      }
    };
    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [gameplayInputOwned, launchMachine, search.length, selected]);

  const page = currentPage;
  const selectedIndex = useMemo(() => {
    if (!page || !selected) return -1;
    return page.items.findIndex((item) => item.shortName === selected.shortName);
  }, [page, selected]);

  const range = page ? machinePageRange(page.offset, page.items.length, page.total) : null;

  const moveSelection = (index: number) => {
    if (!page?.items[index]) return;
    setSelected(page.items[index]);
    window.requestAnimationFrame(() => rowRefs.current[index]?.focus());
  };

  const handleRowKeyDown = (
    event: ReactKeyboardEvent<HTMLButtonElement>,
    index: number,
    machine: MachineListItem,
  ) => {
    if (event.key === "Enter") {
      event.preventDefault();
      launchMachine(machine);
      return;
    }
    const next = nextMachineIndex(
      event.key,
      index,
      page?.items.length ?? 0,
      PAGE_NAVIGATION_ROWS,
    );
    if (next !== null) {
      event.preventDefault();
      moveSelection(next);
    }
  };

  const selectedAvailability =
    selected && page ? (page.availabilityByShortName[selected.shortName] ?? "unknown") : "unknown";

  return (
    <div className="mame-browser" data-testid="mame-machine-browser">
      <aside className="mame-browser__filters" aria-label="Machine filters">
        <h2>Filters</h2>
        <div className="mame-browser__filter-list" role="list">
          {MAME_MACHINE_FILTERS.map((definition) => (
            <button
              key={definition.id}
              type="button"
              className={definition.id === filter ? "is-selected" : undefined}
              aria-pressed={definition.id === filter}
              title={definition.description}
              onClick={() => setFilter(definition.id)}
            >
              {definition.label}
            </button>
          ))}
        </div>
        <p className="mame-browser__filter-note">
          Additional canonical MAME filters are being enabled as their typed query predicates land.
        </p>
      </aside>

      <section className="mame-browser__center" aria-label="Machine selection">
        <div className="mame-browser__searchbar">
          <label htmlFor="mame-machine-search">Search</label>
          <input
            id="mame-machine-search"
            ref={searchInputRef}
            type="search"
            value={search}
            autoComplete="off"
            spellCheck={false}
            placeholder="Type to search systems…"
            onChange={(event) => setSearch(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape" && search.length > 0) {
                event.preventDefault();
                setSearch("");
              }
            }}
          />
          <kbd>/</kbd>
          <span className="mame-browser__range">{range ?? "No matching systems"}</span>
        </div>

        {loadState.status === "error" && (
          <div className="mame-browser__inline-error" role="alert">
            {loadState.message}
          </div>
        )}

        <div className="mame-browser__list-wrap" aria-busy={loadState.status === "loading"}>
          {page && page.items.length > 0 ? (
            <ul className="mame-browser__list" role="listbox" aria-label="MAME systems">
              {page.items.map((machine, index) => {
                const availability = page.availabilityByShortName[machine.shortName] ?? "unknown";
                const active = machine.shortName === selected?.shortName;
                return (
                  <li
                    key={machine.shortName}
                    role="option"
                    aria-selected={active}
                    className={active ? "is-selected" : undefined}
                  >
                    <button
                      ref={(node) => {
                        rowRefs.current[index] = node;
                      }}
                      type="button"
                      tabIndex={active || (selectedIndex < 0 && index === 0) ? 0 : -1}
                      onClick={() => setSelected(machine)}
                      onDoubleClick={() => launchMachine(machine)}
                      onKeyDown={(event) => handleRowKeyDown(event, index, machine)}
                    >
                      <span className="mame-browser__machine-name">{machine.description}</span>
                      <code>{machine.shortName}</code>
                      <span className="mame-browser__machine-meta">
                        {machine.year ?? "—"} · {machine.manufacturer ?? "Unknown manufacturer"}
                      </span>
                      <span className="mame-browser__machine-flags">
                        <span>{machineStatusLabel(machine)}</span>
                        <span>{machineAvailabilityLabel(availability)}</span>
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          ) : loadState.status === "loading" ? (
            <div className="mame-browser__empty" aria-live="polite">
              Loading MAME systems…
            </div>
          ) : (
            <div className="mame-browser__empty">No systems match this filter and search.</div>
          )}
        </div>

        {page && page.total > page.limit && (
          <div className="mame-browser__pagination" aria-label="Machine list pages">
            <button
              type="button"
              disabled={page.offset === 0 || loadState.status === "loading"}
              onClick={() => setOffset(Math.max(0, page.offset - page.limit))}
            >
              Previous
            </button>
            <span>{range}</span>
            <button
              type="button"
              disabled={
                page.offset + page.items.length >= page.total || loadState.status === "loading"
              }
              onClick={() => setOffset(page.offset + page.limit)}
            >
              Next
            </button>
          </div>
        )}
      </section>

      <aside className="mame-browser__right" aria-label="Selected machine details">
        <div className="mame-browser__right-tabs" role="tablist" aria-label="Machine detail mode">
          <button
            type="button"
            role="tab"
            aria-selected={rightPanelMode === "images"}
            className={rightPanelMode === "images" ? "is-selected" : undefined}
            onClick={() => setRightPanelMode("images")}
          >
            Images
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={rightPanelMode === "info"}
            className={rightPanelMode === "info" ? "is-selected" : undefined}
            onClick={() => setRightPanelMode("info")}
          >
            Info
          </button>
        </div>

        {selected ? (
          <>
            <div className="mame-browser__selected-heading">
              <div>
                <h2>{selected.description}</h2>
                <code>{selected.shortName}</code>
              </div>
              <span>{machineAvailabilityLabel(selectedAvailability)}</span>
            </div>

            <div className="mame-browser__toolbar" role="toolbar" aria-label="Selected machine actions">
              <FavoriteToggleButton
                shortName={selected.shortName}
                revision={favoritesRevision}
                onChanged={() => setFavoritesRevision((value) => value + 1)}
              />
              <button
                type="button"
                className="mame-browser__start"
                disabled={!selected.runnable || gameplayInputOwned || launchState.status === "launching"}
                title={gameplayInputOwned ? "A native MAME session currently owns gameplay input" : undefined}
                onClick={() => launchMachine(selected)}
              >
                {launchState.status === "launching" && launchState.shortName === selected.shortName
                  ? "Starting…"
                  : "Start"}
              </button>
            </div>

            {launchState.status === "error" && launchState.shortName === selected.shortName && (
              <div className="mame-browser__inline-error" role="alert">
                {launchState.message}
              </div>
            )}

            <div className="mame-browser__right-content" role="tabpanel">
              {rightPanelMode === "images" ? (
                <MachineArtworkPanel machine={selected.shortName} />
              ) : detailState.status === "ready" &&
                detailState.detail.shortName === selected.shortName ? (
                <MachineInfo detail={detailState.detail} availability={selectedAvailability} />
              ) : detailState.status === "error" &&
                detailState.shortName === selected.shortName ? (
                <div className="mame-browser__inline-error" role="alert">
                  {detailState.message}
                </div>
              ) : (
                <div className="mame-browser__empty" aria-live="polite">
                  Loading machine information…
                </div>
              )}
            </div>
          </>
        ) : (
          <div className="mame-browser__empty">Select a system to view artwork and information.</div>
        )}
      </aside>
    </div>
  );
}

function MachineInfo({
  detail,
  availability,
}: {
  detail: MachineDetail;
  availability: "available" | "missing" | "unknown";
}) {
  return (
    <div className="mame-browser__info">
      <dl>
        <div>
          <dt>Description</dt>
          <dd>{detail.description}</dd>
        </div>
        <div>
          <dt>Short name</dt>
          <dd>
            <code>{detail.shortName}</code>
          </dd>
        </div>
        <div>
          <dt>Year</dt>
          <dd>{detail.year ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Manufacturer</dt>
          <dd>{detail.manufacturer ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{detail.driverStatus ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Media</dt>
          <dd>{machineAvailabilityLabel(availability)}</dd>
        </div>
        <div>
          <dt>Parent</dt>
          <dd>{detail.parentDescription ?? detail.cloneOf ?? "—"}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>{detail.sourceFile ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Software lists</dt>
          <dd>{detail.softwareLists.length}</dd>
        </div>
        <div>
          <dt>Displays</dt>
          <dd>{detail.displays.length}</dd>
        </div>
      </dl>
      <div className="mame-browser__info-flags" aria-label="Machine characteristics">
        {detail.isBios && <span>BIOS</span>}
        {detail.isMechanical && <span>Mechanical</span>}
        {detail.driverSavestate === "supported" && <span>Save supported</span>}
        {detail.driverRequiresArtwork && <span>Requires artwork</span>}
        {detail.driverIncomplete && <span>Incomplete</span>}
        {detail.driverUnofficial && <span>Unofficial</span>}
      </div>
    </div>
  );
}
