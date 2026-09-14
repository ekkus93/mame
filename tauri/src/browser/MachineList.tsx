import type { KeyboardEvent as ReactKeyboardEvent } from "react";

import type { MachineListItem, MachinePage } from "../backend/types";
import { machineAvailabilityLabel, machineStatusLabel } from "../library/libraryQuery";

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
  const selectedIndex = Math.max(
    0,
    page.items.findIndex((machine) => machine.shortName === selected?.shortName),
  );

  return (
    <ul className="mame-machine-list" role="listbox" aria-label="MAME machines">
      {page.items.map((machine, index) => {
        const availability = page.availabilityByShortName[machine.shortName] ?? "unknown";
        const isSelected = selected?.shortName === machine.shortName;
        return (
          <li key={machine.shortName} role="presentation">
            <button
              ref={(element) => registerRow(index, element)}
              type="button"
              role="option"
              aria-selected={isSelected}
              tabIndex={index === selectedIndex ? 0 : -1}
              className={isSelected ? "mame-machine-row is-selected" : "mame-machine-row"}
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
