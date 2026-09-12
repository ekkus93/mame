import { invoke } from "@tauri-apps/api/core";

import type { PathValidation, PlatformPath } from "./pathConfiguration";

export type ArtworkKind =
  | "screenshot"
  | "cabinet"
  | "marquee"
  | "flyer"
  | "icon"
  | "systemImage";

export type ArtworkProvenance = {
  kind: "localFile";
  rootIndex: number;
};

export type ArtworkDescriptor = {
  schemaVersion: number;
  assetId: string;
  machine: string;
  kind: ArtworkKind;
  mimeType: string;
  bytes: number;
  provenance: ArtworkProvenance;
};

export type ArtworkSlot = {
  kind: ArtworkKind;
  asset: ArtworkDescriptor | null;
};

export type MachineArtwork = {
  schemaVersion: number;
  machine: string;
  slots: ArtworkSlot[];
};

export type ArtworkConfiguration = {
  schemaVersion: number;
  roots: PlatformPath[];
  validations: PathValidation[];
};

export async function getArtworkConfiguration(): Promise<ArtworkConfiguration> {
  return invoke<ArtworkConfiguration>("get_artwork_configuration");
}

export async function setArtworkConfiguration(
  roots: PlatformPath[],
): Promise<ArtworkConfiguration> {
  return invoke<ArtworkConfiguration>("set_artwork_configuration", {
    request: { roots },
  });
}

export async function pickArtworkDirectory(): Promise<PlatformPath | null> {
  return invoke<PlatformPath | null>("pick_artwork_directory");
}

export async function discoverMachineArtwork(machine: string): Promise<MachineArtwork> {
  return invoke<MachineArtwork>("discover_machine_artwork", { machine });
}
