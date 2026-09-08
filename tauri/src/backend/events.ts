import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { AppReadyEventV1 } from "./types";

export const APP_READY_EVENT = "app.ready" as const;

export function onAppReady(handler: (payload: AppReadyEventV1) => void): Promise<UnlistenFn> {
  return listen<AppReadyEventV1>(APP_READY_EVENT, (event) => {
    handler(event.payload);
  });
}
