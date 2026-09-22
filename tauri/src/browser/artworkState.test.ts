import { describe, expect, it } from "vitest";

import {
  artworkAssetError,
  initialArtworkAssetState,
  isCurrentArtworkRequest,
} from "./artworkState";

describe("MAME artwork asset state", () => {
  it("distinguishes a selected asset that is loading from a missing slot", () => {
    expect(initialArtworkAssetState(true)).toEqual({ status: "loading" });
    expect(initialArtworkAssetState(false)).toEqual({ status: "missing" });
  });

  it("uses the shared formatter for artwork errors", () => {
    const reason = { code: "asset-failed" };
    expect(artworkAssetError(reason, () => "formatted artwork failure")).toEqual({
      status: "error",
      message: "formatted artwork failure",
    });
  });
  it("rejects out-of-order artwork responses after request identity changes", () => {
    expect(isCurrentArtworkRequest(8, 7)).toBe(false);
    expect(isCurrentArtworkRequest(8, 8)).toBe(true);
  });
});
