import { useState } from "react";

import type { LaunchPreferences } from "../backend/generalSettings";
import type { MameUiPanelMode } from "../backend/mameUiState";
import type { MachineDetail, SessionSnapshot } from "../backend/types";
import { SoftwareBrowser } from "../browser/SoftwareBrowser";

// MUH-004: compatibility adapter only. The MAME SoftwareBrowser is the single
// authoritative BIOS- and part-aware frontend software launch implementation.
export function SoftwareListBrowser({
  detail,
  launchOverrides,
  onSessionStarted,
}: {
  detail: MachineDetail;
  launchOverrides: LaunchPreferences | null;
  onSessionStarted: (session: SessionSnapshot) => void;
}) {
  const [panelMode, setPanelMode] = useState<MameUiPanelMode>("info");

  return (
    <SoftwareBrowser
      detail={detail}
      gameplayInputOwned={false}
      launchOverrides={launchOverrides}
      panelMode={panelMode}
      onPanelModeChange={setPanelMode}
      onBack={() => undefined}
      onSessionStarted={onSessionStarted}
    />
  );
}
