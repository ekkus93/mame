import { MAME_BROWSER_FILTERS, type MameBrowserFilter } from "./model";

export function MachineFilterPanel({
  active,
  onChange,
}: {
  active: MameBrowserFilter;
  onChange: (filter: MameBrowserFilter) => void;
}) {
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
      <p className="mame-filter-note">
        Additional canonical MAME filters are added as their typed catalog predicates land.
      </p>
    </aside>
  );
}
