import type {
  MachineAvailability,
  MachineListItem,
  MachineSearchRequest,
  MachineSort,
} from "../backend/types";

export const LIBRARY_PAGE_SIZE = 100;

export type LibraryFilters = {
  text: string;
  manufacturer: string;
  year: string;
  driverStatus: "" | "good" | "imperfect" | "preliminary";
  availability: "" | MachineAvailability;
  cloneFilter: "all" | "parentsOnly" | "clonesOnly";
  sort: MachineSort;
};

export const DEFAULT_LIBRARY_FILTERS: LibraryFilters = {
  text: "",
  manufacturer: "",
  year: "",
  driverStatus: "",
  availability: "",
  cloneFilter: "all",
  sort: "descriptionAsc",
};

export function buildMachineSearchRequest(
  filters: LibraryFilters,
  offset = 0,
): MachineSearchRequest {
  return {
    text: normalize(filters.text),
    manufacturer: normalize(filters.manufacturer),
    year: normalize(filters.year),
    driverStatus: filters.driverStatus || null,
    availability: filters.availability || null,
    cloneFilter: filters.cloneFilter,
    sort: filters.sort,
    includeDevices: false,
    limit: LIBRARY_PAGE_SIZE,
    offset: Math.max(0, offset),
  };
}

export function machineAvailabilityLabel(availability: MachineAvailability): string {
  switch (availability) {
    case "available":
      return "Available";
    case "missing":
      return "Missing";
    case "unknown":
      return "Unknown";
  }
}

export function machineStatusLabel(
  machine: Pick<MachineListItem, "runnable" | "driverStatus">,
): string {
  if (!machine.runnable) {
    return "Not runnable";
  }

  switch (machine.driverStatus) {
    case "good":
      return "Working";
    case "imperfect":
      return "Imperfect";
    case "preliminary":
      return "Preliminary";
    default:
      return "Runnable";
  }
}

function normalize(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}
