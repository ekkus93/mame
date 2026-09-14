import { useEffect, useMemo, useRef, useState } from "react";

import {
  discoverMachineArtwork,
  readArtworkAsset,
  type ArtworkAssetPayload,
  type ArtworkKind,
  type MachineArtwork,
} from "../backend/artwork";
import type { LaunchPreferences } from "../backend/generalSettings";
import type { MachineDetail, SessionSnapshot } from "../backend/types";
import { MachineAuditPanel } from "../library/MachineAuditPanel";
import { SoftwareListBrowser } from "../library/SoftwareListBrowser";
import { machineStatusLabel } from "../library/libraryQuery";
import { MachineSettingsPanel } from "../settings/MachineSettingsPanel";

export type MachineRightView = "images" | "info" | "audit" | "settings" | "software";

const ARTWORK_LABELS: Record<ArtworkKind, string> = {
  screenshot: "Snapshots",
  cabinet: "Cabinet",
  marquee: "Marquee",
  flyer: "Flyer",
  icon: "Icon",
  systemImage: "System image",
};

export function MachineRightPanel({
  detail,
  view,
  onViewChange,
  artworkKind,
  onArtworkKindChange,
  pendingLaunchOverrides,
  onPendingLaunchOverridesChanged,
  onAuditResultChanged,
  onSoftwareSessionStarted,
}: {
  detail: MachineDetail;
  view: MachineRightView;
  onViewChange: (view: MachineRightView) => void;
  artworkKind: ArtworkKind;
  onArtworkKindChange: (kind: ArtworkKind) => void;
  pendingLaunchOverrides: LaunchPreferences | null;
  onPendingLaunchOverridesChanged: (preferences: LaunchPreferences | null) => void;
  onAuditResultChanged: () => void;
  onSoftwareSessionStarted: (session: SessionSnapshot) => void;
}) {
  return (
    <aside className="mame-right-panel" aria-label="Selected machine context">
      <div className="mame-right-tabs" role="tablist" aria-label="Machine detail view">
        <button
          type="button"
          role="tab"
          aria-selected={view === "images"}
          className={view === "images" ? "is-selected" : ""}
          onClick={() => onViewChange("images")}
        >
          Images
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={view === "info"}
          className={view === "info" ? "is-selected" : ""}
          onClick={() => onViewChange("info")}
        >
          Info
        </button>
      </div>

      {view === "images" && (
        <ArtworkPane
          machine={detail.shortName}
          selectedKind={artworkKind}
          onSelectedKindChange={onArtworkKindChange}
        />
      )}
      {view === "info" && <InfoPane detail={detail} />}
      {view === "audit" && (
        <MachineAuditPanel
          shortName={detail.shortName}
          onAuditResultChanged={onAuditResultChanged}
        />
      )}
      {view === "settings" && (
        <MachineSettingsPanel
          shortName={detail.shortName}
          pendingLaunchOverrides={pendingLaunchOverrides}
          onPendingLaunchOverridesChanged={onPendingLaunchOverridesChanged}
        />
      )}
      {view === "software" && (
        <SoftwareListBrowser
          detail={detail}
          launchOverrides={pendingLaunchOverrides}
          onSessionStarted={onSoftwareSessionStarted}
        />
      )}
    </aside>
  );
}

function ArtworkPane({
  machine,
  selectedKind,
  onSelectedKindChange,
}: {
  machine: string;
  selectedKind: ArtworkKind;
  onSelectedKindChange: (kind: ArtworkKind) => void;
}) {
  const discoveryRequestId = useRef(0);
  const assetRequestId = useRef(0);
  const [artwork, setArtwork] = useState<MachineArtwork | null>(null);
  const [asset, setAsset] = useState<ArtworkAssetPayload | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const id = ++discoveryRequestId.current;
    assetRequestId.current += 1;
    setArtwork(null);
    setAsset(null);
    setLoading(true);
    setError(null);
    void discoverMachineArtwork(machine)
      .then((result) => {
        if (discoveryRequestId.current === id) setArtwork(result);
      })
      .catch((reason: unknown) => {
        if (discoveryRequestId.current === id) setError(String(reason));
      })
      .finally(() => {
        if (discoveryRequestId.current === id) setLoading(false);
      });
  }, [machine]);

  const slot = useMemo(
    () => artwork?.slots.find((candidate) => candidate.kind === selectedKind) ?? null,
    [artwork, selectedKind],
  );

  useEffect(() => {
    const descriptor = slot?.asset;
    const id = ++assetRequestId.current;
    if (!descriptor) {
      setAsset(null);
      return;
    }
    setAsset(null);
    setError(null);
    void readArtworkAsset(descriptor.assetId)
      .then((payload) => {
        if (assetRequestId.current === id) setAsset(payload);
      })
      .catch((reason: unknown) => {
        if (assetRequestId.current === id) setError(String(reason));
      });
  }, [slot]);

  if (loading) return <div className="mame-panel-state">Loading artwork…</div>;
  if (error) {
    return (
      <div className="mame-panel-state" role="alert">
        Artwork unavailable: {error}
      </div>
    );
  }

  return (
    <div className="mame-artwork-pane">
      <div className="mame-artwork-categories" aria-label="Artwork category">
        {(artwork?.slots ?? []).map((candidate) => (
          <button
            key={candidate.kind}
            type="button"
            className={selectedKind === candidate.kind ? "is-selected" : ""}
            aria-pressed={selectedKind === candidate.kind}
            onClick={() => onSelectedKindChange(candidate.kind)}
          >
            {ARTWORK_LABELS[candidate.kind]}
          </button>
        ))}
      </div>
      <div className="mame-artwork-frame">
        {asset ? (
          <img src={asset.dataUrl} alt={`${machine} ${ARTWORK_LABELS[selectedKind]}`} />
        ) : (
          <div className="mame-panel-state">
            No {ARTWORK_LABELS[selectedKind].toLowerCase()} artwork found.
          </div>
        )}
      </div>
    </div>
  );
}

function InfoPane({ detail }: { detail: MachineDetail }) {
  const orientation = detail.displays.length
    ? detail.displays.some((display) => display.rotate === 90 || display.rotate === 270)
      ? "Vertical"
      : "Horizontal"
    : "Unknown";

  return (
    <div className="mame-info-pane">
      <h2>{detail.description}</h2>
      <p className="mame-info-short">{detail.shortName}</p>
      <dl>
        <div>
          <dt>Year</dt>
          <dd>{detail.year ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Manufacturer</dt>
          <dd>{detail.manufacturer ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{machineStatusLabel(detail)}</dd>
        </div>
        <div>
          <dt>Parent</dt>
          <dd>{detail.cloneOf ? (detail.parentDescription ?? detail.cloneOf) : "Parent"}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>{detail.sourceFile ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Save states</dt>
          <dd>{detail.driverSavestate ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Orientation</dt>
          <dd>{orientation}</dd>
        </div>
        <div>
          <dt>Software lists</dt>
          <dd>{detail.softwareLists.length}</dd>
        </div>
      </dl>
      {(detail.driverRequiresArtwork ||
        detail.driverUnofficial ||
        detail.driverNoSoundHardware ||
        detail.driverIncomplete) && (
        <div className="mame-info-flags" aria-label="Driver notes">
          {detail.driverRequiresArtwork && <span>Requires artwork</span>}
          {detail.driverUnofficial && <span>Unofficial</span>}
          {detail.driverNoSoundHardware && <span>No sound hardware</span>}
          {detail.driverIncomplete && <span>Incomplete</span>}
        </div>
      )}
    </div>
  );
}
