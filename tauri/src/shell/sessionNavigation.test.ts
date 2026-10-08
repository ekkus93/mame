import { describe, expect, it } from "vitest";

import { libraryReturnAction } from "./sessionNavigation";

describe("library return and supervised MAME lifecycle", () => {
  it("stops an active game before hiding its in-app surface", () => {
    expect(libraryReturnAction("session", "running")).toBe("stop");
  });

  it("keeps the surface visible while the session cannot yet be stopped", () => {
    for (const state of ["created", "starting", "stopping"] as const) {
      expect(libraryReturnAction("session", state)).toBe("wait");
    }
  });

  it("navigates immediately from non-game surfaces or terminal sessions", () => {
    expect(libraryReturnAction("settings", "running")).toBe("navigate");
    expect(libraryReturnAction("session", null)).toBe("navigate");
    for (const state of ["exited", "failed", "crashed"] as const) {
      expect(libraryReturnAction("session", state)).toBe("navigate");
    }
  });
});
