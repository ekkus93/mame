import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const sourcePath = fileURLToPath(new URL("./MameCatalogStatePanel.tsx", import.meta.url));
const source = readFileSync(sourcePath, "utf8");

describe("MameCatalogStatePanel metadata bootstrap", () => {
  it("starts missing or stale metadata import automatically once per catalog state", () => {
    expect(source).toContain("autoImportKeyRef");
    expect(source).toContain('state.status !== "importNeeded"');
    expect(source).toContain("onImportMetadata();");
    expect(source).toContain("Metadata import is starting automatically");
  });

  it("does not tell normal users to choose a MAME executable", () => {
    expect(source).not.toContain("Choose a MAME executable");
    expect(source).toContain("effective MAME runtime");
    expect(source).toContain("advanced runtime override");
  });
});
