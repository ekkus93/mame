import type { MameUiBootstrapStatus } from "../backend/mameBootstrap";

export type ImportableBootstrapStatus = Extract<
  MameUiBootstrapStatus,
  { status: "metadataMissing" | "metadataStale" }
>;

export type MameBrowserBootstrapState =
  | { status: "checking" }
  | MameUiBootstrapStatus
  | { status: "importing"; previous: ImportableBootstrapStatus }
  | { status: "importFailed"; previous: ImportableBootstrapStatus; message: string };

export function bootstrapFromBackend(status: MameUiBootstrapStatus): MameBrowserBootstrapState {
  return status;
}

export function beginMetadataImport(state: MameBrowserBootstrapState): MameBrowserBootstrapState {
  if (state.status === "metadataMissing" || state.status === "metadataStale") {
    return { status: "importing", previous: state };
  }
  if (state.status === "importFailed") {
    return { status: "importing", previous: state.previous };
  }
  return state;
}

export function failMetadataImport(
  state: MameBrowserBootstrapState,
  message: string,
): MameBrowserBootstrapState {
  if (state.status !== "importing") return state;
  return { status: "importFailed", previous: state.previous, message };
}

export function catalogIsReady(
  state: MameBrowserBootstrapState,
): state is Extract<MameUiBootstrapStatus, { status: "ready" }> {
  return state.status === "ready";
}
