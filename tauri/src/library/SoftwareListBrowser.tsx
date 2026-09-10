import { type FormEvent, useEffect, useMemo, useState } from "react";

import { launchLibrarySoftware, queryMameSoftwareList } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import type {
  MachineDetail,
  SessionSnapshot,
  SoftwareItemSummary,
  SoftwareListPage,
} from "../backend/types";
import "./softwareListBrowser.css";

const SOFTWARE_PAGE_SIZE = 25;

type BrowseState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; page: SoftwareListPage }
  | { status: "error"; message: string };

type SoftwareLaunchState =
  | { status: "idle" }
  | { status: "launching"; shortName: string }
  | { status: "launched"; shortName: string; session: SessionSnapshot }
  | { status: "error"; shortName: string; message: string };

export function SoftwareListBrowser({
  detail,
  launchOverrides,
  onSessionStarted,
}: {
  detail: MachineDetail;
  launchOverrides: LaunchPreferences | null;
  onSessionStarted: (session: SessionSnapshot) => void;
}) {
  const [selectedListName, setSelectedListName] = useState(detail.softwareLists[0]?.name ?? "");
  const [searchText, setSearchText] = useState("");
  const [appliedSearch, setAppliedSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [browseState, setBrowseState] = useState<BrowseState>({ status: "idle" });
  const [launchState, setLaunchState] = useState<SoftwareLaunchState>({
    status: "idle",
  });

  useEffect(() => {
    setSelectedListName(detail.softwareLists[0]?.name ?? "");
    setSearchText("");
    setAppliedSearch("");
    setOffset(0);
    setLaunchState({ status: "idle" });
  }, [detail.shortName, detail.softwareLists]);

  useEffect(() => {
    if (!selectedListName) {
      setBrowseState({ status: "idle" });
      return;
    }

    let cancelled = false;
    setBrowseState({ status: "loading" });
    void queryMameSoftwareList({
      shortName: detail.shortName,
      softwareList: selectedListName,
      text: appliedSearch || null,
      limit: SOFTWARE_PAGE_SIZE,
      offset,
    })
      .then((page) => {
        if (!cancelled) {
          setBrowseState({ status: "ready", page });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setBrowseState({ status: "error", message: errorMessage(error) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [appliedSearch, detail.shortName, offset, selectedListName]);

  const selectedList = useMemo(
    () =>
      detail.softwareLists.find((softwareList) => softwareList.name === selectedListName) ?? null,
    [detail.softwareLists, selectedListName],
  );
  const page = browseState.status === "ready" ? browseState.page : null;
  const range = useMemo(() => {
    if (!page || page.total === 0) {
      return null;
    }
    const first = page.offset + 1;
    const last = Math.min(page.offset + page.items.length, page.total);
    return `${first.toLocaleString()}–${last.toLocaleString()} of ${page.total.toLocaleString()}`;
  }, [page]);

  function selectList(name: string) {
    setSelectedListName(name);
    setSearchText("");
    setAppliedSearch("");
    setOffset(0);
    setLaunchState({ status: "idle" });
  }

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setAppliedSearch(searchText.trim());
    setOffset(0);
  }

  function clearSearch() {
    setSearchText("");
    setAppliedSearch("");
    setOffset(0);
  }

  function launchSoftware(item: SoftwareItemSummary) {
    if (!selectedList) {
      return;
    }
    setLaunchState({ status: "launching", shortName: item.shortName });
    void launchLibrarySoftware({
      shortName: detail.shortName,
      softwareList: selectedList.name,
      softwareItem: item.shortName,
      launchOverrides,
    })
      .then((session) => {
        setLaunchState({ status: "launched", shortName: item.shortName, session });
        onSessionStarted(session);
      })
      .catch((error: unknown) => {
        setLaunchState({
          status: "error",
          shortName: item.shortName,
          message: errorMessage(error),
        });
      });
  }

  return (
    <section className="software-browser" aria-labelledby={`software-${detail.shortName}`}>
      <div className="software-browser-heading">
        <div>
          <h4 id={`software-${detail.shortName}`}>Software lists</h4>
          <p>Browse MAME-reported software associated with this machine.</p>
        </div>
        {range && <span>{range}</span>}
      </div>

      {detail.softwareLists.length === 0 ? (
        <p className="detail-note">No software lists are associated with this machine.</p>
      ) : (
        <>
          <div className="software-list-tabs" role="group" aria-label="Associated software lists">
            {detail.softwareLists.map((softwareList) => (
              <button
                key={`${softwareList.tag}:${softwareList.name}`}
                type="button"
                className={softwareList.name === selectedListName ? "selected" : undefined}
                aria-pressed={softwareList.name === selectedListName}
                onClick={() => selectList(softwareList.name)}
              >
                <strong>{softwareList.name}</strong>
                <span>{softwareList.status}</span>
              </button>
            ))}
          </div>

          {selectedList && (
            <div className="software-list-context">
              <code>{selectedList.tag}</code>
              <span>Status: {selectedList.status}</span>
              {selectedList.filter && <span>Filter: {selectedList.filter}</span>}
            </div>
          )}

          <form className="software-search" onSubmit={submitSearch}>
            <label>
              <span>Search software</span>
              <input
                type="search"
                value={searchText}
                onChange={(event) => setSearchText(event.target.value)}
                placeholder="Title, short name, year, or publisher"
                autoComplete="off"
              />
            </label>
            <button type="submit">Search</button>
            <button
              type="button"
              className="secondary-button"
              disabled={!searchText && !appliedSearch}
              onClick={clearSearch}
            >
              Clear
            </button>
          </form>

          {browseState.status === "loading" && (
            <div className="software-state" role="status">
              Reading software metadata from MAME…
            </div>
          )}
          {browseState.status === "error" && (
            <div className="software-state launch-error" role="alert">
              {browseState.message}
            </div>
          )}
          {page && page.softwareListDescription && (
            <p className="software-list-description">{page.softwareListDescription}</p>
          )}
          {page && page.items.length === 0 && (
            <div className="software-state">No matching software items were reported by MAME.</div>
          )}
          {page && page.items.length > 0 && (
            <ul className="software-item-list">
              {page.items.map((item) => {
                const launching =
                  launchState.status === "launching" && launchState.shortName === item.shortName;
                return (
                  <li key={item.shortName}>
                    <div className="software-item-primary">
                      <strong>{item.description}</strong>
                      <code>{item.shortName}</code>
                    </div>
                    <div className="software-item-meta">
                      <span>{item.year}</span>
                      <span>{item.publisher}</span>
                      <span>Support: {item.supported}</span>
                      {item.cloneOf && <span>Clone of {item.cloneOf}</span>}
                    </div>
                    <button type="button" disabled={launching} onClick={() => launchSoftware(item)}>
                      {launching ? "Launching…" : "Launch software"}
                    </button>
                    {launchState.status === "error" && launchState.shortName === item.shortName && (
                      <p className="launch-error" role="alert">
                        {launchState.message}
                      </p>
                    )}
                    {launchState.status === "launched" &&
                      launchState.shortName === item.shortName && (
                        <p className="launch-result" role="status">
                          Session {launchState.session.sessionId} started.
                        </p>
                      )}
                  </li>
                );
              })}
            </ul>
          )}

          {page && page.total > page.limit && (
            <nav className="software-pagination" aria-label="Software-list pages">
              <button
                type="button"
                className="secondary-button"
                disabled={page.offset === 0}
                onClick={() => setOffset(Math.max(0, page.offset - page.limit))}
              >
                Previous
              </button>
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
        </>
      )}
    </section>
  );
}
