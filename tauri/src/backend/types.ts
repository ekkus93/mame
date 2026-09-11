import type { LaunchPreferences } from "./generalSettings";

export const APP_PROTOCOL_VERSION = 1 as const;

export type AppErrorEnvelope = {
  code: string;
  message: string;
  details: unknown;
  retryable: boolean;
};

export type AppInfoRequest = {
  protocolVersion: typeof APP_PROTOCOL_VERSION;
};

export type AppInfoResponse = {
  protocolVersion: typeof APP_PROTOCOL_VERSION;
  appVersion: string;
  backend: "rust-tauri";
};

export type AppReadyEventV1 = {
  schemaVersion: 1;
  appVersion: string;
};

export type MameExecutableSelectionKind = "external" | "developmentTree";

export type MameExecutableRequest = {
  source: MameExecutableSelectionKind;
  path: string;
};

export type MameExecutableIdentity = {
  source: "bundled" | "external" | "developmentTree";
  trust: "qualifiedBundled" | "userConfigured" | "development";
  path: string;
  version: string;
  build: string | null;
  rawVersionLine: string;
};

export type MetadataGenerationSummary = {
  generationId: number;
  sourceKind: string;
  trust: string;
  executablePath: string;
  mameVersion: string;
  mameBuild: string | null;
  rawVersionLine: string;
  listxmlBuild: string | null;
  mameConfig: string | null;
  generatedAtEpochMs: number;
  importedAtEpochMs: number;
  machineCount: number;
};

export type MetadataFreshness = "empty" | "fresh" | "stale";

export type MetadataStatus = {
  schemaVersion: 1;
  freshness: MetadataFreshness;
  currentExecutable: MameExecutableIdentity;
  activeGeneration: MetadataGenerationSummary | null;
};

export type MetadataRefreshResult = {
  schemaVersion: 1;
  generation: MetadataGenerationSummary;
};

export type RefreshMameMetadataRequest = {
  executable: MameExecutableRequest;
};

export type MetadataStatusRequest = {
  executable: MameExecutableRequest;
};

export type CloneFilter = "all" | "parentsOnly" | "clonesOnly";

export type MachineAvailability = "available" | "missing" | "unknown";

export type MachineSort =
  | "descriptionAsc"
  | "descriptionDesc"
  | "shortNameAsc"
  | "yearAsc"
  | "yearDesc"
  | "manufacturerAsc"
  | "manufacturerDesc";

export type MachineSearchRequest = {
  text?: string | null;
  manufacturer?: string | null;
  year?: string | null;
  driverStatus?: "good" | "imperfect" | "preliminary" | null;
  availability?: MachineAvailability | null;
  cloneFilter?: CloneFilter;
  sort?: MachineSort;
  includeDevices?: boolean;
  limit?: number;
  offset?: number;
};

export type MachineListItem = {
  shortName: string;
  description: string;
  year: string | null;
  manufacturer: string | null;
  sourceFile: string | null;
  cloneOf: string | null;
  runnable: boolean;
  isDevice: boolean;
  driverStatus: string | null;
  displayCount: number;
  softwareListCount: number;
};

export type MachinePage = {
  schemaVersion: 1;
  generationId: number;
  total: number;
  offset: number;
  limit: number;
  items: MachineListItem[];
  availabilityByShortName: Record<string, MachineAvailability>;
};

export type MachineDetailRequest = {
  shortName: string;
};

export type MachineDisplayInfo = {
  tag: string | null;
  displayType: string;
  rotate: number | null;
  flipX: boolean;
  width: number | null;
  height: number | null;
  refreshHz: number;
  pixelClockHz: number | null;
};

export type MachineSoftwareListInfo = {
  tag: string;
  name: string;
  status: string;
  filter: string | null;
};

export type MachineDetail = {
  schemaVersion: 1;
  generationId: number;
  shortName: string;
  description: string;
  year: string | null;
  manufacturer: string | null;
  sourceFile: string | null;
  cloneOf: string | null;
  parentDescription: string | null;
  romOf: string | null;
  isBios: boolean;
  isDevice: boolean;
  isMechanical: boolean;
  runnable: boolean;
  driverStatus: string | null;
  driverEmulation: string | null;
  driverCocktail: string | null;
  driverSavestate: string | null;
  driverRequiresArtwork: boolean;
  driverUnofficial: boolean;
  driverNoSoundHardware: boolean;
  driverIncomplete: boolean;
  displays: MachineDisplayInfo[];
  softwareLists: MachineSoftwareListInfo[];
};

export type LaunchLibraryMachineRequest = {
  shortName: string;
  launchOverrides?: LaunchPreferences | null;
};

export type SoftwareItemSummary = {
  shortName: string;
  description: string;
  year: string;
  publisher: string;
  cloneOf: string | null;
  supported: "yes" | "partial" | "no";
};

export type SoftwareListQueryRequest = {
  shortName: string;
  softwareList: string;
  text?: string | null;
  limit?: number;
  offset?: number;
};

export type SoftwareListPage = {
  schemaVersion: 1;
  machineShortName: string;
  softwareListName: string;
  softwareListDescription: string | null;
  total: number;
  offset: number;
  limit: number;
  items: SoftwareItemSummary[];
};

export type LaunchLibrarySoftwareRequest = {
  shortName: string;
  softwareList: string;
  softwareItem: string;
  launchOverrides?: LaunchPreferences | null;
};

export type FavoriteState = {
  schemaVersion: 1;
  shortName: string;
  favorite: boolean;
  createdAtEpochMs: number | null;
};

export type FavoriteEntry = {
  shortName: string;
  createdAtEpochMs: number;
  machine: MachineListItem | null;
};

export type FavoritePage = {
  schemaVersion: 1;
  total: number;
  offset: number;
  limit: number;
  items: FavoriteEntry[];
};

export type FavoritePageRequest = {
  limit?: number;
  offset?: number;
};

export type SetLibraryFavoriteRequest = {
  shortName: string;
  favorite: boolean;
};

export type SessionState =
  "created" | "starting" | "running" | "stopping" | "exited" | "failed" | "crashed";

export type SessionSnapshot = {
  schemaVersion: 1;
  sessionId: string;
  state: SessionState;
  machine: string;
  software: string | null;
  pid: number | null;
};

export type PauseMameRequest = {
  sessionId: string;
};

export type PauseMameResult = {
  schemaVersion: 1;
  sessionId: string;
  paused: boolean;
};

export type SessionPauseEventV1 = PauseMameResult;
