import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { launchMameSoftware, queryMameSoftware } from "./mameSoftware";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("authoritative software-list backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("queries software through the part-aware typed command", async () => {
    const request = {
      shortName: "apple2e",
      softwareList: "apple2_flop_orig",
      text: "archon",
      filter: "all" as const,
      limit: 25,
      offset: 0,
    };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryMameSoftware(request);
    expect(invoke).toHaveBeenCalledWith("query_mame_software_list", { request });
  });

  it("propagates software part and BIOS through the authoritative launch command", async () => {
    const request = {
      shortName: "apple2e",
      softwareList: "apple2_flop_orig",
      softwareItem: "archon2",
      softwarePart: "flop1",
      bios: "enhanced",
    };
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "session-1",
      state: "running",
      machine: "apple2e",
      software: "apple2_flop_orig:archon2:flop1",
      pid: 123,
    });

    await launchMameSoftware(request);
    expect(invoke).toHaveBeenCalledWith("launch_library_software", { request });
  });
});
