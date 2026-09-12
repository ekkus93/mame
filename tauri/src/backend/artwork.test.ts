import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  discoverMachineArtwork,
  getArtworkConfiguration,
  pickArtworkDirectory,
  readArtworkAsset,
  setArtworkConfiguration,
} from "./artwork";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("artwork backend commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads artwork roots through the dedicated Rust command", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, roots: [], validations: [] });
    await getArtworkConfiguration();
    expect(invoke).toHaveBeenCalledWith("get_artwork_configuration");
  });

  it("persists ordered roots without flattening encoded platform paths", async () => {
    const roots = ["/artwork", { encoding: "unixBytesHex", data: "617274ff" }];
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, roots, validations: [] });

    await setArtworkConfiguration(roots);
    expect(invoke).toHaveBeenCalledWith("set_artwork_configuration", {
      request: { roots },
    });
  });

  it("requests native artwork directory selection", async () => {
    vi.mocked(invoke).mockResolvedValue("/artwork");
    await expect(pickArtworkDirectory()).resolves.toBe("/artwork");
    expect(invoke).toHaveBeenCalledWith("pick_artwork_directory");
  });

  it("discovers artwork by machine identifier only", async () => {
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1, machine: "pacman", slots: [] });
    await discoverMachineArtwork("pacman");
    expect(invoke).toHaveBeenCalledWith("discover_machine_artwork", { machine: "pacman" });
  });

  it("reads artwork through an opaque asset identifier without sending a host path", async () => {
    vi.mocked(invoke).mockResolvedValue({
      schemaVersion: 1,
      assetId: "local:0:pacman:screenshot:png",
      mimeType: "image/png",
      bytes: 3,
      dataUrl: "data:image/png;base64,Zm9v",
    });

    await readArtworkAsset("local:0:pacman:screenshot:png");
    expect(invoke).toHaveBeenCalledWith("read_artwork_asset", {
      assetId: "local:0:pacman:screenshot:png",
    });
  });
});
