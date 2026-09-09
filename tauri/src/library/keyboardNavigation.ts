import type { SessionState } from "../backend/types";

export type LibraryShortcut = "focusSearch" | "launchSelected";

export type LibraryShortcutInput = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  repeat: boolean;
  defaultPrevented: boolean;
  editing: boolean;
  gameplayActive: boolean;
};

export function isGameplaySessionState(state: SessionState | null | undefined): boolean {
  return state === "created" || state === "starting" || state === "running" || state === "stopping";
}

export function resolveLibraryShortcut(input: LibraryShortcutInput): LibraryShortcut | null {
  if (
    input.gameplayActive ||
    input.editing ||
    input.repeat ||
    input.defaultPrevented ||
    input.altKey
  ) {
    return null;
  }

  if (input.key === "/" && !input.ctrlKey && !input.metaKey && !input.shiftKey) {
    return "focusSearch";
  }

  if (input.key === "Enter" && (input.ctrlKey || input.metaKey) && !input.shiftKey) {
    return "launchSelected";
  }

  return null;
}

export function nextMachineIndex(
  key: string,
  currentIndex: number,
  itemCount: number,
): number | null {
  if (itemCount <= 0 || currentIndex < 0 || currentIndex >= itemCount) {
    return null;
  }

  switch (key) {
    case "ArrowDown":
      return Math.min(currentIndex + 1, itemCount - 1);
    case "ArrowUp":
      return Math.max(currentIndex - 1, 0);
    case "Home":
      return 0;
    case "End":
      return itemCount - 1;
    default:
      return null;
  }
}

export function isEditableElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  return (
    target.isContentEditable ||
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.tagName === "SELECT"
  );
}
