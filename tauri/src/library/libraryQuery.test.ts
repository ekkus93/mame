import { describe, expect, it } from "vitest";

import type { MachineListItem } from "../backend/types";
import {
  buildMachineSearchRequest,
  DEFAULT_LIBRARY_FILTERS,
  LIBRARY_PAGE_SIZE,
  machineStatusLabel,
} from "./libraryQuery";

describe("library query view model", () => {
  it("normalizes filters and always requests a bounded page", () => {
    expect(
      buildMachineSearchRequest(
        {
          ...DEFAULT_LIBRARY_FILTERS,
          text: "  galax  ",
          manufacturer: " Namco ",
          driverStatus: "good",
          cloneFilter: "parentsOnly",
          sort: "yearDesc",
        },
        200,
      ),
    ).toEqual({
      text: "galax",
      manufacturer: "Namco",
      year: null,
      driverStatus: "good",
      cloneFilter: "parentsOnly",
      sort: "yearDesc",
      includeDevices: false,
      limit: LIBRARY_PAGE_SIZE,
      offset: 200,
    });
  });

  it("uses deterministic description sorting by default", () => {
    expect(buildMachineSearchRequest(DEFAULT_LIBRARY_FILTERS).sort).toBe("descriptionAsc");
  });

  it("preserves an explicit manufacturer sort mode", () => {
    expect(
      buildMachineSearchRequest({
        ...DEFAULT_LIBRARY_FILTERS,
        sort: "manufacturerDesc",
      }).sort,
    ).toBe("manufacturerDesc");
  });

  it("clamps a negative page offset", () => {
    expect(buildMachineSearchRequest(DEFAULT_LIBRARY_FILTERS, -50).offset).toBe(0);
  });

  it("maps backend driver state to concise user-facing status", () => {
    const machine = (overrides: Partial<MachineListItem> = {}): MachineListItem => ({
      shortName: "galaxian",
      description: "Galaxian",
      year: "1979",
      manufacturer: "Namco",
      sourceFile: "galaxian/galaxian.cpp",
      cloneOf: null,
      runnable: true,
      isDevice: false,
      driverStatus: "good",
      displayCount: 1,
      softwareListCount: 0,
      ...overrides,
    });

    expect(machineStatusLabel(machine())).toBe("Working");
    expect(machineStatusLabel(machine({ driverStatus: "imperfect" }))).toBe("Imperfect");
    expect(machineStatusLabel(machine({ runnable: false }))).toBe("Not runnable");
  });
});
