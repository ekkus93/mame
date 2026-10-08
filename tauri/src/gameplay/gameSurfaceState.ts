import type { FrameOrientation } from "./frameProtocol";

export type FramePollFailureDecision =
  | { state: "stalled"; message: string }
  | { state: "error"; message: string };

export function frameDisplaySize(
  width: number,
  height: number,
  orientation: FrameOrientation,
): { width: number; height: number } {
  return orientation === 90 || orientation === 270
    ? { width: height, height: width }
    : { width, height };
}

export function acceptFrameSequence(previous: bigint, next: bigint): bigint {
  if (next <= previous) {
    throw new Error("The MAME gameplay frame sequence did not advance.");
  }
  return next;
}

export function framePollFailureDecision({
  errorCode,
  errorMessage,
  nowMs,
  startedAtMs,
  firstFrameAtMs,
  lastPresentedAtMs,
  firstFrameTimeoutMs,
  frameStallTimeoutMs,
}: {
  errorCode: string | null;
  errorMessage: string;
  nowMs: number;
  startedAtMs: number;
  firstFrameAtMs: number | null;
  lastPresentedAtMs: number;
  firstFrameTimeoutMs: number;
  frameStallTimeoutMs: number;
}): FramePollFailureDecision | null {
  if (errorCode !== "MAME_FRAME_NOT_READY") {
    return { state: "error", message: errorMessage };
  }
  if (firstFrameAtMs === null) {
    if (nowMs - startedAtMs >= firstFrameTimeoutMs) {
      return {
        state: "stalled",
        message: "MAME is running, but no gameplay frame has arrived yet.",
      };
    }
    return null;
  }
  if (nowMs - lastPresentedAtMs >= frameStallTimeoutMs) {
    return {
      state: "stalled",
      message: "The MAME gameplay frame stream has stalled.",
    };
  }
  return null;
}
