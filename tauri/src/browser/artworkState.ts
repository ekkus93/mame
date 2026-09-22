export type ArtworkAssetState<T> =
  | { status: "missing" }
  | { status: "loading" }
  | { status: "ready"; asset: T }
  | { status: "error"; message: string };

export function initialArtworkAssetState(hasDescriptor: boolean): ArtworkAssetState<never> {
  return hasDescriptor ? { status: "loading" } : { status: "missing" };
}

export function artworkAssetError(
  reason: unknown,
  format: (reason: unknown) => string,
): ArtworkAssetState<never> {
  return { status: "error", message: format(reason) };
}


export function isCurrentArtworkRequest(currentRequestId: number, responseRequestId: number): boolean {
  return currentRequestId === responseRequestId;
}
