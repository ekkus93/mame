import { invoke } from "@tauri-apps/api/core";

import {
  APP_PROTOCOL_VERSION,
  type AppInfoRequest,
  type AppInfoResponse,
} from "./types";

export async function getAppInfo(): Promise<AppInfoResponse> {
  const request: AppInfoRequest = {
    protocolVersion: APP_PROTOCOL_VERSION,
  };

  return invoke<AppInfoResponse>("get_app_info", { request });
}
