import { describe, expect, it } from "vitest";

import type { SessionSnapshot } from "../backend/types";
import type { StoredSaveStateRecord } from "../backend/saveStateRecords";
import { saveStateCompatibility } from "./saveStateCompatibility";

const record: StoredSaveStateRecord = {
  id: 7,
  filePresent: true,
  record: {
    schemaVersion: 1,
    machine: "pacman",
    software: null,
    mame: {
      version: "0.281",
      build: "mame0281",
      rawVersionLine: "MAME v0.281",
    },
    savedAtEpochMs: 1,
    slot: "quick",
    path: "/application-owned/quick.sta",
    bytes: 4096,
    screenshot: null,
  },
};

const session: SessionSnapshot = {
  schemaVersion: 1,
  sessionId: "session-1",
  state: "running",
  machine: "pacman",
  software: null,
  pid: 123,
};

describe("saveStateCompatibility", () => {
  it("never labels a matching record as proven compatible", () => {
    const result = saveStateCompatibility(record, session);
    expect(result.loadable).toBe(true);
    expect(result.message).toContain("validated at load time");
  });

  it("blocks missing files and context mismatches", () => {
    expect(
      saveStateCompatibility({ ...record, filePresent: false }, session).loadable,
    ).toBe(false);
    expect(
      saveStateCompatibility(record, { ...session, machine: "galaga" }).loadable,
    ).toBe(false);
  });

  it("warns about an observed different MAME build but still defers to the probe", () => {
    const current = {
      ...session,
      executable: {
        rawVersionLine: "MAME v0.282",
        version: "0.282",
        build: "mame0282",
      },
    } as SessionSnapshot;
    const result = saveStateCompatibility(record, current);
    expect(result.loadable).toBe(true);
    expect(result.tone).toBe("warning");
    expect(result.message).toContain("structural compatibility probe");
  });
});
