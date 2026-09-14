import type {
  MameUiMachineFilter,
  MameUiMachineSearchRequest,
} from "../backend/mameUi";

export type MameBrowserFilter = MameUiMachineFilter;
export type MameBrowserFilterValueKind = "manufacturer" | "year" | "sourceFile";

export type MameBrowserFilterDefinition = {
  id: MameBrowserFilter;
  label: string;
  description: string;
  valueKind?: MameBrowserFilterValueKind;
};

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
    id: "verticalScreen",
    label: "Vertical Screen",
    description: "Systems with a rotated vertical display",
  },
  {
    id: "horizontalScreen",
    label: "Horizontal Screen",
    description: "Systems not marked with a rotated vertical display",
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
): MameUiMachineSearchRequest {
  return {
    text: text.trim() || null,
    filter,
    filterValue: filterValue.trim() || null,
    limit: MAME_BROWSER_PAGE_SIZE,
    offset: Math.max(0, offset),
  };
}

export function nextBrowserIndex(
  key: string,
  current: number,
  count: number,
  pageStep = 10,
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
