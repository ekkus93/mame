import { useEffect, useState } from "react";

import { getLibraryFavorite, setLibraryFavorite } from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { FavoriteState } from "../backend/types";

export function FavoriteToggleButton({
  shortName,
  revision,
  onChanged,
}: {
  shortName: string;
  revision: number;
  onChanged: () => void;
}) {
  const [favorite, setFavorite] = useState<FavoriteState | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setFavorite(null);
    setError(null);
    void getLibraryFavorite({ shortName })
      .then((state) => {
        if (!cancelled) {
          setFavorite(state);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setError(errorMessage(reason));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [shortName, revision]);

  function toggleFavorite() {
    if (!favorite || pending) {
      return;
    }
    setPending(true);
    setError(null);
    void setLibraryFavorite({ shortName, favorite: !favorite.favorite })
      .then((state) => {
        setFavorite(state);
        onChanged();
      })
      .catch((reason: unknown) => setError(errorMessage(reason)))
      .finally(() => setPending(false));
  }

  return (
    <>
      <button
        type="button"
        className="secondary-button"
        disabled={!favorite || pending}
        aria-pressed={favorite?.favorite ?? false}
        onClick={toggleFavorite}
      >
        {pending
          ? "Saving…"
          : favorite?.favorite
            ? "★ Favorited"
            : favorite
              ? "☆ Add favorite"
              : "Loading favorite…"}
      </button>
      {error && (
        <span className="launch-error" role="alert">
          {error}
        </span>
      )}
    </>
  );
}
