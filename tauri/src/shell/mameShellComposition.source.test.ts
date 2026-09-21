import { describe, expect, it } from "vitest";

import browserSource from "../browser/MameBrowser.tsx?raw";
import shellSource from "./MameShell.tsx?raw";

describe("default MAME shell composition", () => {
  it("keeps the machine driver status as the bottom persistent browser region", () => {
    expect(shellSource).not.toContain('<footer className="mame-status-bar"');
    expect(browserSource.lastIndexOf("<MachineDriverStatus")).toBeGreaterThan(
      browserSource.lastIndexOf("mame-browser-grid"),
    );
  });

  it("keeps secondary tools reachable without permanent footer chrome", () => {
    expect(browserSource).toContain('className="mame-utility-menu"');
    for (const label of ["Session", "Configure Options", "Audit", "History", "Collections", "Diagnostics"]) {
      expect(browserSource).toContain(`>${label}</button>`);
    }
    expect(shellSource).toContain('view === "history"');
    expect(shellSource).toContain('view === "collections"');
    expect(shellSource).toContain('view === "diagnostics"');
  });
});
