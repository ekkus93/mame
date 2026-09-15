import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const appSource = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");

describe("App composition contract", () => {
  it("keeps session shortcut ownership inside MameShell", () => {
    expect(appSource).toContain("<MameShell");
    expect(appSource).not.toContain("window.dispatchEvent(new Event(\"focus\"))");
    expect(appSource).not.toContain("SESSION_EXITED_EVENT");
    expect(appSource).not.toContain("SESSION_CRASHED_EVENT");
    expect(appSource).not.toContain("SESSION_FAILED_EVENT");
  });
});
