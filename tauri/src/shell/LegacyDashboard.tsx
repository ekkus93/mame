import { BulkAuditPanel } from "../library/BulkAuditPanel";
import { CollectionManager } from "../library/CollectionManager";
import { LibraryBrowser } from "../library/LibraryBrowser";
import { RecentHistoryPanel } from "../library/RecentHistoryPanel";
import { SessionControlPanel } from "../session/SessionControlPanel";
import { DiagnosticsPanel } from "../settings/DiagnosticsPanel";
import { GeneralSettingsPanel } from "../settings/GeneralSettingsPanel";

export function LegacyDashboard({
  availabilityRevision,
  onAvailabilityChanged,
}: {
  availabilityRevision: number;
  onAvailabilityChanged: () => void;
}) {
  return (
    <div className="legacy-dashboard" aria-label="Legacy project tools">
      <p className="legacy-dashboard__notice">
        Temporary migration view. These capabilities are being moved into contextual MAME-style
        surfaces before this dashboard is retired.
      </p>
      <GeneralSettingsPanel onContentPathsChanged={onAvailabilityChanged} />
      <DiagnosticsPanel />
      <SessionControlPanel />
      <BulkAuditPanel onAuditResultsChanged={onAvailabilityChanged} />
      <LibraryBrowser
        availabilityRevision={availabilityRevision}
        onAuditResultsChanged={onAvailabilityChanged}
      />
      <RecentHistoryPanel />
      <CollectionManager />
    </div>
  );
}
