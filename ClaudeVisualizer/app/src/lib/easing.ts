// Small math helpers shared by the animated widgets.

export const clamp = (value: number, min: number, max: number): number =>
  Math.max(min, Math.min(max, value));

export const lerp = (from: number, to: number, t: number): number =>
  from + (to - from) * t;

/**
 * Frame-rate-independent easing factor (ported from mockup.html).
 * Each second, a value eased with this factor closes 99.9% of the gap to its
 * target, and it behaves identically at 30fps and 120fps because the factor
 * is derived from the elapsed time, not the frame count.
 */
export const easeFactor = (deltaSeconds: number): number =>
  1 - Math.pow(0.001, deltaSeconds);
