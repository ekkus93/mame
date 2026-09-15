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
  | "chdRequired"
  | "noChdRequired"
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

export type ExportMameUiDisplayedListRequest = {
  text?: string | null;
  filter: MameUiMachineFilter;
  filterValue?: string | null;
};

export type ExportMameUiDisplayedListResult = {
  schemaVersion: number;
  canceled: boolean;
  rows: number;
  path: string | null;
};

export async function queryMameUiLibrary(
  request: MameUiMachineSearchRequest,
): Promise<MachinePage> {
  return invoke<MachinePage>("query_mame_ui_library", { request });
}

export async function exportMameUiDisplayedList(
  request: ExportMameUiDisplayedListRequest,
): Promise<ExportMameUiDisplayedListResult> {
  return invoke<ExportMameUiDisplayedListResult>("export_mame_ui_displayed_list", { request });
}
