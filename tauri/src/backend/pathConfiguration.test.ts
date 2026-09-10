import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getContentPathConfiguration,
  pickContentDirectory,
  setContentPathConfiguration,
} from "./pathConfiguration";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("content path backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads path configuration through the dedicated Rust command", async () => {
    vi.mocked(invoke).mockResolvedValue({ contentPaths: {}, validations: {} });
    await getContentPathConfiguration();
    expect(invoke).toHaveBeenCalledWith("get_content_path_configuration");
  });

  it("persists the ordered path model without flattening encoded platform paths", async () => {
    const contentPaths = {
      romPaths: ["/roms", { encoding: "unixBytesHex", data: "726f6dff" }],
      softwarePaths: ["/software"],
      chdPaths: [],
    };
    vi.mocked(invoke).mockResolvedValue({ contentPaths, validations: {} });

    await setContentPathConfiguration(contentPaths);
    expect(invoke).toHaveBeenCalledWith("set_content_path_configuration", {
      request: { contentPaths },
    });
  });

  it("requests native directory selection from the Rust backend", async () => {
    vi.mocked(invoke).mockResolvedValue("/roms");
    await expect(pickContentDirectory()).resolves.toBe("/roms");
    expect(invoke).toHaveBeenCalledWith("pick_content_directory");
  });
});
