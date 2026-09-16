import { describe, expect, it } from "vitest";

import mainSource from "../main.tsx?raw";
import themeCss from "../mameTheme.css?raw";
import shellSource from "../shell/MameShell.tsx?raw";
import shellCss from "../shell/MameShell.css?raw";
import filterSource from "./MachineFilterPanel.tsx?raw";
import listSource from "./MachineList.tsx?raw";

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

  it("keeps the default shell on explicit MAME tokens instead of system colors", () => {
    expect(shellCss).toContain("background: var(--mame-bg)");
    expect(shellCss).toContain("background: var(--mame-toolbar)");
    expect(shellCss).toContain("var(--mame-status)");
    expect(shellCss).toContain("color: var(--mame-selected-text)");
    expect(shellCss).not.toContain("background: Canvas");
    expect(shellCss).not.toContain("color: CanvasText");
  });

  it("removes the generic dashboard tab row from the default shell", () => {
    expect(shellSource).not.toContain('className="mame-shell-nav"');
    expect(shellSource).not.toContain('aria-label="Application views"');
    expect(shellSource).toContain('aria-label="MAME machine browser"');
  });

  it("preserves visible original MAME selection landmarks", () => {
    expect(filterSource).toContain("mame-filter-indicator");
    expect(filterSource).toContain("Category");
    expect(filterSource).toContain("Custom Filter");
    expect(listSource).toContain("is-selected");
    expect(listSource).toContain("is-unavailable");
  });
});
