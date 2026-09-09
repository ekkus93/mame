import { invoke } from "@tauri-apps/api/core";

import type { MachineListItem } from "./types";

export type RecentHistoryEntry = {
  id: number;
  machineShortName: string;
  softwareItem: string | null;
  launchedAtEpochMs: number;
  succeeded: boolean | null;
  machine: MachineListItem | null;
};

export type RecentHistoryPage = {
  schemaVersion: 1;
  total: number;
  offset: number;
  limit: number;
  retentionLimit: number;
  items: RecentHistoryEntry[];
};

export function queryLibraryHistory(limit = 50, offset = 0): Promise<RecentHistoryPage> {
  return invoke<RecentHistoryPage>("query_library_history", {
    request: { limit, offset },
  });
}
