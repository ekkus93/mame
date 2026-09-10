import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  createBrowserControllerProfile,
  getControllerProfileConfiguration,
  setControllerProfileSelection,
} from "./controllerProfiles";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("controller profile backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue({});
  });

  it("inspects an exact machine scope", async () => {
    const scope = { kind: "machine" as const, shortName: "pacman" };
    await getControllerProfileConfiguration(scope);
    expect(invoke).toHaveBeenCalledWith("get_controller_profile_configuration", {
      request: { scope },
    });
  });

  it("selects or clears exactly one profile for a scope", async () => {
    const scope = { kind: "global" as const };
    await setControllerProfileSelection(scope, 7);
    expect(invoke).toHaveBeenLastCalledWith("set_controller_profile_selection", {
      request: { scope, profileId: 7 },
    });

    await setControllerProfileSelection(scope, null);
    expect(invoke).toHaveBeenLastCalledWith("set_controller_profile_selection", {
      request: { scope, profileId: null },
    });
  });

  it("captures browser identity and mapping without claiming runtime activation", async () => {
    const request = { name: "Arcade pad", gamepadId: "045e gamepad", mapping: "standard" };
    await createBrowserControllerProfile(request);
    expect(invoke).toHaveBeenCalledWith("create_browser_controller_profile", { request });
  });
});
