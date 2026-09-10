import { invoke } from "@tauri-apps/api/core";

export type EncodedPlatformPath = {
  encoding: string;
  data: string;
};

export type PlatformPath = string | EncodedPlatformPath;

export type PathValidationStatus =
  | "accessible"
  | "missing"
  | "notDirectory"
  | "permissionDenied"
  | "unreadable";

export type ContentPaths = {
  romPaths: PlatformPath[];
  softwarePaths: PlatformPath[];
  chdPaths: PlatformPath[];
};

export type PathValidation = {
  path: PlatformPath;
  status: PathValidationStatus;
  message: string | null;
};

export type ContentPathValidations = {
  romPaths: PathValidation[];
  softwarePaths: PathValidation[];
  chdPaths: PathValidation[];
};

export type ContentPathConfiguration = {
  contentPaths: ContentPaths;
  validations: ContentPathValidations;
};

export async function getContentPathConfiguration(): Promise<ContentPathConfiguration> {
  return invoke<ContentPathConfiguration>("get_content_path_configuration");
}

export async function setContentPathConfiguration(
  contentPaths: ContentPaths,
): Promise<ContentPathConfiguration> {
  return invoke<ContentPathConfiguration>("set_content_path_configuration", {
    request: { contentPaths },
  });
}

export async function pickContentDirectory(): Promise<PlatformPath | null> {
  return invoke<PlatformPath | null>("pick_content_directory");
}
