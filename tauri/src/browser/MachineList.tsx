import { type KeyboardEvent as ReactKeyboardEvent, useEffect, useRef } from "react";

import type { MachineListItem, MachinePage } from "../backend/types";
import { machineAvailabilityLabel, machineStatusLabel } from "../library/libraryQuery";
import { scrollSelectedMachineIntoView } from "./machineListVisibility";
import { viewportBrowserIndex } from "./model";

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
  const listRef = useRef<HTMLUListElement | null>(null);
  const rowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const selectedIndex = Math.max(
    0,
    page.items.findIndex((machine) => machine.shortName === selected?.shortName),
  );

  // Query/filter/search replacement may select a row outside the old viewport. Measure against
  // the actual scrolling list region (the list parent), not the WebView window. Nearest scrolling
  // reveals clipped rows while leaving a fully visible row stationary. Keyboard focus remains the
  // normal browser-driven scrolling path for direct navigation.
  useEffect(() => {
    scrollSelectedMachineIntoView(
      page.items,
      selected,
      rowRefs.current,
      listRef.current?.parentElement ?? null,
    );
  }, [page.items, selected]);

  function handleNavigation(event: ReactKeyboardEvent<HTMLButtonElement>, index: number) {
    if (event.key !== "PageUp" && event.key !== "PageDown") {
      onNavigate(event, index);
      return;
    }

    const viewport = listRef.current?.parentElement;
    const row = rowRefs.current[index];
    const viewportHeight = viewport?.clientHeight ?? 0;
    const rowHeight = row?.getBoundingClientRect().height ?? 0;
    const nextIndex = viewportBrowserIndex(
      event.key,
      index,
      page.items.length,
      viewportHeight,
      rowHeight,
    );
    if (nextIndex === null) return;

    event.preventDefault();
    const next = page.items[nextIndex];
    if (!next) return;
    onSelect(next);
    rowRefs.current[nextIndex]?.focus();
  }

  return (
    <ul ref={listRef} className="mame-machine-list" role="listbox" aria-label="MAME machines">
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
              onKeyDown={(event) => handleNavigation(event, index)}
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
