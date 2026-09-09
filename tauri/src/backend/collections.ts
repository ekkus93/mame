import { invoke } from "@tauri-apps/api/core";

import type { MachineListItem } from "./types";

export type CollectionSummary = {
  id: number;
  name: string;
  createdAtEpochMs: number;
  updatedAtEpochMs: number;
  memberCount: number;
};

export type CollectionListPage = {
  schemaVersion: 1;
  total: number;
  offset: number;
  limit: number;
  items: CollectionSummary[];
};

export type CollectionMemberEntry = {
  shortName: string;
  addedAtEpochMs: number;
  machine: MachineListItem | null;
};

export type CollectionMemberPage = {
  schemaVersion: 1;
  collection: CollectionSummary;
  total: number;
  offset: number;
  limit: number;
  items: CollectionMemberEntry[];
};

export type CollectionMembershipState = {
  schemaVersion: 1;
  collectionId: number;
  shortName: string;
  member: boolean;
  addedAtEpochMs: number | null;
};

export type DeleteCollectionResult = {
  schemaVersion: 1;
  collectionId: number;
  deleted: boolean;
};

export function createLibraryCollection(name: string): Promise<CollectionSummary> {
  return invoke<CollectionSummary>("create_library_collection", { request: { name } });
}

export function renameLibraryCollection(
  collectionId: number,
  name: string,
): Promise<CollectionSummary> {
  return invoke<CollectionSummary>("rename_library_collection", {
    request: { collectionId, name },
  });
}

export function deleteLibraryCollection(collectionId: number): Promise<DeleteCollectionResult> {
  return invoke<DeleteCollectionResult>("delete_library_collection", {
    request: { collectionId },
  });
}

export function queryLibraryCollections(limit = 100, offset = 0): Promise<CollectionListPage> {
  return invoke<CollectionListPage>("query_library_collections", {
    request: { limit, offset },
  });
}

export function queryLibraryCollectionMembers(
  collectionId: number,
  limit = 100,
  offset = 0,
): Promise<CollectionMemberPage> {
  return invoke<CollectionMemberPage>("query_library_collection_members", {
    request: { collectionId, limit, offset },
  });
}

export function setLibraryCollectionMachine(
  collectionId: number,
  shortName: string,
  member: boolean,
): Promise<CollectionMembershipState> {
  return invoke<CollectionMembershipState>("set_library_collection_machine", {
    request: { collectionId, shortName, member },
  });
}
