import type { MameUiMachineFilter, MameUiMachineSearchRequest } from "../backend/mameUi";
import type { MachineListItem } from "../backend/types";

export type MameBrowserFilter = MameUiMachineFilter;
export type MameBrowserFilterValueKind = "manufacturer" | "year" | "sourceFile";

export type MameBrowserFilterDefinition = {
  id: MameBrowserFilter;
  label: string;
  description: string;
  valueKind?: MameBrowserFilterValueKind;
};

export type MameBrowserDeferredFilterDefinition = {
  id: "categoryDeferred" | "customFilterDeferred";
  label: string;
  description: string;
  deferred: true;
};

export type MameBrowserFilterNavItem =
  MameBrowserFilterDefinition | MameBrowserDeferredFilterDefinition;

export const MAME_BROWSER_FILTERS: MameBrowserFilterDefinition[] = [
  { id: "all", label: "Unfiltered", description: "All runnable catalog systems" },
  { id: "available", label: "Available", description: "Media verified as available" },
  {
    id: "unavailable",
    label: "Unavailable",
    description: "Required media is missing or incorrect",
  },
  { id: "working", label: "Working", description: "Systems not marked not-working" },
  {
    id: "notWorking",
    label: "Not Working",
    description: "Systems marked preliminary / not working",
  },
  { id: "mechanical", label: "Mechanical", description: "Mechanical systems" },
  {
    id: "notMechanical",
    label: "Not Mechanical",
    description: "Non-mechanical systems",
  },
  { id: "favorites", label: "Favorites", description: "User-favorited systems" },
  { id: "bios", label: "BIOS", description: "BIOS root systems" },
  { id: "notBios", label: "Not BIOS", description: "Systems that are not BIOS roots" },
  { id: "parents", label: "Parents", description: "Parent systems only" },
  { id: "clones", label: "Clones", description: "Clone systems only" },
  {
    id: "manufacturer",
    label: "Manufacturer",
    description: "Systems from a manufacturer",
    valueKind: "manufacturer",
  },
  { id: "year", label: "Year", description: "Systems from a year", valueKind: "year" },
  {
    id: "sourceFile",
    label: "Source File",
    description: "Systems imported from a MAME source file",
    valueKind: "sourceFile",
  },
  {
    id: "saveSupported",
    label: "Save Supported",
    description: "Systems whose MAME driver supports save states",
  },
  {
    id: "saveUnsupported",
    label: "Save Unsupported",
    description: "Systems whose MAME driver does not support save states",
  },
  {
    id: "chdRequired",
    label: "CHD Required",
    description: "Systems whose authoritative MAME metadata contains disk media",
  },
  {
    id: "noChdRequired",
    label: "No CHD Required",
    description: "Systems whose authoritative MAME metadata contains no disk media",
  },
  {
    id: "verticalScreen",
    label: "Vertical Screen",
    description: "Systems with a rotated vertical display",
  },
  {
    id: "horizontalScreen",
    label: "Horizontal Screen",
    description: "Systems with displays that are not vertically rotated",
  },
];

export const MAME_BROWSER_FILTER_NAV_ITEMS: MameBrowserFilterNavItem[] = [
  ...MAME_BROWSER_FILTERS.slice(0, 7),
  {
    id: "categoryDeferred",
    label: "Category",
    description: "Deferred until authoritative category data exists",
    deferred: true,
  },
  ...MAME_BROWSER_FILTERS.slice(7),
  {
    id: "customFilterDeferred",
    label: "Custom Filter",
    description: "Deferred until persisted composite custom filters exist",
    deferred: true,
  },
];

export const MAME_BROWSER_PAGE_SIZE = 100;

export function filterRequiresValue(filter: MameBrowserFilter): boolean {
  return MAME_BROWSER_FILTERS.some(
    (definition) => definition.id === filter && definition.valueKind !== undefined,
  );
}

export function buildMameBrowserRequest(
  filter: MameBrowserFilter,
  text: string,
  filterValue: string,
  offset = 0,
  preferredMachine: string | null = null,
): MameUiMachineSearchRequest {
  return {
    text: text.trim() || null,
    filter,
    filterValue: filterValue.trim() || null,
    preferredMachine,
    limit: MAME_BROWSER_PAGE_SIZE,
    offset: Math.max(0, offset),
  };
}

export function reconcileMachineSelection(
  items: MachineListItem[],
  current: MachineListItem | null,
  preferredMachine: string | null,
): MachineListItem | null {
  if (current) {
    const refreshed = items.find((item) => item.shortName === current.shortName);
    if (refreshed) return refreshed;
  }
  if (preferredMachine) {
    const preferred = items.find((item) => item.shortName === preferredMachine);
    if (preferred) return preferred;
  }
  return items[0] ?? null;
}

export function browserPageStep(viewportHeight: number, rowHeight: number): number {
  if (
    !Number.isFinite(viewportHeight) ||
    !Number.isFinite(rowHeight) ||
    viewportHeight <= 0 ||
    rowHeight <= 0
  ) {
    return 1;
  }
  return Math.max(1, Math.floor(viewportHeight / rowHeight) - 1);
}

// Home/End intentionally target the current fetched catalog page. The explicit pager is the
// authoritative way to cross backend page boundaries.
export function nextBrowserIndex(
  key: string,
  current: number,
  count: number,
  pageStep = 1,
): number | null {
  if (count <= 0) return null;
  const bounded = Math.min(Math.max(current, 0), count - 1);
  switch (key) {
    case "ArrowDown":
      return Math.min(count - 1, bounded + 1);
    case "ArrowUp":
      return Math.max(0, bounded - 1);
    case "Home":
      return 0;
    case "End":
      return count - 1;
    case "PageDown":
      return Math.min(count - 1, bounded + Math.max(1, pageStep));
    case "PageUp":
      return Math.max(0, bounded - Math.max(1, pageStep));
    default:
      return null;
  }
}

export function viewportBrowserIndex(
  key: string,
  current: number,
  count: number,
  viewportHeight: number,
  rowHeight: number,
): number | null {
  return nextBrowserIndex(key, current, count, browserPageStep(viewportHeight, rowHeight));
}
