import { describe, expect, it } from "vitest";

import type { ArtworkAssetPayload } from "../backend/artwork";
import { ArtworkPreviewCache } from "./artworkPreviewCache";

function payload(assetId: string, bytes: number): ArtworkAssetPayload {
  return {
    schemaVersion: 1,
    assetId,
    mimeType: "image/png",
    bytes,
    dataUrl: "data:image/png;base64,Zm9v",
  };
}

describe("ArtworkPreviewCache", () => {
  it("evicts the least recently used entry when the entry limit is exceeded", () => {
    const cache = new ArtworkPreviewCache(2, 100);
    cache.set(payload("a", 10));
    cache.set(payload("b", 10));
    expect(cache.get("a")?.assetId).toBe("a");

    cache.set(payload("c", 10));

    expect(cache.get("b")).toBeNull();
    expect(cache.get("a")?.assetId).toBe("a");
    expect(cache.get("c")?.assetId).toBe("c");
  });

  it("evicts old entries until the byte budget is satisfied", () => {
    const cache = new ArtworkPreviewCache(4, 20);
    cache.set(payload("a", 12));
    cache.set(payload("b", 12));

    expect(cache.get("a")).toBeNull();
    expect(cache.get("b")?.assetId).toBe("b");
    expect(cache.bytes()).toBe(12);
  });

  it("accounts for replacement payloads without double counting", () => {
    const cache = new ArtworkPreviewCache(4, 100);
    cache.set(payload("a", 10));
    cache.set(payload("a", 25));

    expect(cache.size()).toBe(1);
    expect(cache.bytes()).toBe(25);
  });

  it("does not cache a payload larger than the configured byte budget", () => {
    const cache = new ArtworkPreviewCache(4, 20);
    cache.set(payload("oversized", 21));

    expect(cache.size()).toBe(0);
    expect(cache.bytes()).toBe(0);
  });
});
