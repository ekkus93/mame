import { useEffect, useMemo, useState } from "react";

import {
  discoverMachineArtwork,
  readArtworkAsset,
  type ArtworkAssetPayload,
  type ArtworkKind,
  type MachineArtwork,
} from "../backend/artwork";
import { errorMessage } from "../backend/errors";
import { artworkPreviewCache } from "./artworkPreviewCache";
import "./MachineArtworkPanel.css";

const ARTWORK_LABELS: Record<ArtworkKind, string> = {
  screenshot: "Screenshot",
  cabinet: "Cabinet",
  marquee: "Marquee",
  flyer: "Flyer",
  icon: "Icon",
  systemImage: "System image",
};

type DiscoveryState =
  | { status: "loading" }
  | { status: "ready"; artwork: MachineArtwork }
  | { status: "error"; message: string };

type PreviewState =
  | { status: "idle" }
  | { status: "loading"; assetId: string }
  | { status: "ready"; payload: ArtworkAssetPayload }
  | { status: "error"; assetId: string; message: string };

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KiB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}

export function MachineArtworkPanel({ machine }: { machine: string }) {
  const [discovery, setDiscovery] = useState<DiscoveryState>({ status: "loading" });
  const [selectedKind, setSelectedKind] = useState<ArtworkKind | null>(null);
  const [preview, setPreview] = useState<PreviewState>({ status: "idle" });

  useEffect(() => {
    let cancelled = false;
    setDiscovery({ status: "loading" });
    setSelectedKind(null);
    setPreview({ status: "idle" });

    void discoverMachineArtwork(machine)
      .then((artwork) => {
        if (cancelled) {
          return;
        }
        setDiscovery({ status: "ready", artwork });
        setSelectedKind(artwork.slots.find((slot) => slot.asset !== null)?.kind ?? null);
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setDiscovery({ status: "error", message: errorMessage(reason) });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [machine]);

  const selectedSlot = useMemo(() => {
    if (discovery.status !== "ready" || selectedKind === null) {
      return null;
    }
    return discovery.artwork.slots.find((slot) => slot.kind === selectedKind) ?? null;
  }, [discovery, selectedKind]);

  useEffect(() => {
    const asset = selectedSlot?.asset ?? null;
    if (!asset) {
      setPreview({ status: "idle" });
      return;
    }

    const cached = artworkPreviewCache.get(asset.assetId);
    if (cached) {
      setPreview({ status: "ready", payload: cached });
      return;
    }

    let cancelled = false;
    setPreview({ status: "loading", assetId: asset.assetId });
    void readArtworkAsset(asset.assetId)
      .then((payload) => {
        if (cancelled) {
          return;
        }
        if (payload.assetId !== asset.assetId) {
          setPreview({
            status: "error",
            assetId: asset.assetId,
            message: "The artwork response did not match the requested asset.",
          });
          return;
        }
        artworkPreviewCache.set(payload);
        setPreview({ status: "ready", payload });
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setPreview({
            status: "error",
            assetId: asset.assetId,
            message: errorMessage(reason),
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedSlot]);

  const headingId = `machine-artwork-${machine}`;

  return (
    <section className="machine-artwork" aria-labelledby={headingId}>
      <div className="machine-artwork__heading">
        <div>
          <h4 id={headingId}>Artwork</h4>
          <p>Local previews are discovered lazily for the selected machine only.</p>
        </div>
      </div>

      {discovery.status === "loading" && (
        <div className="machine-artwork__placeholder" aria-live="polite">
          Loading local artwork…
        </div>
      )}

      {discovery.status === "error" && (
        <div className="machine-artwork__error" role="alert">
          <strong>Artwork unavailable</strong>
          <span>{discovery.message}</span>
        </div>
      )}

      {discovery.status === "ready" && (
        <>
          <div className="machine-artwork__tabs" aria-label="Artwork category">
            {discovery.artwork.slots.map((slot) => {
              const label = ARTWORK_LABELS[slot.kind];
              return (
                <button
                  key={slot.kind}
                  type="button"
                  className={slot.kind === selectedKind ? "is-selected" : undefined}
                  aria-pressed={slot.kind === selectedKind}
                  disabled={slot.asset === null}
                  title={slot.asset === null ? `No local ${label.toLowerCase()} found` : undefined}
                  onClick={() => setSelectedKind(slot.kind)}
                >
                  {label}
                </button>
              );
            })}
          </div>

          {selectedKind === null && (
            <div className="machine-artwork__placeholder">
              No local artwork found for this machine.
            </div>
          )}

          {selectedKind !== null && preview.status === "loading" && (
            <div className="machine-artwork__placeholder" aria-live="polite">
              Loading {ARTWORK_LABELS[selectedKind].toLowerCase()}…
            </div>
          )}

          {selectedKind !== null && preview.status === "error" && (
            <div className="machine-artwork__error" role="alert">
              <strong>{ARTWORK_LABELS[selectedKind]} unavailable</strong>
              <span>{preview.message}</span>
            </div>
          )}

          {selectedKind !== null && preview.status === "ready" && (
            <figure className="machine-artwork__preview">
              <img
                src={preview.payload.dataUrl}
                alt={`${ARTWORK_LABELS[selectedKind]} for ${machine}`}
                loading="lazy"
                decoding="async"
              />
              <figcaption>
                {ARTWORK_LABELS[selectedKind]} · {formatBytes(preview.payload.bytes)}
              </figcaption>
            </figure>
          )}
        </>
      )}
    </section>
  );
}
