import { describe, expect, it } from "vitest";

import {
  acceptFrameSequence,
  frameDisplaySize,
  framePollFailureDecision,
} from "./frameProtocol";

describe("game surface state", () => {
  it("swaps backing dimensions for quarter-turn rotation only", () => {
    expect(frameDisplaySize(320, 240, 0)).toEqual({ width: 320, height: 240 });
    expect(frameDisplaySize(320, 240, 90)).toEqual({ width: 240, height: 320 });
    expect(frameDisplaySize(320, 240, 180)).toEqual({ width: 320, height: 240 });
    expect(frameDisplaySize(320, 240, 270)).toEqual({ width: 240, height: 320 });
  });

  it("accepts only strictly advancing gameplay frame sequences", () => {
    expect(acceptFrameSequence(-1n, 0n)).toBe(0n);
    expect(acceptFrameSequence(7n, 8n)).toBe(8n);
    expect(() => acceptFrameSequence(8n, 8n)).toThrow("did not advance");
    expect(() => acceptFrameSequence(8n, 7n)).toThrow("did not advance");
  });

  it("keeps waiting before the first-frame deadline then reports a distinct timeout", () => {
    const base = {
      errorCode: "MAME_FRAME_NOT_READY",
      errorMessage: "not ready",
      startedAtMs: 1_000,
      firstFrameAtMs: null,
      lastPresentedAtMs: 1_000,
      firstFrameTimeoutMs: 5_000,
      frameStallTimeoutMs: 2_000,
    };
    expect(framePollFailureDecision({ ...base, nowMs: 5_999 })).toBeNull();
    expect(framePollFailureDecision({ ...base, nowMs: 6_000 })).toEqual({
      state: "stalled",
      message: "MAME is running, but no gameplay frame has arrived yet.",
    });
  });

  it("distinguishes a post-start stream stall from the first-frame timeout", () => {
    expect(
      framePollFailureDecision({
        errorCode: "MAME_FRAME_NOT_READY",
        errorMessage: "not ready",
        nowMs: 9_000,
        startedAtMs: 1_000,
        firstFrameAtMs: 2_000,
        lastPresentedAtMs: 7_000,
        firstFrameTimeoutMs: 5_000,
        frameStallTimeoutMs: 2_000,
      }),
    ).toEqual({
      state: "stalled",
      message: "The MAME gameplay frame stream has stalled.",
    });
  });

  it("maps non-waiting frame failures to the error state verbatim", () => {
    expect(
      framePollFailureDecision({
        errorCode: "MAME_FRAME_VERSION_UNSUPPORTED",
        errorMessage: "Unsupported gameplay frame version.",
        nowMs: 2_000,
        startedAtMs: 1_000,
        firstFrameAtMs: null,
        lastPresentedAtMs: 1_000,
        firstFrameTimeoutMs: 5_000,
        frameStallTimeoutMs: 2_000,
      }),
    ).toEqual({
      state: "error",
      message: "Unsupported gameplay frame version.",
    });
  });
});
