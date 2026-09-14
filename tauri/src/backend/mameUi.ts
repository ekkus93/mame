import { invoke } from "@tauri-apps/api/core";

import type { MachinePage } from "./types";

export type MameUiMachineFilter =
  | "all"
  | "available"
  | "unavailable"
  | "working"
  | "notWorking"
  | "mechanical"
  | "notMechanical"
  | "favorites"
  | "bios"
  | "notBios"
  | "parents"
  | "clones"
  | "manufacturer"
  | "year"
  | "sourceFile"
  | "saveSupported"
  | "saveUnsupported"
  | "verticalScreen"
  | "horizontalScreen";

export type MameUiMachineSearchRequest = {
  text?: string | null;
  filter: MameUiMachineFilter;
  filterValue?: string | null;
  preferredMachine?: string | null;
  limit?: number;
  offset?: number;
};

export async function queryMameUiLibrary(
  request: MameUiMachineSearchRequest,
): Promise<MachinePage> {
  return invoke<MachinePage>("query_mame_ui_library", { request });
}
