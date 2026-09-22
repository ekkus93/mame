import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { MameSoftwareItem, MameSoftwarePage } from "../backend/mameSoftware";
import { SoftwareResultsList } from "./SoftwareBrowser";

function item(overrides: Partial<MameSoftwareItem> = {}): MameSoftwareItem {
  return {
    shortName: "cart1",
    description: "Cart One",
    year: "1984",
    publisher: "Publisher",
    cloneOf: null,
    supported: "yes",
    parts: [{ name: "cart", interface: "cart" }],
    ...overrides,
  };
}

function page(items: MameSoftwareItem[]): MameSoftwarePage {
  return {
    schemaVersion: 1,
    machineShortName: "a2600",
    softwareListName: "a2600",
    softwareListDescription: "Atari 2600 cartridges",
    total: items.length,
    offset: 0,
    limit: 50,
    items,
  };
}

function renderResults(selected: MameSoftwareItem | null, items: MameSoftwareItem[]) {
  return renderToStaticMarkup(
    <SoftwareResultsList
      page={page(items)}
      selected={selected}
      registerRow={() => undefined}
      onSelect={() => undefined}
      onActivate={() => undefined}
      onNavigate={() => undefined}
    />,
  );
}

describe("SoftwareResultsList component", () => {
  it("renders the selected software row with selected aria state and MAME row treatment", () => {
    const selected = item({ shortName: "cart2", description: "Cart Two", supported: "partial" });
    const html = renderResults(selected, [item(), selected]);

    expect(html).toContain('role="listbox"');
    expect(html).toContain('role="option"');
    expect(html).toContain('aria-selected="true"');
    expect(html).toContain('class="mame-software-row is-selected is-partial"');
    expect(html).toContain('data-support="partial"');
    expect(html).toContain('tabindex="0"');
    expect(html).toContain('class="mame-software-title"');
    expect(html).toContain("Cart Two");
  });

  it("renders unsupported software rows with the muted unsupported class and support label", () => {
    const unsupported = item({ shortName: "badcart", description: "Bad Cart", supported: "no" });
    const html = renderResults(null, [unsupported]);

    expect(html).toContain('class="mame-software-row is-unsupported"');
    expect(html).toContain('data-support="no"');
    expect(html).toContain('class="mame-software-support"');
    expect(html).toContain("no");
  });
});
