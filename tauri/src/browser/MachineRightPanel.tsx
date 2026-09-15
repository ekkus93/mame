import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type RefObject,
} from "react";

import {
  discoverMachineArtwork,
  readArtworkAsset,
  type ArtworkAssetPayload,
  type ArtworkKind,
  type MachineArtwork,
} from "../backend/artwork";
import type { LaunchPreferences } from "../backend/generalSettings";
import type { MachineDetail } from "../backend/types";
import { MachineAuditPanel } from "../library/MachineAuditPanel";
import { machineStatusLabel } from "../library/libraryQuery";
import { MachineSettingsPanel } from "../settings/MachineSettingsPanel";

export type MachineRightView = "images" | "info" | "audit" | "settings";

const ARTWORK_LABELS: Record<ArtworkKind, string> = {
  screenshot: "Snapshots",
  cabinet: "Cabinet",
  controlPanel: "Control Panel",
  pcb: "PCB",
  flyer: "Flyer",
  titleScreen: "Title Screen",
  ending: "Ending",
  artworkPreview: "Artwork Preview",
  bosses: "Bosses",
  logo: "Logo",
  versus: "Versus",
  gameOver: "Game Over",
  howTo: "HowTo",
  scores: "Scores",
  select: "Select",
  marquee: "Marquee",
  cover: "Covers",
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
  firstTabRef,
  onNavigateToMachines,
  onSettingsClose,
}: {
  detail: MachineDetail;
  view: MachineRightView;
  onViewChange: (view: MachineRightView) => void;
  artworkKind: ArtworkKind;
  onArtworkKindChange: (kind: ArtworkKind) => void;
  pendingLaunchOverrides: LaunchPreferences | null;
  onPendingLaunchOverridesChanged: (preferences: LaunchPreferences | null) => void;
  onAuditResultChanged: () => void;
  firstTabRef: RefObject<HTMLButtonElement | null>;
  onNavigateToMachines: () => void;
  onSettingsClose: () => void;
}) {
  const handleRegionKey = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      onNavigateToMachines();
    }
  };

  return (
    <aside className="mame-right-panel" aria-label="Selected machine context">
      <div className="mame-right-tabs" role="tablist" aria-label="Machine detail view">
        <button
          ref={firstTabRef}
          type="button"
          role="tab"
          aria-selected={view === "images"}
          className={view === "images" ? "is-selected" : ""}
          onClick={() => onViewChange("images")}
          onKeyDown={handleRegionKey}
        >
          Images
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={view === "info"}
          className={view === "info" ? "is-selected" : ""}
          onClick={() => onViewChange("info")}
          onKeyDown={handleRegionKey}
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
        <div className="mame-context-subview" aria-label="Selected machine configuration">
          <div className="mame-context-subview-toolbar">
            <button type="button" className="secondary-button" autoFocus onClick={onSettingsClose}>
              ← Machine details
            </button>
            <span>Configure {detail.description}</span>
          </div>
          <MachineSettingsPanel
            shortName={detail.shortName}
            pendingLaunchOverrides={pendingLaunchOverrides}
            onPendingLaunchOverridesChanged={onPendingLaunchOverridesChanged}
          />
        </div>
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
    setAsset(null);
    setError(null);
    if (!descriptor) {
      return;
    }
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
          <img
            src={asset.dataUrl}
            alt={`${slot?.asset?.machine ?? machine} ${ARTWORK_LABELS[selectedKind]}`}
          />
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
