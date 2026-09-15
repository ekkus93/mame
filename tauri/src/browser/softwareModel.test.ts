import { describe, expect, it } from "vitest";

import type { MameSoftwareItem } from "../backend/mameSoftware";
import {
  buildEmptyLaunchRequest,
  buildSoftwareLaunchRequest,
  canBeginSoftwareLaunch,
  nextSoftwareIndex,
  selectedPartForSoftware,
  shouldPreserveLaunchOnSelection,
  softwareFilterRequiresValue,
  softwareGlobalShortcutAction,
} from "./softwareModel";

function software(parts: MameSoftwareItem["parts"]): MameSoftwareItem {
  return {
    shortName: "game",
    description: "Game",
    year: "1990",
    publisher: "Publisher",
    cloneOf: null,
    supported: "yes",
    parts,
  };
}

function shortcut(overrides: Partial<Parameters<typeof softwareGlobalShortcutAction>[0]> = {}) {
  return softwareGlobalShortcutAction({
    key: "/",
    gameplayInputOwned: false,
    defaultPrevented: false,
    repeat: false,
    altKey: false,
    ctrlKey: false,
    metaKey: false,
    editableTarget: false,
    searchTarget: false,
    searchHasValue: false,
    ...overrides,
  });
}

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

  it("suppresses software-browser global shortcuts while gameplay owns input", () => {
    expect(shortcut({ gameplayInputOwned: true, key: "/" })).toBe("none");
    expect(shortcut({ gameplayInputOwned: true, key: "Escape" })).toBe("none");
  });

  it("routes software-browser shortcuts only outside editable fields", () => {
    expect(shortcut({ key: "/" })).toBe("focusSearch");
    expect(shortcut({ key: "Escape" })).toBe("back");
    expect(shortcut({ key: "/", editableTarget: true })).toBe("none");
    expect(shortcut({ key: "Escape", editableTarget: true })).toBe("none");
    expect(shortcut({ key: "Escape", editableTarget: true, searchTarget: true, searchHasValue: true })).toBe(
      "clearSearch",
    );
  });

  it("auto-selects only a single software part", () => {
    expect(selectedPartForSoftware(software([]))).toBeNull();
    expect(selectedPartForSoftware(software([{ name: "cart", interface: "cart" }]))).toBe("cart");
    expect(
      selectedPartForSoftware(
        software([
          { name: "cart", interface: "cart" },
          { name: "flop", interface: "floppy" },
        ]),
      ),
    ).toBeNull();
  });

  it("preserves omitted BIOS semantics in software and empty launch requests", () => {
    expect(
      buildSoftwareLaunchRequest({
        shortName: "machine",
        softwareList: "list",
        item: software([{ name: "cart", interface: "cart" }]),
        softwarePart: "cart",
        bios: null,
        launchOverrides: null,
      }),
    ).toMatchObject({ softwareItem: "game", softwarePart: "cart", bios: null });
    expect(
      buildEmptyLaunchRequest({ shortName: "machine", bios: null, launchOverrides: null }),
    ).toEqual({ shortName: "machine", bios: null, launchOverrides: null });
  });

  it("propagates an explicitly selected BIOS to both launch request types", () => {
    expect(
      buildSoftwareLaunchRequest({
        shortName: "machine",
        softwareList: "list",
        item: software([]),
        softwarePart: null,
        bios: "rev3",
        launchOverrides: null,
      }).bios,
    ).toBe("rev3");
    expect(
      buildEmptyLaunchRequest({ shortName: "machine", bios: "rev3", launchOverrides: null }).bios,
    ).toBe("rev3");
  });

  it("keeps a pending launch across selection changes and rejects duplicate activation", () => {
    const item = software([{ name: "cart", interface: "cart" }]);
    expect(shouldPreserveLaunchOnSelection("launching")).toBe(true);
    expect(shouldPreserveLaunchOnSelection("launched")).toBe(false);
    expect(
      canBeginSoftwareLaunch({ listName: "list", launchInFlight: false, item, softwarePart: "cart" }),
    ).toBe(true);
    expect(
      canBeginSoftwareLaunch({ listName: "list", launchInFlight: true, item, softwarePart: "cart" }),
    ).toBe(false);
  });

  it("requires an explicit part for multi-part software", () => {
    const item = software([
      { name: "cart", interface: "cart" },
      { name: "flop", interface: "floppy" },
    ]);
    expect(
      canBeginSoftwareLaunch({ listName: "list", launchInFlight: false, item, softwarePart: null }),
    ).toBe(false);
    expect(
      canBeginSoftwareLaunch({ listName: "list", launchInFlight: false, item, softwarePart: "flop" }),
    ).toBe(true);
  });
});
