import { describe, expect, it } from "vitest";

import { TAURI_EVENT_NAMES } from "./events";

describe("Tauri event-name contract", () => {
  it("uses only characters accepted by Tauri", () => {
    const valid = /^[A-Za-z0-9/:_-]+$/;
    for (const eventName of TAURI_EVENT_NAMES) {
      expect(eventName).toMatch(valid);
    }
  });

  it("does not use dotted event names", () => {
    for (const eventName of TAURI_EVENT_NAMES) {
      expect(eventName).not.toContain(".");
    }
  });
});
