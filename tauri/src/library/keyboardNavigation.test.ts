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

  it("provides bounded arrow and edge navigation", () => {
    expect(nextMachineIndex("ArrowDown", 0, 3)).toBe(1);
    expect(nextMachineIndex("ArrowDown", 2, 3)).toBe(2);
    expect(nextMachineIndex("ArrowUp", 0, 3)).toBe(0);
    expect(nextMachineIndex("Home", 2, 3)).toBe(0);
    expect(nextMachineIndex("End", 0, 3)).toBe(2);
    expect(nextMachineIndex("PageDown", 0, 3)).toBeNull();
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
