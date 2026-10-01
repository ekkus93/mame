import { createRef } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { MachineDetail } from "../backend/types";
import { MachineRightPanel } from "./MachineRightPanel";

function detail(): MachineDetail {
  return {
    schemaVersion: 1,
    generationId: 7,
    shortName: "pacman",
    description: "Pac-Man",
    year: "1980",
    manufacturer: "Namco",
    sourceFile: "pacman.cpp",
    cloneOf: null,
    parentDescription: null,
    romOf: null,
    isBios: false,
    isDevice: false,
    isMechanical: false,
    runnable: true,
    canStartEmpty: false,
    driverStatus: "good",
    driverEmulation: "good",
    driverCocktail: "good",
    driverSavestate: "supported",
    driverRequiresArtwork: false,
    driverUnofficial: false,
    driverNoSoundHardware: false,
    driverIncomplete: false,
    displays: [],
    softwareLists: [],
  };
}

function render(
  view: "images" | "info",
  availability: "available" | "missing" | "unknown" = "unknown",
) {
  return renderToStaticMarkup(
    <MachineRightPanel
      detail={detail()}
      availability={availability}
      view={view}
      onViewChange={() => undefined}
      artworkKind="screenshot"
      onArtworkKindChange={() => undefined}
      pendingLaunchOverrides={null}
      onPendingLaunchOverridesChanged={() => undefined}
      onAuditResultChanged={() => undefined}
      onConfigureContent={() => undefined}
      firstTabRef={createRef<HTMLButtonElement>()}
      onNavigateToMachines={() => undefined}
      onSettingsClose={() => undefined}
      gameplayInputOwned={false}
    />,
  );
}

describe("MachineRightPanel primary tabs", () => {
  it("renders Images and Infos as a keyboard tablist", () => {
    const html = render("images");
    expect(html).toContain('role="tablist"');
    expect(html).toContain('aria-label="Machine Images and Infos"');
    expect(html).toContain('role="tab"');
    expect(html).toContain("Images");
    expect(html).toContain("Infos");
  });

  it("keeps Images selected and tabbable when artwork is active", () => {
    const html = render("images");
    expect(html).toContain('aria-selected="true" tabindex="0" class="is-selected"');
    expect(html).toContain('aria-selected="false" tabindex="-1" class=""');
  });

  it("keeps Infos selected and tabbable when information is active", () => {
    const html = render("info");
    expect(html).toContain('aria-selected="false" tabindex="-1" class=""');
    expect(html).toContain('aria-selected="true" tabindex="0" class="is-selected"');
  });

  it("prioritizes unaudited content guidance before optional media", () => {
    const html = render("images", "unknown");
    expect(html).toContain("Not audited");
    expect(html).toContain("Audit before");
    expect(html).toContain("Configure content");
  });

  it("explains missing content with actionable content guidance", () => {
    const html = render("images", "missing");
    expect(html).toContain("Missing content");
    expect(html).toContain("Required ROM");
    expect(html).toContain("missing or incorrect");
    expect(html).toContain("Configure content");
  });

  it("reports audited content as ready to start", () => {
    const html = render("info", "available");
    expect(html).toContain("Available locally");
    expect(html).toContain("Audited content is ready to start");
  });
});
