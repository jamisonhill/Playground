// Trace — the live token-throughput chart, rendered with uPlot (SPEC §5).
//
// A 240-point ring buffer is pushed every ~120ms from the latest snapshot
// (the mockup's cadence) and handed to uPlot via setData. Series are drawn
// cache-read first, then input, then output, so the teal output line sits on
// top exactly like the mockup.

import { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import type { TraceSample } from "../types/telemetry";

const POINT_COUNT = 240;
const PUSH_INTERVAL_MS = 120;
const Y_MAX = 2600; // fixed scale, same as the mockup — keeps the trace stable

const SERIES_COLORS = {
  output: "#14E3CC",
  input: "#B47CFF",
  cacheRead: "#FFB93B",
};

export function Trace({ trace }: { trace: TraceSample }) {
  const containerRef = useRef<HTMLDivElement>(null);

  // The push interval reads the newest sample through a ref so the uPlot
  // instance is created once and never rebuilt on snapshot re-renders.
  const latestSampleRef = useRef(trace);
  latestSampleRef.current = trace;

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Ring buffers. x is just the sample index — uPlot needs an x series but
    // we hide the axis, so the values never show.
    const xs = Array.from({ length: POINT_COUNT }, (_, i) => i);
    const outputBuf = new Array<number>(POINT_COUNT).fill(0);
    const inputBuf = new Array<number>(POINT_COUNT).fill(0);
    const cacheBuf = new Array<number>(POINT_COUNT).fill(0);

    const plot = new uPlot(
      {
        width: Math.max(1, container.clientWidth),
        height: Math.max(1, container.clientHeight),
        padding: [6, 4, 2, 4],
        scales: {
          x: { time: false },
          y: { range: [0, Y_MAX] },
        },
        // No axes: the mockup shows only faint horizontal gridlines, drawn below.
        axes: [{ show: false }, { show: false }],
        legend: { show: false },
        cursor: { show: false },
        series: [
          {},
          { stroke: SERIES_COLORS.cacheRead, width: 2, fill: "rgba(255,185,59,.05)", points: { show: false } },
          { stroke: SERIES_COLORS.input, width: 2, fill: "rgba(180,124,255,.06)", points: { show: false } },
          { stroke: SERIES_COLORS.output, width: 2, fill: "rgba(20,227,204,.10)", points: { show: false } },
        ],
        hooks: {
          // Repaint the mockup's four faint gridlines before each series draw.
          // u.bbox is in device pixels, which is what u.ctx draws in.
          drawClear: [
            (u) => {
              const { left, top, width, height } = u.bbox;
              const ctx = u.ctx;
              ctx.save();
              ctx.strokeStyle = "rgba(30,42,56,.55)";
              ctx.lineWidth = 1;
              for (let i = 1; i < 5; i++) {
                const y = top + (height / 5) * i;
                ctx.beginPath();
                ctx.moveTo(left, y);
                ctx.lineTo(left + width, y);
                ctx.stroke();
              }
              ctx.restore();
            },
          ],
        },
      },
      [xs, cacheBuf, inputBuf, outputBuf],
      container,
    );

    // Push the newest snapshot sample into the ring buffers on a fixed cadence.
    const pushIntervalId = window.setInterval(() => {
      const sample = latestSampleRef.current;
      outputBuf.push(sample.outputTokensPerSec); outputBuf.shift();
      inputBuf.push(sample.inputTokensPerSec); inputBuf.shift();
      cacheBuf.push(sample.cacheReadTokensPerSec); cacheBuf.shift();
      plot.setData([xs, cacheBuf, inputBuf, outputBuf]);
    }, PUSH_INTERVAL_MS);

    // Track the panel's size so the chart reshapes with the window. Guard
    // against 0×0 (the trace panel is display:none on very narrow windows —
    // uPlot would throw on a zero-sized canvas).
    const resizeObserver = new ResizeObserver(() => {
      const w = container.clientWidth;
      const h = container.clientHeight;
      if (w > 0 && h > 0) plot.setSize({ width: w, height: h });
    });
    resizeObserver.observe(container);

    return () => {
      window.clearInterval(pushIntervalId);
      resizeObserver.disconnect();
      plot.destroy();
    };
  }, []);

  return (
    <section className="panel a-trace">
      <div className="panel-head">
        <span className="dot" />
        <h2>Token Throughput</h2>
        <span className="right">live · 60fps</span>
      </div>
      <div className="panel-body">
        <div ref={containerRef} className="trace-plot" />
      </div>
      <div className="trace-legend">
        <span className="leg">
          <i style={{ background: SERIES_COLORS.output }} />output <b>{Math.round(trace.outputTokensPerSec)}</b>
        </span>
        <span className="leg">
          <i style={{ background: SERIES_COLORS.input }} />input <b>{Math.round(trace.inputTokensPerSec)}</b>
        </span>
        <span className="leg">
          <i style={{ background: SERIES_COLORS.cacheRead }} />cache read <b>{Math.round(trace.cacheReadTokensPerSec)}</b> tok/s
        </span>
      </div>
    </section>
  );
}
