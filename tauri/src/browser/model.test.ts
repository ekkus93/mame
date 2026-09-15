import { describe, expect, it } from "vitest";

import type { MachineListItem } from "../backend/types";
import {
  buildMameBrowserRequest,
  filterRequiresValue,
  nextBrowserIndex,
  reconcileMachineSelection,
} from "./model";

function machine(shortName: string, description: string): MachineListItem {
  return {
    shortName,
    description,
    year: null,
    manufacturer: null,
    sourceFile: null,
    cloneOf: null,
    runnable: true,
    isDevice: false,
    driverStatus: "good",
    displayCount: 1,
    softwareListCount: 0,
  };
}

describe("MAME browser model", () => {
  it("maps canonical frontend filters to bounded MAME UI queries", () => {
    expect(buildMameBrowserRequest("available", " pac ", "", 10)).toEqual({
      text: "pac",
      filter: "available",
      filterValue: null,
      preferredMachine: null,
      limit: 100,
      offset: 10,
    });
    expect(buildMameBrowserRequest("notWorking", "", "")).toMatchObject({
      filter: "notWorking",
    });
    expect(buildMameBrowserRequest("manufacturer", "", " Namco ")).toMatchObject({
      filter: "manufacturer",
      filterValue: "Namco",
    });
    expect(buildMameBrowserRequest("chdRequired", "", "", 0, "area51")).toMatchObject({
      filter: "chdRequired",
      preferredMachine: "area51",
    });
    expect(buildMameBrowserRequest("noChdRequired", "", "")).toMatchObject({
      filter: "noChdRequired",
    });
  });

  it("identifies canonical filters that require a bounded value", () => {
    expect(filterRequiresValue("manufacturer")).toBe(true);
    expect(filterRequiresValue("year")).toBe(true);
    expect(filterRequiresValue("sourceFile")).toBe(true);
    expect(filterRequiresValue("favorites")).toBe(false);
    expect(filterRequiresValue("chdRequired")).toBe(false);
  });

  it("refreshes a retained selection from the authoritative result row", () => {
    const stale = machine("pacman", "Old Pac-Man metadata");
    const refreshed = machine("pacman", "PAC-MAN");
    expect(reconcileMachineSelection([refreshed], stale, null)).toBe(refreshed);
  });

  it("falls back deterministically when a selected machine disappears", () => {
    const first = machine("galaga", "Galaga");
    const preferred = machine("pacman", "Pac-Man");
    expect(reconcileMachineSelection([first, preferred], machine("missing", "Missing"), "pacman")).toBe(
      preferred,
    );
    expect(reconcileMachineSelection([first], machine("missing", "Missing"), null)).toBe(first);
    expect(reconcileMachineSelection([], machine("missing", "Missing"), null)).toBeNull();
  });

  it("supports bounded row, edge and page navigation", () => {
    expect(nextBrowserIndex("ArrowDown", 0, 30)).toBe(1);
    expect(nextBrowserIndex("ArrowUp", 0, 30)).toBe(0);
    expect(nextBrowserIndex("Home", 17, 30)).toBe(0);
    expect(nextBrowserIndex("End", 2, 30)).toBe(29);
    expect(nextBrowserIndex("PageDown", 4, 30, 10)).toBe(14);
    expect(nextBrowserIndex("PageUp", 4, 30, 10)).toBe(0);
    expect(nextBrowserIndex("x", 4, 30)).toBeNull();
  });
});
