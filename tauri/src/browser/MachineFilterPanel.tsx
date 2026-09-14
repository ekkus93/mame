import { MAME_BROWSER_FILTERS, type MameBrowserFilter } from "./model";

export function MachineFilterPanel({
  active,
  filterValue,
  onChange,
  onFilterValueChange,
}: {
  active: MameBrowserFilter;
  filterValue: string;
  onChange: (filter: MameBrowserFilter) => void;
  onFilterValueChange: (value: string) => void;
}) {
  const activeDefinition = MAME_BROWSER_FILTERS.find((filter) => filter.id === active);

  return (
    <aside className="mame-filter-panel" aria-label="Machine filters">
      <div className="mame-region-heading">Filters</div>
      <div className="mame-filter-list" role="listbox" aria-label="MAME machine filters">
        {MAME_BROWSER_FILTERS.map((filter) => (
          <button
            key={filter.id}
            type="button"
            role="option"
            aria-selected={active === filter.id}
            className={active === filter.id ? "mame-filter is-selected" : "mame-filter"}
            title={filter.description}
            onClick={() => onChange(filter.id)}
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
        and a persisted composite-filter model exist. CHD filters land with listxml disk metadata.
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
