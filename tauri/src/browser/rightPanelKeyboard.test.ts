import { describe, expect, it } from "vitest";

import { nextPrimaryRightView } from "./rightPanelKeyboard";

describe("right-panel primary tab keyboard navigation", () => {
  it("moves from Images to Infos with ArrowRight", () => {
    expect(nextPrimaryRightView("images", "ArrowRight")).toBe("info");
  });

  it("moves from Infos to Images with ArrowLeft", () => {
    expect(nextPrimaryRightView("info", "ArrowLeft")).toBe("images");
  });

  it("clamps at Infos for ArrowRight", () => {
    expect(nextPrimaryRightView("info", "ArrowRight")).toBe("info");
  });

  it("returns null at the Images left edge so focus can return to the machine list", () => {
    expect(nextPrimaryRightView("images", "ArrowLeft")).toBeNull();
  });

  it("ignores unrelated keys without changing tabs", () => {
    expect(nextPrimaryRightView("images", "Home")).toBe("images");
    expect(nextPrimaryRightView("info", "Escape")).toBe("info");
  });
});
