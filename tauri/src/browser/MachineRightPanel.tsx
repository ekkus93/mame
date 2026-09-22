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
import { errorMessage } from "../backend/errors";
import type { LaunchPreferences } from "../backend/generalSettings";
import type { MachineDetail } from "../backend/types";
import { MachineAuditPanel } from "../library/MachineAuditPanel";
import { machineStatusLabel } from "../library/libraryQuery";
import { MachineSettingsPanel } from "../settings/MachineSettingsPanel";
import { ArtworkAssetFrame } from "./ArtworkAssetFrame";
import {
  artworkAssetError,
  initialArtworkAssetState,
  isCurrentArtworkRequest,
  type ArtworkAssetState,
} from "./artworkState";
import { nextPrimaryRightView, type PrimaryRightView } from "./rightPanelKeyboard";

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

function panelDataView(view: MachineRightView): string {
  switch (view) {
    case "images":
      return "artwork";
    case "info":
      return "details";
    case "settings":
      return "configure";
    case "audit":
      return "audit";
  }
}

function emptyMachinePanelText(status: "idle" | "loading" | "error", message?: string): string {
  switch (status) {
    case "loading":
      return "Loading machine details…";
    case "error":
      return message ?? "Machine details unavailable.";
    case "idle":
      return "Select a machine.";
  }
}

export function EmptyMachineRightPanel({
  status,
  message,
}: {
  status: "idle" | "loading" | "error";
  message?: string;
}) {
  return (
    <aside className="mame-right-panel" data-view="artwork" aria-label="Selected machine context">
      <div className="mame-right-tabs" role="tablist" aria-label="Machine Images and Infos">
        <button type="button" role="tab" aria-selected="true" className="is-selected">
          Images
        </button>
        <button type="button" role="tab" aria-selected="false" disabled>
          Infos
        </button>
      </div>
      <div className="mame-artwork-pane">
        <div className="mame-artwork-categories" aria-label="Artwork category">
          <button type="button" className="is-selected" aria-pressed="true">
            Snapshots
          </button>
        </div>
        <div className="mame-artwork-frame">
          <div className="mame-no-image-placeholder">
            <strong>No image Available</strong>
            <span>{emptyMachinePanelText(status, message)}</span>
          </div>
        </div>
      </div>
    </aside>
  );
}

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
  gameplayInputOwned,
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
  gameplayInputOwned: boolean;
}) {
  const infoTabRef = useRef<HTMLButtonElement | null>(null);

  const selectPrimaryTab = (next: PrimaryRightView) => {
    onViewChange(next);
    if (next === "images") firstTabRef.current?.focus();
    else infoTabRef.current?.focus();
  };

  const handlePrimaryTabKey = (
    event: ReactKeyboardEvent<HTMLButtonElement>,
    current: PrimaryRightView,
  ) => {
    if (gameplayInputOwned) return;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const next = nextPrimaryRightView(current, event.key);
    if (next) {
      selectPrimaryTab(next);
    } else {
      onNavigateToMachines();
    }
  };

  return (
    <aside
      className="mame-right-panel"
      data-view={panelDataView(view)}
      aria-label="Selected machine context"
    >
      <div className="mame-right-tabs" role="tablist" aria-label="Machine Images and Infos">
        <button
          ref={firstTabRef}
          type="button"
          role="tab"
          aria-selected={view === "images"}
          tabIndex={view === "images" ? 0 : -1}
          className={view === "images" ? "is-selected" : ""}
          onClick={() => onViewChange("images")}
          onKeyDown={(event) => handlePrimaryTabKey(event, "images")}
        >
          Images
        </button>
        <button
          ref={infoTabRef}
          type="button"
          role="tab"
          aria-selected={view === "info"}
          tabIndex={view === "info" ? 0 : -1}
          className={view === "info" ? "is-selected" : ""}
          onClick={() => onViewChange("info")}
          onKeyDown={(event) => handlePrimaryTabKey(event, "info")}
        >
          Infos
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
  const [assetState, setAssetState] = useState<ArtworkAssetState<ArtworkAssetPayload>>({
    status: "missing",
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const id = ++discoveryRequestId.current;
    assetRequestId.current += 1;
    setArtwork(null);
    setAssetState({ status: "missing" });
    setLoading(true);
    setError(null);
    void discoverMachineArtwork(machine)
      .then((result) => {
        if (isCurrentArtworkRequest(discoveryRequestId.current, id)) setArtwork(result);
      })
      .catch((reason: unknown) => {
        if (isCurrentArtworkRequest(discoveryRequestId.current, id)) setError(errorMessage(reason));
      })
      .finally(() => {
        if (isCurrentArtworkRequest(discoveryRequestId.current, id)) setLoading(false);
      });
  }, [machine]);

  const slot = useMemo(
    () => artwork?.slots.find((candidate) => candidate.kind === selectedKind) ?? null,
    [artwork, selectedKind],
  );

  const categoryKinds = useMemo<ArtworkKind[]>(() => {
    const kinds = new Set<ArtworkKind>();
    kinds.add("screenshot");
    for (const candidate of artwork?.slots ?? []) kinds.add(candidate.kind);
    kinds.add(selectedKind);
    return Array.from(kinds);
  }, [artwork, selectedKind]);

  useEffect(() => {
    const descriptor = slot?.asset;
    const id = ++assetRequestId.current;
    setError(null);
    setAssetState(initialArtworkAssetState(Boolean(descriptor)));
    if (!descriptor) return;
    void readArtworkAsset(descriptor.assetId)
      .then((payload) => {
        if (isCurrentArtworkRequest(assetRequestId.current, id)) setAssetState({ status: "ready", asset: payload });
      })
      .catch((reason: unknown) => {
        if (isCurrentArtworkRequest(assetRequestId.current, id)) {
          setAssetState(artworkAssetError(reason, errorMessage));
        }
      });
  }, [slot]);

  if (loading) {
    return (
      <div className="mame-artwork-frame">
        <div className="mame-panel-state">Loading artwork…</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="mame-artwork-frame" role="alert">
        <div className="mame-no-image-placeholder">
          <strong>Artwork unavailable</strong>
          <span>{error}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="mame-artwork-pane">
      <div className="mame-artwork-categories" aria-label="Artwork category">
        {categoryKinds.map((kind) => (
          <button
            key={kind}
            type="button"
            className={selectedKind === kind ? "is-selected" : ""}
            aria-pressed={selectedKind === kind}
            onClick={() => onSelectedKindChange(kind)}
          >
            {ARTWORK_LABELS[kind]}
          </button>
        ))}
      </div>
      <ArtworkAssetFrame
        state={assetState}
        label={ARTWORK_LABELS[selectedKind]}
        machine={slot?.asset?.machine ?? machine}
      />
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
          <dt>Graphics</dt>
          <dd>{detail.driverEmulation ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Sound</dt>
          <dd>{detail.driverNoSoundHardware ? "No sound hardware" : "Driver-reported"}</dd>
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
