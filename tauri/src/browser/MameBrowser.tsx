import {
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";

import type { ArtworkKind } from "../backend/artwork";
import {
  getMameMachineDetail,
  getMameMetadataStatus,
  launchLibraryMachine,
  refreshMameMetadata,
} from "../backend/commands";
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import { exportMameUiDisplayedList, queryMameUiLibrary } from "../backend/mameUi";
import { getMameUiState, setMameUiState, type MameUiPanelMode } from "../backend/mameUiState";
import type {
  MameVersionReport,
  MachineDetail,
  MachineListItem,
  MachinePage,
  SessionSnapshot,
} from "../backend/types";
import { FavoriteToggleButton } from "../library/FavoriteToggleButton";
import { isEditableElement } from "../library/keyboardNavigation";
import { MameCatalogStatePanel } from "./MameCatalogStatePanel";
import { MachineDriverStatus } from "./MachineDriverStatus";
import { MachineFilterPanel } from "./MachineFilterPanel";
import { MachineList } from "./MachineList";
import {
  catalogCanQuery,
  catalogRangeLabel,
  catalogStateFromMetadata,
  initialMameCatalogState,
  machineAvailabilityNotice,
  metadataExecutableRequest,
  type MameCatalogState,
} from "./mameCatalogState";
import {
  EmptyMachineRightPanel,
  MachineRightPanel,
  type MachineRightView,
} from "./MachineRightPanel";
import {
  buildMameBrowserRequest,
  filterRequiresValue,
  nextBrowserIndex,
  reconcileMachineSelection,
  type MameBrowserFilter,
} from "./model";
import { SoftwareBrowser } from "./SoftwareBrowser";
import {
  activationMayCommit,
  detailMayCommit,
  selectMachineIdentity,
  type MachineAsyncIdentity,
} from "./machineAsyncIdentity";

type LoadState =
  | { status: "awaitingFilterValue" }
  | { status: "loading" }
  | { status: "ready"; page: MachinePage }
  | { status: "error"; message: string };

type DetailState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; detail: MachineDetail }
  | { status: "error"; message: string };

type LaunchState =
  | { status: "idle" }
  | { status: "launching" }
  | { status: "launched"; session: SessionSnapshot }
  | { status: "error"; message: string };

type ExportState =
  | { status: "idle" }
  | { status: "exporting" }
  | { status: "success"; message: string }
  | { status: "error"; message: string };

type SoftwareCapableMachineDetail = MachineDetail & { canStartEmpty: boolean };
type EmptyRightPanelStatus = "idle" | "loading" | "error";

const MAME_MACHINE_ACTION_LABELS = {
  start: "Start",
  startEmpty: "Start Empty",
  configureMachine: "Configure Machine",
  softwareList: "Software List",
  audit: "Audit",
} as const;

function emptyRightPanelStatusFor(detailState: DetailState): EmptyRightPanelStatus {
  switch (detailState.status) {
    case "loading":
      return "loading";
    case "error":
      return "error";
    case "idle":
    case "ready":
      return "idle";
  }
}

function primaryLaunchButtonLabel(detail: MachineDetail, launchState: LaunchState): string {
  if (launchState.status === "launching") return "Starting…";
  if (detail.canStartEmpty && detail.softwareLists.length > 0) {
    return MAME_MACHINE_ACTION_LABELS.startEmpty;
  }
  return MAME_MACHINE_ACTION_LABELS.start;
}

function primaryLaunchButtonAriaLabel(detail: MachineDetail): string {
  if (detail.canStartEmpty && detail.softwareLists.length > 0) {
    return `Start Empty ${detail.description}`;
  }
  return `Start ${detail.description}`;
}

export function MameBrowser({
  availabilityRevision,
  gameplayInputOwned,
  mame,
  onAuditResultsChanged,
  onConfigureOptions,
  onOpenAudit,
  onOpenCollections,
  onOpenDiagnostics,
  onOpenHistory,
  onOpenSession,
  sessionLabel,
  onSessionStarted,
}: {
  availabilityRevision: number;
  gameplayInputOwned: boolean;
  mame: MameVersionReport;
  onAuditResultsChanged: () => void;
  onConfigureOptions: () => void;
  onOpenAudit: () => void;
  onOpenCollections: () => void;
  onOpenDiagnostics: () => void;
  onOpenHistory: () => void;
  onOpenSession: () => void;
  sessionLabel: string;
  onSessionStarted: (session: SessionSnapshot) => void;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const machineRowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const activeFilterButtonRef = useRef<HTMLButtonElement | null>(null);
  const rightPanelFirstTabRef = useRef<HTMLButtonElement | null>(null);
  const configureButtonRef = useRef<HTMLButtonElement | null>(null);
  const catalogSequence = useRef(0);
  const querySequence = useRef(0);
  const detailSequence = useRef(0);
  const activationSequence = useRef(0);
  const asyncIdentityRef = useRef<MachineAsyncIdentity>({
    shortName: null,
    detailGeneration: 0,
    activationGeneration: 0,
  });
  const selectedRef = useRef<MachineListItem | null>(null);
  const [uiStateHydrated, setUiStateHydrated] = useState(false);
  const [uiStateError, setUiStateError] = useState<string | null>(null);
  const [filter, setFilter] = useState<MameBrowserFilter>("all");
  const [filterValue, setFilterValue] = useState("");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [debouncedFilterValue, setDebouncedFilterValue] = useState("");
  const [offset, setOffset] = useState(0);
  const [preferredMachine, setPreferredMachine] = useState<string | null>(null);
  const [rememberedMachine, setRememberedMachine] = useState<string | null>(null);
  const [catalogState, setCatalogState] = useState<MameCatalogState>(() =>
    initialMameCatalogState(mame),
  );
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });
  const [selected, setSelected] = useState<MachineListItem | null>(null);
  const [detailState, setDetailState] = useState<DetailState>({ status: "idle" });
  const [rightView, setRightView] = useState<MachineRightView>("images");
  const [primaryRightView, setPrimaryRightView] = useState<MameUiPanelMode>("images");
  const [artworkKind, setArtworkKind] = useState<ArtworkKind>("screenshot");
  const [softwareRightPanelMode, setSoftwareRightPanelMode] = useState<MameUiPanelMode>("images");
  const [softwareArtworkKind, setSoftwareArtworkKind] = useState<ArtworkKind>("screenshot");
  const [softwareMode, setSoftwareMode] = useState(false);
  const [showNarrowDetails, setShowNarrowDetails] = useState(false);
  const [pendingLaunchOverrides, setPendingLaunchOverrides] = useState<LaunchPreferences | null>(
    null,
  );
  const [launchState, setLaunchState] = useState<LaunchState>({ status: "idle" });
  const [exportState, setExportState] = useState<ExportState>({ status: "idle" });
  const [favoriteRevision, bumpFavoriteRevision] = useReducer((value: number) => value + 1, 0);

  const selectMachine = useCallback((next: MachineListItem | null) => {
    const previousShortName = selectedRef.current?.shortName ?? null;
    const nextShortName = next?.shortName ?? null;
    selectedRef.current = next;
    if (previousShortName !== nextShortName) {
      asyncIdentityRef.current = selectMachineIdentity(asyncIdentityRef.current, nextShortName);
      detailSequence.current = asyncIdentityRef.current.detailGeneration;
      activationSequence.current = asyncIdentityRef.current.activationGeneration;
      setPendingLaunchOverrides(null);
      setLaunchState({ status: "idle" });
    }
    setSelected(next);
  }, []);

  useEffect(() => {
    let cancelled = false;
    void getMameUiState()
      .then((saved) => {
        if (cancelled) return;
        setFilter(saved.filter);
        setFilterValue(saved.filterValue ?? "");
        setPreferredMachine(saved.lastMachine);
        setRememberedMachine(saved.lastMachine);
        setRightView(saved.rightPanelMode);
        setPrimaryRightView(saved.rightPanelMode);
        setArtworkKind(saved.artworkKind);
        setSoftwareRightPanelMode(saved.softwareRightPanelMode);
        setSoftwareArtworkKind(saved.softwareArtworkKind);
      })
      .catch((reason: unknown) => {
        if (!cancelled) setUiStateError(`Saved browser state unavailable: ${errorMessage(reason)}`);
      })
      .finally(() => {
        if (!cancelled) setUiStateHydrated(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const sequence = ++catalogSequence.current;
    const initial = initialMameCatalogState(mame);
    setCatalogState(initial);
    setLoadState({ status: "loading" });
    selectMachine(null);

    if (initial.status !== "checking") return;

    const executable = metadataExecutableRequest(mame);
    if (!executable) {
      setCatalogState({
        status: "executableUnavailable",
        message: "The active MAME executable cannot be used to inspect metadata.",
      });
      return;
    }

    void getMameMetadataStatus({ executable })
      .then((status) => {
        if (catalogSequence.current === sequence) {
          setCatalogState(catalogStateFromMetadata(status));
        }
      })
      .catch((reason: unknown) => {
        if (catalogSequence.current === sequence) {
          setCatalogState({
            status: "importFailed",
            message: `Metadata status unavailable: ${errorMessage(reason)}`,
          });
        }
      });
  }, [mame, selectMachine]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedFilterValue(filterValue.trim()), 160);
    return () => window.clearTimeout(timer);
  }, [filterValue]);

  useEffect(() => setOffset(0), [filter, debouncedSearch, debouncedFilterValue]);

  useEffect(() => {
    if (selected) setRememberedMachine(selected.shortName);
  }, [selected]);

  useEffect(() => {
    setSoftwareMode(false);
    setShowNarrowDetails(false);
  }, [selected?.shortName]);

  useEffect(() => {
    if (!uiStateHydrated) return;
    const timer = window.setTimeout(() => {
      void setMameUiState({
        schemaVersion: 1,
        lastMachine: rememberedMachine,
        filter,
        filterValue: filterValue.trim() || null,
        rightPanelMode: primaryRightView,
        artworkKind,
        softwareRightPanelMode,
        softwareArtworkKind,
      })
        .then(() => setUiStateError(null))
        .catch((reason: unknown) => {
          setUiStateError(`Browser state could not be saved: ${errorMessage(reason)}`);
        });
    }, 300);
    return () => window.clearTimeout(timer);
  }, [
    artworkKind,
    filter,
    filterValue,
    primaryRightView,
    rememberedMachine,
    softwareArtworkKind,
    softwareRightPanelMode,
    uiStateHydrated,
  ]);

  const importMetadata = useCallback(() => {
    if (catalogState.status === "importing") return;
    const executable = metadataExecutableRequest(mame);
    if (!executable) {
      setCatalogState({
        status: "importFailed",
        message: "The active MAME executable cannot be used to refresh metadata.",
      });
      return;
    }

    const sequence = ++catalogSequence.current;
    setCatalogState({ status: "importing" });
    setLoadState({ status: "loading" });
    selectMachine(null);

    void refreshMameMetadata({ executable })
      .then((result) => {
        if (catalogSequence.current !== sequence) return;
        setCatalogState({
          status: "ready",
          machineCount: result.generation.machineCount,
        });
        setOffset(0);
        setPreferredMachine(rememberedMachine);
      })
      .catch((reason: unknown) => {
        if (catalogSequence.current === sequence) {
          setCatalogState({
            status: "importFailed",
            message: errorMessage(reason),
          });
        }
      });
  }, [catalogState.status, mame, rememberedMachine, selectMachine]);

  const valueRequired = filterRequiresValue(filter);
  const request = useMemo(
    () =>
      buildMameBrowserRequest(
        filter,
        debouncedSearch,
        debouncedFilterValue,
        offset,
        offset === 0 ? preferredMachine : null,
      ),
    [filter, debouncedSearch, debouncedFilterValue, offset, preferredMachine],
  );

  useEffect(() => {
    if (!uiStateHydrated) return;
    if (!catalogCanQuery(catalogState)) {
      querySequence.current += 1;
      selectMachine(null);
      return;
    }
    const sequence = ++querySequence.current;
    if (valueRequired && !debouncedFilterValue) {
      setLoadState({ status: "awaitingFilterValue" });
      selectMachine(null);
      return;
    }

    setLoadState({ status: "loading" });
    void queryMameUiLibrary(request)
      .then((page) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "ready", page });
        if (page.offset !== offset) setOffset(page.offset);
        selectMachine(reconcileMachineSelection(page.items, selectedRef.current, preferredMachine));
        if (preferredMachine) setPreferredMachine(null);
      })
      .catch((reason: unknown) => {
        if (querySequence.current !== sequence) return;
        setLoadState({ status: "error", message: errorMessage(reason) });
        selectMachine(null);
      });
  }, [
    availabilityRevision,
    catalogState,
    debouncedFilterValue,
    offset,
    preferredMachine,
    request,
    uiStateHydrated,
    valueRequired,
    selectMachine,
  ]);

  useEffect(() => {
    const sequence = detailSequence.current;
    if (!selected) {
      setDetailState({ status: "idle" });
      return;
    }
    const shortName = selected.shortName;
    setDetailState({ status: "loading" });
    void getMameMachineDetail({ shortName })
      .then((detail) => {
        if (detailMayCommit(asyncIdentityRef.current, sequence, shortName)) {
          setDetailState({ status: "ready", detail });
        }
      })
      .catch((reason: unknown) => {
        if (detailMayCommit(asyncIdentityRef.current, sequence, shortName)) {
          setDetailState({ status: "error", message: errorMessage(reason) });
        }
      });
  }, [selected]);

  const page = loadState.status === "ready" ? loadState.page : null;
  const catalogReady = catalogCanQuery(catalogState);
  const availabilityNotice = catalogReady ? machineAvailabilityNotice(page) : null;
  const range =
    !catalogReady || !page
      ? catalogRangeLabel(catalogState)
      : page.total > 0
        ? `${(page.offset + 1).toLocaleString()}–${Math.min(
            page.offset + page.items.length,
            page.total,
          ).toLocaleString()} of ${page.total.toLocaleString()}`
        : "Metadata loaded";

  const focusSelectedMachine = useCallback(() => {
    if (!page || !selected) return;
    const index = page.items.findIndex((machine) => machine.shortName === selected.shortName);
    if (index >= 0) machineRowRefs.current[index]?.focus();
  }, [page, selected]);

  const launchDetail = useCallback(
    (detail: MachineDetail) => {
      if (!detail.runnable || launchState.status === "launching") return;
      setLaunchState({ status: "launching" });
      void launchLibraryMachine({
        shortName: detail.shortName,
        launchOverrides: pendingLaunchOverrides,
      })
        .then((session) => {
          setLaunchState({ status: "launched", session });
          setPendingLaunchOverrides(null);
          onSessionStarted(session);
        })
        .catch((reason: unknown) => {
          setLaunchState({ status: "error", message: errorMessage(reason) });
        });
    },
    [launchState.status, onSessionStarted, pendingLaunchOverrides],
  );

  const exportDisplayedList = useCallback(() => {
    if (
      !catalogReady ||
      exportState.status === "exporting" ||
      (valueRequired && !debouncedFilterValue)
    ) {
      return;
    }
    setExportState({ status: "exporting" });
    void exportMameUiDisplayedList({
      text: debouncedSearch || null,
      filter,
      filterValue: debouncedFilterValue || null,
    })
      .then((result) => {
        if (result.canceled) {
          setExportState({ status: "idle" });
          return;
        }
        setExportState({
          status: "success",
          message: `Exported ${result.rows.toLocaleString()} machine${result.rows === 1 ? "" : "s"}.`,
        });
      })
      .catch((reason: unknown) => {
        setExportState({ status: "error", message: errorMessage(reason) });
      });
  }, [
    catalogReady,
    debouncedFilterValue,
    debouncedSearch,
    exportState.status,
    filter,
    valueRequired,
  ]);

  const activateMachine = useCallback(
    (machine: MachineListItem) => {
      if (!machine.runnable || selectedRef.current?.shortName !== machine.shortName) {
        return;
      }
      const activation = ++activationSequence.current;
      asyncIdentityRef.current = {
        ...asyncIdentityRef.current,
        activationGeneration: activation,
      };
      const detailGeneration = detailSequence.current;
      if (detailState.status === "ready" && detailState.detail.shortName === machine.shortName) {
        launchDetail(detailState.detail);
        return;
      }
      void getMameMachineDetail({ shortName: machine.shortName })
        .then((detail) => {
          if (
            activationMayCommit(
              asyncIdentityRef.current,
              detailGeneration,
              activation,
              machine.shortName,
            )
          ) {
            launchDetail(detail);
          }
        })
        .catch((reason: unknown) => {
          if (
            activationMayCommit(
              asyncIdentityRef.current,
              detailGeneration,
              activation,
              machine.shortName,
            )
          ) {
            setLaunchState({ status: "error", message: errorMessage(reason) });
          }
        });
    },
    [detailState, launchDetail],
  );

  function handleMachineRowKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>, index: number) {
    if (!page) return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      activeFilterButtonRef.current?.focus();
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      setShowNarrowDetails(true);
      rightPanelFirstTabRef.current?.focus();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setShowNarrowDetails(false);
      searchInputRef.current?.focus();
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      const machine = page.items[index];
      if (machine) activateMachine(machine);
      return;
    }
    const nextIndex = nextBrowserIndex(event.key, index, page.items.length);
    if (nextIndex === null) return;
    event.preventDefault();
    const next = page.items[nextIndex];
    if (!next) return;
    selectMachine(next);
    machineRowRefs.current[nextIndex]?.focus();
  }

  useEffect(() => {
    function handleShortcut(event: KeyboardEvent) {
      if (
        event.defaultPrevented ||
        event.repeat ||
        event.altKey ||
        gameplayInputOwned ||
        softwareMode ||
        !document.hasFocus()
      ) {
        return;
      }

      const editing = isEditableElement(event.target);
      if (event.key === "Escape" && event.target === searchInputRef.current && search) {
        event.preventDefault();
        setSearch("");
        return;
      }
      if (editing) return;

      if (event.key === "Escape" && showNarrowDetails) {
        event.preventDefault();
        setShowNarrowDetails(false);
        focusSelectedMachine();
        return;
      }
      if (event.key === "/" && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        searchInputRef.current?.focus();
        return;
      }
      if (
        event.key === "Enter" &&
        (event.ctrlKey || event.metaKey) &&
        detailState.status === "ready"
      ) {
        event.preventDefault();
        launchDetail(detailState.detail);
      }
    }

    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [
    detailState,
    focusSelectedMachine,
    gameplayInputOwned,
    launchDetail,
    search,
    showNarrowDetails,
    softwareMode,
  ]);

  const detail = detailState.status === "ready" ? detailState.detail : null;
  const detailErrorMessage = detailState.status === "error" ? detailState.message : null;
  const emptyRightPanelStatus = emptyRightPanelStatusFor(detailState);
