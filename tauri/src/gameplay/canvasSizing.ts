/** Canvas width/height setters clear the bitmap and reset its 2D context. */
export function ensureCanvasBackingSize(
  canvas: Pick<HTMLCanvasElement, "width" | "height">,
  width: number,
  height: number,
): void {
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
}
