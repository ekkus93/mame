import { describe, expect, it } from "vitest";

import { formatFrameAges } from "./frameMetrics";

describe("frame-metric clock-domain labels", () => {
  it("distinguishes backend receive age and presentation age without claiming latency", () => {
    expect(formatFrameAges({ latestReceivedAgeMs: 15, lastPresentedAgeMs: 25 })).toBe(
      "last recv age 15ms · last present age 25ms",
    );
    expect(formatFrameAges({ latestReceivedAgeMs: null, lastPresentedAgeMs: null })).toBe(
      "last recv age — · last present age —",
    );
  });
});
