import { useEffect, useMemo, useState } from "react";

import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import {
  getMachineLaunchSettings,
  resetMachineLaunchSettings,
  setMachineLaunchSettings,
  type MachineLaunchSettings,
} from "../backend/machineSettings";
import "./machineSettings.css";

const INHERITED_PREFERENCES: LaunchPreferences = {
  windowMode: "inherit",
  renderer: "inherit",
  audio: "inherit",
};

function preferenceLabel(value: string): string {
  switch (value) {
    case "inherit":
      return "MAME / inherited";
    case "windowed":
      return "Windowed";
    case "fullscreen":
      return "Fullscreen";
    case "auto":
      return "Automatic";
    case "bgfx":
      return "BGFX";
    case "openGl":
      return "OpenGL";
    case "software":
      return "Software";
    case "disabled":
      return "Disabled";
    default:
      return value;
  }
}

export function MachineSettingsPanel({ shortName }: { shortName: string }) {
  const [settings, setSettings] = useState<MachineLaunchSettings | null>(null);
  const [draft, setDraft] = useState<LaunchPreferences>(INHERITED_PREFERENCES);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setSettings(null);
    setDraft(INHERITED_PREFERENCES);
    setError(null);
    setNotice(null);

    void getMachineLaunchSettings(shortName)
      .then((loaded) => {
        if (!cancelled) {
          setSettings(loaded);
          setDraft(loaded.overrides);
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
  }, [shortName]);

  const hasOverride = useMemo(
    () =>
      draft.windowMode !== "inherit" || draft.renderer !== "inherit" || draft.audio !== "inherit",
    [draft],
  );

  async function save() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setMachineLaunchSettings(shortName, draft);
      setSettings(saved);
      setDraft(saved.overrides);
      setNotice(
        hasOverride
          ? "Machine launch overrides saved. They apply to new sessions for this machine."
          : "Machine settings now inherit the general launch preferences.",
      );
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function reset() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await resetMachineLaunchSettings(shortName);
      setSettings(saved);
      setDraft(saved.overrides);
      setNotice("Machine overrides cleared; this machine now inherits general launch preferences.");
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="machine-settings" aria-labelledby={`machine-settings-${shortName}`}>
      <div className="machine-settings__heading">
        <div>
          <h4 id={`machine-settings-${shortName}`}>Machine launch settings</h4>
          <p>
            Override the general launch preferences for this machine only. “Inherit” falls back to
            the corresponding general setting without editing MAME-owned INI files.
          </p>
        </div>
      </div>

      {error && (
        <p className="machine-settings__error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="machine-settings__notice" role="status">
          {notice}
        </p>
      )}

      {!settings ? (
        <p className="machine-settings__loading" aria-live="polite">
          Loading machine settings…
        </p>
      ) : (
        <>
          <div className="machine-settings__grid">
            <label>
              <span>Window mode override</span>
              <select
                value={draft.windowMode}
                disabled={busy}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    windowMode: event.target.value as LaunchPreferences["windowMode"],
                  }))
                }
              >
                <option value="inherit">Inherit general setting</option>
                <option value="windowed">Windowed</option>
                <option value="fullscreen">Fullscreen</option>
              </select>
            </label>

            <label>
              <span>Renderer override</span>
              <select
                value={draft.renderer}
                disabled={busy}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    renderer: event.target.value as LaunchPreferences["renderer"],
                  }))
                }
              >
                <option value="inherit">Inherit general setting</option>
                <option value="auto">Automatic provider</option>
                <option value="bgfx">BGFX</option>
                <option value="openGl">OpenGL</option>
                <option value="software">Software</option>
              </select>
            </label>

            <label>
              <span>Audio override</span>
              <select
                value={draft.audio}
                disabled={busy}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    audio: event.target.value as LaunchPreferences["audio"],
                  }))
                }
              >
                <option value="inherit">Inherit general setting</option>
                <option value="auto">Automatic provider</option>
                <option value="disabled">Disabled</option>
              </select>
            </label>
          </div>

          <div className="machine-settings__actions">
            <button type="button" disabled={busy} onClick={() => void save()}>
              Save machine settings
            </button>
            <button
              type="button"
              className="secondary-button"
              disabled={busy || !hasOverride}
              onClick={() => void reset()}
            >
              Reset to inherited
            </button>
          </div>

          <div className="machine-settings__effective" aria-label="Effective machine launch values">
            <strong>Effective values for new launches</strong>
            <dl>
              <div>
                <dt>Window</dt>
                <dd>{preferenceLabel(settings.effective.windowMode)}</dd>
              </div>
              <div>
                <dt>Renderer</dt>
                <dd>{preferenceLabel(settings.effective.renderer)}</dd>
              </div>
              <div>
                <dt>Audio</dt>
                <dd>{preferenceLabel(settings.effective.audio)}</dd>
              </div>
            </dl>
          </div>
        </>
      )}
    </section>
  );
}
