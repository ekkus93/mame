import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { queryLibraryHistory } from "./history";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("play-history backend commands", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it("uses a bounded typed history query envelope", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryLibraryHistory();
    expect(invoke).toHaveBeenLastCalledWith("query_library_history", {
      request: { limit: 50, offset: 0 },
    });

    await queryLibraryHistory(25, 75);
    expect(invoke).toHaveBeenLastCalledWith("query_library_history", {
      request: { limit: 25, offset: 75 },
    });
  });
});
