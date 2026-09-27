import { useEffect, useState } from "react";

import { getAppInfo } from "../backend/commands";
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

function runtimeTitle(runtime: MameVersionReport | null): string {
  if (!runtime) return "Checking installed MAME runtime…";
  if (runtime.status === "available") {
    return runtime.identity.source === "bundled"
      ? `Bundled MAME ${runtime.identity.version}`
      : `Custom MAME ${runtime.identity.version}`;
  }
  if (runtime.status === "unavailable") return "MAME runtime unavailable";
  return "MAME runtime unavailable";
}

function runtimeDetail(runtime: MameVersionReport | null): string {
  if (!runtime) return "Reading the MAME runtime installed with this application.";
  if (runtime.status === "available") {
    return runtime.identity.source === "bundled"
      ? "Installed with this application. No executable setup is required."
      : "Advanced external executable override is active.";
  }
  if (runtime.status === "unavailable") return runtime.errorMessage;
  return "The installed MAME runtime could not be resolved.";
}

export function GeneralSettingsPanel({
  onContentPathsChanged,
}: {
  onContentPathsChanged?: () => void;
}) {
  const [settings, setSettings] = useState<GeneralSettings | null>(null);
  const [runtime, setRuntime] = useState<MameVersionReport | null>(null);
  const [advancedRuntimeOpen, setAdvancedRuntimeOpen] = useState(false);
  const [executableDraft, setExecutableDraft] = useState("");
  const [preferencesDraft, setPreferencesDraft] = useState<LaunchPreferences>(DEFAULT_PREFERENCES);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void Promise.all([getGeneralSettings(), getAppInfo()])
      .then(([loaded, info]) => {
        if (!cancelled) {
          setSettings(loaded);
          setExecutableDraft(loaded.mameExecutable ?? "");
          setPreferencesDraft(loaded.launchPreferences);
          setRuntime(info.mame);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) setError(errorMessage(reason));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function refreshRuntime() {
    const info = await getAppInfo();
    setRuntime(info.mame);
  }

  async function browseExecutable() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const selected = await pickMameExecutable();
      if (selected !== null) setExecutableDraft(selected);
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function saveExternalOverride() {
    if (!executableDraft.trim()) {
      setError("Choose a custom MAME executable before enabling the override.");
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setGeneralMameExecutable(executableDraft.trim());
      setSettings(saved);
      setExecutableDraft(saved.mameExecutable ?? "");
      await refreshRuntime();
      setNotice("Custom MAME override validated and enabled.");
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function restoreBundledMame() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setGeneralMameExecutable(null);
      setSettings(saved);
      setExecutableDraft("");
      await refreshRuntime();
      setNotice("Using the MAME runtime installed with this application.");
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

  const externalOverrideActive = settings?.mameExecutable !== null;

  return (
    <section className="general-settings" aria-labelledby="general-settings-heading">
      <div className="general-settings__heading">
        <div>
          <p className="eyebrow">Application configuration</p>
          <h2 id="general-settings-heading">General settings</h2>
          <p>
            Configure content search paths, local artwork, controllers, and bounded launch
            preferences. MAME itself is installed with this application.
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
              <h3 id="mame-runtime-heading">MAME runtime</h3>
              <p>
                The packaged MAME runtime is the default. A custom executable is an advanced
                override and is never required for normal use.
              </p>
            </div>

            <div className="general-settings__runtime-status" role="status">
              <strong>{runtimeTitle(runtime)}</strong>
              <span>{runtimeDetail(runtime)}</span>
              {runtime?.status === "available" && (
                <span className="general-settings__runtime-path">
                  Source: {runtime.identity.source}
                  {runtime.identity.source === "external" ? ` · ${runtime.identity.path}` : ""}
                </span>
              )}
            </div>

            <div className="general-settings__actions">
              <button
                type="button"
                className="secondary-button"
                aria-expanded={advancedRuntimeOpen}
                onClick={() => setAdvancedRuntimeOpen((current) => !current)}
              >
                {advancedRuntimeOpen
                  ? "Hide advanced runtime override"
                  : "Advanced runtime override"}
              </button>
              {externalOverrideActive && (
                <button type="button" disabled={busy} onClick={() => void restoreBundledMame()}>
                  Use bundled MAME
                </button>
              )}
            </div>

            {advancedRuntimeOpen && (
              <div className="general-settings__advanced-runtime">
                <p>
                  Developer/expert option only. Selecting another executable changes the effective
                  MAME identity and will cause metadata to refresh before catalog-backed launches.
                </p>
                <div className="general-settings__executable-row">
                  <label>
                    <span>Custom executable override</span>
                    <input
                      type="text"
                      value={executableDraft}
                      disabled={busy}
                      spellCheck={false}
                      placeholder="/path/to/custom/mame"
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
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void saveExternalOverride()}
                    >
                      Use custom MAME
                    </button>
                    <button
                      type="button"
                      className="secondary-button"
                      disabled={busy || !externalOverrideActive}
                      onClick={() => void restoreBundledMame()}
                    >
                      Use bundled MAME
                    </button>
                  </div>
                </div>
              </div>
            )}
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

          <ControllerConfigurationPanel allowCapture />
          <PathConfigurationPanel onContentPathsChanged={onContentPathsChanged} />
          <ArtworkConfigurationPanel />
        </div>
      )}
    </section>
  );
}
