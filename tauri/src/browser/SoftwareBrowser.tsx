import {
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { launchLibraryMachine } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import {
  launchMameSoftware,
  queryMameSoftware,
  type MameSoftwareItem,
  type MameSoftwarePage,
  type SoftwareListFilter,
} from "../backend/mameSoftware";
import type { MameUiPanelMode } from "../backend/mameUiState";
import type { MachineDetail, SessionSnapshot } from "../backend/types";
import { isEditableElement } from "../library/keyboardNavigation";
import {
  nextSoftwareIndex,
  SOFTWARE_FILTERS,
  SOFTWARE_PAGE_SIZE,
  softwareFilterRequiresValue,
} from "./softwareModel";
import "./SoftwareBrowser.css";

type SoftwareMachineDetail = MachineDetail & { canStartEmpty: boolean };

type BrowseState =
  | { status: "awaitingFilterValue" }
  | { status: "loading" }
  | { status: "ready"; page: MameSoftwarePage }
  | { status: "error"; message: string };

type LaunchState =
  | { status: "idle" }
  | { status: "launching"; target: string }
  | { status: "launched"; session: SessionSnapshot }
  | { status: "error"; message: string };

export function SoftwareBrowser({
  detail,
  launchOverrides,
  panelMode,
  onPanelModeChange,
  onBack,
  onSessionStarted,
}: {
  detail: SoftwareMachineDetail;
  launchOverrides: LaunchPreferences | null;
  panelMode: MameUiPanelMode;
  onPanelModeChange: (mode: MameUiPanelMode) => void;
  onBack: () => void;
  onSessionStarted: (session: SessionSnapshot) => void;
}) {
  const searchRef = useRef<HTMLInputElement>(null);
  const rowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const requestSequence = useRef(0);
  const [listName, setListName] = useState(detail.softwareLists[0]?.name ?? "");
  const [filter, setFilter] = useState<SoftwareListFilter>("all");
  const [filterValue, setFilterValue] = useState("");
  const [debouncedFilterValue, setDebouncedFilterValue] = useState("");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [browse, setBrowse] = useState<BrowseState>({ status: "loading" });
  const [selected, setSelected] = useState<MameSoftwareItem | null>(null);
  const [selectedPart, setSelectedPart] = useState<string | null>(null);
  const [launch, setLaunch] = useState<LaunchState>({ status: "idle" });

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedFilterValue(filterValue.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [filterValue]);

  useEffect(() => setOffset(0), [listName, filter, debouncedFilterValue, debouncedSearch]);

  useEffect(() => {
    setSelectedPart(selected?.parts.length === 1 ? (selected.parts[0]?.name ?? null) : null);
    setLaunch({ status: "idle" });
  }, [selected]);

  const requiresValue = softwareFilterRequiresValue(filter);
  useEffect(() => {
    if (!listName) return;
    const sequence = ++requestSequence.current;
    if (requiresValue && !debouncedFilterValue) {
      setBrowse({ status: "awaitingFilterValue" });
      setSelected(null);
      return;
    }
    setBrowse({ status: "loading" });
    void queryMameSoftware({
      shortName: detail.shortName,
      softwareList: listName,
      text: debouncedSearch || null,
      filter,
      filterValue: debouncedFilterValue || null,
      limit: SOFTWARE_PAGE_SIZE,
      offset,
    })
      .then((page) => {
        if (requestSequence.current !== sequence) return;
        setBrowse({ status: "ready", page });
        setSelected((current) => {
          if (current && page.items.some((item) => item.shortName === current.shortName)) {
            return current;
          }
          return page.items[0] ?? null;
        });
      })
      .catch((reason: unknown) => {
        if (requestSequence.current !== sequence) return;
        setBrowse({ status: "error", message: errorMessage(reason) });
        setSelected(null);
      });
  }, [
    debouncedFilterValue,
    debouncedSearch,
    detail.shortName,
    filter,
    listName,
    offset,
    requiresValue,
  ]);

  const page = browse.status === "ready" ? browse.page : null;
  const range = useMemo(() => {
    if (!page || page.total === 0) return "0 software items";
    return `${(page.offset + 1).toLocaleString()}–${Math.min(
      page.offset + page.items.length,
      page.total,
    ).toLocaleString()} of ${page.total.toLocaleString()}`;
  }, [page]);

  const launchItem = useCallback(
    (item: MameSoftwareItem, part: string | null) => {
      if (!listName || launch.status === "launching") return;
      if (item.parts.length > 1 && !part) return;
      setLaunch({ status: "launching", target: item.shortName });
      void launchMameSoftware({
        shortName: detail.shortName,
        softwareList: listName,
        softwareItem: item.shortName,
        softwarePart: part,
        launchOverrides,
      })
        .then((session) => {
          setLaunch({ status: "launched", session });
          onSessionStarted(session);
        })
        .catch((reason: unknown) => setLaunch({ status: "error", message: errorMessage(reason) }));
    },
    [detail.shortName, launch.status, launchOverrides, listName, onSessionStarted],
  );

  const launchSelected = useCallback(() => {
    if (!selected) return;
    launchItem(selected, selectedPart);
  }, [launchItem, selected, selectedPart]);

  const startEmpty = useCallback(() => {
    if (!detail.canStartEmpty || launch.status === "launching") return;
    setLaunch({ status: "launching", target: "empty" });
    void launchLibraryMachine({ shortName: detail.shortName, launchOverrides })
      .then((session) => {
        setLaunch({ status: "launched", session });
        onSessionStarted(session);
      })
      .catch((reason: unknown) => setLaunch({ status: "error", message: errorMessage(reason) }));
  }, [detail.canStartEmpty, detail.shortName, launch.status, launchOverrides, onSessionStarted]);

  function activateItem(item: MameSoftwareItem) {
    setSelected(item);
    if (item.parts.length <= 1) {
      launchItem(item, item.parts[0]?.name ?? null);
    }
  }

  function handleRowKey(event: ReactKeyboardEvent<HTMLButtonElement>, index: number) {
    if (!page) return;
    if (event.key === "Enter") {
      event.preventDefault();
      const item = page.items[index];
      if (item) activateItem(item);
      return;
    }
    const nextIndex = nextSoftwareIndex(event.key, index, page.items.length);
    if (nextIndex === null) return;
    event.preventDefault();
    const next = page.items[nextIndex];
    if (!next) return;
    setSelected(next);
    rowRefs.current[nextIndex]?.focus();
  }

  useEffect(() => {
    function handleKey(event: KeyboardEvent) {
      if (event.defaultPrevented || event.repeat || event.altKey) return;
      if (event.key === "Escape") {
        if (event.target === searchRef.current && search) {
          event.preventDefault();
          setSearch("");
          return;
        }
        if (!isEditableElement(event.target)) {
          event.preventDefault();
          onBack();
        }
        return;
      }
      if (isEditableElement(event.target)) return;
      if (event.key === "/" && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        searchRef.current?.focus();
      }
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onBack, search]);

  return (
    <section className="mame-software-browser" aria-label={`Software for ${detail.description}`}>
      <div className="mame-software-toolbar">
        <button type="button" className="secondary-button" onClick={onBack}>
          ← Machines
        </button>
        <strong>{detail.description}</strong>
        <label className="mame-search">
          <span className="visually-hidden">Search software</span>
          <input
            ref={searchRef}
            type="search"
            value={search}
            placeholder="Search software..."
            autoComplete="off"
            onChange={(event) => setSearch(event.target.value)}
          />
        </label>
        <span className="mame-browser-range" aria-live="polite">
          {range}
        </span>
        {detail.canStartEmpty && (
          <button type="button" className="secondary-button" onClick={startEmpty}>
            Start Empty
          </button>
        )}
        <button
          type="button"
          className="mame-start-button"
          disabled={
            !selected ||
            (selected.parts.length > 1 && !selectedPart) ||
            launch.status === "launching"
          }
          onClick={launchSelected}
        >
          {launch.status === "launching" ? "Starting…" : "Start"}
        </button>
      </div>

      {launch.status === "error" && (
        <div className="mame-browser-banner is-error" role="alert">
          {launch.message}
        </div>
      )}
      {launch.status === "launched" && (
        <div className="mame-browser-banner" role="status">
          Session {launch.session.sessionId} started.
        </div>
      )}

      <div className="mame-software-grid">
        <aside className="mame-filter-panel" aria-label="Software filters">
          <div className="mame-region-heading">Software List</div>
          <div className="mame-filter-list">
            {detail.softwareLists.map((list) => (
              <button
                key={`${list.tag}:${list.name}`}
                type="button"
                className={`mame-filter ${list.name === listName ? "is-selected" : ""}`}
                aria-pressed={list.name === listName}
                onClick={() => {
                  setListName(list.name);
                  setSelected(null);
                  setOffset(0);
                }}
              >
                {list.name}
              </button>
            ))}
          </div>
          <div className="mame-region-heading">Filter</div>
          <div className="mame-filter-list">
            {SOFTWARE_FILTERS.map((candidate) => (
              <button
                key={candidate.id}
                type="button"
                className={`mame-filter ${candidate.id === filter ? "is-selected" : ""}`}
                aria-pressed={candidate.id === filter}
                onClick={() => {
                  setFilter(candidate.id);
                  setFilterValue("");
                }}
              >
                {candidate.label}
              </button>
            ))}
          </div>
          {requiresValue && (
            <label className="mame-software-filter-value">
              <span>{filter === "year" ? "Year" : "Publisher"}</span>
              <input value={filterValue} onChange={(event) => setFilterValue(event.target.value)} />
            </label>
          )}
        </aside>

        <section
          className="mame-list-region"
          aria-label="Software list"
          aria-busy={browse.status === "loading"}
        >
          <div className="mame-region-heading">Software</div>
          {browse.status === "awaitingFilterValue" && (
            <div className="mame-panel-state">Enter a value for the selected filter.</div>
          )}
          {browse.status === "loading" && <div className="mame-panel-state">Loading software…</div>}
          {browse.status === "error" && (
            <div className="mame-panel-state" role="alert">
              {browse.message}
            </div>
          )}
          {page && page.items.length === 0 && (
            <div className="mame-panel-state">No software matches.</div>
          )}
          {page && page.items.length > 0 && (
            <ul className="mame-software-listbox" aria-label="Software results">
              {page.items.map((item, index) => (
                <li key={item.shortName}>
                  <button
                    ref={(element) => {
                      rowRefs.current[index] = element;
                    }}
                    type="button"
                    className={`mame-software-row ${selected?.shortName === item.shortName ? "is-selected" : ""}`}
                    aria-current={selected?.shortName === item.shortName ? "true" : undefined}
                    onClick={() => setSelected(item)}
                    onDoubleClick={() => activateItem(item)}
                    onKeyDown={(event) => handleRowKey(event, index)}
                  >
                    <span className="mame-software-title">{item.description}</span>
                    <span className="mame-machine-short">{item.shortName}</span>
                    <span>{item.year}</span>
                    <span className="mame-software-publisher">{item.publisher}</span>
                    <span>{item.supported}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
          {page && page.total > page.limit && (
            <nav className="mame-pager" aria-label="Software result pages">
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

        <aside className="mame-right-panel" aria-label="Selected software context">
          <div className="mame-right-tabs" role="tablist" aria-label="Software detail view">
            <button
              type="button"
              role="tab"
              aria-selected={panelMode === "images"}
              className={panelMode === "images" ? "is-selected" : ""}
              onClick={() => onPanelModeChange("images")}
            >
              Images
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={panelMode === "info"}
              className={panelMode === "info" ? "is-selected" : ""}
              onClick={() => onPanelModeChange("info")}
            >
              Info
            </button>
          </div>
          {!selected ? (
            <div className="mame-panel-state">Select software.</div>
          ) : panelMode === "images" ? (
            <div className="mame-artwork-frame">
              <div className="mame-panel-state">
                No configured software artwork source is available.
              </div>
            </div>
          ) : (
            <div className="mame-info-pane">
              <h2>{selected.description}</h2>
              <p className="mame-info-short">{selected.shortName}</p>
              <dl>
                <div>
                  <dt>Year</dt>
                  <dd>{selected.year}</dd>
                </div>
                <div>
                  <dt>Publisher</dt>
                  <dd>{selected.publisher}</dd>
                </div>
                <div>
                  <dt>Support</dt>
                  <dd>{selected.supported}</dd>
                </div>
                <div>
                  <dt>Parent</dt>
                  <dd>{selected.cloneOf ?? "Parent"}</dd>
                </div>
                <div>
                  <dt>Software list</dt>
                  <dd>{listName}</dd>
                </div>
              </dl>
              {selected.parts.length > 1 && (
                <label className="mame-software-part-selector">
                  <span>Launch part</span>
                  <select
                    value={selectedPart ?? ""}
                    onChange={(event) => setSelectedPart(event.target.value || null)}
                  >
                    <option value="">Choose a part…</option>
                    {selected.parts.map((part) => (
                      <option key={`${part.name}:${part.interface}`} value={part.name}>
                        {part.name} ({part.interface})
                      </option>
                    ))}
                  </select>
                </label>
              )}
              {selected.parts.length === 1 && selected.parts[0] && (
                <p className="mame-software-part-note">
                  Part: {selected.parts[0].name} ({selected.parts[0].interface})
                </p>
              )}
            </div>
          )}
        </aside>
      </div>
    </section>
  );
}
