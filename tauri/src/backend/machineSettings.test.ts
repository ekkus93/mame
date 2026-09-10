import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { LaunchPreferences } from "./generalSettings";
import {
  getMachineLaunchSettings,
  resetMachineLaunchSettings,
  setMachineLaunchSettings,
} from "./machineSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("per-machine settings backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads settings for exactly one machine short name", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await getMachineLaunchSettings("pacman");
    expect(invoke).toHaveBeenCalledWith("get_machine_launch_settings", {
      request: { shortName: "pacman" },
    });
  });

  it("persists only bounded launch preference overrides", async () => {
    const overrides: LaunchPreferences = {
      windowMode: "fullscreen",
      renderer: "inherit",
      audio: "disabled",
    };
    vi.mocked(invoke).mockResolvedValue({});
    await setMachineLaunchSettings("pacman", overrides);
    expect(invoke).toHaveBeenCalledWith("set_machine_launch_settings", {
      request: { shortName: "pacman", overrides },
    });
  });

  it("resets a machine back to inherited settings through a dedicated command", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await resetMachineLaunchSettings("pacman");
    expect(invoke).toHaveBeenCalledWith("reset_machine_launch_settings", {
      request: { shortName: "pacman" },
    });
  });
});
