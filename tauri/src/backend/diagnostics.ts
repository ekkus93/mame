import { invoke } from "@tauri-apps/api/core";

import type { AppInfoResponse } from "./types";

export type DiagnosticLogEntry = {
  timestampEpochMs: number;
  level: string;
  category: string;
  message: string;
  context: unknown;
};

export type CatalogSchemaDiagnostics = {
  supportedVersion: number;
  currentVersion: number | null;
  migrationState:
    | "notInitialized"
    | "current"
    | "migrationRequired"
    | "newerThanSupported"
    | "invalid"
    | "unavailable";
  errorCode: string | null;
};

export type DiagnosticsSnapshot = {
  schemaVersion: 2;
  app: AppInfoResponse;
  platform: string;
  architecture: string;
  settingsPath: string;
  catalogPath: string;
  catalogSchema: CatalogSchemaDiagnostics;
  logPath: string;
  recentLogs: DiagnosticLogEntry[];
};

export type DiagnosticsExportResult = {
  schemaVersion: 1;
  path: string;
};

export async function getDiagnostics(): Promise<DiagnosticsSnapshot> {
  return invoke<DiagnosticsSnapshot>("get_diagnostics");
}

export async function exportDiagnosticsBundle(): Promise<DiagnosticsExportResult> {
  return invoke<DiagnosticsExportResult>("export_diagnostics_bundle");
}
