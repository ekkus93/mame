import { describe, expect, it } from "vitest";

import {
  beginMetadataImport,
  bootstrapFromBackend,
  catalogIsReady,
  failMetadataImport,
} from "./bootstrapModel";

const configuredMame = {
  version: "0.288",
  build: "test",
  rawVersionLine: "0.288 test",
};

const generation = {
  generationId: 7,
  mameVersion: "0.288",
  importedAtEpochMs: 1234,
  machineCount: 38190,
};

describe("MAME browser bootstrap model", () => {
  it("keeps configuration and metadata readiness states distinct", () => {
    expect(bootstrapFromBackend({ status: "notConfigured" }).status).toBe("notConfigured");
    expect(
      bootstrapFromBackend({
        status: "executableUnavailable",
        errorCode: "MAME_EXECUTABLE_NOT_FOUND",
        errorMessage: "The configured MAME executable path is not usable.",
      }).status,
    ).toBe("executableUnavailable");
    expect(bootstrapFromBackend({ status: "metadataMissing", mame: configuredMame }).status).toBe(
      "metadataMissing",
    );
    expect(
      bootstrapFromBackend({
        status: "metadataStale",
        mame: configuredMame,
        activeGeneration: generation,
      }).status,
    ).toBe("metadataStale");
    expect(
      catalogIsReady(
        bootstrapFromBackend({ status: "ready", mame: configuredMame, generation }),
      ),
    ).toBe(true);
  });

  it("models import progress, failure and retry without pretending the catalog is ready", () => {
    const missing = bootstrapFromBackend({ status: "metadataMissing", mame: configuredMame });
    const importing = beginMetadataImport(missing);
    expect(importing.status).toBe("importing");
    expect(catalogIsReady(importing)).toBe(false);

    const failed = failMetadataImport(importing, "MAME metadata generation failed.");
    expect(failed).toMatchObject({
      status: "importFailed",
      message: "MAME metadata generation failed.",
    });
    expect(beginMetadataImport(failed).status).toBe("importing");
  });

  it("does not start an import from a ready or unconfigured state", () => {
    const ready = bootstrapFromBackend({ status: "ready", mame: configuredMame, generation });
    expect(beginMetadataImport(ready)).toEqual(ready);
    const notConfigured = bootstrapFromBackend({ status: "notConfigured" });
    expect(beginMetadataImport(notConfigured)).toEqual(notConfigured);
  });
});
