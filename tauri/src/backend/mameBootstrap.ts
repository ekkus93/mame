import { invoke } from "@tauri-apps/api/core";

import type { MetadataRefreshResult } from "./types";

export type MameUiConfiguredMame = {
  version: string;
  build: string | null;
  rawVersionLine: string;
};

export type MameUiGenerationSummary = {
  generationId: number;
  mameVersion: string;
  importedAtEpochMs: number;
  machineCount: number;
};

export type MameUiBootstrapStatus =
  | { status: "notConfigured" }
  | { status: "executableUnavailable"; errorCode: string; errorMessage: string }
  | { status: "metadataMissing"; mame: MameUiConfiguredMame }
  | {
      status: "metadataStale";
      mame: MameUiConfiguredMame;
      activeGeneration: MameUiGenerationSummary;
    }
  | { status: "ready"; mame: MameUiConfiguredMame; generation: MameUiGenerationSummary };

export async function getMameUiBootstrapStatus(): Promise<MameUiBootstrapStatus> {
  return invoke<MameUiBootstrapStatus>("get_mame_ui_bootstrap_status");
}

export async function refreshConfiguredMameMetadata(): Promise<MetadataRefreshResult> {
  return invoke<MetadataRefreshResult>("refresh_configured_mame_metadata");
}
