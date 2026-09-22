/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import source from "./SoftwareBrowser.tsx?raw";

const SOURCE_DIR = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(join(SOURCE_DIR, "SoftwareBrowser.css"), "utf8");

describe("Software Browser parity surface", () => {
  it("keeps Software Browser on explicit MAME palette tokens", () => {
    for (const prohibited of ["Canvas", "CanvasText", "ButtonFace", "ButtonText", "currentColor"]) {
      expect(css).not.toContain(prohibited);
    }

    for (const requiredToken of [
      "var(--mame-bg)",
      "var(--mame-toolbar)",
      "var(--mame-border)",
      "var(--mame-border-muted)",
      "var(--mame-selected)",
      "var(--mame-selected-text)",
      "var(--mame-disabled)",
      "var(--mame-focus)",
      "var(--mame-muted)",
      "var(--mame-warning)",
    ]) {
      expect(css).toContain(requiredToken);
    }
  });

  it("uses dense original-like rows with selected, muted, disabled, and focus states", () => {
    expect(css).toContain("min-height: 1.45rem");
    expect(css).toContain("padding: 0.04rem 0.4rem");
    expect(css).toContain(".mame-software-row.is-selected");
    expect(css).toContain(".mame-software-row.is-selected .mame-software-title");
    expect(css).toContain(".mame-software-row.is-unsupported:not(.is-selected)");
    expect(css).toContain(".mame-software-row.is-partial:not(.is-selected)");
    expect(css).toContain(".mame-software-row:disabled");
    expect(css).toContain(".mame-software-row:focus-visible");
  });

  it("keeps software selection and activation wired to the row component", () => {
    expect(source).toContain("<SoftwareResultsList");
    expect(source).toContain("onSelect={setSelected}");
    expect(source).toContain("onActivate={activateItem}");
    expect(source).toContain("onNavigate={handleRowKey}");
    expect(source).toContain("aria-selected={isSelected}");
    expect(source).toContain("data-support={item.supported}");
    expect(source).toContain("mame-software-support");
  });

  it("keeps launch, BIOS, software-part, error, and Back/Escape behavior in the MAME surface", () => {
    expect(source).toContain("launchMameSoftware");
    expect(source).toContain("buildSoftwareLaunchRequest");
    expect(source).toContain("bios: selectedBios");
    expect(source).toContain("selected.parts.length > 1");
    expect(source).toContain('launch.status === "launching"');
    expect(source).toContain("launchMameEmpty");
    expect(source).toContain("onClick={onBack}");
    expect(source).toContain('event.key === "Escape"');
    expect(source).toContain('className="mame-browser-banner is-error"');
    expect(source).toContain('className="mame-panel-state" role="alert"');
  });
});
