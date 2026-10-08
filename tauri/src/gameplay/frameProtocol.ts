export const FRAME_MAGIC = "MTGFRM01";
export const FRAME_FIXED_HEADER_BYTES = 52;
export const FRAME_PROTOCOL_VERSION = 1;
export const MAX_FRAME_DIMENSION = 8192;
export const MAX_FRAME_PAYLOAD = 64 * 1024 * 1024;

const PIXEL_FORMAT_BGRX8888_LE = 1;
const KNOWN_FLAG_MASK = 0b11;
const decoder = new TextDecoder("utf-8", { fatal: true });

export type FrameOrientation = 0 | 90 | 180 | 270;

export type GameFrame = {
  sessionId: string;
  sequence: bigint;
  width: number;
  height: number;
  stride: number;
  payloadBytes: number;
  captureTimestampUs: bigint;
  orientationDegrees: FrameOrientation;
  flipX: boolean;
  flipY: boolean;
  pixelFormat: "bgrx8888Le";
  pixels: Uint8Array;
};

export function hasFrameMagic(data: Uint8Array): boolean {
  return (
    data.length >= FRAME_FIXED_HEADER_BYTES &&
    data[0] === 77 &&
    data[1] === 84 &&
    data[2] === 71 &&
    data[3] === 70 &&
    data[4] === 82 &&
    data[5] === 77 &&
    data[6] === 48 &&
    data[7] === 49
  );
}

export function parseGameFrame(buffer: ArrayBuffer, expectedSessionId: string): GameFrame {
  const data = new Uint8Array(buffer);
  if (!hasFrameMagic(data)) {
    throw new Error("The MAME gameplay frame has an invalid magic value.");
  }

  const view = new DataView(buffer);
  const version = view.getUint16(8, true);
  if (version !== FRAME_PROTOCOL_VERSION) {
    throw new Error(`Unsupported MAME gameplay frame protocol version ${version}.`);
  }

  const headerBytes = view.getUint16(10, true);
  const sequence = view.getBigUint64(12, true);
  const width = view.getUint32(20, true);
  const height = view.getUint32(24, true);
  const stride = view.getUint32(28, true);
  const payloadBytes = view.getUint32(32, true);
  const captureTimestampUs = view.getBigUint64(36, true);
  const orientation = view.getUint16(44, true);
  const flags = view.getUint16(46, true);
  const pixelFormat = view.getUint16(48, true);
  const sessionBytes = view.getUint16(50, true);

  if (width === 0 || height === 0 || width > MAX_FRAME_DIMENSION || height > MAX_FRAME_DIMENSION) {
    throw new Error("The MAME gameplay frame dimensions are outside the supported bounds.");
  }
  const expectedStride = width * 4;
  if (stride !== expectedStride) {
    throw new Error("The MAME gameplay frame stride is invalid.");
  }
  const expectedPayloadBytes = stride * height;
  if (payloadBytes !== expectedPayloadBytes || payloadBytes > MAX_FRAME_PAYLOAD) {
    throw new Error("The MAME gameplay frame payload size is invalid.");
  }
  if (sessionBytes === 0 || sessionBytes > 96) {
    throw new Error("The MAME gameplay frame session identifier length is invalid.");
  }
  if (headerBytes !== FRAME_FIXED_HEADER_BYTES + sessionBytes) {
    throw new Error("The MAME gameplay frame header length is invalid.");
  }
  if (headerBytes + payloadBytes !== data.byteLength) {
    throw new Error("The MAME gameplay frame length does not match its header.");
  }
  if (pixelFormat !== PIXEL_FORMAT_BGRX8888_LE) {
    throw new Error("The MAME gameplay frame pixel format is unsupported.");
  }
  if ((flags & ~KNOWN_FLAG_MASK) !== 0) {
    throw new Error("The MAME gameplay frame contains unsupported flags.");
  }
  if (orientation !== 0 && orientation !== 90 && orientation !== 180 && orientation !== 270) {
    throw new Error("The MAME gameplay frame orientation is unsupported.");
  }

  let sessionId: string;
  try {
    sessionId = decoder.decode(data.subarray(FRAME_FIXED_HEADER_BYTES, headerBytes));
  } catch {
    throw new Error("The MAME gameplay frame session identifier is invalid UTF-8.");
  }
  if (sessionId !== expectedSessionId) {
    throw new Error("The MAME gameplay frame belongs to a stale or different session.");
  }

  return {
    sessionId,
    sequence,
    width,
    height,
    stride,
    payloadBytes,
    captureTimestampUs,
    orientationDegrees: orientation,
    flipX: (flags & 1) !== 0,
    flipY: (flags & 2) !== 0,
    pixelFormat: "bgrx8888Le",
    // The invoke result is an owned binary buffer. Avoid copying the full
    // frame again before the separate BGRX-to-RGBA canvas conversion.
    pixels: data.subarray(headerBytes),
  };
}

export function bgrxToRgba(frame: GameFrame): Uint8ClampedArray<ArrayBuffer> {
  const rgba = new Uint8ClampedArray(new ArrayBuffer(frame.payloadBytes));
  for (let offset = 0; offset < frame.payloadBytes; offset += 4) {
    rgba[offset] = frame.pixels[offset + 2] ?? 0;
    rgba[offset + 1] = frame.pixels[offset + 1] ?? 0;
    rgba[offset + 2] = frame.pixels[offset] ?? 0;
    rgba[offset + 3] = 255;
  }
  return rgba;
}

export type FramePollFailureDecision = {
  state: "stalled" | "unsupported" | "error";
  message: string;
};

export interface FramePollContext {
  errorCode: string | null;
  errorMessage: string;
  nowMs: number;
  startedAtMs: number;
  firstFrameAtMs: number | null;
  lastPresentedAtMs: number;
  firstFrameTimeoutMs: number;
  frameStallTimeoutMs: number;
}

export function frameDisplaySize(width: number, height: number, orientation: FrameOrientation) {
  if (orientation === 90 || orientation === 270) {
    return { width: height, height: width };
  }
  return { width, height };
}

export function acceptFrameSequence(previous: bigint, next: bigint): bigint {
  if (next <= previous) {
    throw new Error("The MAME gameplay frame sequence did not advance.");
  }
  return next;
}

export function framePollFailureDecision(
  context: FramePollContext,
): FramePollFailureDecision | null {
  if (
    context.errorCode === "MAME_FRAME_VERSION_UNSUPPORTED" ||
    context.errorCode === "MAME_FRAME_PIXEL_FORMAT_UNSUPPORTED" ||
    context.errorCode === "MAME_IN_APP_GAMEPLAY_UNSUPPORTED"
  ) {
    return { state: "unsupported", message: context.errorMessage };
  }
  if (context.errorCode !== "MAME_FRAME_NOT_READY") {
    return { state: "error", message: context.errorMessage };
  }
  if (context.firstFrameAtMs === null) {
    if (context.nowMs - context.startedAtMs >= context.firstFrameTimeoutMs) {
      return {
        state: "stalled",
        message: "MAME is running, but no gameplay frame has arrived yet.",
      };
    }
    return null;
  }
  if (context.nowMs - context.lastPresentedAtMs >= context.frameStallTimeoutMs) {
    return {
      state: "stalled",
      message: "The MAME gameplay frame stream has stalled.",
    };
  }
  return null;
}
