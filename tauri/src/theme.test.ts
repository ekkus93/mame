import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const SOURCE_DIR = dirname(fileURLToPath(import.meta.url));

function readSource(relativePath: string): string {
  return readFileSync(join(SOURCE_DIR, relativePath), "utf8");
}

describe("MAME visual theme", () => {
  it("loads the explicit theme after the base CSS", () => {
    const main = readSource("main.tsx");
    const baseImport = 'import "./index.css";';
    const themeImport = 'import "./theme.css";';

    expect(main).toContain(baseImport);
    expect(main).toContain(themeImport);
    expect(main.indexOf(baseImport)).toBeLessThan(main.indexOf(themeImport));
  });

  it("defines an arcade palette instead of relying on host toolkit system colors", () => {
    const theme = readSource("theme.css");

    for (const token of [
      "--mame-bg:",
      "--mame-surface:",
      "--mame-surface-selected:",
      "--mame-border-strong:",
      "--mame-accent:",
      "--mame-accent-strong:",
      "--mame-text:",
      "--mame-muted:",
      "color-scheme: dark;",
      "accent-color: var(--mame-accent);",
    ]) {
      expect(theme).toContain(token);
    }
  });

  it("themes the visible MAME shell surfaces that appeared monochrome", () => {
    const theme = readSource("theme.css");

    for (const token of [
      ".mame-shell",
      ".mame-shell-header",
      ".mame-browser-toolbar",
      ".mame-region-heading",
      ".mame-filter.is-selected",
      ".mame-machine-row.is-selected",
      ".mame-software-row.is-selected",
      ".mame-search input",
      ".mame-start-button",
    ]) {
      expect(theme).toContain(token);
    }

    expect(theme).not.toContain("background: Canvas;");
    expect(theme).not.toContain("color: CanvasText;");
  });
});
