import type { LaunchPreferences } from "../backend/generalSettings";
import type {
  MameEmptyLaunch,
  MameSoftwareItem,
  MameSoftwareLaunch,
  SoftwareListFilter,
} from "../backend/mameSoftware";

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

export type SoftwareGlobalShortcutAction = "none" | "clearSearch" | "back" | "focusSearch";

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

export function softwareGlobalShortcutAction({
  key,
  gameplayInputOwned,
  defaultPrevented,
  repeat,
  altKey,
  ctrlKey,
  metaKey,
  editableTarget,
  searchTarget,
  searchHasValue,
}: {
  key: string;
  gameplayInputOwned: boolean;
  defaultPrevented: boolean;
  repeat: boolean;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  editableTarget: boolean;
  searchTarget: boolean;
  searchHasValue: boolean;
}): SoftwareGlobalShortcutAction {
  if (defaultPrevented || repeat || altKey || gameplayInputOwned) return "none";
  if (key === "Escape") {
    if (searchTarget && searchHasValue) return "clearSearch";
    return editableTarget ? "none" : "back";
  }
  if (editableTarget) return "none";
  if (key === "/" && !ctrlKey && !metaKey) return "focusSearch";
  return "none";
}

export function selectedPartForSoftware(item: MameSoftwareItem | null): string | null {
  return item?.parts.length === 1 ? (item.parts[0]?.name ?? null) : null;
}

export function softwareSupportClassName(supported: MameSoftwareItem["supported"]): string {
  switch (supported) {
    case "partial":
      return "is-partial";
    case "no":
      return "is-unsupported";
    case "yes":
      return "";
  }
}

export function softwareRowClassName({
  isSelected,
  supported,
}: {
  isSelected: boolean;
  supported: MameSoftwareItem["supported"];
}): string {
  return [
    "mame-software-row",
    isSelected ? "is-selected" : "",
    softwareSupportClassName(supported),
  ]
    .filter(Boolean)
    .join(" ");
}

export function softwareStartButtonDisabled({
  selected,
  selectedPart,
  launchStatus,
}: {
  selected: MameSoftwareItem | null;
  selectedPart: string | null;
  launchStatus: string;
}): boolean {
  return (
    !selected ||
    (selected.parts.length > 1 && !selectedPart) ||
    launchStatus === "launching"
  );
}

export function canBeginSoftwareLaunch({
  listName,
  launchInFlight,
  item,
  softwarePart,
}: {
  listName: string;
  launchInFlight: boolean;
  item: MameSoftwareItem;
  softwarePart: string | null;
}): boolean {
  return Boolean(listName) && !launchInFlight && (item.parts.length <= 1 || Boolean(softwarePart));
}

export function shouldPreserveLaunchOnSelection(status: string): boolean {
  return status === "launching";
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
