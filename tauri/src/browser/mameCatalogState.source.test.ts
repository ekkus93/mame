import { describe, expect, it } from "vitest";

import shellSource from "../shell/MameShell.tsx?raw";
import browserSource from "./MameBrowser.tsx?raw";
import catalogSource from "./MachineCatalogState.tsx?raw";

describe("MAME startup and metadata empty-state parity tripwires", () => {
  it("keeps catalog empty states inside the default MAME browser layout", () => {
    expect(browserSource).toContain("<MachineCatalogState");
    expect(browserSource).toContain("<MachineCatalogSummary");
    expect(browserSource).toContain("mameReport={appInfo.mame}");
    expect(shellSource).toContain("onConfigureOptions={() => setView(\"settings\")}");
  });

  it(
    "distinguishes not-configured, import-needed, importing, failed, loaded, and no-match states",
    () => {
      for (const phrase of [
        "MAME executable is not configured",
        "MAME executable is unavailable",
        "MAME metadata is not imported",
        "MAME metadata import is in progress",
        "MAME metadata import failed",
        "MAME metadata loaded",
        "No machines match this filter",
      ]) {
        expect(catalogSource).toContain(phrase);
      }
    },
  );

  it(
    "keeps no-ROMs and unknown-availability states visible without a generic blank page",
    () => {
      expect(catalogSource).toContain("ROM availability unknown");
      expect(catalogSource).toContain("No ROMs available for displayed systems");
      expect(catalogSource).toContain("mame-catalog-summary");
      expect(catalogSource).toContain("mame-catalog-state");
    },
  );

  it("replaces the generic zero-count range with action-oriented catalog state labels", () => {
    expect(browserSource).toContain("Configure MAME");
    expect(browserSource).toContain("Reading catalog");
    expect(browserSource).toContain("Metadata not imported");
    expect(browserSource).not.toContain(': "0 machines"');
  });

  it("provides a Configure Options path for states that need setup or import work", () => {
    expect(catalogSource).toContain("Configure Options");
    expect(catalogSource).toContain("showConfigureOptions: true");
    expect(catalogSource).toContain("onConfigureOptions");
  });
});
