import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const sourcePath = fileURLToPath(new URL("./GeneralSettingsPanel.tsx", import.meta.url));
const source = readFileSync(sourcePath, "utf8");

describe("GeneralSettingsPanel bundled runtime UX", () => {
  it("keeps executable selection behind an advanced override affordance", () => {
    expect(source).toContain("Bundled MAME runtime");
    expect(source).toContain("External runtime override");
    expect(source).toContain("Use bundled MAME");
    expect(source).toContain("showAdvancedRuntimeOverride &&");
  });

  it("does not restore the obsolete normal executable-picker copy", () => {
    expect(source).not.toContain("Choose a MAME executable");
    expect(source).not.toContain("Save executable");
    expect(source).not.toContain("Configured MAME executable cleared.");
  });
});
