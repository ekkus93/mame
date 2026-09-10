import { useEffect, useMemo, useState } from "react";

import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import {
  getMachineLaunchSettings,
  resetMachineLaunchSettings,
  setMachineLaunchSettings,
  type ConfigurationLayer,
  type MachineLaunchSettings,
} from "../backend/machineSettings";
import { ControllerConfigurationPanel } from "./ControllerConfigurationPanel";
import "./machineSettings.css";

const INHERITED_PREFERENCES: LaunchPreferences = {
  windowMode: "inherit",
  renderer: "inherit",
  audio: "inherit",
};

function preferenceLabel(value: string): string {
  switch (value) {
    case "inherit":
      return "MAME decides at runtime";
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

function sourceLayerLabel(layer: ConfigurationLayer): string {
  switch (layer) {
    case "applicationDefaults":
      return "Application default";
    case "mameGlobalDefaults":
      return "MAME / global configuration";
    case "profileDefaults":
      return "General settings";
    case "machineOverrides":
      return "Machine override";
    case "transientLaunchOverrides":
      return "Next-launch override";
  }
}

function normalizePending(preferences: LaunchPreferences): LaunchPreferences | null {
  return preferences.windowMode === "inherit" &&
    preferences.renderer === "inherit" &&
    preferences.audio === "inherit"
    ? null
    : preferences;
}

export function MachineSettingsPanel({
  shortName,
  pendingLaunchOverrides,
  onPendingLaunchOverridesChanged,
}: {
  shortName: string;
  pendingLaunchOverrides: LaunchPreferences | null;
  onPendingLaunchOverridesChanged: (preferences: LaunchPreferences | null) => void;
}) {
  const [settings, setSettings] = useState<MachineLaunchSettings | null>(null);
  const [draft, setDraft] = useState<LaunchPreferences>(INHERITED_PREFERENCES);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const pendingDraft = pendingLaunchOverrides ?? INHERITED_PREFERENCES;

  useEffect(() => {
    let cancelled = false;
    setSettings(null);
    setDraft(INHERITED_PREFERENCES);
    setError(null);
    setNotice(null);

    void getMachineLaunchSettings(shortName, pendingLaunchOverrides)
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
    // The persisted draft is reset only when the selected machine changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [shortName]);

  useEffect(() => {
    let cancelled = false;
    void getMachineLaunchSettings(shortName, pendingLaunchOverrides)
      .then((loaded) => {
        if (!cancelled) {
          setSettings(loaded);
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
  }, [pendingLaunchOverrides, shortName]);

  const hasOverride = useMemo(
    () =>
      draft.windowMode !== "inherit" || draft.renderer !== "inherit" || draft.audio !== "inherit",
    [draft],
  );
  const hasPendingOverride = pendingLaunchOverrides !== null;

  function updatePending<K extends keyof LaunchPreferences>(key: K, value: LaunchPreferences[K]) {
    onPendingLaunchOverridesChanged(normalizePending({ ...pendingDraft, [key]: value }));
  }

  async function save() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = await setMachineLaunchSettings(shortName, draft, pendingLaunchOverrides);
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
      const saved = await resetMachineLaunchSettings(shortName, pendingLaunchOverrides);
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
            Persistent machine overrides sit above general settings. A next-launch override is
            transient, wins over both layers, and clears only after a successful machine or software
            launch. “Inherit” emits no app-owned override and leaves MAME in control.
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

          <section className="machine-settings__transient" aria-label="Next-launch overrides">
            <div className="machine-settings__transient-heading">
              <div>
                <strong>Next launch only</strong>
                <p>
                  These values are not persisted. They apply to the next successful launch for this
                  machine, including software-list launches.
                </p>
              </div>
              <button
                type="button"
                className="secondary-button"
                disabled={!hasPendingOverride}
                onClick={() => onPendingLaunchOverridesChanged(null)}
              >
                Clear next-launch override
              </button>
            </div>
            <div className="machine-settings__grid">
              <label>
                <span>Window</span>
                <select
                  value={pendingDraft.windowMode}
                  onChange={(event) =>
                    updatePending(
                      "windowMode",
                      event.target.value as LaunchPreferences["windowMode"],
                    )
                  }
                >
                  <option value="inherit">No transient override</option>
                  <option value="windowed">Windowed</option>
                  <option value="fullscreen">Fullscreen</option>
                </select>
              </label>
              <label>
                <span>Renderer</span>
                <select
                  value={pendingDraft.renderer}
                  onChange={(event) =>
                    updatePending("renderer", event.target.value as LaunchPreferences["renderer"])
                  }
                >
                  <option value="inherit">No transient override</option>
                  <option value="auto">Automatic provider</option>
                  <option value="bgfx">BGFX</option>
                  <option value="openGl">OpenGL</option>
                  <option value="software">Software</option>
                </select>
              </label>
              <label>
                <span>Audio</span>
                <select
                  value={pendingDraft.audio}
                  onChange={(event) =>
                    updatePending("audio", event.target.value as LaunchPreferences["audio"])
                  }
                >
                  <option value="inherit">No transient override</option>
                  <option value="auto">Automatic provider</option>
                  <option value="disabled">Disabled</option>
                </select>
              </label>
            </div>
          </section>

          <div
            className="machine-settings__effective machine-settings__explanation"
            aria-label="Effective machine launch configuration explanation"
          >
            <strong>Effective values for the next launch</strong>
            <p>
              Renderer and audio values are requested launch configuration. MAME may still choose a
              runtime fallback; this panel does not claim an unobserved provider as active.
            </p>
            <div className="machine-settings__explanation-table" role="table">
              <div className="machine-settings__explanation-header" role="row">
                <span role="columnheader">Option</span>
                <span role="columnheader">Effective value</span>
                <span role="columnheader">Source layer</span>
                <span role="columnheader">Pending next launch</span>
              </div>
              {(
                [
                  ["Window", settings.explanation.windowMode],
                  ["Renderer", settings.explanation.renderer],
                  ["Audio", settings.explanation.audio],
                ] as const
              ).map(([label, explanation]) => (
                <div className="machine-settings__explanation-row" role="row" key={label}>
                  <strong role="cell">{label}</strong>
                  <span role="cell">{preferenceLabel(explanation.effectiveValue)}</span>
                  <span role="cell">{sourceLayerLabel(explanation.sourceLayer)}</span>
                  <span role="cell">
                    {explanation.pendingLaunchOverride === null
                      ? "None"
                      : preferenceLabel(explanation.pendingLaunchOverride)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      <ControllerConfigurationPanel shortName={shortName} />
    </section>
  );
}
