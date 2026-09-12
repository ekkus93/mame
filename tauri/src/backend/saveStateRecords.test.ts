import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  deleteSaveStateRecord,
  listSaveStateRecords,
  loadKnownSaveState,
  saveKnownState,
} from "./saveStateRecords";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("save-state record backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("lists records with a bounded page request", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      total: 0,
      offset: 0,
      limit: 100,
      items: [],
    });
    await listSaveStateRecords({ limit: 100, offset: 0 });
    expect(invoke).toHaveBeenCalledWith("list_save_state_records", {
      request: { limit: 100, offset: 0 },
    });
  });

  it("saves through the known-state wrapper", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await saveKnownState({ sessionId: "session-1", slot: "quick" });
    expect(invoke).toHaveBeenCalledWith("save_known_state", {
      request: { sessionId: "session-1", slot: "quick" },
    });
  });

  it("loads and deletes records by backend-issued id rather than path", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    await loadKnownSaveState({ sessionId: "session-1", recordId: 7 });
    expect(invoke).toHaveBeenCalledWith("load_known_save_state", {
      request: { sessionId: "session-1", recordId: 7 },
    });

    await deleteSaveStateRecord({ recordId: 7 });
    expect(invoke).toHaveBeenLastCalledWith("delete_save_state_record", {
      request: { recordId: 7 },
    });
  });
});
