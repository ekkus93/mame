import { describe, expect, it } from "vitest";

import type { MameVersionReport } from "../backend/types";
import { metadataExecutableRequest } from "./mameCatalogState";

describe("metadataExecutableRequest", () => {
  it("requests backend-owned bundled resolution for a bundled runtime", () => {
    const report: MameVersionReport = {
      status: "available",
      identity: {
        source: "bundled",
        trust: "qualifiedBundled",
        path: "/package/resources/mame-runtime/bin/mame",
        version: "0.288",
        build: null,
        rawVersionLine: "0.288",
      },
    };

    expect(metadataExecutableRequest(report)).toEqual({ source: "bundled", path: "" });
  });

  it("continues to pass external overrides by path", () => {
    const report: MameVersionReport = {
      status: "available",
      identity: {
        source: "external",
        trust: "userConfigured",
        path: "/opt/mame/mame",
        version: "0.288",
        build: null,
        rawVersionLine: "0.288",
      },
    };

    expect(metadataExecutableRequest(report)).toEqual({ source: "external", path: "/opt/mame/mame" });
  });
});
