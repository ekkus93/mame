/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import appSource from "../App.tsx?raw";
import mainSource from "../main.tsx?raw";
import browserSource from "../browser/MameBrowser.tsx?raw";

const SOURCE_DIR = dirname(fileURLToPath(import.meta.url));

function readSource(relativePath: string): string {
  return readFileSync(join(SOURCE_DIR, relativePath), "utf8");
}

describe("Linux desktop compatibility parity tripwires", () => {
  const themeCss = readSource("../mameTheme.css");
  const shellCss = readSource("./MameShell.css");
  const appCss = readSource("../App.css");
  const tauriConfig = readSource("../../src-tauri/tauri.conf.json");

  it("keeps the product on the Tauri WebView shell instead of a native selector route", () => {
    expect(appSource).toContain("<MameShell");
    expect(tauriConfig).toContain('"frontendDist": "../dist"');
    expect(tauriConfig).toContain('"devUrl": "http://localhost:1420"');
    expect(appSource).not.toContain("Legacy UI");
    expect(appSource).not.toContain("native selector");
  });

  it("loads explicit MAME theme tokens after base styling and pins host-independent dark colors", () => {
    expect(mainSource.indexOf('import "./index.css";')).toBeLessThan(
      mainSource.indexOf('import "./mameTheme.css";'),
    );
    expect(themeCss).toContain("color-scheme: dark;");
    expect(themeCss).toContain("--mame-bg:");
    expect(themeCss).toContain("--mame-text:");
    expect(themeCss).toContain("--mame-focus:");
    expect(shellCss).toContain("background: var(--mame-bg)");
  });

  it("keeps startup presentation on the same explicit palette", () => {
    expect(appCss).toContain(".startup-shell");
    expect(appCss).toContain("background: var(--mame-bg);");
    expect(appCss).toContain("color: var(--mame-text);");
    expect(appCss).toContain("border: 1px solid var(--mame-border);");
  });

  it("fails browser shortcuts closed when the WebView or gameplay input does not own focus", () => {
    expect(browserSource).toContain("!document.hasFocus()");
    expect(browserSource).toContain("gameplayInputOwned");
    expect(browserSource).toContain("isEditableElement(event.target)");
    expect(browserSource).toContain("searchInputRef.current?.focus()");
    expect(browserSource).toContain("activeFilterButtonRef.current?.focus()");
    expect(browserSource).toContain("rightPanelFirstTabRef.current?.focus()");
  });

  it("retains a high-contrast visible focus affordance", () => {
    expect(shellCss).toContain("button:focus-visible");
    expect(shellCss).toContain("input:focus-visible");
    expect(shellCss).toContain("outline: 2px solid var(--mame-focus)");
  });
});
