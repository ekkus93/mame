import { useEffect, useState } from "react";

import {
  getContentPathConfiguration,
  pickContentDirectory,
  setContentPathConfiguration,
  type ContentPathConfiguration,
  type ContentPaths,
  type PathValidationStatus,
  type PlatformPath,
} from "../backend/pathConfiguration";
import { errorMessage } from "../backend/errors";
import {
  addUniquePath,
  CONTENT_PATH_GROUPS,
  displayPlatformPath,
  movePath,
  removePath,
  type ContentPathKey,
} from "./pathConfigurationModel";
import "./pathConfiguration.css";

const STATUS_LABELS: Record<PathValidationStatus, string> = {
  accessible: "Accessible",
  missing: "Missing",
  notDirectory: "Not a directory",
  permissionDenied: "Permission denied",
  unreadable: "Unreadable",
};

export function PathConfigurationPanel() {
  const [configuration, setConfiguration] = useState<ContentPathConfiguration | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getContentPathConfiguration()
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

  async function persist(key: ContentPathKey, paths: PlatformPath[]) {
    if (!configuration) {
      return;
    }
    const contentPaths: ContentPaths = {
      ...configuration.contentPaths,
      [key]: paths,
    };

    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setContentPathConfiguration(contentPaths);
      setConfiguration(saved);
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function addPath(key: ContentPathKey) {
    if (!configuration) {
      return;
    }

    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const selected = await pickContentDirectory();
      if (selected === null) {
        return;
      }
      const current = configuration.contentPaths[key];
      const next = addUniquePath(current, selected);
      if (next === current) {
        setNotice("That directory is already configured in this path group.");
        return;
      }
      const saved = await setContentPathConfiguration({
        ...configuration.contentPaths,
        [key]: next,
      });
      setConfiguration(saved);
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="path-configuration" aria-labelledby="content-paths-heading">
      <div className="path-configuration__heading">
        <div>
          <p className="eyebrow">Content configuration</p>
          <h2 id="content-paths-heading">ROM and software paths</h2>
          <p>
            Paths are searched in the order shown. Unavailable paths stay configured and are
            reported explicitly instead of being discarded.
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
          Loading content paths…
        </p>
      ) : (
        <div className="path-configuration__groups">
          {CONTENT_PATH_GROUPS.map((group) => {
            const paths = configuration.contentPaths[group.key];
            const validations = configuration.validations[group.key];
            return (
              <section className="path-group" key={group.key} aria-labelledby={`${group.key}-title`}>
                <div className="path-group__header">
                  <h3 id={`${group.key}-title`}>{group.label}</h3>
                  <button type="button" disabled={busy} onClick={() => void addPath(group.key)}>
                    {group.addLabel}
                  </button>
                </div>

                {paths.length === 0 ? (
                  <p className="path-group__empty">No paths configured.</p>
                ) : (
                  <ol className="path-group__list">
                    {paths.map((path, index) => {
                      const validation = validations[index];
                      return (
                        <li className="path-entry" key={`${displayPlatformPath(path)}-${index}`}>
                          <div className="path-entry__details">
                            <code>{displayPlatformPath(path)}</code>
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
                          <div className="path-entry__actions" aria-label={`Actions for path ${index + 1}`}>
                            <button
                              type="button"
                              aria-label="Move path up"
                              disabled={busy || index === 0}
                              onClick={() => void persist(group.key, movePath(paths, index, -1))}
                            >
                              ↑
                            </button>
                            <button
                              type="button"
                              aria-label="Move path down"
                              disabled={busy || index === paths.length - 1}
                              onClick={() => void persist(group.key, movePath(paths, index, 1))}
                            >
                              ↓
                            </button>
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() => void persist(group.key, removePath(paths, index))}
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
            );
          })}
        </div>
      )}
    </section>
  );
}
