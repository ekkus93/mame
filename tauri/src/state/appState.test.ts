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
        build: {
          gitSha: "0123456789abcdef",
          profile: "debug",
          target: "x86_64-unknown-linux-gnu",
        },
        databaseSchemaVersion: 4,
        settingsSchemaVersion: 2,
        runtimeProtocolVersion: 1,
        mame: { status: "notConfigured" },
      },
    });

    expect(next).toEqual({
      status: "ready",
      info: {
        protocolVersion: 1,
        appVersion: "0.1.0",
        backend: "rust-tauri",
        build: {
          gitSha: "0123456789abcdef",
          profile: "debug",
          target: "x86_64-unknown-linux-gnu",
        },
        databaseSchemaVersion: 4,
        settingsSchemaVersion: 2,
        runtimeProtocolVersion: 1,
        mame: { status: "notConfigured" },
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
