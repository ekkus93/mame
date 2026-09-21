/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import mainSource from "../main.tsx?raw";
import browserSource from "./MameBrowser.tsx?raw";
import driverStatusSource from "./MachineDriverStatus.tsx?raw";
import filterSource from "./MachineFilterPanel.tsx?raw";
import listSource from "./MachineList.tsx?raw";
import rightPanelSource from "./MachineRightPanel.tsx?raw";
import shellSource from "../shell/MameShell.tsx?raw";

const SOURCE_DIR = dirname(fileURLToPath(import.meta.url));

function readSource(relativePath: string): string {
  return readFileSync(join(SOURCE_DIR, relativePath), "utf8");
}

describe("MAME visual parity tripwires", () => {
  const themeCss = readSource("../mameTheme.css");
  const shellCss = readSource("../shell/MameShell.css");

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

  it("removes generic Tauri branding from the default machine browser", () => {
    expect(shellCss).not.toContain("MAME Tauri Frontend");
    expect(shellCss).not.toContain(".mame-browser::before");
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

  it("preserves original right panel Images and Infos landmarks", () => {
    expect(rightPanelSource).toContain("Machine Images and Infos");
    expect(rightPanelSource).toContain("Images");
    expect(rightPanelSource).toContain("Infos");
    expect(rightPanelSource).toContain("Snapshots");
    expect(rightPanelSource).toContain("No image Available");
    expect(rightPanelSource).toContain("mame-no-image-placeholder");
  });

  it("routes selected driver metadata into a green bottom status region", () => {
    expect(browserSource).toContain("<MachineDriverStatus");
    expect(driverStatusSource).toContain("driverEmulation");
    expect(driverStatusSource).toContain("driverNoSoundHardware");
    expect(driverStatusSource).toContain("machineStatusLabel(detail)");
    expect(shellCss).toContain(".mame-driver-status");
  });
});
