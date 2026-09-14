import { invoke } from "@tauri-apps/api/core";

import type { ArtworkKind } from "./artwork";
import type { MameUiMachineFilter } from "./mameUi";

export type MameUiPanelMode = "images" | "info";

export type MameUiState = {
  schemaVersion: number;
  lastMachine: string | null;
  filter: MameUiMachineFilter;
  filterValue: string | null;
  rightPanelMode: MameUiPanelMode;
  artworkKind: ArtworkKind;
  softwareRightPanelMode: MameUiPanelMode;
  softwareArtworkKind: ArtworkKind;
};

export async function getMameUiState(): Promise<MameUiState> {
  return invoke<MameUiState>("get_mame_ui_state");
}

export async function setMameUiState(state: MameUiState): Promise<MameUiState> {
  return invoke<MameUiState>("set_mame_ui_state", { state });
}
