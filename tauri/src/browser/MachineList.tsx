import { type KeyboardEvent as ReactKeyboardEvent, useEffect, useRef } from "react";

import type { MachineListItem, MachinePage } from "../backend/types";
import { machineAvailabilityLabel, machineStatusLabel } from "../library/libraryQuery";

type ScrollableMachineRow = Pick<HTMLElement, "scrollIntoView">;

export function scrollSelectedMachineIntoView(
  items: MachineListItem[],
  selected: MachineListItem | null,
  rows: Array<ScrollableMachineRow | null>,
): boolean {
  if (!selected) return false;
  const index = items.findIndex((machine) => machine.shortName === selected.shortName);
  const row = index >= 0 ? rows[index] : null;
  if (!row) return false;
  row.scrollIntoView({ block: "nearest" });
  return true;
}

export function MachineList({
  page,
  selected,
  registerRow,
  onSelect,
  onNavigate,
  onActivate,
}: {
  page: MachinePage;
  selected: MachineListItem | null;
  registerRow: (index: number, element: HTMLButtonElement | null) => void;
  onSelect: (machine: MachineListItem) => void;
  onNavigate: (event: ReactKeyboardEvent<HTMLButtonElement>, index: number) => void;
  onActivate: (machine: MachineListItem) => void;
}) {
  const rowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const selectedIndex = Math.max(
    0,
    page.items.findIndex((machine) => machine.shortName === selected?.shortName),
  );

  // Query/filter/search replacement may select a row outside the old viewport. "nearest"
  // reveals it while leaving an already-visible row stationary; keyboard focus continues to
  // provide the normal browser-driven scrolling path for direct navigation.
  useEffect(() => {
    scrollSelectedMachineIntoView(page.items, selected, rowRefs.current);
  }, [page.items, selected]);

  return (
    <ul className="mame-machine-list" role="listbox" aria-label="MAME machines">
      {page.items.map((machine, index) => {
        const availability = page.availabilityByShortName[machine.shortName] ?? "unknown";
        const isSelected = selected?.shortName === machine.shortName;
        const unavailable = !machine.runnable || availability === "missing";
        const rowClass = [
          "mame-machine-row",
          isSelected ? "is-selected" : "",
          unavailable ? "is-unavailable" : "",
        ]
          .filter(Boolean)
          .join(" ");
        return (
          <li key={machine.shortName} role="presentation">
            <button
              ref={(element) => {
                rowRefs.current[index] = element;
                registerRow(index, element);
              }}
              type="button"
              role="option"
              aria-selected={isSelected}
              tabIndex={index === selectedIndex ? 0 : -1}
              className={rowClass}
              onFocus={() => onSelect(machine)}
              onClick={() => onSelect(machine)}
              onDoubleClick={() => onActivate(machine)}
              onKeyDown={(event) => onNavigate(event, index)}
            >
              <span className="mame-machine-title">{machine.description}</span>
              <span className="mame-machine-short">{machine.shortName}</span>
              <span className="mame-machine-year">{machine.year ?? "—"}</span>
              <span className="mame-machine-maker">{machine.manufacturer ?? "Unknown"}</span>
              <span className={`mame-machine-state state-${machine.driverStatus ?? "unknown"}`}>
                {machineStatusLabel(machine)}
              </span>
              <span className={`mame-machine-availability availability-${availability}`}>
                {machineAvailabilityLabel(availability)}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
