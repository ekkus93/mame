import { invoke } from "@tauri-apps/api/core";

import type { LoadMameStateResult, SaveMameStateRequest } from "./types";

export type SaveStateMameIdentity = {
  version: string;
  build: string | null;
  rawVersionLine: string;
};

export type SaveStateRecord = {
  schemaVersion: 1;
  machine: string;
  software: string | null;
  mame: SaveStateMameIdentity;
  savedAtEpochMs: number;
  slot: string;
  path: string;
  bytes: number;
  screenshot: { schemaVersion: number; assetId: string } | null;
};

export type StoredSaveStateRecord = {
  id: number;
  record: SaveStateRecord;
  filePresent: boolean;
};

export type SaveStateRecordPage = {
  schemaVersion: 1;
  total: number;
  offset: number;
  limit: number;
  items: StoredSaveStateRecord[];
};

export type ListSaveStateRecordsRequest = {
  limit?: number;
  offset?: number;
};

export type LoadKnownSaveStateRequest = {
  sessionId: string;
  recordId: number;
};

export type DeleteSaveStateRecordRequest = {
  recordId: number;
};

export type DeleteSaveStateRecordResult = {
  schemaVersion: 1;
  recordId: number;
  fileDeleted: boolean;
};

export async function listSaveStateRecords(
  request: ListSaveStateRecordsRequest = {},
): Promise<SaveStateRecordPage> {
  return invoke<SaveStateRecordPage>("list_save_state_records", { request });
}

export async function saveKnownState(
  request: SaveMameStateRequest,
): Promise<StoredSaveStateRecord> {
  return invoke<StoredSaveStateRecord>("save_known_state", { request });
}

export async function loadKnownSaveState(
  request: LoadKnownSaveStateRequest,
): Promise<LoadMameStateResult> {
  return invoke<LoadMameStateResult>("load_known_save_state", { request });
}

export async function deleteSaveStateRecord(
  request: DeleteSaveStateRecordRequest,
): Promise<DeleteSaveStateRecordResult> {
  return invoke<DeleteSaveStateRecordResult>("delete_save_state_record", {
    request,
  });
}
