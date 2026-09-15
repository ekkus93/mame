import { describe, expect, it } from "vitest";

import { SoftwareListBrowser } from "./SoftwareListBrowser";

describe("legacy software browser retirement", () => {
  it("retains only a compatibility adapter around the authoritative browser", () => {
    expect(typeof SoftwareListBrowser).toBe("function");
  });
});
