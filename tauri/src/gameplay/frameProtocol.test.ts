import { describe, expect, it } from "vitest";

import {
  bgrxToRgba,
  FRAME_FIXED_HEADER_BYTES,
  FRAME_PROTOCOL_VERSION,
  parseGameFrame,
} from "./frameProtocol";

const SESSION = "mame-123-1";

function writeUint64(view: DataView, offset: number, value: bigint) {
  view.setBigUint64(offset, value, true);
}

function makeFrame(overrides: { sessionId?: string; orientation?: number; flags?: number } = {}) {
  const sessionId = overrides.sessionId ?? SESSION;
  const session = new TextEncoder().encode(sessionId);
  const width = 2;
  const height = 1;
  const stride = width * 4;
  const payloadBytes = stride * height;
  const headerBytes = FRAME_FIXED_HEADER_BYTES + session.length;
  const buffer = new ArrayBuffer(headerBytes + payloadBytes);
  const bytes = new Uint8Array(buffer);
  bytes.set(new TextEncoder().encode("MTGFRM01"), 0);
  const view = new DataView(buffer);
  view.setUint16(8, FRAME_PROTOCOL_VERSION, true);
  view.setUint16(10, headerBytes, true);
  writeUint64(view, 12, 7n);
  view.setUint32(20, width, true);
  view.setUint32(24, height, true);
  view.setUint32(28, stride, true);
  view.setUint32(32, payloadBytes, true);
  writeUint64(view, 36, 42n);
  view.setUint16(44, overrides.orientation ?? 90, true);
  view.setUint16(46, overrides.flags ?? 1, true);
  view.setUint16(48, 1, true);
  view.setUint16(50, session.length, true);
  bytes.set(session, FRAME_FIXED_HEADER_BYTES);
  bytes.set([3, 2, 1, 0, 30, 20, 10, 0], headerBytes);
  return buffer;
}

describe("gameplay frame protocol", () => {
  it("parses the Rust client frame header and session identity", () => {
    const frame = parseGameFrame(makeFrame(), SESSION);
    expect(frame.sequence).toBe(7n);
    expect(frame.width).toBe(2);
    expect(frame.height).toBe(1);
    expect(frame.orientationDegrees).toBe(90);
    expect(frame.flipX).toBe(true);
    expect(frame.flipY).toBe(false);
    expect(Array.from(frame.pixels)).toEqual([3, 2, 1, 0, 30, 20, 10, 0]);
  });

  it("views frame pixels without allocating another full-frame copy", () => {
    const buffer = makeFrame();
    const frame = parseGameFrame(buffer, SESSION);
    expect(frame.pixels.buffer).toBe(buffer);
    expect(frame.pixels.byteOffset).toBe(FRAME_FIXED_HEADER_BYTES + SESSION.length);
  });

  it("converts little-endian packed RGB bytes to opaque RGBA", () => {
    const rgba = bgrxToRgba(parseGameFrame(makeFrame(), SESSION));
    expect(Array.from(rgba)).toEqual([1, 2, 3, 255, 10, 20, 30, 255]);
  });

  it("rejects stale sessions and unsupported orientation", () => {
    expect(() => parseGameFrame(makeFrame({ sessionId: "other" }), SESSION)).toThrow(
      "stale or different session",
    );
    expect(() => parseGameFrame(makeFrame({ orientation: 45 }), SESSION)).toThrow(
      "orientation is unsupported",
    );
  });

  it("rejects empty session identity and inconsistent payload length", () => {
    const emptySession = makeFrame();
    new DataView(emptySession).setUint16(50, 0, true);
    expect(() => parseGameFrame(emptySession, SESSION)).toThrow(
      "session identifier length is invalid",
    );

    const badPayload = makeFrame();
    new DataView(badPayload).setUint32(32, 9, true);
    expect(() => parseGameFrame(badPayload, SESSION)).toThrow("payload size is invalid");
  });

  it("rejects unknown frame flags", () => {
    expect(() => parseGameFrame(makeFrame({ flags: 4 }), SESSION)).toThrow("unsupported flags");
  });
});
