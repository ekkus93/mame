import { describe, expect, it } from "vitest";

import appCss from "../App.css?raw";
import mainSource from "../main.tsx?raw";
import themeCss from "../mameTheme.css?raw";
import shellSource from "../shell/MameShell.tsx?raw";
import shellCss from "../shell/MameShell.css?raw";
import browserSource from "./MameBrowser.tsx?raw";
import rightPanelSource from "./MachineRightPanel.tsx?raw";

describe("MAME visual parity tripwires", () => {
  it("loads explicit MAME theme tokens after base CSS", () => {
    expect(mainSource).toContain('import "./index.css";');
    expect(mainSource).toContain('import "./mameTheme.css";');
    expect(mainSource.indexOf('import "./index.css";')).toBeLessThan(
      mainSource.indexOf('import "./mameTheme.css";'),
    );
  });

  it("defines the original-MAME-like palette required by the product contract", () => {
    for (const token of [
      "--mame-bg:",
      "--mame-toolbar:",
      "--mame-selected:",
      "--mame-selected-text:",
      "--mame-text:",
      "--mame-muted:",
      "--mame-border:",
      "--mame-status:",
      "color-scheme: dark;",
    ]) {
      expect(themeCss).toContain(token);
    }
  });

  it("keeps primary shell styling on explicit MAME tokens instead of system colors", () => {
    expect(shellCss).toContain("background: var(--mame-bg)");
    expect(shellCss).toContain("background: var(--mame-toolbar)");
    expect(shellCss).toContain("background: var(--mame-status)");
    expect(shellCss).toContain("color: var(--mame-selected-text)");
    expect(shellCss).not.toContain("background: Canvas");
    expect(shellCss).not.toContain("color: CanvasText");
    expect(appCss).not.toContain("background: Canvas");
    expect(appCss).not.toContain("color: CanvasText");
  });

  it("does not expose the generic dashboard tab row as the default shell", () => {
    expect(shellSource).not.toContain('className="mame-shell-nav"');
    expect(shellSource).not.toContain('aria-label="Application views"');
    expect(browserSource).toContain("mame-browser-titlebar");
    expect(browserSource).toContain("mame-action-strip");
  });

  it("preserves visible original MAME selection and status landmarks", () => {
    for (const token of [
      "mame-filter-indicator",
      "mame-main-actions",
      "Configure Options",
      "Configure Machine",
      "MachineStatusRegion",
      "mame-driver-status",
    ]) {
      expect(browserSource).toContain(token);
    }
    expect(rightPanelSource).toContain("Infos");
    expect(rightPanelSource).toContain("mame-no-image-placeholder");
  });
});
