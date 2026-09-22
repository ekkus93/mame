import { describe, expect, it, vi } from "vitest";

import type { MachineListItem } from "../backend/types";
import {
  scrollSelectedMachineIntoView,
  selectedRowVisibilityAction,
} from "./machineListVisibility";
import { reconcileMachineSelection } from "./model";

function machine(index: number): MachineListItem {
  return {
    shortName: `machine-${index}`,
    description: `Machine ${index}`,
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

function rows(count: number) {
  return Array.from({ length: count }, () => ({ scrollIntoView: vi.fn() }));
}

function geometricRow(top: number, bottom: number, viewportEnd = 600) {
  return {
    ownerDocument: { defaultView: { innerHeight: viewportEnd } } as unknown as Document,
    getBoundingClientRect: () => ({ top, bottom }) as DOMRect,
    scrollIntoView: vi.fn(),
  };
}

describe("machine-list selection visibility", () => {
  it("documents nearest-scroll behavior for rows outside the viewport", () => {
    expect(
      selectedRowVisibilityAction({
        viewportStart: 0,
        viewportEnd: 500,
        rowStart: -32,
        rowEnd: 0,
      }),
    ).toBe("scroll-nearest");
    expect(
      selectedRowVisibilityAction({
        viewportStart: 0,
        viewportEnd: 500,
        rowStart: 480,
        rowEnd: 532,
      }),
    ).toBe("scroll-nearest");
  });

  it("documents no-op behavior when the selected row is already fully visible", () => {
    expect(
      selectedRowVisibilityAction({
        viewportStart: 0,
        viewportEnd: 500,
        rowStart: 25,
        rowEnd: 75,
      }),
    ).toBe("already-visible");
  });

  it("reveals an asynchronously selected result near the beginning with nearest scrolling", () => {
    const items = Array.from({ length: 100 }, (_, index) => machine(index));
    const selected = reconcileMachineSelection(items, null, "machine-2");
    const rowRefs = rows(items.length);

    expect(scrollSelectedMachineIntoView(items, selected, rowRefs)).toBe(true);
    expect(rowRefs[2]!.scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });
    expect(rowRefs[99]!.scrollIntoView).not.toHaveBeenCalled();
  });

  it("reveals an asynchronously selected result near the end without forcing an edge alignment", () => {
    const items = Array.from({ length: 100 }, (_, index) => machine(index));
    const selected = reconcileMachineSelection(items, null, "machine-97");
    const rowRefs = rows(items.length);

    expect(scrollSelectedMachineIntoView(items, selected, rowRefs)).toBe(true);
    expect(rowRefs[97]!.scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });
  });

  it("avoids an unnecessary scroll jump when the selected row is already visible", () => {
    const items = [machine(0), machine(1), machine(2)];
    const selected = reconcileMachineSelection(items, null, "machine-1");
    const rowRefs = [geometricRow(-50, -1), geometricRow(100, 140), geometricRow(700, 760)];

    expect(scrollSelectedMachineIntoView(items, selected, rowRefs)).toBe(false);
    expect(rowRefs[1]!.scrollIntoView).not.toHaveBeenCalled();
  });

  it("scrolls a partially hidden selected row using nearest alignment", () => {
    const items = [machine(0), machine(1), machine(2)];
    const selected = reconcileMachineSelection(items, null, "machine-2");
    const rowRefs = [geometricRow(10, 40), geometricRow(100, 140), geometricRow(580, 640)];

    expect(scrollSelectedMachineIntoView(items, selected, rowRefs)).toBe(true);
    expect(rowRefs[2]!.scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });
  });

  it("does not scroll and clears selection when replacement results are empty", () => {
    const previous = machine(40);
    const selected = reconcileMachineSelection([], previous, null);
    const rowRefs = rows(100);

    expect(selected).toBeNull();
    expect(scrollSelectedMachineIntoView([], selected, rowRefs)).toBe(false);
    expect(rowRefs.every((row) => row.scrollIntoView.mock.calls.length === 0)).toBe(true);
  });

  it("does not scroll when the selected identity is absent from replacement results", () => {
    const items = [machine(0), machine(1)];
    const rowRefs = rows(items.length);
    expect(scrollSelectedMachineIntoView(items, machine(99), rowRefs)).toBe(false);
  });
});
