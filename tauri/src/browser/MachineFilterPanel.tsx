import { useEffect, useRef, type KeyboardEvent as ReactKeyboardEvent } from "react";

import { MAME_BROWSER_FILTERS, type MameBrowserFilter } from "./model";

export function MachineFilterPanel({
  active,
  filterValue,
  onChange,
  onFilterValueChange,
  registerActiveButton,
  onNavigateToMachines,
}: {
  active: MameBrowserFilter;
  filterValue: string;
  onChange: (filter: MameBrowserFilter) => void;
  onFilterValueChange: (value: string) => void;
  registerActiveButton: (element: HTMLButtonElement | null) => void;
  onNavigateToMachines: () => void;
}) {
  const buttonRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const activeDefinition = MAME_BROWSER_FILTERS.find((filter) => filter.id === active);
  const activeIndex = Math.max(
    0,
    MAME_BROWSER_FILTERS.findIndex((filter) => filter.id === active),
  );

  useEffect(() => {
    registerActiveButton(buttonRefs.current[activeIndex] ?? null);
  }, [activeIndex, registerActiveButton]);

  function handleKey(event: ReactKeyboardEvent<HTMLButtonElement>, index: number) {
    if (event.key === "ArrowRight") {
      event.preventDefault();
      onNavigateToMachines();
      return;
    }

    let nextIndex: number | null = null;
    switch (event.key) {
      case "ArrowDown":
        nextIndex = Math.min(MAME_BROWSER_FILTERS.length - 1, index + 1);
        break;
      case "ArrowUp":
        nextIndex = Math.max(0, index - 1);
        break;
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = MAME_BROWSER_FILTERS.length - 1;
        break;
      default:
        return;
    }

    event.preventDefault();
    const next = MAME_BROWSER_FILTERS[nextIndex];
    if (!next) return;
    onChange(next.id);
    buttonRefs.current[nextIndex]?.focus();
  }

  return (
    <aside className="mame-filter-panel" aria-label="Machine filters">
      <div className="mame-region-heading">Filters</div>
      <div className="mame-filter-list" role="listbox" aria-label="MAME machine filters">
        {MAME_BROWSER_FILTERS.map((filter, index) => (
          <button
            key={filter.id}
            ref={(element) => {
              buttonRefs.current[index] = element;
            }}
            type="button"
            role="option"
            aria-selected={active === filter.id}
            tabIndex={active === filter.id ? 0 : -1}
            className={active === filter.id ? "mame-filter is-selected" : "mame-filter"}
            title={filter.description}
            onClick={() => onChange(filter.id)}
            onKeyDown={(event) => handleKey(event, index)}
          >
            {filter.label}
          </button>
        ))}
      </div>
      {activeDefinition?.valueKind && (
        <label className="mame-filter-value">
          <span>{filterValueLabel(activeDefinition.valueKind)}</span>
          <input
            type={activeDefinition.valueKind === "year" ? "text" : "search"}
            inputMode={activeDefinition.valueKind === "year" ? "numeric" : undefined}
            value={filterValue}
            autoComplete="off"
            onChange={(event) => onFilterValueChange(event.target.value)}
          />
        </label>
      )}
      <p className="mame-filter-note">
        Category and Custom Filter remain deliberately deferred until authoritative category data
        and a persisted composite-filter model exist.
      </p>
    </aside>
  );
}

function filterValueLabel(kind: "manufacturer" | "year" | "sourceFile"): string {
  switch (kind) {
    case "manufacturer":
      return "Manufacturer";
    case "year":
      return "Year";
    case "sourceFile":
      return "Source file";
  }
}
