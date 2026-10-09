import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const source = readFileSync(fileURLToPath(new URL("./GameSurface.tsx", import.meta.url)), "utf8");

describe("GameSurface session-owned input lifecycle", () => {
  it("does not recreate destructive input cleanup when only controller preference changes", () => {
    expect(source).toContain("preferredGamepadIdRef.current = preferredGamepadId");
    expect(source).toContain("preferredGamepadIdRef.current,");
    expect(source).toContain("}, [session.sessionId]);");
    expect(source).not.toContain("[preferredGamepadId, session.sessionId, session.state]");
  });
});
