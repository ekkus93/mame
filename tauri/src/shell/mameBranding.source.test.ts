import { describe, expect, it } from "vitest";

import appSource from "../App.tsx?raw";
import browserSource from "../browser/MameBrowser.tsx?raw";
import shellSource from "./MameShell.tsx?raw";

const defaultSurfaceSources = [appSource, browserSource, shellSource];

describe("default MAME branding", () => {
  it("does not expose generic Tauri rewrite branding on startup or browser surfaces", () => {
    for (const source of defaultSurfaceSources) {
      expect(source).not.toContain("MAME Tauri Frontend");
      expect(source).not.toContain("MAME Tauri");
    }
  });

  it("keeps the browser title treatment compact instead of adding a pseudo-title row", () => {
    expect(browserSource).toContain('className="mame-browser-toolbar"');
    expect(browserSource).toContain('placeholder="Search systems..."');
    expect(browserSource).not.toContain("MAME Tauri Frontend");
    expect(browserSource).not.toContain("mame-browser-title");
  });
});
