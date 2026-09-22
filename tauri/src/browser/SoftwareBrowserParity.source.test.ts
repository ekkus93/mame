/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import source from "./SoftwareBrowser.tsx?raw";

const SOURCE_DIR = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(join(SOURCE_DIR, "SoftwareBrowser.css"), "utf8");

describe("Software Browser parity surface", () => {
  it("uses explicit MAME palette and dense selected-row treatment", () => {
    expect(css).toContain("background: var(--mame-bg)");
    expect(css).toContain("var(--mame-selected)");
    expect(css).toContain("var(--mame-selected-text)");
    expect(css).toContain("var(--mame-focus)");
    expect(css).toContain("min-height: 1.75rem");
    expect(css).not.toMatch(/\b(?:Canvas|CanvasText|currentColor)\b/);
  });

  it("preserves software selection, activation, BIOS, and back affordances", () => {
    expect(source).toContain("onClick={() => setSelected(item)}");
    expect(source).toContain("onDoubleClick={() => activateItem(item)}");
    expect(source).toContain('event.key === "Enter"');
    expect(source).toContain("selectedBios");
    expect(source).toContain("onBack");
    expect(source).toContain('event.key === "Escape"');
  });

  it("keeps software errors and state feedback inside the MAME surface", () => {
    expect(source).toContain('className="mame-browser-banner is-error"');
    expect(source).toContain('className="mame-panel-state" role="alert"');
  });
});
