export const FRAME_HEADER_BYTES = 80;
export const FRAME_PROTOCOL_VERSION = 1;
export const MAX_FRAME_PAYLOAD = 32 * 1024 * 1024;

export function hasFrameMagic(data: Uint8Array): boolean {
  return (
    data.length >= FRAME_HEADER_BYTES &&
    data[0] === 77 &&
    data[1] === 70 &&
    data[2] === 82 &&
    data[3] === 77
  );
}
