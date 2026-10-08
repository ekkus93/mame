import { invoke } from "@tauri-apps/api/core";

import type { AppInfoResponse, FrameMetricsSnapshot, SessionSnapshot } from "./types";

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

export type EncodedPlatformPath = {
  encoding: "unixBytesHex" | "windowsWideHex";
  data: string;
};

export type ContentPathDiagnosticsEntry = {
  kind: "rom" | "software" | "chd";
  source: "configured" | "mameDefault";
  validation: {
    path: string | EncodedPlatformPath;
    status: "accessible" | "missing" | "notDirectory" | "permissionDenied" | "unreadable";
    message: string | null;
  };
};

export type ContentPathDiagnostics = {
  schemaVersion: 1;
  resolutionPolicy: "configuredWithConventionalMameDefaults";
  entries: ContentPathDiagnosticsEntry[];
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

export type RuntimeDiagnostics = {
  status: "available" | "unavailable" | "notConfigured";
  activeSource: "bundled" | "external" | "developmentTree" | null;
  trust: "qualifiedBundled" | "userConfigured" | "development" | null;
  failureDomain: string | null;
  errorCode: string | null;
  message: string | null;
};

export type GameplayInputDiagnostics = {
  state: "ready" | "starting" | "stopping" | "ended" | "inactive";
  bridge: "boundedAuthenticatedRuntimeControl";
  maxUpdatesPerBatch: number;
};

export type GameplayDiagnostics = {
  session: SessionSnapshot | null;
  videoTransport: "privateAuthenticatedFifo" | "unsupported";
  video: FrameMetricsSnapshot | null;
  input: GameplayInputDiagnostics;
};

export type DiagnosticsSnapshot = {
  schemaVersion: 3;
  app: AppInfoResponse;
  runtime: RuntimeDiagnostics;
  platform: string;
  architecture: string;
  settingsPath: string;
  catalogPath: string;
  catalogSchema: CatalogSchemaDiagnostics;
  contentPaths: ContentPathDiagnostics;
  gameplay: GameplayDiagnostics;
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
