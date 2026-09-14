import type { MachineSearchRequest } from "../backend/types";

export type MameBrowserFilter =
  | "all"
  | "available"
  | "unavailable"
  | "working"
  | "parents"
  | "clones";

export type MameBrowserFilterDefinition = {
  id: MameBrowserFilter;
  label: string;
  description: string;
};

export const MAME_BROWSER_FILTERS: MameBrowserFilterDefinition[] = [
  { id: "all", label: "Unfiltered", description: "All runnable catalog systems" },
  { id: "available", label: "Available", description: "Media verified as available" },
  {
    id: "unavailable",
    label: "Unavailable",
    description: "Required media is missing or incorrect",
  },
  { id: "working", label: "Working", description: "Drivers reported as working" },
  { id: "parents", label: "Parents", description: "Parent systems only" },
  { id: "clones", label: "Clones", description: "Clone systems only" },
];

export const MAME_BROWSER_PAGE_SIZE = 100;

export function buildMameBrowserRequest(
  filter: MameBrowserFilter,
  text: string,
  offset = 0,
): MachineSearchRequest {
  const request: MachineSearchRequest = {
    text: text.trim() || null,
    cloneFilter: "all",
    sort: "descriptionAsc",
    includeDevices: false,
    limit: MAME_BROWSER_PAGE_SIZE,
    offset: Math.max(0, offset),
  };

  switch (filter) {
    case "available":
      request.availability = "available";
      break;
    case "unavailable":
      request.availability = "missing";
      break;
    case "working":
      request.driverStatus = "good";
      break;
    case "parents":
      request.cloneFilter = "parentsOnly";
      break;
    case "clones":
      request.cloneFilter = "clonesOnly";
      break;
    case "all":
      break;
  }

  return request;
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
