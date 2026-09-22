import type { ArtworkAssetPayload } from "../backend/artwork";
import type { ArtworkAssetState } from "./artworkState";

export function ArtworkAssetFrame({
  state,
  label,
  machine,
}: {
  state: ArtworkAssetState<ArtworkAssetPayload>;
  label: string;
  machine: string;
}) {
  return (
    <div className="mame-artwork-frame">
      {state.status === "loading" && <div className="mame-panel-state">Loading {label}…</div>}
      {state.status === "error" && (
        <div className="mame-no-image-placeholder" role="alert">
          <strong>Artwork unavailable</strong>
          <span>{state.message}</span>
        </div>
      )}
      {state.status === "ready" && <img src={state.asset.dataUrl} alt={`${machine} ${label}`} />}
      {state.status === "missing" && (
        <div className="mame-no-image-placeholder">
          <strong>No image Available</strong>
          <span>{label}</span>
        </div>
      )}
    </div>
  );
}
