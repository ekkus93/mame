import type { MachineDetail, MachineListItem } from "../backend/types";
import { machineStatusLabel } from "../library/libraryQuery";

export type MachineDriverStatusLoadState = "idle" | "loading" | "ready" | "error";

function parentageLabel(detail: MachineDetail): string {
  if (detail.isBios) return "BIOS";
  if (detail.cloneOf) return `Clone of ${detail.parentDescription ?? detail.cloneOf}`;
  return "Parent";
}

function displayLabel(detail: MachineDetail): string {
  if (detail.displays.length === 0) return "Graphics: unknown";
  const orientation = detail.displays.some(
    (display) => display.rotate === 90 || display.rotate === 270,
  )
    ? "vertical"
    : "horizontal";
  return `Graphics: ${detail.driverEmulation ?? "unknown"} · ${orientation}`;
}

function soundLabel(detail: MachineDetail): string {
  if (detail.driverNoSoundHardware) return "Sound: no sound hardware";
  return "Sound: present or driver-reported";
}

export function machineDriverStatusText({
  detail,
  selected,
  detailStatus,
  errorMessage,
}: {
  detail: MachineDetail | null;
  selected: MachineListItem | null;
  detailStatus: MachineDriverStatusLoadState;
  errorMessage: string | null;
}): string {
  if (detail) {
    return [
      `${detail.description} (${detail.shortName})`,
      detail.year ?? "Year unknown",
      detail.manufacturer ?? "Manufacturer unknown",
      parentageLabel(detail),
      `Overall: ${machineStatusLabel(detail)}`,
      displayLabel(detail),
      soundLabel(detail),
      `Save: ${detail.driverSavestate ?? "unknown"}`,
    ].join("  |  ");
  }

  if (detailStatus === "loading" && selected) {
    return `Loading driver status for ${selected.description} (${selected.shortName})…`;
  }

  if (detailStatus === "error") {
    return `Driver status unavailable${errorMessage ? `: ${errorMessage}` : ""}`;
  }

  return "Select a system to view year, manufacturer, driver, graphics, and sound status.";
}

export function MachineDriverStatus({
  detail,
  selected,
  detailStatus,
  errorMessage,
}: {
  detail: MachineDetail | null;
  selected: MachineListItem | null;
  detailStatus: MachineDriverStatusLoadState;
  errorMessage: string | null;
}) {
  return (
    <footer
      className="mame-driver-status"
      aria-label="Selected machine driver status"
      aria-live="polite"
    >
      {machineDriverStatusText({ detail, selected, detailStatus, errorMessage })}
    </footer>
  );
}
