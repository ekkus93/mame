import { describe, expect, it } from "vitest";

import shellSource from "../shell/MameShell.tsx?raw";
import browserSource from "./MameBrowser.tsx?raw";
import filterSource from "./MachineFilterPanel.tsx?raw";
import rightPanelSource from "./MachineRightPanel.tsx?raw";
import { nextBrowserIndex } from "./model";

describe("MAME command and interaction parity tripwires", () => {
  it("keeps original-like command labels in the default browser flow", () => {
    expect(shellSource).toContain("Configure Options");
    expect(browserSource).toContain("Configure Machine");
    expect(browserSource).toContain("Software List");
    expect(browserSource).toContain("Start Empty");
    expect(browserSource).toContain("mame-configure-machine-button");
    expect(browserSource).toContain("mame-software-list-button");
    expect(browserSource).not.toContain(">\n              Configure\n            </button>");
    expect(browserSource).not.toContain(">\n                Software\n              </button>");
  });

  it("keeps model-level keyboard movement for rows, pages, and list edges", () => {
    expect(nextBrowserIndex("ArrowDown", 0, 30)).toBe(1);
    expect(nextBrowserIndex("ArrowUp", 0, 30)).toBe(0);
    expect(nextBrowserIndex("PageDown", 4, 30, 10)).toBe(14);
    expect(nextBrowserIndex("PageUp", 4, 30, 10)).toBe(0);
    expect(nextBrowserIndex("Home", 17, 30)).toBe(0);
    expect(nextBrowserIndex("End", 2, 30)).toBe(29);
  });

  it("preserves default browser keyboard shortcuts and back/focus behavior", () => {
    expect(browserSource).toContain('event.key === "Enter"');
    expect(browserSource).toContain('event.key === "Escape"');
    expect(browserSource).toContain('event.key === "/"');
    expect(browserSource).toContain("searchInputRef.current?.focus()");
    expect(browserSource).toContain("activeFilterButtonRef.current?.focus()");
    expect(browserSource).toContain("rightPanelFirstTabRef.current?.focus()");
    expect(browserSource).toContain("setShowNarrowDetails(false)");
  });

  it("preserves click paths for filters, rows, activation, and right-panel tabs", () => {
    expect(browserSource).toContain("onChange={changeFilter}");
    expect(browserSource).toContain("onSelect={selectMachine}");
    expect(browserSource).toContain("onActivate={activateMachine}");
    expect(browserSource).toContain("onViewChange={changeRightView}");
    expect(filterSource).toContain("if (!deferred) onChange(filter.id);");
    expect(rightPanelSource).toContain('onClick={() => onViewChange("images")}');
    expect(rightPanelSource).toContain('onClick={() => onViewChange("info")}');
  });

  it("keeps launch errors inside the MAME browser surface instead of a generic page", () => {
    expect(browserSource).toContain("mame-browser-banner is-error");
    expect(browserSource).toContain('role="alert"');
    expect(browserSource).toContain("launchLibraryMachine");
    expect(browserSource).toContain("pendingLaunchOverrides");
  });
});
