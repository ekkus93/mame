import { describe, expect, it } from "vitest";

import {
  addUniquePath,
  displayPlatformPath,
  movePath,
  removePath,
} from "./pathConfigurationModel";

describe("path configuration model", () => {
  it("preserves ordering while adding, moving, and removing paths", () => {
    const added = addUniquePath(["/one", "/two"], "/three");
    expect(movePath(added, 2, -1)).toEqual(["/one", "/three", "/two"]);
    expect(removePath(added, 1)).toEqual(["/one", "/three"]);
  });

  it("does not add duplicate encoded paths", () => {
    const encoded = { encoding: "unixBytesHex", data: "726f6dff" };
    const paths = [encoded];
    expect(addUniquePath(paths, { ...encoded })).toBe(paths);
  });

  it("renders encoded paths explicitly instead of applying lossy string conversion", () => {
    expect(displayPlatformPath({ encoding: "unixBytesHex", data: "726f6dff" })).toContain(
      "unixBytesHex",
    );
  });

  it("leaves out-of-range moves unchanged", () => {
    const paths = ["/one", "/two"];
    expect(movePath(paths, 0, -1)).toBe(paths);
    expect(movePath(paths, 1, 1)).toBe(paths);
  });
});
