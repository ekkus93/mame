import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { ArtworkAssetFrame } from "./ArtworkAssetFrame";

describe("ArtworkAssetFrame", () => {
  it("renders loading without claiming artwork is missing", () => {
    const html = renderToStaticMarkup(
      <ArtworkAssetFrame state={{ status: "loading" }} label="Snapshots" machine="pacman" />,
    );
    expect(html).toContain("Loading Snapshots");
    expect(html).not.toContain("No image Available");
  });

  it("renders a true missing-artwork state separately", () => {
    const html = renderToStaticMarkup(
      <ArtworkAssetFrame state={{ status: "missing" }} label="Cabinet" machine="pacman" />,
    );
    expect(html).toContain("No image Available");
    expect(html).toContain("Cabinet");
    expect(html).not.toContain("Loading Cabinet");
  });

  it("renders formatted artwork errors inside an alert", () => {
    const html = renderToStaticMarkup(
      <ArtworkAssetFrame
        state={{ status: "error", message: "formatted artwork failure" }}
        label="Snapshots"
        machine="pacman"
      />,
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain("formatted artwork failure");
    expect(html).not.toContain("No image Available");
  });
});
