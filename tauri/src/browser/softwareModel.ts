import type {
  MameEmptyLaunch,
  MameSoftwareItem,
  MameSoftwareLaunch,
  SoftwareListFilter,
} from "../backend/mameSoftware";
import type { LaunchPreferences } from "../backend/generalSettings";

export const SOFTWARE_PAGE_SIZE = 50;
export const SOFTWARE_PAGE_NAVIGATION_STEP = 10;

export const SOFTWARE_FILTERS: ReadonlyArray<{ id: SoftwareListFilter; label: string }> = [
  { id: "all", label: "All" },
  { id: "parents", label: "Parents" },
  { id: "clones", label: "Clones" },
  { id: "year", label: "Year" },
  { id: "publisher", label: "Publisher" },
  { id: "supported", label: "Supported" },
  { id: "partiallySupported", label: "Partially Supported" },
  { id: "unsupported", label: "Unsupported" },
];

export function softwareFilterRequiresValue(filter: SoftwareListFilter): boolean {
  return filter === "year" || filter === "publisher";
}

export function nextSoftwareIndex(key: string, index: number, count: number): number | null {
  if (count <= 0) return null;
  switch (key) {
    case "ArrowUp":
      return Math.max(0, index - 1);
    case "ArrowDown":
      return Math.min(count - 1, index + 1);
    case "Home":
      return 0;
    case "End":
      return count - 1;
    case "PageUp":
      return Math.max(0, index - SOFTWARE_PAGE_NAVIGATION_STEP);
    case "PageDown":
      return Math.min(count - 1, index + SOFTWARE_PAGE_NAVIGATION_STEP);
    default:
      return null;
  }
}

export function selectedPartForSoftware(item: MameSoftwareItem | null): string | null {
  return item?.parts.length === 1 ? (item.parts[0]?.name ?? null) : null;
}

export function buildSoftwareLaunchRequest({
  shortName,
  softwareList,
  item,
  softwarePart,
  bios,
  launchOverrides,
}: {
  shortName: string;
  softwareList: string;
  item: MameSoftwareItem;
  softwarePart: string | null;
  bios: string | null;
  launchOverrides: LaunchPreferences | null;
}): MameSoftwareLaunch {
  return {
    shortName,
    softwareList,
    softwareItem: item.shortName,
    softwarePart,
    bios,
    launchOverrides,
  };
}

export function buildEmptyLaunchRequest({
  shortName,
  bios,
  launchOverrides,
}: {
  shortName: string;
  bios: string | null;
  launchOverrides: LaunchPreferences | null;
}): MameEmptyLaunch {
  return { shortName, bios, launchOverrides };
}
