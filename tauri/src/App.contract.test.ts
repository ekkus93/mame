import { describe, expect, it } from "vitest";

import { APP_OWNS_SESSION_SHORTCUTS } from "./App";

describe("App composition contract", () => {
  it("delegates session shortcut ownership to MameShell", () => {
    expect(APP_OWNS_SESSION_SHORTCUTS).toBe(false);
  });
});
