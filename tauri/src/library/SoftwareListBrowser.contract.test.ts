import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const source = readFileSync(
  fileURLToPath(new URL("./SoftwareListBrowser.tsx", import.meta.url)),
  "utf8",
);

describe("legacy software browser compatibility surface", () => {
  it("delegates to the authoritative BIOS/part-aware SoftwareBrowser", () => {
    expect(source).toContain('import { SoftwareBrowser } from "../browser/SoftwareBrowser"');
    expect(source).toContain("<SoftwareBrowser");
    expect(source).not.toContain("launchLibrarySoftware");
    expect(source).not.toContain("queryMameSoftwareList");
  });

  it("does not recreate an untyped software launch request", () => {
    expect(source).not.toContain("softwareItem:");
    expect(source).not.toContain('invoke<SessionSnapshot>("launch_library_software"');
  });
});
