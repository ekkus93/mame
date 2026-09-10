import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  cancelLibraryBulkAudit,
  getLibraryBulkAuditStatus,
  getLibraryMachineAudit,
  runLibraryMachineAudit,
  startLibraryBulkAudit,
} from "./auditCommands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("machine audit backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads the current persisted result through the typed command envelope", async () => {
    const request = { shortName: "pacman" };
    vi.mocked(invoke).mockResolvedValue(null);

    await expect(getLibraryMachineAudit(request)).resolves.toBeNull();
    expect(invoke).toHaveBeenCalledWith("get_library_machine_audit", { request });
  });

  it("runs one machine audit through the typed command envelope", async () => {
    const request = { shortName: "pacman" };
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      machineShortName: "pacman",
      auditedAtEpochMs: 1234,
      result: {
        classification: "complete",
        facts: {},
        exitCode: 0,
        rawExcerpt: "stdout:\nromset pacman is good\n",
        rawTruncated: false,
      },
    });

    await expect(runLibraryMachineAudit(request)).resolves.toMatchObject({
      schemaVersion: 1,
      machineShortName: "pacman",
    });
    expect(invoke).toHaveBeenCalledWith("run_library_machine_audit", { request });
  });

  it("loads bulk audit status without an untyped payload", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, state: "idle" });

    await expect(getLibraryBulkAuditStatus()).resolves.toMatchObject({ state: "idle" });
    expect(invoke).toHaveBeenCalledWith("get_library_bulk_audit_status");
  });

  it("starts a bounded bulk audit with explicit parallelism", async () => {
    const request = { maxParallelism: 2 };
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, state: "running" });

    await expect(startLibraryBulkAudit(request)).resolves.toMatchObject({ state: "running" });
    expect(invoke).toHaveBeenCalledWith("start_library_bulk_audit", { request });
  });

  it("cancels the active bulk audit without accepting arbitrary input", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, state: "cancelling" });

    await expect(cancelLibraryBulkAudit()).resolves.toMatchObject({ state: "cancelling" });
    expect(invoke).toHaveBeenCalledWith("cancel_library_bulk_audit");
  });
});
