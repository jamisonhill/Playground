// Keeps a <canvas> backing store matched to its CSS size × devicePixelRatio.
// Ported from mockup.html: the backing store is only reallocated when the CSS
// size actually changes (reallocating every frame would flicker and waste time).

export interface FittedCanvas {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
}

// Cap DPR at 2 — retina sharpness without paying 3×3 pixel cost on future displays.
const devicePixelRatioCapped = Math.min(window.devicePixelRatio || 1, 2);

// Remembers each canvas's last CSS size so we can skip reallocation. WeakMap so
// removed canvases don't leak.
const lastSize = new WeakMap<HTMLCanvasElement, { w: number; h: number }>();

/**
 * Fit the canvas to its current layout size and return a cleared, DPR-scaled
 * 2D context. Returns null if the canvas has no 2D context (effectively never
 * happens in a WebView, but TypeScript requires we handle it) or is not laid
 * out yet (zero size).
 */
export function fitCanvas(canvas: HTMLCanvasElement): FittedCanvas | null {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  const ctx = canvas.getContext("2d");
  if (!ctx || width === 0 || height === 0) return null;

  const previous = lastSize.get(canvas);
  if (!previous || previous.w !== width || previous.h !== height) {
    canvas.width = Math.max(1, Math.round(width * devicePixelRatioCapped));
    canvas.height = Math.max(1, Math.round(height * devicePixelRatioCapped));
    lastSize.set(canvas, { w: width, h: height });
  }

  // Draw in CSS-pixel coordinates; the transform maps them to device pixels.
  ctx.setTransform(devicePixelRatioCapped, 0, 0, devicePixelRatioCapped, 0, 0);
  ctx.clearRect(0, 0, width, height);
  return { ctx, width, height };
}
