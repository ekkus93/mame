import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  createLibraryCollection,
  deleteLibraryCollection,
  queryLibraryCollectionMembers,
  queryLibraryCollections,
  renameLibraryCollection,
  setLibraryCollectionMachine,
} from "./collections";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("collection backend commands", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it("uses typed create rename and delete envelopes", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1 });

    await createLibraryCollection("Arcade");
    expect(invoke).toHaveBeenLastCalledWith("create_library_collection", {
      request: { name: "Arcade" },
    });

    await renameLibraryCollection(7, "Classics");
    expect(invoke).toHaveBeenLastCalledWith("rename_library_collection", {
      request: { collectionId: 7, name: "Classics" },
    });

    await deleteLibraryCollection(7);
    expect(invoke).toHaveBeenLastCalledWith("delete_library_collection", {
      request: { collectionId: 7 },
    });
  });

  it("keeps collection and member queries bounded", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, items: [] });

    await queryLibraryCollections();
    expect(invoke).toHaveBeenLastCalledWith("query_library_collections", {
      request: { limit: 100, offset: 0 },
    });

    await queryLibraryCollectionMembers(3);
    expect(invoke).toHaveBeenLastCalledWith("query_library_collection_members", {
      request: { collectionId: 3, limit: 100, offset: 0 },
    });
  });

  it("passes explicit membership intent without constructing SQL", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, member: true });

    await setLibraryCollectionMachine(3, "galaxian", true);
    expect(invoke).toHaveBeenLastCalledWith("set_library_collection_machine", {
      request: { collectionId: 3, shortName: "galaxian", member: true },
    });
  });
});
