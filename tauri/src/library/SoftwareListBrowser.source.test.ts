import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const source = readFileSync(fileURLToPath(new URL("./SoftwareListBrowser.tsx", import.meta.url)), "utf8");

describe("legacy software browser retirement", () => {
  it("delegates the historical export to the authoritative MAME SoftwareBrowser", () => {
    expect(source).toContain('export { SoftwareBrowser as SoftwareListBrowser } from "../browser/SoftwareBrowser"');
    expect(source).not.toContain("launchLibrarySoftware");
    expect(source).not.toContain("queryMameSoftwareList");
  });
});
