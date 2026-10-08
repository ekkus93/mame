import { describe, expect, it } from "vitest";
import { parseGameFrame } from "./frameProtocol";

describe("malformed gameplay frames", () => {
  it("rejects empty and short binary payloads", () => {
    expect(() => parseGameFrame(new ArrayBuffer(0), "session")).toThrow("invalid magic");
    expect(() => parseGameFrame(new ArrayBuffer(51), "session")).toThrow("invalid magic");
  });
});
