import { type FormEvent, useCallback, useEffect, useState } from "react";

import {
  createLibraryCollection,
  deleteLibraryCollection,
  queryLibraryCollectionMembers,
  queryLibraryCollections,
  renameLibraryCollection,
  setLibraryCollectionMachine,
  type CollectionListPage,
  type CollectionMemberPage,
  type CollectionSummary,
} from "../backend/collections";
import { errorMessage } from "../backend/errors";
import "./collections.css";

type CollectionsState =
  | { status: "loading" }
  | { status: "ready"; page: CollectionListPage }
  | { status: "error"; message: string };

type MembersState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; page: CollectionMemberPage }
  | { status: "error"; message: string };

export function CollectionManager() {
  const [collections, setCollections] = useState<CollectionsState>({ status: "loading" });
  const [selected, setSelected] = useState<CollectionSummary | null>(null);
  const [members, setMembers] = useState<MembersState>({ status: "idle" });
  const [newName, setNewName] = useState("");
  const [renameName, setRenameName] = useState("");
  const [machineShortName, setMachineShortName] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const loadCollections = useCallback(async () => {
    setCollections({ status: "loading" });
    try {
      const page = await queryLibraryCollections();
      setCollections({ status: "ready", page });
      setSelected((current) => {
        if (current) {
          return page.items.find((item) => item.id === current.id) ?? page.items[0] ?? null;
        }
        return page.items[0] ?? null;
      });
    } catch (error: unknown) {
      setCollections({ status: "error", message: errorMessage(error) });
    }
  }, []);

  const loadMembers = useCallback(async (collectionId: number) => {
    setMembers({ status: "loading" });
    try {
      setMembers({
        status: "ready",
        page: await queryLibraryCollectionMembers(collectionId),
      });
    } catch (error: unknown) {
      setMembers({ status: "error", message: errorMessage(error) });
    }
  }, []);

  useEffect(() => {
    void loadCollections();
  }, [loadCollections]);

  useEffect(() => {
    if (!selected) {
      setMembers({ status: "idle" });
      setRenameName("");
      return;
    }
    setRenameName(selected.name);
    void loadMembers(selected.id);
  }, [loadMembers, selected]);

  async function createCollection(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setActionError(null);
    try {
      const created = await createLibraryCollection(newName);
      setNewName("");
      await loadCollections();
      setSelected(created);
    } catch (error: unknown) {
      setActionError(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function renameCollection(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) return;
    setBusy(true);
    setActionError(null);
    try {
      const renamed = await renameLibraryCollection(selected.id, renameName);
      await loadCollections();
      setSelected(renamed);
    } catch (error: unknown) {
      setActionError(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function removeCollection() {
    if (!selected || !window.confirm(`Delete collection “${selected.name}”?`)) return;
    setBusy(true);
    setActionError(null);
    try {
      await deleteLibraryCollection(selected.id);
      setSelected(null);
      setMembers({ status: "idle" });
      await loadCollections();
    } catch (error: unknown) {
      setActionError(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function addMachine(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) return;
    setBusy(true);
    setActionError(null);
    try {
      await setLibraryCollectionMachine(selected.id, machineShortName, true);
      setMachineShortName("");
      await Promise.all([loadCollections(), loadMembers(selected.id)]);
    } catch (error: unknown) {
      setActionError(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  async function removeMachine(shortName: string) {
    if (!selected) return;
    setBusy(true);
    setActionError(null);
    try {
      await setLibraryCollectionMachine(selected.id, shortName, false);
      await Promise.all([loadCollections(), loadMembers(selected.id)]);
    } catch (error: unknown) {
      setActionError(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  const page = collections.status === "ready" ? collections.page : null;

  return (
    <section className="collections-workspace" aria-labelledby="collections-heading">
      <div className="collections-heading-row">
        <div>
          <p className="eyebrow">User library</p>
          <h2 id="collections-heading">Collections</h2>
        </div>
        {page && <span>{page.total.toLocaleString()} collections</span>}
      </div>

      <form className="collection-create" onSubmit={createCollection}>
        <label>
          <span>New collection</span>
          <input
            value={newName}
            onChange={(event) => setNewName(event.target.value)}
            placeholder="Arcade classics"
            maxLength={80}
            disabled={busy}
          />
        </label>
        <button type="submit" disabled={busy || newName.trim().length === 0}>
          Create
        </button>
      </form>

      {actionError && (
        <p className="collection-error" role="alert">
          {actionError}
        </p>
      )}

      <div className="collections-layout">
        <div className="collection-list-panel">
          {collections.status === "loading" && <p>Loading collections…</p>}
          {collections.status === "error" && (
            <div role="alert">
              <p>{collections.message}</p>
              <button type="button" onClick={() => void loadCollections()}>
                Retry
              </button>
            </div>
          )}
          {page && page.items.length === 0 && <p>No collections yet.</p>}
          {page && page.items.length > 0 && (
            <ul className="collection-list">
              {page.items.map((collection) => (
                <li key={collection.id}>
                  <button
                    type="button"
                    className={selected?.id === collection.id ? "collection-selected" : undefined}
                    onClick={() => setSelected(collection)}
                  >
                    <strong>{collection.name}</strong>
                    <span>{collection.memberCount.toLocaleString()} machines</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
          {page && page.total > page.items.length && (
            <p className="collection-note">Showing the first {page.items.length} collections.</p>
          )}
        </div>

        <div className="collection-detail-panel">
          {!selected && <p>Select a collection to manage its machines.</p>}
          {selected && (
            <>
              <div className="collection-detail-heading">
                <div>
                  <p className="eyebrow">Selected collection</p>
                  <h3>{selected.name}</h3>
                </div>
                <button
                  type="button"
                  className="danger-button"
                  onClick={removeCollection}
                  disabled={busy}
                >
                  Delete
                </button>
              </div>

              <form className="collection-rename" onSubmit={renameCollection}>
                <label>
                  <span>Rename</span>
                  <input
                    value={renameName}
                    onChange={(event) => setRenameName(event.target.value)}
                    maxLength={80}
                    disabled={busy}
                  />
                </label>
                <button type="submit" disabled={busy || renameName.trim().length === 0}>
                  Save name
                </button>
              </form>

              <form className="collection-add-machine" onSubmit={addMachine}>
                <label>
                  <span>Add machine by short name</span>
                  <input
                    value={machineShortName}
                    onChange={(event) => setMachineShortName(event.target.value)}
                    placeholder="galaxian"
                    autoComplete="off"
                    disabled={busy}
                  />
                </label>
                <button type="submit" disabled={busy || machineShortName.trim().length === 0}>
                  Add
                </button>
              </form>

              {members.status === "loading" && <p>Loading collection members…</p>}
              {members.status === "error" && <p role="alert">{members.message}</p>}
              {members.status === "ready" && members.page.items.length === 0 && (
                <p>This collection is empty.</p>
              )}
              {members.status === "ready" && members.page.items.length > 0 && (
                <ul className="collection-members">
                  {members.page.items.map((entry) => (
                    <li key={entry.shortName}>
                      <span>
                        <strong>{entry.machine?.description ?? entry.shortName}</strong>
                        <code>{entry.shortName}</code>
                        {!entry.machine && <em>Not present in the active MAME catalog</em>}
                      </span>
                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => void removeMachine(entry.shortName)}
                        disabled={busy}
                      >
                        Remove
                      </button>
                    </li>
                  ))}
                </ul>
              )}
              {members.status === "ready" && members.page.total > members.page.items.length && (
                <p className="collection-note">
                  Showing the first {members.page.items.length} of {members.page.total} members.
                </p>
              )}
            </>
          )}
        </div>
      </div>
    </section>
  );
}
