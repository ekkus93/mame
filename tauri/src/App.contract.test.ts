import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const appSource = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");
const executableSource = appSource.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "");

describe("App composition contract", () => {
  it("keeps session shortcut ownership inside MameShell", () => {
    expect(executableSource).toContain("<MameShell");
    expect(executableSource).not.toContain("window.dispatchEvent");
    expect(executableSource).not.toContain("listen(");
    expect(executableSource).not.toContain("SESSION_EXITED_EVENT");
    expect(executableSource).not.toContain("SESSION_CRASHED_EVENT");
    expect(executableSource).not.toContain("SESSION_FAILED_EVENT");
  });
});
