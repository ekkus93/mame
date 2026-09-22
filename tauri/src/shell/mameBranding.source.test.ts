import { describe, expect, it } from "vitest";

import appSource from "../App.tsx?raw";
import browserSource from "../browser/MameBrowser.tsx?raw";
import shellCss from "./MameShell.css?raw";
import shellSource from "./MameShell.tsx?raw";

const defaultSurfaceSources = [appSource, browserSource, shellSource, shellCss];

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
    expect(shellCss).toContain("grid-template-rows: auto auto minmax(0, 1fr) auto;");
    expect(shellCss).not.toContain("::before");
  });
});
