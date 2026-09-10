import { invoke } from "@tauri-apps/api/core";

import type { LaunchPreferences } from "./generalSettings";

export type ConfigurationLayer =
  | "applicationDefaults"
  | "mameGlobalDefaults"
  | "profileDefaults"
  | "machineOverrides"
  | "transientLaunchOverrides";

export type PreferenceExplanation<T> = {
  effectiveValue: T;
  sourceLayer: ConfigurationLayer;
  pendingLaunchOverride: T | null;
};

export type LaunchPreferencesExplanation = {
  windowMode: PreferenceExplanation<LaunchPreferences["windowMode"]>;
  renderer: PreferenceExplanation<LaunchPreferences["renderer"]>;
  audio: PreferenceExplanation<LaunchPreferences["audio"]>;
};

export type MachineLaunchSettings = {
  schemaVersion: 1;
  shortName: string;
  overrides: LaunchPreferences;
  effective: LaunchPreferences;
  explanation: LaunchPreferencesExplanation;
};

function pendingRequest(pendingLaunchOverrides: LaunchPreferences | null) {
  return pendingLaunchOverrides ? { pendingLaunchOverrides } : {};
}

export async function getMachineLaunchSettings(
  shortName: string,
  pendingLaunchOverrides: LaunchPreferences | null = null,
): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("get_machine_launch_settings", {
    request: { shortName, ...pendingRequest(pendingLaunchOverrides) },
  });
}

export async function setMachineLaunchSettings(
  shortName: string,
  overrides: LaunchPreferences,
  pendingLaunchOverrides: LaunchPreferences | null = null,
): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("set_machine_launch_settings", {
    request: { shortName, overrides, ...pendingRequest(pendingLaunchOverrides) },
  });
}

export async function resetMachineLaunchSettings(
  shortName: string,
  pendingLaunchOverrides: LaunchPreferences | null = null,
): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("reset_machine_launch_settings", {
    request: { shortName, ...pendingRequest(pendingLaunchOverrides) },
  });
}
