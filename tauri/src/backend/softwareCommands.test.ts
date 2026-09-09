import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { launchLibrarySoftware, queryMameSoftwareList } from "./commands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("software-list backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("queries an associated software list through the typed command", async () => {
    const request = {
      shortName: "apple2e",
      softwareList: "apple2_flop_orig",
      text: "archon",
      limit: 25,
      offset: 0,
    };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryMameSoftwareList(request);
    expect(invoke).toHaveBeenCalledWith("query_mame_software_list", { request });
  });

  it("launches the selected machine and software target through Rust", async () => {
    const request = {
      shortName: "apple2e",
      softwareList: "apple2_flop_orig",
      softwareItem: "archon2",
    };
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "session-1",
      state: "running",
      machine: "apple2e",
      software: "apple2_flop_orig:archon2",
      pid: 123,
    });

    await launchLibrarySoftware(request);
    expect(invoke).toHaveBeenCalledWith("launch_library_software", { request });
  });
});
