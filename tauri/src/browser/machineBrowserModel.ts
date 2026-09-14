import type { MachineSearchRequest } from "../backend/types";

export type MameMachineFilterId =
  | "all"
  | "available"
  | "unavailable"
  | "working"
  | "parents"
  | "clones";

export type MameMachineFilterDefinition = {
  id: MameMachineFilterId;
  label: string;
  description: string;
};

export const MAME_MACHINE_FILTERS: readonly MameMachineFilterDefinition[] = [
  { id: "all", label: "Unfiltered", description: "Show all runnable systems." },
  {
    id: "available",
    label: "Available",
    description: "Show systems with verified media.",
  },
  {
    id: "unavailable",
    label: "Unavailable",
    description: "Show systems with missing or incorrect required media.",
  },
  {
    id: "working",
    label: "Working",
    description: "Show systems reported working by MAME.",
  },
  { id: "parents", label: "Parents", description: "Show parent systems only." },
  { id: "clones", label: "Clones", description: "Show clone systems only." },
] as const;

export const MACHINE_BROWSER_PAGE_SIZE = 100;

export function normalizeMachineSearch(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function buildMachineBrowserRequest(
  filter: MameMachineFilterId,
  search: string,
  offset = 0,
): MachineSearchRequest {
  const request: MachineSearchRequest = {
    text: normalizeMachineSearch(search),
    cloneFilter: "all",
    sort: "descriptionAsc",
    includeDevices: false,
    limit: MACHINE_BROWSER_PAGE_SIZE,
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

export function machinePageRange(
  offset: number,
  itemCount: number,
  total: number,
): string | null {
  if (total <= 0 || itemCount <= 0) {
    return null;
  }
  const first = offset + 1;
  const last = Math.min(offset + itemCount, total);
  return `${first.toLocaleString()}–${last.toLocaleString()} of ${total.toLocaleString()}`;
}
