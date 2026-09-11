import { invoke } from "@tauri-apps/api/core";

import {
  APP_PROTOCOL_VERSION,
  type AppInfoRequest,
  type AppInfoResponse,
  type FavoritePage,
  type FavoritePageRequest,
  type FavoriteState,
  type LaunchLibraryMachineRequest,
  type LaunchLibrarySoftwareRequest,
  type MachineDetail,
  type MachineDetailRequest,
  type MachinePage,
  type MachineSearchRequest,
  type MetadataRefreshResult,
  type MetadataStatus,
  type MetadataStatusRequest,
  type PauseMameRequest,
  type PauseMameResult,
  type RefreshMameMetadataRequest,
  type ResetMameRequest,
  type ResetMameResult,
  type SessionSnapshot,
  type SetLibraryFavoriteRequest,
  type SoftwareListPage,
  type SoftwareListQueryRequest,
} from "./types";

export async function getAppInfo(): Promise<AppInfoResponse> {
  const request: AppInfoRequest = {
    protocolVersion: APP_PROTOCOL_VERSION,
  };

  return invoke<AppInfoResponse>("get_app_info", { request });
}

export async function refreshMameMetadata(
  request: RefreshMameMetadataRequest,
): Promise<MetadataRefreshResult> {
  return invoke<MetadataRefreshResult>("refresh_mame_metadata", { request });
}

export async function getMameMetadataStatus(
  request: MetadataStatusRequest,
): Promise<MetadataStatus> {
  return invoke<MetadataStatus>("get_mame_metadata_status", { request });
}

export async function queryMameLibrary(request: MachineSearchRequest): Promise<MachinePage> {
  return invoke<MachinePage>("query_mame_library", { request });
}

export async function getMameMachineDetail(request: MachineDetailRequest): Promise<MachineDetail> {
  return invoke<MachineDetail>("get_mame_machine_detail", { request });
}

export async function launchLibraryMachine(
  request: LaunchLibraryMachineRequest,
): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("launch_library_machine", { request });
}

export async function queryMameSoftwareList(
  request: SoftwareListQueryRequest,
): Promise<SoftwareListPage> {
  return invoke<SoftwareListPage>("query_mame_software_list", { request });
}

export async function launchLibrarySoftware(
  request: LaunchLibrarySoftwareRequest,
): Promise<SessionSnapshot> {
  return invoke<SessionSnapshot>("launch_library_software", { request });
}

export async function getMameSession(): Promise<SessionSnapshot | null> {
  return invoke<SessionSnapshot | null>("get_mame_session");
}

export async function pauseMame(request: PauseMameRequest): Promise<PauseMameResult> {
  return invoke<PauseMameResult>("pause_mame", { request });
}

export async function resumeMame(request: PauseMameRequest): Promise<PauseMameResult> {
  return invoke<PauseMameResult>("resume_mame", { request });
}

export async function resetMame(request: ResetMameRequest): Promise<ResetMameResult> {
  return invoke<ResetMameResult>("reset_mame", { request });
}

export async function getLibraryFavorite(request: MachineDetailRequest): Promise<FavoriteState> {
  return invoke<FavoriteState>("get_library_favorite", { request });
}

export async function setLibraryFavorite(
  request: SetLibraryFavoriteRequest,
): Promise<FavoriteState> {
  return invoke<FavoriteState>("set_library_favorite", { request });
}

export async function queryLibraryFavorites(request: FavoritePageRequest): Promise<FavoritePage> {
  return invoke<FavoritePage>("query_library_favorites", { request });
}
