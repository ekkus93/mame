import { invoke } from "@tauri-apps/api/core";

export type ControllerProfileScope = { kind: "global" } | { kind: "machine"; shortName: string };

export type ControllerDeviceIdentityKind = "browserGamepadId" | "mameInputDeviceId" | "userDefined";

export type ControllerMappingProvenanceKind =
  "browserStandardGamepad" | "mameControllerConfig" | "mameOsdControllerMap" | "projectOwned";

export type ControllerProfile = {
  schemaVersion: 1;
  id: number;
  name: string;
  targetDevice: {
    kind: ControllerDeviceIdentityKind;
    value: string;
    reportedMapping: string | null;
  };
  mappingProvenance: {
    kind: ControllerMappingProvenanceKind;
    sourceReference: string | null;
  };
  createdAtEpochMs: number;
  updatedAtEpochMs: number;
};

export type ControllerProfileApplicationStatus = "unassigned" | "assignedNotApplied";

export type ControllerProfileConfiguration = {
  schemaVersion: 1;
  profiles: ControllerProfile[];
  assignedProfile: ControllerProfile | null;
  effectiveProfile: ControllerProfile | null;
  effectiveScope: ControllerProfileScope | null;
  activeProfile: ControllerProfile | null;
  applicationStatus: ControllerProfileApplicationStatus;
  statusMessage: string;
};

export async function getControllerProfileConfiguration(
  scope: ControllerProfileScope,
): Promise<ControllerProfileConfiguration> {
  return invoke<ControllerProfileConfiguration>("get_controller_profile_configuration", {
    request: { scope },
  });
}

export async function setControllerProfileSelection(
  scope: ControllerProfileScope,
  profileId: number | null,
): Promise<ControllerProfileConfiguration> {
  return invoke<ControllerProfileConfiguration>("set_controller_profile_selection", {
    request: { scope, profileId },
  });
}

export async function createBrowserControllerProfile(request: {
  name: string;
  gamepadId: string;
  mapping: string;
}): Promise<ControllerProfile> {
  return invoke<ControllerProfile>("create_browser_controller_profile", { request });
}
