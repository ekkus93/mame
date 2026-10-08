import type { SessionState } from "../backend/types";

/**
 * A library return must not hide a supervised child that is still running.
 * During startup/teardown the session cannot safely accept the stop command.
 */
export function libraryReturnAction(
  view: string,
  sessionState: SessionState | null,
): "navigate" | "stop" | "wait" {
  if (view !== "session" || sessionState === null) return "navigate";
  if (sessionState === "running") return "stop";
  if (sessionState === "created" || sessionState === "starting" || sessionState === "stopping") {
    return "wait";
  }
  return "navigate";
}
