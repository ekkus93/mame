import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getGeneralSettings,
  pickMameExecutable,
  setGeneralLaunchPreferences,
  setGeneralMameExecutable,
  type LaunchPreferences,
} from "./generalSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("general settings backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads general settings from the dedicated Rust command", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await getGeneralSettings();
    expect(invoke).toHaveBeenCalledWith("get_general_settings");
  });

  it("persists a configured MAME executable as an exact path", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await setGeneralMameExecutable("/opt/MAME builds/mame");
    expect(invoke).toHaveBeenCalledWith("set_general_mame_executable", {
      request: { mameExecutable: "/opt/MAME builds/mame" },
    });
  });

  it("persists bounded launch preferences without arbitrary argv strings", async () => {
    const launchPreferences: LaunchPreferences = {
      windowMode: "fullscreen",
      renderer: "bgfx",
      audio: "disabled",
    };
    vi.mocked(invoke).mockResolvedValue({});
    await setGeneralLaunchPreferences(launchPreferences);
    expect(invoke).toHaveBeenCalledWith("set_general_launch_preferences", {
      request: { launchPreferences },
    });
  });

  it("requests executable selection through the native backend picker", async () => {
    vi.mocked(invoke).mockResolvedValue("/usr/bin/mame");
    await expect(pickMameExecutable()).resolves.toBe("/usr/bin/mame");
    expect(invoke).toHaveBeenCalledWith("pick_mame_executable");
  });
});
