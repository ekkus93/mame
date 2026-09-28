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
import type { MameVersionReport } from "../backend/types";
import { ArtworkConfigurationPanel } from "./ArtworkConfigurationPanel";
import { ControllerConfigurationPanel } from "./ControllerConfigurationPanel";
import { PathConfigurationPanel } from "./PathConfigurationPanel";
import "./generalSettings.css";

const DEFAULT_PREFERENCES: LaunchPreferences = {
  windowMode: "inherit",
  renderer: "inherit",
  audio: "inherit",
};

function runtimeSourceLabel(report: MameVersionReport | undefined): string {
  if (!report) return "Checking runtime status";
  if (report.status === "available") {
    return report.identity.source === "bundled"
      ? "Bundled MAME runtime"
      : "External MAME override";
  }
  if (report.status === "unavailable") return "Runtime unavailable";
  return "Runtime not configured";
}

function runtimeStatusText(report: MameVersionReport | undefined): string {
  if (!report) return "The application is checking the active runtime.";
  if (report.status === "available") {
    const build = report.identity.build ? ` · ${report.identity.build}` : "";
    return `${report.identity.version}${build} · ${report.identity.trust}`;
  }
  if (report.status === "unavailable") {
    return `${report.errorCode}: ${report.errorMessage}`;
  }
  return "No active runtime was reported by the backend.";
}

export function GeneralSettingsPanel({
  mame,
  onContentPathsChanged,
}: {
  mame?: MameVersionReport;
  onContentPathsChanged?: () => void;
}) {
  const [settings, setSettings] = useState<GeneralSettings | null>(null);
  const [executableDraft, setExecutableDraft] = useState("");
  const [preferencesDraft, setPreferencesDraft] = useState<LaunchPreferences>(DEFAULT_PREFERENCES);
  const [showAdvancedRuntimeOverride, setShowAdvancedRuntimeOverride] = useState(false);
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
      const saved = await setGeneralMameExecutable(executableDraft.trim() === "" ? null : executableDraft);
      setSettings(saved);
      setExecutableDraft(saved.mameExecutable ?? "");
      setNotice(
        saved.mameExecutable
          ? "External MAME override validated and saved. Refresh metadata before treating it as the active catalog runtime."
          : "Using bundled MAME runtime.",
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
      setNotice("Using bundled MAME runtime.");
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
            Review the bundled runtime, content search paths, local artwork, and bounded launch
            preferences. Existing MAME-owned INI and CFG files remain authoritative unless a launch
            preference explicitly supplies a command-line override.
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
          <section className="general-settings__group" aria-labelledby="mame-runtime-heading">
            <div>
              <h3 id="mame-runtime-heading">Bundled MAME runtime</h3>
              <p>
                Normal installs use the package-owned MAME runtime. ROMs, CHDs, and software lists
                are still supplied by the user through content paths.
              </p>
            </div>
            <div className="general-settings__executable-row">
              <div>
                <span>{runtimeSourceLabel(mame)}</span>
                <p>{runtimeStatusText(mame)}</p>
                {settings.mameExecutable ? (
                  <p>Advanced override: {settings.mameExecutable}</p>
                ) : (
                  <p>No external runtime override is active.</p>
                )}
              </div>
              <div className="general-settings__actions">
                <button
                  type="button"
                  className="secondary-button"
                  disabled={busy}
                  onClick={() => setShowAdvancedRuntimeOverride((current) => !current)}
                >
                  {showAdvancedRuntimeOverride ? "Hide advanced override" : "Advanced override"}
                </button>
                <button
                  type="button"
                  className="secondary-button"
                  disabled={busy || settings.mameExecutable === null}
                  onClick={() => void clearExecutable()}
                >
                  Use bundled MAME
                </button>
              </div>
            </div>
          </section>

          {showAdvancedRuntimeOverride && (
            <section
              className="general-settings__group"
              aria-labelledby="external-runtime-override-heading"
            >
              <div>
                <h3 id="external-runtime-override-heading">External runtime override</h3>
                <p>
                  Developers and advanced users can point the app at a different MAME executable.
                  Clearing the override restores the bundled runtime.
                </p>
              </div>
              <div className="general-settings__executable-row">
                <label>
                  <span>Override executable path</span>
                  <input
                    type="text"
                    value={executableDraft}
                    disabled={busy}
                    spellCheck={false}
                    placeholder="Optional external override"
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
                    Browse for override…
                  </button>
                  <button type="button" disabled={busy} onClick={() => void saveExecutable()}>
                    Save override
                  </button>
                  <button
                    type="button"
                    className="secondary-button"
                    disabled={busy || settings.mameExecutable === null}
                    onClick={() => void clearExecutable()}
                  >
                    Use bundled MAME
                  </button>
                </div>
              </div>
            </section>
          )}

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

          <ControllerConfigurationPanel allowCapture />

          <PathConfigurationPanel onContentPathsChanged={onContentPathsChanged} />

          <ArtworkConfigurationPanel />
        </div>
      )}
    </section>
  );
}
