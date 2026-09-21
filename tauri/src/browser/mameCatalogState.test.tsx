import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import type { MameVersionReport, MachinePage, MetadataStatus } from "../backend/types";
import { MameCatalogStatePanel } from "./MameCatalogStatePanel";
import {
  catalogCanQuery,
  catalogRangeLabel,
  catalogStateFromMetadata,
  initialMameCatalogState,
  machineAvailabilityNotice,
  metadataExecutableRequest,
  type MameCatalogState,
} from "./mameCatalogState";

const availableMame: MameVersionReport = {
  status: "available",
  identity: {
    source: "external",
    trust: "userConfigured",
    path: "/opt/mame/mame",
    version: "0.280",
    build: null,
    rawVersionLine: "0.280",
  },
};

function metadataStatus(freshness: MetadataStatus["freshness"]): MetadataStatus {
  return {
    schemaVersion: 1,
    freshness,
    currentExecutable: availableMame.status === "available" ? availableMame.identity : neverValue(),
    activeGeneration:
      freshness === "empty"
        ? null
        : {
            generationId: 7,
            sourceKind: "external",
            trust: "userConfigured",
            executablePath: "/opt/mame/mame",
            mameVersion: "0.280",
            mameBuild: null,
            rawVersionLine: "0.280",
            listxmlBuild: null,
            mameConfig: null,
            generatedAtEpochMs: 1,
            importedAtEpochMs: 2,
            machineCount: 42,
          },
  };
}

function neverValue(): never {
  throw new Error("unreachable");
}

function renderState(state: Exclude<MameCatalogState, { status: "ready" }>): string {
  return renderToStaticMarkup(
    <MameCatalogStatePanel
      state={state}
      onConfigureOptions={vi.fn()}
      onImportMetadata={vi.fn()}
    />,
  );
}

function pageWithAvailability(values: Array<"available" | "missing" | "unknown">): MachinePage {
  const items = values.map((_, index) => ({
    shortName: `machine_${index}`,
    description: `Machine ${index}`,
    year: null,
    manufacturer: null,
    sourceFile: null,
    cloneOf: null,
    runnable: true,
    isDevice: false,
    driverStatus: "good",
    displayCount: 1,
    softwareListCount: 0,
  }));
  return {
    schemaVersion: 1,
    generationId: 7,
    total: items.length,
    offset: 0,
    limit: 100,
    items,
    availabilityByShortName: Object.fromEntries(
      items.map((item, index) => [item.shortName, values[index] ?? "unknown"]),
    ),
  };
}

describe("MAME startup and metadata parity state", () => {
  it("classifies executable readiness without falling through to a generic zero-machine state", () => {
    expect(initialMameCatalogState({ status: "notConfigured" })).toEqual({
      status: "notConfigured",
    });
    expect(initialMameCatalogState(availableMame)).toEqual({ status: "checking" });
    expect(metadataExecutableRequest(availableMame)).toEqual({
      source: "external",
      path: "/opt/mame/mame",
    });
  });

  it("distinguishes empty, stale, and fresh metadata generations", () => {
    expect(catalogStateFromMetadata(metadataStatus("empty"))).toMatchObject({
      status: "importNeeded",
      freshness: "empty",
    });
    expect(catalogStateFromMetadata(metadataStatus("stale"))).toMatchObject({
      status: "importNeeded",
      freshness: "stale",
      previousMachineCount: 42,
    });
    expect(catalogStateFromMetadata(metadataStatus("fresh"))).toEqual({
      status: "ready",
      machineCount: 42,
    });
  });

  it("only permits catalog queries once metadata is fresh", () => {
    expect(catalogCanQuery({ status: "checking" })).toBe(false);
    expect(catalogCanQuery({ status: "importing" })).toBe(false);
    expect(catalogCanQuery({ status: "ready", machineCount: 42 })).toBe(true);
    expect(catalogRangeLabel({ status: "importing" })).toBe("Importing metadata…");
    expect(catalogRangeLabel({ status: "ready", machineCount: 42 })).toBe("Metadata loaded");
  });

  it("renders original-shell configuration, import, progress, and failure states", () => {
    expect(renderState({ status: "notConfigured" })).toContain("MAME is not configured");
    expect(renderState({ status: "notConfigured" })).toContain("Configure Options");

    const needed = renderState({
      status: "importNeeded",
      freshness: "empty",
      previousMachineCount: null,
    });
    expect(needed).toContain("MAME metadata needs import");
    expect(needed).toContain("Import Metadata");
    expect(needed).toContain("Configure Options");

    expect(renderState({ status: "importing" })).toContain("Importing MAME metadata");

    const failed = renderState({ status: "importFailed", message: "listxml failed" });
    expect(failed).toContain("MAME metadata import failed");
    expect(failed).toContain("Retry Metadata Import");
    expect(failed).toContain("listxml failed");
  });

  it("distinguishes unknown availability, no available ROMs, and populated availability", () => {
    expect(machineAvailabilityNotice(pageWithAvailability(["unknown", "unknown"]))).toBe(
      "unknown",
    );
    expect(machineAvailabilityNotice(pageWithAvailability(["missing", "missing"]))).toBe(
      "noneAvailable",
    );
    expect(machineAvailabilityNotice(pageWithAvailability(["available", "missing"]))).toBeNull();
  });
});
