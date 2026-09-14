import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { exportMameUiDisplayedList, queryMameUiLibrary } from "./mameUi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("MAME UI backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("keeps the machine query behind the typed MAME UI command", async () => {
    const request = {
      text: "galax",
      filter: "working" as const,
      filterValue: null,
      preferredMachine: null,
      limit: 100,
      offset: 0,
    };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryMameUiLibrary(request);
    expect(invoke).toHaveBeenCalledWith("query_mame_ui_library", { request });
  });

  it("exports only typed filter and search state through the bounded Rust command", async () => {
    const request = {
      text: "pac",
      filter: "manufacturer" as const,
      filterValue: "Namco",
    };
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      canceled: false,
      rows: 12,
      path: "/tmp/mame-displayed-list.csv",
    });

    await expect(exportMameUiDisplayedList(request)).resolves.toMatchObject({ rows: 12 });
    expect(invoke).toHaveBeenCalledWith("export_mame_ui_displayed_list", { request });
  });
});
