import { useState } from "react";

import type { LaunchPreferences } from "../backend/generalSettings";
import type { MameUiPanelMode } from "../backend/mameUiState";
import type { MachineDetail, SessionSnapshot } from "../backend/types";
import { SoftwareBrowser } from "../browser/SoftwareBrowser";

/**
 * Compatibility surface for callers that still render the pre-MameShell software-list browser.
 *
 * Software querying and launch semantics are intentionally delegated to the authoritative
 * `browser/SoftwareBrowser` implementation so this compatibility component cannot bypass
 * part selection, BIOS validation/override semantics, or the typed Start Empty path.
 */
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
