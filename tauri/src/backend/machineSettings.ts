import { invoke } from "@tauri-apps/api/core";

import type { LaunchPreferences } from "./generalSettings";

export type MachineLaunchSettings = {
  schemaVersion: 1;
  shortName: string;
  overrides: LaunchPreferences;
  effective: LaunchPreferences;
};

export async function getMachineLaunchSettings(shortName: string): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("get_machine_launch_settings", {
    request: { shortName },
  });
}

export async function setMachineLaunchSettings(
  shortName: string,
  overrides: LaunchPreferences,
): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("set_machine_launch_settings", {
    request: { shortName, overrides },
  });
}

export async function resetMachineLaunchSettings(
  shortName: string,
): Promise<MachineLaunchSettings> {
  return invoke<MachineLaunchSettings>("reset_machine_launch_settings", {
    request: { shortName },
  });
}
