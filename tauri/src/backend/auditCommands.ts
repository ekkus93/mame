import { invoke } from "@tauri-apps/api/core";

export type MameAuditClassification =
  "complete" | "bestAvailable" | "missingRequired" | "incorrect" | "mixedFailure" | "unknown";

export type MameAuditFacts = {
  optionalMissing: boolean;
  noGoodDumpKnown: boolean;
  needsRedump: boolean;
  missingRequired: boolean;
  incorrectChecksum: boolean;
  incorrectLength: boolean;
  targetNotFound: boolean;
};

export type MameAuditParseResult = {
  classification: MameAuditClassification;
  facts: MameAuditFacts;
  exitCode: number | null;
  rawExcerpt: string;
  rawTruncated: boolean;
};

export type MachineAuditRequest = {
  shortName: string;
};

export type MachineAuditResponse = {
  schemaVersion: 1;
  machineShortName: string;
  result: MameAuditParseResult;
  auditedAtEpochMs: number;
};

export async function getLibraryMachineAudit(
  request: MachineAuditRequest,
): Promise<MachineAuditResponse | null> {
  return invoke<MachineAuditResponse | null>("get_library_machine_audit", { request });
}

export async function runLibraryMachineAudit(
  request: MachineAuditRequest,
): Promise<MachineAuditResponse> {
  return invoke<MachineAuditResponse>("run_library_machine_audit", { request });
}
