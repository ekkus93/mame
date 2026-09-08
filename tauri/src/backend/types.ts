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
