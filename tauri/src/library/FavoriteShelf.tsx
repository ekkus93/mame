import { useEffect, useMemo, useState } from "react";

import { queryLibraryFavorites, setLibraryFavorite } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { FavoritePage, MachineListItem } from "../backend/types";
import { machineStatusLabel } from "./libraryQuery";
import "./favorites.css";

const FAVORITES_PAGE_SIZE = 12;

type LoadState =
  | { status: "loading" }
  | { status: "ready"; page: FavoritePage }
  | { status: "error"; message: string };

export function FavoriteShelf({
  revision,
  onSelect,
  onChanged,
}: {
  revision: number;
  onSelect: (machine: MachineListItem) => void;
  onChanged: () => void;
}) {
  const [offset, setOffset] = useState(0);
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    setLoadState({ status: "loading" });
    void queryLibraryFavorites({ limit: FAVORITES_PAGE_SIZE, offset })
      .then((page) => {
        if (cancelled) {
          return;
        }
        if (page.items.length === 0 && page.offset > 0 && page.total > 0) {
          setOffset(Math.max(0, page.offset - page.limit));
          return;
        }
        setLoadState({ status: "ready", page });
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setLoadState({ status: "error", message: errorMessage(reason) });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [offset, revision]);

  const page = loadState.status === "ready" ? loadState.page : null;
  const range = useMemo(() => {
    if (!page || page.total === 0) {
      return null;
    }
    const first = page.offset + 1;
    const last = Math.min(page.offset + page.items.length, page.total);
    return `${first.toLocaleString()}–${last.toLocaleString()} of ${page.total.toLocaleString()}`;
  }, [page]);

  function removeFavorite(shortName: string) {
    void setLibraryFavorite({ shortName, favorite: false })
      .then(() => onChanged())
      .catch((reason: unknown) => setLoadState({ status: "error", message: errorMessage(reason) }));
  }

  return (
    <section className="favorites-shelf" aria-labelledby="favorites-heading">
      <div className="favorites-heading-row">
        <div>
          <p className="eyebrow">Saved machines</p>
          <h3 id="favorites-heading">Favorites</h3>
        </div>
        {range && <p className="library-range">{range}</p>}
      </div>

      {loadState.status === "loading" && (
        <div className="library-state">
          <strong>Loading favorites…</strong>
        </div>
      )}
      {loadState.status === "error" && (
        <div className="library-state error-state" role="alert">
          <strong>Favorites unavailable</strong>
          <span>{loadState.message}</span>
        </div>
      )}
      {page && page.total === 0 && (
        <div className="library-state favorites-empty">
          <strong>No favorites yet</strong>
          <span>Select a machine and use “Add favorite” to keep it here.</span>
        </div>
      )}
      {page && page.items.length > 0 && (
        <ul className="favorites-list" aria-label="Favorite machines">
          {page.items.map((entry) => (
            <li key={entry.shortName} className="favorite-entry">
              {entry.machine ? (
                <button
                  type="button"
                  className="favorite-machine-button"
                  onClick={() => onSelect(entry.machine as MachineListItem)}
                >
                  <span>
                    <strong>{entry.machine.description}</strong>
                    <code>{entry.machine.shortName}</code>
                  </span>
                  <span>{machineStatusLabel(entry.machine)}</span>
                </button>
              ) : (
                <div className="favorite-missing">
                  <strong>{entry.shortName}</strong>
                  <span>Unavailable in the current MAME catalog</span>
                </div>
              )}
              <button
                type="button"
                className="secondary-button favorite-remove"
                onClick={() => removeFavorite(entry.shortName)}
                aria-label={`Remove ${entry.shortName} from favorites`}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}

      {page && page.total > page.limit && (
        <nav className="pagination" aria-label="Favorite pages">
          <button
            type="button"
            className="secondary-button"
            disabled={page.offset === 0}
            onClick={() => setOffset(Math.max(0, page.offset - page.limit))}
          >
            Previous favorites
          </button>
          <button
            type="button"
            className="secondary-button"
            disabled={page.offset + page.items.length >= page.total}
            onClick={() => setOffset(page.offset + page.limit)}
          >
            Next favorites
          </button>
        </nav>
      )}
    </section>
  );
}
