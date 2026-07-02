// Odometers — the bottom strip of daily running totals (tokens, cost, tool
// calls, lines, commits) plus the visualizer's own uptime.

import type { OdometerTotals } from "../types/telemetry";

const formatCount = (n: number): string => Math.round(n).toLocaleString("en-US");

/** "7m" under an hour, "1.4h" after — the mockup's compact uptime format. */
function formatUptime(appStartMs: number): string {
  const minutes = (Date.now() - appStartMs) / 60_000;
  return minutes < 60
    ? `${Math.max(1, Math.floor(minutes))}m`
    : `${(minutes / 60).toFixed(1)}h`;
}

export function Odometers({ odometers }: { odometers: OdometerTotals }) {
  return (
    <footer className="odos a-odos">
      <div className="odo">
        <span className="ol">Tokens Today</span>
        <span className="ov"><u>{formatCount(odometers.tokensToday)}</u></span>
      </div>
      <div className="odo">
        <span className="ol">Cost Today</span>
        <span className="ov"><small>$</small>{odometers.costTodayUsd.toFixed(2)}</span>
      </div>
      <div className="odo">
        <span className="ol">Tool Calls</span>
        <span className="ov">{formatCount(odometers.toolCalls)}</span>
      </div>
      <div className="odo">
        <span className="ol">Lines Edited</span>
        <span className="ov">{formatCount(odometers.linesEdited)}</span>
      </div>
      <div className="odo">
        <span className="ol">Commits</span>
        <span className="ov">{odometers.commits}</span>
      </div>
      <div className="odo">
        <span className="ol">Uptime</span>
        <span className="ov">{formatUptime(odometers.appStartMs)}</span>
      </div>
    </footer>
  );
}
