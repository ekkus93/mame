import { describe, expect, it } from "vitest";

import {
  buildMachineBrowserRequest,
  machinePageRange,
  normalizeMachineSearch,
} from "./machineBrowserModel";

describe("MAME machine browser model", () => {
  it("maps canonical implemented filters to bounded backend requests", () => {
    expect(buildMachineBrowserRequest("available", " pac ", -10)).toMatchObject({
      text: "pac",
      availability: "available",
      cloneFilter: "all",
      limit: 100,
      offset: 0,
    });
    expect(buildMachineBrowserRequest("unavailable", "", 100).availability).toBe("missing");
    expect(buildMachineBrowserRequest("working", "", 0).driverStatus).toBe("good");
    expect(buildMachineBrowserRequest("parents", "", 0).cloneFilter).toBe("parentsOnly");
    expect(buildMachineBrowserRequest("clones", "", 0).cloneFilter).toBe("clonesOnly");
  });

  it("normalizes live search and formats dense-list ranges", () => {
    expect(normalizeMachineSearch("   ")).toBeNull();
    expect(normalizeMachineSearch(" Galaga ")).toBe("Galaga");
    expect(machinePageRange(0, 100, 245)).toBe("1–100 of 245");
    expect(machinePageRange(200, 45, 245)).toBe("201–245 of 245");
    expect(machinePageRange(0, 0, 0)).toBeNull();
  });
});
