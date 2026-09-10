import { useEffect, useState } from "react";

import { errorMessage } from "../backend/errors";
import {
  getGeneralSettings,
  pickMameExecutable,
  setGeneralLaunchPreferences,
  setGeneralMameExecutable,
  type GeneralSettings,
  type LaunchPreferences,
} from "../backend/generalSettings";
import { PathConfigurationPanel } from "./PathConfigurationPanel";
import "./generalSettings.css";

const DEFAULT_PREFERENCES: LaunchPreferences = {
  windowMode: "inherit",
  renderer: "inherit",
  audio: "inherit",
};

export function GeneralSettingsPanel({
  onContentPathsChanged,
}: {
  onContentPathsChanged?: () => void;
}) {
  const [settings, setSettings] = useState<GeneralSettings | null>(null);
  const [executableDraft, setExecutableDraft] = useState("");
  const [preferencesDraft, setPreferencesDraft] = useState<LaunchPreferences>(DEFAULT_PREFERENCES);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getGeneralSettings()
      .then((loaded) => {
        if (!cancelled) {
          setSettings(loaded);
          setExecutableDraft(loaded.mameExecutable ?? "");
          setPreferencesDraft(loaded.launchPreferences);
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

  async function browseExecutable() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const selected = await pickMameExecutable();
      if (selected !== null) {
        setExecutableDraft(selected);
      }
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function saveExecutable() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setGeneralMameExecutable(executableDraft === "" ? null : executableDraft);
      setSettings(saved);
      setExecutableDraft(saved.mameExecutable ?? "");
      setNotice(
        saved.mameExecutable
          ? "MAME executable validated and saved. Refresh metadata before treating it as the active catalog executable."
          : "Configured MAME executable cleared.",
      );
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function clearExecutable() {
    setExecutableDraft("");
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setGeneralMameExecutable(null);
      setSettings(saved);
      setNotice("Configured MAME executable cleared.");
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function saveLaunchPreferences() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setGeneralLaunchPreferences(preferencesDraft);
      setSettings(saved);
      setPreferencesDraft(saved.launchPreferences);
      setNotice("Launch preferences saved. They apply to new MAME sessions.");
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="general-settings" aria-labelledby="general-settings-heading">
      <div className="general-settings__heading">
        <div>
          <p className="eyebrow">Application configuration</p>
          <h2 id="general-settings-heading">General settings</h2>
          <p>
            Configure the MAME executable, content search paths, and bounded launch preferences.
            Existing MAME-owned INI and CFG files remain authoritative unless a launch preference
            explicitly supplies a command-line override.
          </p>
        </div>
      </div>

      {error && (
        <p className="general-settings__error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="general-settings__notice" role="status">
          {notice}
        </p>
      )}

      {!settings ? (
        <p className="general-settings__loading" aria-live="polite">
          Loading general settings…
        </p>
      ) : (
        <div className="general-settings__body">
          <section className="general-settings__group" aria-labelledby="mame-executable-heading">
            <div>
              <h3 id="mame-executable-heading">MAME executable</h3>
              <p>
                A newly selected executable is validated before it is saved. Changing this setting
                does not silently retarget the active metadata catalog; refresh metadata before the
                new executable becomes authoritative for catalog-backed launches.
              </p>
            </div>
            <div className="general-settings__executable-row">
              <label>
                <span>Executable path</span>
                <input
                  type="text"
                  value={executableDraft}
                  disabled={busy}
                  spellCheck={false}
                  placeholder="Choose a MAME executable"
                  onChange={(event) => setExecutableDraft(event.target.value)}
                />
              </label>
              <div className="general-settings__actions">
                <button
                  type="button"
                  className="secondary-button"
                  disabled={busy}
                  onClick={() => void browseExecutable()}
                >
                  Browse…
                </button>
                <button type="button" disabled={busy} onClick={() => void saveExecutable()}>
                  Save executable
                </button>
                <button
                  type="button"
                  className="secondary-button"
                  disabled={busy || settings.mameExecutable === null}
                  onClick={() => void clearExecutable()}
                >
                  Clear
                </button>
              </div>
            </div>
          </section>

          <section className="general-settings__group" aria-labelledby="launch-preferences-heading">
            <div>
              <h3 id="launch-preferences-heading">Launch preferences</h3>
              <p>
                “Inherit” leaves MAME and its normal INI stack in control. Renderer values are
                requested providers; MAME may fall back when a provider is unavailable. Later
                explainability work will distinguish requested and observed effective providers.
              </p>
            </div>
            <div className="general-settings__preference-grid">
              <label>
                <span>Window mode</span>
                <select
                  value={preferencesDraft.windowMode}
                  disabled={busy}
                  onChange={(event) =>
                    setPreferencesDraft((current) => ({
                      ...current,
                      windowMode: event.target.value as LaunchPreferences["windowMode"],
                    }))
                  }
                >
                  <option value="inherit">Inherit MAME setting</option>
                  <option value="windowed">Windowed</option>
                  <option value="fullscreen">Fullscreen</option>
                </select>
              </label>

              <label>
                <span>Renderer</span>
                <select
                  value={preferencesDraft.renderer}
                  disabled={busy}
                  onChange={(event) =>
                    setPreferencesDraft((current) => ({
                      ...current,
                      renderer: event.target.value as LaunchPreferences["renderer"],
                    }))
                  }
                >
                  <option value="inherit">Inherit MAME setting</option>
                  <option value="auto">Automatic provider</option>
                  <option value="bgfx">BGFX</option>
                  <option value="openGl">OpenGL</option>
                  <option value="software">Software</option>
                </select>
              </label>

              <label>
                <span>Audio</span>
                <select
                  value={preferencesDraft.audio}
                  disabled={busy}
                  onChange={(event) =>
                    setPreferencesDraft((current) => ({
                      ...current,
                      audio: event.target.value as LaunchPreferences["audio"],
                    }))
                  }
                >
                  <option value="inherit">Inherit MAME setting</option>
                  <option value="auto">Automatic provider</option>
                  <option value="disabled">Disabled</option>
                </select>
              </label>
            </div>
            <div className="general-settings__actions">
              <button type="button" disabled={busy} onClick={() => void saveLaunchPreferences()}>
                Save launch preferences
              </button>
            </div>
          </section>

          <PathConfigurationPanel onContentPathsChanged={onContentPathsChanged} />
        </div>
      )}
    </section>
  );
}
