import { describe, expect, it } from "vitest";

import {
  buildMameBrowserRequest,
  emptyMachineResultMessage,
  filterRequiresValue,
  nextBrowserIndex,
} from "./model";

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

  it("describes healthy-catalog empty search and filter results truthfully", () => {
    expect(emptyMachineResultMessage("pac", "all")).toBe("No machines match this search.");
    expect(emptyMachineResultMessage("", "favorites")).toBe("No machines match this filter.");
    expect(emptyMachineResultMessage("pac", "favorites")).toBe(
      "No machines match this search and filter.",
    );
    expect(emptyMachineResultMessage("", "all")).toBe(
      "The active catalog contains no runnable machines.",
    );
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
