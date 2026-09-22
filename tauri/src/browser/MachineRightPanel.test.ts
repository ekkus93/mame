import { describe, expect, it } from "vitest";

import { nextPrimaryRightView } from "./MachineRightPanel";

describe("MAME right-panel tab keyboard model", () => {
  it("moves right from Images to Infos and clamps on Infos", () => {
    expect(nextPrimaryRightView("images", "ArrowRight")).toBe("info");
    expect(nextPrimaryRightView("info", "ArrowRight")).toBe("info");
  });

  it("moves left from Infos to Images and returns null at the left boundary", () => {
    expect(nextPrimaryRightView("info", "ArrowLeft")).toBe("images");
    expect(nextPrimaryRightView("images", "ArrowLeft")).toBeNull();
  });

  it("does not change tabs for unrelated keys", () => {
    expect(nextPrimaryRightView("images", "Enter")).toBe("images");
    expect(nextPrimaryRightView("info", "Escape")).toBe("info");
  });
});
