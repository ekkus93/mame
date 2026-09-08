import { describe, expect, it } from "vitest";

import { appStateReducer, initialAppState } from "./appState";

describe("appStateReducer", () => {
  it("keeps backend source-of-truth data in the ready state", () => {
    const next = appStateReducer(initialAppState, {
      type: "ready",
      info: {
        protocolVersion: 1,
        appVersion: "0.1.0",
        backend: "rust-tauri",
      },
    });

    expect(next).toEqual({
      status: "ready",
      info: {
        protocolVersion: 1,
        appVersion: "0.1.0",
        backend: "rust-tauri",
      },
    });
  });

  it("represents backend failure explicitly", () => {
    expect(
      appStateReducer(initialAppState, {
        type: "error",
        message: "backend unavailable",
      }),
    ).toEqual({ status: "error", message: "backend unavailable" });
  });
});
