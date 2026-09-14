import { describe, expect, it } from "vitest";

import { nextSoftwareIndex, softwareFilterRequiresValue } from "./softwareModel";

describe("MAME software browser model", () => {
  it("requires values only for exact year and publisher filters", () => {
    expect(softwareFilterRequiresValue("year")).toBe(true);
    expect(softwareFilterRequiresValue("publisher")).toBe(true);
    expect(softwareFilterRequiresValue("parents")).toBe(false);
    expect(softwareFilterRequiresValue("supported")).toBe(false);
  });

  it("implements bounded MAME-style list navigation", () => {
    expect(nextSoftwareIndex("ArrowDown", 0, 40)).toBe(1);
    expect(nextSoftwareIndex("PageDown", 0, 40)).toBe(10);
    expect(nextSoftwareIndex("PageUp", 3, 40)).toBe(0);
    expect(nextSoftwareIndex("End", 3, 40)).toBe(39);
    expect(nextSoftwareIndex("Home", 39, 40)).toBe(0);
    expect(nextSoftwareIndex("x", 3, 40)).toBeNull();
  });
});
