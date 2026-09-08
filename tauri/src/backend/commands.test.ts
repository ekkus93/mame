import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { getAppInfo } from "./commands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("getAppInfo", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("uses the versioned typed command envelope", async () => {
    vi.mocked(invoke).mockResolvedValue({
      protocolVersion: 1,
      appVersion: "0.1.0",
      backend: "rust-tauri",
    });

    await expect(getAppInfo()).resolves.toMatchObject({
      protocolVersion: 1,
      backend: "rust-tauri",
    });
    expect(invoke).toHaveBeenCalledWith("get_app_info", {
      request: { protocolVersion: 1 },
    });
  });
});
