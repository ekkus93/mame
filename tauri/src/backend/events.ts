import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { AppReadyEventV1 } from "./types";

export const APP_READY_EVENT = "app:ready" as const;
export const SESSION_STARTED_EVENT = "session:started" as const;
export const SESSION_EXITED_EVENT = "session:exited" as const;
export const SESSION_CRASHED_EVENT = "session:crashed" as const;
export const SESSION_FAILED_EVENT = "session:failed" as const;
export const SESSION_PAUSED_EVENT = "session:paused" as const;
export const SESSION_RESUMED_EVENT = "session:resumed" as const;
export const SESSION_STATE_SAVED_EVENT = "session:state-saved" as const;
export const SESSION_STATE_SAVE_FAILED_EVENT = "session:state-save-failed" as const;
export const SESSION_STATE_LOADED_EVENT = "session:state-loaded" as const;
export const SESSION_STATE_LOAD_FAILED_EVENT = "session:state-load-failed" as const;

export const TAURI_EVENT_NAMES = [
  APP_READY_EVENT,
  SESSION_STARTED_EVENT,
  SESSION_EXITED_EVENT,
  SESSION_CRASHED_EVENT,
  SESSION_FAILED_EVENT,
  SESSION_PAUSED_EVENT,
  SESSION_RESUMED_EVENT,
  SESSION_STATE_SAVED_EVENT,
  SESSION_STATE_SAVE_FAILED_EVENT,
  SESSION_STATE_LOADED_EVENT,
  SESSION_STATE_LOAD_FAILED_EVENT,
] as const;

export function onAppReady(handler: (payload: AppReadyEventV1) => void): Promise<UnlistenFn> {
  return listen<AppReadyEventV1>(APP_READY_EVENT, (event) => {
    handler(event.payload);
  });
}
