import type { ArtworkAssetPayload } from "../backend/artwork";

export const MAX_ARTWORK_PREVIEW_CACHE_ENTRIES = 4;
export const MAX_ARTWORK_PREVIEW_CACHE_BYTES = 16 * 1024 * 1024;

export class ArtworkPreviewCache {
  private readonly entries = new Map<string, ArtworkAssetPayload>();
  private totalBytes = 0;

  constructor(
    private readonly maxEntries = MAX_ARTWORK_PREVIEW_CACHE_ENTRIES,
    private readonly maxBytes = MAX_ARTWORK_PREVIEW_CACHE_BYTES,
  ) {}

  get(assetId: string): ArtworkAssetPayload | null {
    const payload = this.entries.get(assetId);
    if (!payload) {
      return null;
    }

    this.entries.delete(assetId);
    this.entries.set(assetId, payload);
    return payload;
  }

  set(payload: ArtworkAssetPayload): void {
    if (!Number.isFinite(payload.bytes) || payload.bytes < 0 || payload.bytes > this.maxBytes) {
      return;
    }

    const existing = this.entries.get(payload.assetId);
    if (existing) {
      this.totalBytes -= existing.bytes;
      this.entries.delete(payload.assetId);
    }

    this.entries.set(payload.assetId, payload);
    this.totalBytes += payload.bytes;
    this.evictToLimits();
  }

  clear(): void {
    this.entries.clear();
    this.totalBytes = 0;
  }

  size(): number {
    return this.entries.size;
  }

  bytes(): number {
    return this.totalBytes;
  }

  private evictToLimits(): void {
    while (this.entries.size > this.maxEntries || this.totalBytes > this.maxBytes) {
      const oldest = this.entries.entries().next().value;
      if (!oldest) {
        this.totalBytes = 0;
        return;
      }
      this.entries.delete(oldest[0]);
      this.totalBytes -= oldest[1].bytes;
    }
  }
}

export const artworkPreviewCache = new ArtworkPreviewCache();
