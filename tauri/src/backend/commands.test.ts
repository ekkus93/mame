import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getAppInfo,
  getMameMetadataStatus,
  loadMameState,
  pauseMame,
  queryMameLibrary,
  queryMameRuntimeState,
  refreshMameMetadata,
  resetMame,
  resumeMame,
  saveMameState,
  setMameMute,
} from "./commands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("typed backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("uses the versioned app-info command envelope", async () => {
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

  it("passes executable identity requests through typed metadata commands", async () => {
    const request = {
      executable: { source: "external" as const, path: "/opt/mame/mame" },
    };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1 });

    await refreshMameMetadata(request);
    expect(invoke).toHaveBeenLastCalledWith("refresh_mame_metadata", { request });

    await getMameMetadataStatus(request);
    expect(invoke).toHaveBeenLastCalledWith("get_mame_metadata_status", { request });
  });

  it("uses dedicated typed pause, resume, and reset commands", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "mame-1-1",
      paused: true,
    });

    await pauseMame({ sessionId: "mame-1-1" });
    expect(invoke).toHaveBeenLastCalledWith("pause_mame", {
      request: { sessionId: "mame-1-1" },
    });

    await resumeMame({ sessionId: "mame-1-1" });
    expect(invoke).toHaveBeenLastCalledWith("resume_mame", {
      request: { sessionId: "mame-1-1" },
    });

    await resetMame({ sessionId: "mame-1-1" });
    expect(invoke).toHaveBeenLastCalledWith("reset_mame", {
      request: { sessionId: "mame-1-1" },
    });
  });

  it("uses typed save/load-state commands with logical slots", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "mame-1-1",
      machine: "pacman",
      software: null,
      slot: "quick1",
      path: "/tmp/state.sta",
      bytes: 1024,
      savedAtEpochMs: 1,
    });

    await saveMameState({ sessionId: "mame-1-1", slot: "quick1" });
    expect(invoke).toHaveBeenLastCalledWith("save_mame_state", {
      request: { sessionId: "mame-1-1", slot: "quick1" },
    });

    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "mame-1-1",
      machine: "pacman",
      software: null,
      slot: "quick1",
      path: "/tmp/state.sta",
      bytes: 1024,
      loadedAtEpochMs: 2,
    });

    await loadMameState({ sessionId: "mame-1-1", slot: "quick1" });
    expect(invoke).toHaveBeenLastCalledWith("load_mame_state", {
      request: { sessionId: "mame-1-1", slot: "quick1" },
    });
  });

  it("uses the typed native user-mute command", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "mame-1-1",
      uiMuted: true,
      effectiveMuted: true,
    });

    await setMameMute({ sessionId: "mame-1-1", muted: true });
    expect(invoke).toHaveBeenLastCalledWith("set_mame_mute", {
      request: { sessionId: "mame-1-1", muted: true },
    });
  });

  it("uses the typed bounded runtime-state query", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      sessionId: "mame-1-1",
      running: true,
      paused: false,
      machine: "pacman",
      software: null,
      uiMuted: false,
      effectiveMuted: false,
    });

    await queryMameRuntimeState({ sessionId: "mame-1-1" });
    expect(invoke).toHaveBeenLastCalledWith("query_mame_runtime_state", {
      request: { sessionId: "mame-1-1" },
    });
  });

  it("sends bounded library query parameters without constructing SQL in the frontend", async () => {
    const request = {
      text: "galax",
      cloneFilter: "parentsOnly" as const,
      limit: 50,
      offset: 0,
    };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryMameLibrary(request);
    expect(invoke).toHaveBeenCalledWith("query_mame_library", { request });
  });
});
