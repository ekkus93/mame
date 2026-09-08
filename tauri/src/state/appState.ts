import type { AppInfoResponse } from "../backend/types";

export type AppState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; info: AppInfoResponse }
  | { status: "error"; message: string };

export type AppAction =
  { type: "load" } | { type: "ready"; info: AppInfoResponse } | { type: "error"; message: string };

export const initialAppState: AppState = { status: "idle" };

export function appStateReducer(_state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "load":
      return { status: "loading" };
    case "ready":
      return { status: "ready", info: action.info };
    case "error":
      return { status: "error", message: action.message };
  }
}
