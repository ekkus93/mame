import type { SessionSnapshot } from "../backend/types";
import type { StoredSaveStateRecord } from "../backend/saveStateRecords";

export type SaveStateCompatibility = {
  loadable: boolean;
  tone: "neutral" | "warning" | "error";
  message: string;
};

type SessionWithExecutable = SessionSnapshot & {
  executable?: {
    version?: string;
    build?: string | null;
    rawVersionLine?: string;
  };
};

export function saveStateCompatibility(
  item: StoredSaveStateRecord,
  session: SessionSnapshot | null,
): SaveStateCompatibility {
  if (!item.filePresent) {
    return {
      loadable: false,
      tone: "error",
      message: "State file is missing from application storage.",
    };
  }
  if (!session || session.state !== "running") {
    return {
      loadable: false,
      tone: "neutral",
      message: "Start the matching machine before loading this state.",
    };
  }
  if (item.record.machine !== session.machine || item.record.software !== session.software) {
    return {
      loadable: false,
      tone: "error",
      message: "This state belongs to a different machine or software context.",
    };
  }

  const executable = (session as SessionWithExecutable).executable;
  if (executable?.rawVersionLine && executable.rawVersionLine !== item.record.mame.rawVersionLine) {
    return {
      loadable: true,
      tone: "warning",
      message:
        "Saved by a different MAME build. Compatibility is unverified; loading will run the structural compatibility probe first.",
    };
  }

  return {
    loadable: true,
    tone: "neutral",
    message: "Compatibility is validated at load time before MAME restores the state.",
  };
}
