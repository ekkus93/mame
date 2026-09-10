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

export type BulkAuditRunState =
  "idle" | "running" | "cancelling" | "completed" | "cancelled" | "failed";

export type BulkAuditStatus = {
  schemaVersion: 1;
  jobId: number | null;
  state: BulkAuditRunState;
  maxParallelism: number;
  total: number;
  completed: number;
  failed: number;
  active: number;
  lastMachine: string | null;
  lastError: string | null;
};

export type StartBulkAuditRequest = {
  maxParallelism: number;
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

export async function getLibraryBulkAuditStatus(): Promise<BulkAuditStatus> {
  return invoke<BulkAuditStatus>("get_library_bulk_audit_status");
}

export async function startLibraryBulkAudit(
  request: StartBulkAuditRequest,
): Promise<BulkAuditStatus> {
  return invoke<BulkAuditStatus>("start_library_bulk_audit", { request });
}

export async function cancelLibraryBulkAudit(): Promise<BulkAuditStatus> {
  return invoke<BulkAuditStatus>("cancel_library_bulk_audit");
}
