import { describe, expect, it } from "vitest";

import {
  isGameplaySessionState,
  nextMachineIndex,
  resolveLibraryShortcut,
  type LibraryShortcutInput,
} from "./keyboardNavigation";

function shortcut(overrides: Partial<LibraryShortcutInput> = {}) {
  return resolveLibraryShortcut({
    key: "/",
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    repeat: false,
    defaultPrevented: false,
    editing: false,
    gameplayActive: false,
    ...overrides,
  });
}

describe("library keyboard navigation", () => {
  it("maps discoverable search and launch shortcuts", () => {
    expect(shortcut()).toBe("focusSearch");
    expect(shortcut({ key: "Enter", ctrlKey: true })).toBe("launchSelected");
    expect(shortcut({ key: "Enter", metaKey: true })).toBe("launchSelected");
  });

  it("never steals shortcuts while editing or MAME owns gameplay input", () => {
    expect(shortcut({ editing: true })).toBeNull();
    expect(shortcut({ gameplayActive: true })).toBeNull();
    expect(shortcut({ repeat: true })).toBeNull();
    expect(shortcut({ defaultPrevented: true })).toBeNull();
    expect(shortcut({ altKey: true })).toBeNull();
  });

  it("provides bounded arrow, paging, and edge navigation", () => {
    expect(nextMachineIndex("ArrowDown", 0, 30)).toBe(1);
    expect(nextMachineIndex("ArrowDown", 29, 30)).toBe(29);
    expect(nextMachineIndex("ArrowUp", 0, 30)).toBe(0);
    expect(nextMachineIndex("PageDown", 2, 30, 12)).toBe(14);
    expect(nextMachineIndex("PageDown", 25, 30, 12)).toBe(29);
    expect(nextMachineIndex("PageUp", 14, 30, 12)).toBe(2);
    expect(nextMachineIndex("PageUp", 5, 30, 12)).toBe(0);
    expect(nextMachineIndex("Home", 20, 30)).toBe(0);
    expect(nextMachineIndex("End", 0, 30)).toBe(29);
  });

  it("treats only active supervised states as gameplay ownership", () => {
    for (const state of ["created", "starting", "running", "stopping"] as const) {
      expect(isGameplaySessionState(state)).toBe(true);
    }
    for (const state of ["exited", "failed", "crashed"] as const) {
      expect(isGameplaySessionState(state)).toBe(false);
    }
    expect(isGameplaySessionState(null)).toBe(false);
  });
});
