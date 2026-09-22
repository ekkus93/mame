import { describe, expect, it, vi } from "vitest";

import type { MachineListItem } from "../backend/types";
import { scrollSelectedMachineIntoView } from "./machineListVisibility";

function machine(shortName: string): MachineListItem {
  return {
    shortName,
    description: shortName,
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

function rect(top: number, bottom: number) {
  return () => ({ top, bottom }) as DOMRect;
}

describe("machine list viewport", () => {
  it("keeps a visible row stationary", () => {
    const selected = machine("selected");
    const scrollIntoView = vi.fn();
    const row = { getBoundingClientRect: rect(200, 240), scrollIntoView };
    const viewport = { getBoundingClientRect: rect(100, 600) };
    const didScroll = scrollSelectedMachineIntoView(
      [selected],
      selected,
      [row],
      viewport,
    );
    expect(didScroll).toBe(false);
    expect(scrollIntoView).not.toHaveBeenCalled();
  });

  it("reveals a row clipped by the list region", () => {
    const selected = machine("selected");
    const scrollIntoView = vi.fn();
    const row = { getBoundingClientRect: rect(50, 90), scrollIntoView };
    const viewport = { getBoundingClientRect: rect(100, 600) };
    const didScroll = scrollSelectedMachineIntoView(
      [selected],
      selected,
      [row],
      viewport,
    );
    expect(didScroll).toBe(true);
    expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });
  });
});
