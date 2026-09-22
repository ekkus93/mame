import { describe, expect, it } from "vitest";

import browserSource from "./MameBrowser.tsx?raw";
import rightPanelSource from "./MachineRightPanel.tsx?raw";
import softwareSource from "./SoftwareBrowser.tsx?raw";

describe("gameplay input ownership regression guards", () => {
  it("keeps global browser shortcuts gated while gameplay owns input", () => {
    expect(browserSource).toContain("gameplayInputOwned ||");
    expect(browserSource).toContain("softwareMode ||");
    expect(browserSource).toContain("!document.hasFocus()");
  });

  it("passes gameplay ownership into secondary interaction surfaces", () => {
    expect(browserSource).toContain("gameplayInputOwned={gameplayInputOwned}");
    expect(rightPanelSource).toContain("if (gameplayInputOwned) return;");
    expect(softwareSource).toContain(
      "event.defaultPrevented || event.repeat || event.altKey || gameplayInputOwned",
    );
  });
});
