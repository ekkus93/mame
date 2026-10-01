import { describe, expect, it } from "vitest";

import { errorMessage } from "./errors";

describe("backend error presentation", () => {
  it("shows actionable missing-content copy instead of secondary runtime-control details", () => {
    const message = errorMessage({
      code: "MAME_CONTENT_LAUNCH_FAILED",
      message:
        "MAME exited before startup because required ROM/content is missing or incorrect. Check the configured content paths and audit this machine again.",
      details: {
        runtimeControlCode: "CONTROL_CHANNEL_CLOSED",
        earlyExit: true,
        contentFailure: true,
      },
      retryable: false,
    });

    expect(message).toContain("required ROM/content is missing or incorrect");
    expect(message).toContain("audit this machine again");
    expect(message).not.toContain("CONTROL_CHANNEL_CLOSED");
  });
});
