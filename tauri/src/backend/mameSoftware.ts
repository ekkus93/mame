import { invoke } from "@tauri-apps/api/core";

import type { LaunchPreferences } from "./generalSettings";
import type { SessionSnapshot } from "./types";

export type SoftwareListFilter =
  | "all"
  | "parents"
  | "clones"
  | "year"
  | "publisher"
  | "supported"
  | "partiallySupported"
  | "unsupported";

export type SoftwarePartSummary = {
  name: string;
  interface: string;
};

export type MameSoftwareItem = {
  shortName: string;
  description: string;
  year: string;
  publisher: string;
  cloneOf: string | null;
  supported: "yes" | "partial" | "no";
  parts: SoftwarePartSummary[];
};

export type MameSoftwarePage = {
  schemaVersion: 1;
  machineShortName: string;
  softwareListName: string;
  softwareListDescription: string | null;
  total: number;
  offset: number;
  limit: number;
  items: MameSoftwareItem[];
};

export type MameSoftwareQuery = {
  shortName: string;
  softwareList: string;
  text?: string | null;
  filter?: SoftwareListFilter;
  filterValue?: string | null;
  limit?: number;
  offset?: number;
};

export type BiosChoice = {
  name: string;
  description: string;
  isDefault: boolean;
};

export type BiosChoicesResponse = {
  schemaVersion: 1;
  machineShortName: string;
  choices: BiosChoice[];
};

export type MameSoftwareLaunch = {
  shortName: string;
  softwareList: string;
  softwareItem: string;
  softwarePart?: string | null;
  bios?: string | null;
  launchOverrides?: LaunchPreferences | null;
};

export async function queryMameSoftware(request: MameSoftwareQuery): Promise<MameSoftwarePage> {
  return invoke<MameSoftwarePage>("query_mame_software_list", { request });
}

export async function queryMameBiosChoices(shortName: string): Promise<BiosChoicesResponse> {
  return invoke<BiosChoicesResponse>("query_mame_bios_choices", {
    request: { shortName },
  });
}

export async function launchMameSoftware(request: MameSoftwareLaunch): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("launch_library_software", { request });
}
