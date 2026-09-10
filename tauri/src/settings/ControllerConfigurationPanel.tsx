import { useCallback, useEffect, useMemo, useState } from "react";

import {
  createBrowserControllerProfile,
  getControllerProfileConfiguration,
  setControllerProfileSelection,
  type ControllerMappingProvenanceKind,
  type ControllerProfileConfiguration,
  type ControllerProfileScope,
} from "../backend/controllerProfiles";
import { errorMessage } from "../backend/errors";
import "./controllerConfiguration.css";

type DetectedGamepad = {
  index: number;
  id: string;
  mapping: string;
};

function provenanceLabel(kind: ControllerMappingProvenanceKind): string {
  switch (kind) {
    case "browserStandardGamepad":
      return "Browser standard-gamepad mapping";
    case "mameControllerConfig":
      return "MAME -ctrlr controller configuration";
    case "mameOsdControllerMap":
      return "MAME OSD controller_map";
    case "projectOwned":
      return "Project-owned mapping";
  }
}

function scopeLabel(scope: ControllerProfileScope | null): string {
  if (scope === null) {
    return "None";
  }
  return scope.kind === "global" ? "Global" : `Machine ${scope.shortName}`;
}

export function ControllerConfigurationPanel({
  shortName,
  allowCapture = false,
}: {
  shortName?: string;
  allowCapture?: boolean;
}) {
  const scope = useMemo<ControllerProfileScope>(
    () => (shortName ? { kind: "machine", shortName } : { kind: "global" }),
    [shortName],
  );
  const [configuration, setConfiguration] = useState<ControllerProfileConfiguration | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [gamepads, setGamepads] = useState<DetectedGamepad[]>([]);
  const [selectedGamepad, setSelectedGamepad] = useState("");
  const [profileName, setProfileName] = useState("");
  const [gamepadApiAvailable, setGamepadApiAvailable] = useState(true);

  const reload = useCallback(async () => {
    const loaded = await getControllerProfileConfiguration(scope);
    setConfiguration(loaded);
  }, [scope]);

  useEffect(() => {
    let cancelled = false;
    setConfiguration(null);
    setError(null);
    void getControllerProfileConfiguration(scope)
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
  }, [scope]);

  async function selectProfile(rawProfileId: string) {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const profileId = rawProfileId === "" ? null : Number(rawProfileId);
      const loaded = await setControllerProfileSelection(scope, profileId);
      setConfiguration(loaded);
      setNotice(
        profileId === null
          ? shortName
            ? "Machine controller assignment cleared; global selection is inherited when present."
            : "Global controller assignment cleared."
          : "Controller profile selection saved. Runtime activation remains separate and is shown below.",
      );
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  function refreshGamepads() {
    setError(null);
    setNotice(null);
    if (typeof navigator.getGamepads !== "function") {
      setGamepadApiAvailable(false);
      setGamepads([]);
      setSelectedGamepad("");
      return;
    }
    setGamepadApiAvailable(true);
    const detected = Array.from(navigator.getGamepads())
      .filter((gamepad): gamepad is Gamepad => gamepad !== null && gamepad.connected)
      .map((gamepad) => ({
        index: gamepad.index,
        id: gamepad.id,
        mapping: gamepad.mapping,
      }));
    setGamepads(detected);
    const firstStandard = detected.find((gamepad) => gamepad.mapping === "standard");
    setSelectedGamepad(firstStandard ? String(firstStandard.index) : "");
    if (firstStandard && profileName === "") {
      setProfileName(firstStandard.id.slice(0, 120));
    }
  }

  async function captureProfile() {
    const gamepad = gamepads.find((candidate) => String(candidate.index) === selectedGamepad);
    if (!gamepad || gamepad.mapping !== "standard") {
      setError("Choose a connected controller reporting the W3C standard mapping.");
      return;
    }
    if (profileName.trim() === "") {
      setError("Enter a profile name.");
      return;
    }

    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await createBrowserControllerProfile({
        name: profileName,
        gamepadId: gamepad.id,
        mapping: gamepad.mapping,
      });
      await reload();
      setNotice(
        "Controller profile captured from the browser-reported device identity. It is saved, not claimed active in MAME.",
      );
    } catch (reason: unknown) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  const assignedId = configuration?.assignedProfile?.id ?? null;
  const effective = configuration?.effectiveProfile ?? null;

  return (
    <section
      className="controller-configuration"
      aria-label={shortName ? "Machine controller profile" : "Global controller profiles"}
    >
      <div className="controller-configuration__heading">
        <div>
          <h3>{shortName ? "Machine controller profile" : "Controller profiles"}</h3>
          <p>
            Profiles store controller identity and mapping provenance. Selection is configuration
            intent only; this build does not yet translate saved profiles into MAME gameplay-input
            configuration, so unsupported or unapplied mappings are never labeled active.
          </p>
        </div>
      </div>

      {error && (
        <p className="controller-configuration__error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="controller-configuration__notice" role="status">
          {notice}
        </p>
      )}

      {!configuration ? (
        <p aria-live="polite">Loading controller profiles…</p>
      ) : (
        <>
          <label className="controller-configuration__selection">
            <span>{shortName ? "Profile for this machine" : "Global profile"}</span>
            <select
              value={assignedId === null ? "" : String(assignedId)}
              disabled={busy}
              onChange={(event) => void selectProfile(event.target.value)}
            >
              <option value="">
                {shortName ? "Inherit global selection" : "No global profile"}
              </option>
              {configuration.profiles.map((profile) => (
                <option key={profile.id} value={profile.id}>
                  {profile.name}
                </option>
              ))}
            </select>
          </label>

          <dl className="controller-configuration__status">
            <div>
              <dt>Assigned here</dt>
              <dd>
                {configuration.assignedProfile?.name ?? (shortName ? "Inherit global" : "None")}
              </dd>
            </div>
            <div>
              <dt>Effective selection</dt>
              <dd>{effective?.name ?? "None"}</dd>
            </div>
            <div>
              <dt>Selection source</dt>
              <dd>{scopeLabel(configuration.effectiveScope)}</dd>
            </div>
            <div>
              <dt>Active in MAME</dt>
              <dd>{configuration.activeProfile?.name ?? "None"}</dd>
            </div>
          </dl>

          <p className="controller-configuration__runtime" role="status">
            {configuration.statusMessage}
          </p>

          {effective && (
            <div className="controller-configuration__provenance">
              <strong>{provenanceLabel(effective.mappingProvenance.kind)}</strong>
              <span>Target identity: {effective.targetDevice.value}</span>
              <span>
                Reported mapping: {effective.targetDevice.reportedMapping ?? "Not reported"}
              </span>
              {effective.mappingProvenance.sourceReference && (
                <span>Source: {effective.mappingProvenance.sourceReference}</span>
              )}
            </div>
          )}
        </>
      )}

      {allowCapture && (
        <section
          className="controller-configuration__capture"
          aria-label="Capture connected controller profile"
        >
          <div className="controller-configuration__capture-heading">
            <div>
              <strong>Capture connected standard gamepad</strong>
              <p>
                Browser device IDs are stored as reported identity evidence, not as guaranteed
                hardware serial numbers. Only controllers reporting mapping “standard” are accepted
                by this capture path.
              </p>
            </div>
            <button
              type="button"
              className="secondary-button"
              disabled={busy}
              onClick={refreshGamepads}
            >
              Refresh controllers
            </button>
          </div>

          {!gamepadApiAvailable && (
            <p className="controller-configuration__runtime">
              The browser Gamepad API is unavailable in this environment.
            </p>
          )}

          {gamepads.length > 0 && (
            <ul className="controller-configuration__devices">
              {gamepads.map((gamepad) => (
                <li key={`${gamepad.index}:${gamepad.id}`}>
                  <code>{gamepad.id}</code>
                  <span>
                    {gamepad.mapping === "standard"
                      ? "Supported standard mapping"
                      : `Unsupported mapping: ${gamepad.mapping || "unmapped"}`}
                  </span>
                </li>
              ))}
            </ul>
          )}

          <div className="controller-configuration__capture-form">
            <label>
              <span>Connected standard controller</span>
              <select
                value={selectedGamepad}
                disabled={busy || gamepads.every((gamepad) => gamepad.mapping !== "standard")}
                onChange={(event) => setSelectedGamepad(event.target.value)}
              >
                {gamepads.filter((gamepad) => gamepad.mapping === "standard").length === 0 && (
                  <option value="">No supported controller detected</option>
                )}
                {gamepads
                  .filter((gamepad) => gamepad.mapping === "standard")
                  .map((gamepad) => (
                    <option key={gamepad.index} value={gamepad.index}>
                      {gamepad.id}
                    </option>
                  ))}
              </select>
            </label>
            <label>
              <span>Profile name</span>
              <input
                value={profileName}
                maxLength={120}
                disabled={busy}
                onChange={(event) => setProfileName(event.target.value)}
                placeholder="Arcade controller"
              />
            </label>
            <button
              type="button"
              disabled={busy || selectedGamepad === "" || profileName.trim() === ""}
              onClick={() => void captureProfile()}
            >
              Save profile
            </button>
          </div>
        </section>
      )}
    </section>
  );
}
