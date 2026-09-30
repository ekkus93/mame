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

export type ContentPathDiagnostics = {
  schemaVersion: 1;
  resolutionPolicy: "configuredOnly";
  total: number;
  rom: number;
  software: number;
  chd: number;
  accessible: number;
  missing: number;
  notDirectory: number;
  permissionDenied: number;
  unreadable: number;
};

export type DiagnosticsSnapshot = {
  schemaVersion: 3;
  app: AppInfoResponse;
  platform: string;
  architecture: string;
  settingsPath: string;
  catalogPath: string;
  catalogSchema: CatalogSchemaDiagnostics;
  contentPaths: ContentPathDiagnostics;
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
