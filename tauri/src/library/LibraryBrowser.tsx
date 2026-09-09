import { type FormEvent, useEffect, useMemo, useState } from "react";

import { queryMameLibrary } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { MachineListItem, MachinePage, MachineSearchRequest } from "../backend/types";
import {
  buildMachineSearchRequest,
  DEFAULT_LIBRARY_FILTERS,
  type LibraryFilters,
  machineStatusLabel,
} from "./libraryQuery";

type LoadState =
  | { status: "loading" }
  | { status: "ready"; page: MachinePage }
  | { status: "error"; message: string };

export function LibraryBrowser() {
  const [filters, setFilters] = useState<LibraryFilters>(DEFAULT_LIBRARY_FILTERS);
  const [request, setRequest] = useState<MachineSearchRequest>(() =>
    buildMachineSearchRequest(DEFAULT_LIBRARY_FILTERS),
  );
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [selected, setSelected] = useState<MachineListItem | null>(null);

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

  return (
    <section className="library-workspace" aria-labelledby="library-heading">
      <div className="library-heading-row">
        <div>
          <p className="eyebrow">Machine catalog</p>
          <h2 id="library-heading">Library</h2>
        </div>
        {range && <p className="library-range">{range}</p>}
      </div>

      <form className="library-filters" onSubmit={submitFilters}>
        <label className="search-field">
          <span>Search</span>
          <input
            type="search"
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
        <div className="filter-actions">
          <button type="submit">Search</button>
          <button type="button" className="secondary-button" onClick={resetFilters}>
            Reset
          </button>
        </div>
      </form>

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
              {page.items.map((machine) => (
                <li key={machine.shortName}>
                  <button
                    type="button"
                    className={
                      selected?.shortName === machine.shortName
                        ? "machine-row machine-row-selected"
                        : "machine-row"
                    }
                    onClick={() => setSelected(machine)}
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
          {selected ? (
            <>
              <p className="eyebrow">Selected machine</p>
              <h3>{selected.description}</h3>
              <p className="machine-short-name">{selected.shortName}</p>
              <dl className="machine-facts">
                <div>
                  <dt>Manufacturer</dt>
                  <dd>{selected.manufacturer ?? "Unknown"}</dd>
                </div>
                <div>
                  <dt>Year</dt>
                  <dd>{selected.year ?? "Unknown"}</dd>
                </div>
                <div>
                  <dt>Status</dt>
                  <dd>{machineStatusLabel(selected)}</dd>
                </div>
                <div>
                  <dt>Relationship</dt>
                  <dd>{selected.cloneOf ? `Clone of ${selected.cloneOf}` : "Parent"}</dd>
                </div>
                <div>
                  <dt>Displays</dt>
                  <dd>{selected.displayCount}</dd>
                </div>
                <div>
                  <dt>Software lists</dt>
                  <dd>{selected.softwareListCount}</dd>
                </div>
                <div>
                  <dt>Source</dt>
                  <dd>{selected.sourceFile ?? "Unknown"}</dd>
                </div>
              </dl>
              <p className="detail-note">
                Launch and full machine-detail actions are added in the next MT-400 slices.
              </p>
            </>
          ) : (
            <div className="library-state">
              <strong>No machine selected</strong>
              <span>Select a catalog row to inspect its indexed metadata.</span>
            </div>
          )}
        </aside>
      </div>
    </section>
  );
}
