import { invoke } from "@tauri-apps/api/core";

import {
  APP_PROTOCOL_VERSION,
  type AppInfoRequest,
  type AppInfoResponse,
  type MachinePage,
  type MachineSearchRequest,
  type MetadataRefreshResult,
  type MetadataStatus,
  type MetadataStatusRequest,
  type RefreshMameMetadataRequest,
} from "./types";

export async function getAppInfo(): Promise<AppInfoResponse> {
  const request: AppInfoRequest = {
    protocolVersion: APP_PROTOCOL_VERSION,
  };

  return invoke<AppInfoResponse>("get_app_info", { request });
}

export async function refreshMameMetadata(
  request: RefreshMameMetadataRequest,
): Promise<MetadataRefreshResult> {
  return invoke<MetadataRefreshResult>("refresh_mame_metadata", { request });
}

export async function getMameMetadataStatus(
  request: MetadataStatusRequest,
): Promise<MetadataStatus> {
  return invoke<MetadataStatus>("get_mame_metadata_status", { request });
}

export async function queryMameLibrary(request: MachineSearchRequest): Promise<MachinePage> {
  return invoke<MachinePage>("query_mame_library", { request });
}
