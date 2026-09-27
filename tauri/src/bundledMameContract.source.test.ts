/// <reference types="node" />

import { describe, expect, it } from "vitest";

import appBackendSource from "../src-tauri/src/app.rs?raw";
import effectiveRuntimeSource from "../src-tauri/src/effective_runtime.rs?raw";
import metadataBackendSource from "../src-tauri/src/metadata.rs?raw";
import browserSource from "./browser/MameBrowser.tsx?raw";
import settingsSource from "./settings/GeneralSettingsPanel.tsx?raw";

describe("bundled MAME product contract", () => {
  it("defaults runtime authority to package-owned MAME rather than an absent override", () => {
    expect(effectiveRuntimeSource).toContain("BundledRuntimeLayout::from_resource_dir");
    expect(effectiveRuntimeSource).toContain("configured_external_source(external_override)");
    expect(appBackendSource).toContain("effective_mame_identity(app)");
    expect(appBackendSource).not.toContain("let Some(path) = settings.mame_executable");
  });

  it("keeps metadata source resolution in Rust and supports the effective bundled runtime", () => {
    expect(metadataBackendSource).toContain("effective_mame_source(app)");
    expect(browserSource).toContain("getMameMetadataStatus({})");
    expect(browserSource).toContain("refreshMameMetadata({})");
  });

  it("automatically bootstraps missing or stale metadata", () => {
    expect(browserSource).toContain("Fresh installs and bundled-runtime upgrades bootstrap metadata");
    expect(browserSource).toContain('setCatalogState({ status: "importing" })');
  });

  it("does not present executable selection as normal-user configuration", () => {
    expect(settingsSource).toContain("MAME itself is installed with this application.");
    expect(settingsSource).toContain("Advanced runtime override");
    expect(settingsSource).toContain("Use bundled MAME");
    expect(settingsSource).toContain("Custom executable override");
    expect(settingsSource).not.toContain("Choose a MAME executable");
    expect(settingsSource).not.toContain(">MAME executable<");
  });

  it("resetting an override means returning to bundled MAME", () => {
    expect(settingsSource).toContain("setGeneralMameExecutable(null)");
    expect(settingsSource).toContain("Using the MAME runtime installed with this application.");
  });
});
