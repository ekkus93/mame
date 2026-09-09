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

export type MachineSearchRequest = {
  text?: string | null;
  manufacturer?: string | null;
  year?: string | null;
  driverStatus?: "good" | "imperfect" | "preliminary" | null;
  cloneFilter?: CloneFilter;
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
};
