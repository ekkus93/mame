import { invoke } from "@tauri-apps/api/core";

export type WindowPreference = "inherit" | "windowed" | "fullscreen";
export type RendererPreference = "inherit" | "auto" | "bgfx" | "openGl" | "software";
export type AudioPreference = "inherit" | "auto" | "disabled";

export type LaunchPreferences = {
  windowMode: WindowPreference;
  renderer: RendererPreference;
  audio: AudioPreference;
};

export type GeneralSettings = {
  schemaVersion: 1;
  mameExecutable: string | null;
  launchPreferences: LaunchPreferences;
};

export async function getGeneralSettings(): Promise<GeneralSettings> {
  return invoke<GeneralSettings>("get_general_settings");
}

export async function setGeneralMameExecutable(
  mameExecutable: string | null,
): Promise<GeneralSettings> {
  return invoke<GeneralSettings>("set_general_mame_executable", {
    request: { mameExecutable },
  });
}

export async function setGeneralLaunchPreferences(
  launchPreferences: LaunchPreferences,
): Promise<GeneralSettings> {
  return invoke<GeneralSettings>("set_general_launch_preferences", {
    request: { launchPreferences },
  });
}

export async function pickMameExecutable(): Promise<string | null> {
  return invoke<string | null>("pick_mame_executable");
}
