// GaugeCluster — the 2×2 grid of canvas gauges (tach, burn rate, fuel, cache).
//
// The snapshot delivers *targets* ~10×/second; a requestAnimationFrame loop
// eases the displayed needle values toward them every frame (SPEC §3). Because
// the gauges redraw every frame anyway, window resizes are picked up naturally
// by fitCanvas() — no separate resize handling needed.

import { useEffect, useRef } from "react";
import type { AggregateGauges } from "../types/telemetry";
import { drawGauge } from "../lib/drawGauge";
import { easeFactor, lerp } from "../lib/easing";
import { useReducedMotion } from "../hooks/useReducedMotion";

export function GaugeCluster({ gauges }: { gauges: AggregateGauges }) {
  const tachCanvasRef = useRef<HTMLCanvasElement>(null);
  const costCanvasRef = useRef<HTMLCanvasElement>(null);
  const contextCanvasRef = useRef<HTMLCanvasElement>(null);
  const cacheCanvasRef = useRef<HTMLCanvasElement>(null);

  // Refs let the long-lived rAF loop read the latest props without the effect
  // re-running (and resetting the needles) on every 10 Hz snapshot.
  const targetsRef = useRef(gauges);
  targetsRef.current = gauges;
  const reducedMotion = useReducedMotion();
  const reducedMotionRef = useRef(reducedMotion);
  reducedMotionRef.current = reducedMotion;

  useEffect(() => {
    // Needles start at rest and sweep up to the live values on launch.
    const displayed = { tach: 0, cost: 0, context: 0, cache: 0 };
    let lastFrameMs = performance.now();
    let rafId = 0;

    const frame = (nowMs: number) => {
      // Cap dt at 50ms so a background-tab pause doesn't make needles jump wildly.
      const deltaSeconds = Math.min(0.05, (nowMs - lastFrameMs) / 1000);
      lastFrameMs = nowMs;

      // Reduced motion: factor 1 = needles snap straight to the target.
      const k = reducedMotionRef.current ? 1 : easeFactor(deltaSeconds);
      const targets = targetsRef.current;
      displayed.tach = lerp(displayed.tach, targets.outputTokensPerSec, k);
      displayed.cost = lerp(displayed.cost, targets.costPerHour, k);
      displayed.context = lerp(displayed.context, targets.contextPercent, k);
      displayed.cache = lerp(displayed.cache, targets.cacheHitPercent, k);

      // Gauge ranges/redlines are the mockup's exact dial calibrations.
      if (tachCanvasRef.current) {
        drawGauge(tachCanvasRef.current, {
          value: displayed.tach, min: 0, max: 2200, redline: 1800,
          color: "#14E3CC", gradientColor: "#8affef", ticks: 10,
          display: (displayed.tach / 1000).toFixed(2), unit: "k tok/s", label: "Throughput",
        });
      }
      if (costCanvasRef.current) {
        drawGauge(costCanvasRef.current, {
          value: displayed.cost, min: 0, max: 40, redline: 32,
          color: "#FFB93B", gradientColor: "#ffd98a", ticks: 8,
          display: "$" + displayed.cost.toFixed(1), unit: "per hour", label: "Burn Rate",
        });
      }
      if (contextCanvasRef.current) {
        drawGauge(contextCanvasRef.current, {
          value: displayed.context, min: 0, max: 100, redline: 85,
          // Fuel goes red past the redline — context exhaustion is the warning state.
          color: displayed.context > 85 ? "#FF4D57" : "#37D67A", gradientColor: "#9dffc6", ticks: 10,
          display: Math.round(displayed.context) + "%", unit: "context", label: "Fuel",
        });
      }
      if (cacheCanvasRef.current) {
        drawGauge(cacheCanvasRef.current, {
          value: displayed.cache, min: 0, max: 100,
          color: "#14E3CC", gradientColor: "#8affef", ticks: 10,
          display: Math.round(displayed.cache) + "%", unit: "cache hit", label: "Cache Temp",
        });
      }

      rafId = requestAnimationFrame(frame);
    };

    rafId = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(rafId);
  }, []);

  return (
    <section className="panel a-gauges">
      <div className="panel-head">
        <span className="dot" />
        <h2>Aggregate Telemetry</h2>
        <span className="right">com.anthropic.claude_code</span>
      </div>
      <div className="panel-body">
        <div className="gauges">
          <div className="gcell"><canvas ref={tachCanvasRef} /></div>
          <div className="gcell"><canvas ref={costCanvasRef} /></div>
          <div className="gcell"><canvas ref={contextCanvasRef} /></div>
          <div className="gcell"><canvas ref={cacheCanvasRef} /></div>
        </div>
      </div>
    </section>
  );
}
