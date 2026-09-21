import type {
  MameExecutableRequest,
  MameVersionReport,
  MachinePage,
  MetadataStatus,
} from "../backend/types";

export type MameCatalogState =
  | { status: "notConfigured" }
  | { status: "executableUnavailable"; message: string }
  | { status: "checking" }
  | { status: "importNeeded"; freshness: "empty" | "stale"; previousMachineCount: number | null }
  | { status: "importing" }
  | { status: "importFailed"; message: string }
  | { status: "ready"; machineCount: number };

export type MachineAvailabilityNotice = "unknown" | "noneAvailable" | null;

export function initialMameCatalogState(mame: MameVersionReport): MameCatalogState {
  switch (mame.status) {
    case "notConfigured":
      return { status: "notConfigured" };
    case "unavailable":
      return { status: "executableUnavailable", message: mame.errorMessage };
    case "available":
      return { status: "checking" };
  }
}

export function metadataExecutableRequest(mame: MameVersionReport): MameExecutableRequest | null {
  if (mame.status !== "available") return null;
  switch (mame.identity.source) {
    case "external":
      return { source: "external", path: mame.identity.path };
    case "developmentTree":
      return { source: "developmentTree", path: mame.identity.path };
    case "bundled":
      return null;
  }
}

export function catalogStateFromMetadata(status: MetadataStatus): MameCatalogState {
  if (status.freshness === "fresh" && status.activeGeneration) {
    return { status: "ready", machineCount: status.activeGeneration.machineCount };
  }
  return {
    status: "importNeeded",
    freshness: status.freshness === "stale" ? "stale" : "empty",
    previousMachineCount: status.activeGeneration?.machineCount ?? null,
  };
}

export function catalogCanQuery(state: MameCatalogState): boolean {
  return state.status === "ready";
}

export function catalogRangeLabel(state: MameCatalogState): string {
  switch (state.status) {
    case "notConfigured":
      return "MAME not configured";
    case "executableUnavailable":
      return "MAME unavailable";
    case "checking":
      return "Checking metadata…";
    case "importNeeded":
      return state.freshness === "stale" ? "Metadata refresh required" : "Metadata import required";
    case "importing":
      return "Importing metadata…";
    case "importFailed":
      return "Metadata import failed";
    case "ready":
      return "Metadata loaded";
  }
}

export function machineAvailabilityNotice(page: MachinePage | null): MachineAvailabilityNotice {
  if (!page || page.items.length === 0) return null;
  const availability = page.items.map(
    (machine) => page.availabilityByShortName[machine.shortName] ?? "unknown",
  );
  if (availability.every((value) => value === "unknown")) return "unknown";
  if (!availability.some((value) => value === "available")) return "noneAvailable";
  return null;
}
