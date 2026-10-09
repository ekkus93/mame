import { describe, expect, it } from "vitest";

import { ensureCanvasBackingSize } from "./canvasSizing";

class TrackingCanvas {
  private currentWidth: number;
  private currentHeight: number;
  resets = 0;

  constructor(width: number, height: number) {
    this.currentWidth = width;
    this.currentHeight = height;
  }

  get width(): number {
    return this.currentWidth;
  }

  set width(value: number) {
    this.resets++;
    this.currentWidth = value;
  }

  get height(): number {
    return this.currentHeight;
  }

  set height(value: number) {
    this.resets++;
    this.currentHeight = value;
  }
}

describe("gameplay canvas backing dimensions", () => {
  it("does not clear the backing canvas on every unchanged-size frame", () => {
    const canvas = new TrackingCanvas(320, 240);
    for (let frame = 0; frame < 120; frame++) {
      ensureCanvasBackingSize(canvas, 320, 240);
    }
    expect(canvas.resets).toBe(0);
  });

  it("resizes on dimension or rotation changes, then reuses that size", () => {
    const canvas = new TrackingCanvas(320, 240);
    ensureCanvasBackingSize(canvas, 240, 320);
    expect(canvas.resets).toBe(2);
    ensureCanvasBackingSize(canvas, 240, 320);
    expect(canvas.resets).toBe(2);
    ensureCanvasBackingSize(canvas, 240, 400);
    expect(canvas.resets).toBe(3);
    expect(canvas.width).toBe(240);
    expect(canvas.height).toBe(400);
  });
});
