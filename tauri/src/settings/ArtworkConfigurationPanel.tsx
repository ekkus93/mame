import { useEffect, useState } from "react";

import {
  getArtworkConfiguration,
  pickArtworkDirectory,
  setArtworkConfiguration,
  type ArtworkConfiguration,
} from "../backend/artwork";
import { errorMessage } from "../backend/errors";
import type { PathValidationStatus, PlatformPath } from "../backend/pathConfiguration";
import {
  addUniquePath,
  displayPlatformPath,
  movePath,
  removePath,
} from "./pathConfigurationModel";
import "./pathConfiguration.css";

const STATUS_LABELS: Record<PathValidationStatus, string> = {
  accessible: "Accessible",
  missing: "Missing",
  notDirectory: "Not a directory",
  permissionDenied: "Permission denied",
  unreadable: "Unreadable",
};

export function ArtworkConfigurationPanel() {
  const [configuration, setConfiguration] = useState<ArtworkConfiguration | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getArtworkConfiguration()
      .then((loaded) => {
        if (!cancelled) {
          setConfiguration(loaded);
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
  }, []);

  async function persist(roots: PlatformPath[]) {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      setConfiguration(await setArtworkConfiguration(roots));
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function addRoot() {
    if (!configuration) {
      return;
    }

    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const selected = await pickArtworkDirectory();
      if (selected === null) {
        return;
      }
      const next = addUniquePath(configuration.roots, selected);
      if (next === configuration.roots) {
        setNotice("That artwork root is already configured.");
        return;
      }
      setConfiguration(await setArtworkConfiguration(next));
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="path-configuration" aria-labelledby="artwork-roots-heading">
      <div className="path-configuration__heading">
        <div>
          <p className="eyebrow">Presentation assets</p>
          <h2 id="artwork-roots-heading">Local artwork roots</h2>
          <p>
            Roots are searched in the order shown. Each root may contain snap, cabinets, marquees,
            flyers, icons, and systems directories with machine-named PNG, JPEG, or WebP files.
            Missing artwork remains an explicit empty result and never changes launch behavior.
          </p>
        </div>
      </div>

      {error && (
        <p className="path-configuration__error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="path-configuration__notice" role="status">
          {notice}
        </p>
      )}

      {!configuration ? (
        <p className="path-configuration__loading" aria-live="polite">
          Loading artwork roots…
        </p>
      ) : (
        <section className="path-group" aria-labelledby="artwork-root-list-heading">
          <div className="path-group__header">
            <h3 id="artwork-root-list-heading">Artwork search roots</h3>
            <button type="button" disabled={busy} onClick={() => void addRoot()}>
              Add artwork directory
            </button>
          </div>

          {configuration.roots.length === 0 ? (
            <p className="path-group__empty">No local artwork roots configured.</p>
          ) : (
            <ol className="path-group__list">
              {configuration.roots.map((root, index) => {
                const validation = configuration.validations[index];
                return (
                  <li className="path-entry" key={`${displayPlatformPath(root)}-${index}`}>
                    <div className="path-entry__details">
                      <code>{displayPlatformPath(root)}</code>
                      {validation ? (
                        <span
                          className={`path-entry__status path-entry__status--${validation.status}`}
                          title={validation.message ?? undefined}
                        >
                          {STATUS_LABELS[validation.status]}
                          {validation.message ? ` — ${validation.message}` : ""}
                        </span>
                      ) : (
                        <span className="path-entry__status">Validation unavailable</span>
                      )}
                    </div>
                    <div
                      className="path-entry__actions"
                      aria-label={`Actions for artwork root ${index + 1}`}
                    >
                      <button
                        type="button"
                        aria-label="Move artwork root up"
                        disabled={busy || index === 0}
                        onClick={() => void persist(movePath(configuration.roots, index, -1))}
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        aria-label="Move artwork root down"
                        disabled={busy || index === configuration.roots.length - 1}
                        onClick={() => void persist(movePath(configuration.roots, index, 1))}
                      >
                        ↓
                      </button>
                      <button
                        type="button"
                        disabled={busy}
                        onClick={() => void persist(removePath(configuration.roots, index))}
                      >
                        Remove
                      </button>
                    </div>
                  </li>
                );
              })}
            </ol>
          )}
        </section>
      )}
    </section>
  );
}
