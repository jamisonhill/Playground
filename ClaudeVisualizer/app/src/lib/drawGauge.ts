// Canvas gauge renderer — a faithful port of drawGauge() from mockup.html.
// Every measurement is proportional to the computed radius, so the same code
// draws a crisp gauge at any panel size (hard requirement: the window reshapes
// without scrolling, and the gauges must scale with it).

import { clamp, lerp } from "./easing";
import { fitCanvas } from "./canvasFit";

const TAU = Math.PI * 2;

// Sweep of the dial: from ~7 o'clock, clockwise 280°, to ~5 o'clock —
// the classic automotive gauge arc (same constants as the mockup).
const ARC_START = Math.PI * 0.72;
const ARC_END = Math.PI * 2.28;

export interface GaugeOptions {
  /** Current (already-eased) value to point the needle at. */
  value: number;
  min: number;
  max: number;
  /** Start of the red zone, in gauge units. Omit for gauges with no redline. */
  redline?: number;
  /** Main dial color, e.g. "#14E3CC". */
  color: string;
  /** Lighter gradient partner for the value arc. Falls back to `color`. */
  gradientColor?: string;
  /** Number of tick intervals around the dial. */
  ticks?: number;
  /** Pre-formatted readout text, e.g. "1.35" or "$12.4". */
  display: string;
  /** Small unit caption under the readout, e.g. "k tok/s". */
  unit?: string;
  /** Gauge title above the dial, e.g. "Throughput". */
  label: string;
}

export function drawGauge(canvas: HTMLCanvasElement, options: GaugeOptions): void {
  const fitted = fitCanvas(canvas);
  if (!fitted) return; // canvas not laid out yet — skip this frame, retry next
  const { ctx, width, height } = fitted;

  const centerX = width / 2;
  const centerY = height * 0.54; // slightly below center: leaves room for the label above
  const radius = Math.min(width * 0.44, height * 0.46);
  if (radius < 12) return; // too small to draw legibly (mid-resize squeeze)

  const fraction = clamp((options.value - options.min) / (options.max - options.min), 0, 1);
  const needleAngle = lerp(ARC_START, ARC_END, fraction);
  const ringWidth = Math.max(4, radius * 0.11);
  ctx.lineCap = "round";

  // Base ring (unlit dial track).
  ctx.lineWidth = ringWidth;
  ctx.strokeStyle = "#0c141d";
  ctx.beginPath();
  ctx.arc(centerX, centerY, radius, ARC_START, ARC_END);
  ctx.stroke();

  // Red zone overlay from the redline to the end of the dial.
  if (options.redline != null) {
    const redFraction = clamp((options.redline - options.min) / (options.max - options.min), 0, 1);
    ctx.strokeStyle = "rgba(255,77,87,.5)";
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, lerp(ARC_START, ARC_END, redFraction), ARC_END);
    ctx.stroke();
  }

  // Tick marks: major every other tick, sized relative to the ring.
  const tickCount = options.ticks ?? 10;
  for (let i = 0; i <= tickCount; i++) {
    const angle = lerp(ARC_START, ARC_END, i / tickCount);
    const isMajor = i % 2 === 0;
    const outer = radius - ringWidth * 0.9;
    const inner = radius - ringWidth * 1.7;
    ctx.strokeStyle = isMajor ? "#46586c" : "#28323f";
    ctx.lineWidth = isMajor ? 1.6 : 1;
    ctx.beginPath();
    ctx.moveTo(centerX + Math.cos(angle) * outer, centerY + Math.sin(angle) * outer);
    ctx.lineTo(centerX + Math.cos(angle) * inner, centerY + Math.sin(angle) * inner);
    ctx.stroke();
  }

  // Lit value arc with a vertical gradient and a soft glow.
  const gradient = ctx.createLinearGradient(0, centerY - radius, 0, centerY + radius);
  gradient.addColorStop(0, options.gradientColor ?? options.color);
  gradient.addColorStop(1, options.color);
  ctx.strokeStyle = gradient;
  ctx.lineWidth = ringWidth;
  ctx.shadowColor = options.color;
  ctx.shadowBlur = radius * 0.14;
  ctx.beginPath();
  ctx.arc(centerX, centerY, radius, ARC_START, needleAngle);
  ctx.stroke();
  ctx.shadowBlur = 0;

  // Needle: a bright line from just behind the hub out to the dial.
  const needleTipX = centerX + Math.cos(needleAngle) * (radius - ringWidth * 1.3);
  const needleTipY = centerY + Math.sin(needleAngle) * (radius - ringWidth * 1.3);
  ctx.strokeStyle = "#eef4f8";
  ctx.lineWidth = Math.max(1.6, radius * 0.028);
  ctx.shadowColor = options.color;
  ctx.shadowBlur = radius * 0.08;
  ctx.beginPath();
  ctx.moveTo(centerX - Math.cos(needleAngle) * radius * 0.15, centerY - Math.sin(needleAngle) * radius * 0.15);
  ctx.lineTo(needleTipX, needleTipY);
  ctx.stroke();
  ctx.shadowBlur = 0;

  // Hub: dark disc, colored rim, colored center dot.
  ctx.fillStyle = "#1a2431";
  ctx.beginPath();
  ctx.arc(centerX, centerY, radius * 0.1, 0, TAU);
  ctx.fill();
  ctx.strokeStyle = options.color;
  ctx.lineWidth = Math.max(1.4, radius * 0.02);
  ctx.stroke();
  ctx.fillStyle = options.color;
  ctx.beginPath();
  ctx.arc(centerX, centerY, radius * 0.032, 0, TAU);
  ctx.fill();

  // Readout, unit caption, and title — all font sizes scale with the radius.
  ctx.textAlign = "center";
  ctx.fillStyle = "#e9f0f6";
  ctx.font = `600 ${(radius * 0.42).toFixed(1)}px "SF Mono", Menlo, monospace`;
  ctx.fillText(options.display, centerX, centerY + radius * 0.66);
  ctx.fillStyle = "#7c8ba0";
  ctx.font = `600 ${(radius * 0.135).toFixed(1)}px "Avenir Next Condensed", sans-serif`;
  ctx.fillText((options.unit ?? "").toUpperCase().split("").join(" "), centerX, centerY + radius * 0.9);
  ctx.fillStyle = "#566372";
  ctx.font = `600 ${(radius * 0.145).toFixed(1)}px "Avenir Next Condensed", sans-serif`;
  ctx.fillText(options.label.toUpperCase().split("").join(" "), centerX, centerY - radius * 0.82);
}
