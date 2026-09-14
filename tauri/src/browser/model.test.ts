import { describe, expect, it } from "vitest";

import { buildMameBrowserRequest, nextBrowserIndex } from "./model";

describe("MAME browser model", () => {
  it("maps canonical frontend filters to bounded backend queries", () => {
    expect(buildMameBrowserRequest("available", " pac ", 10)).toMatchObject({
      text: "pac",
      availability: "available",
      cloneFilter: "all",
      limit: 100,
      offset: 10,
    });
    expect(buildMameBrowserRequest("unavailable", "")).toMatchObject({ availability: "missing" });
    expect(buildMameBrowserRequest("working", "")).toMatchObject({ driverStatus: "good" });
    expect(buildMameBrowserRequest("parents", "")).toMatchObject({ cloneFilter: "parentsOnly" });
    expect(buildMameBrowserRequest("clones", "")).toMatchObject({ cloneFilter: "clonesOnly" });
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
