// HUD — top strip: brand, host, session count, clock, and the telltale lamps.
// The whole strip is a Tauri drag region so the window (which has no title bar)
// can still be moved by grabbing the top edge.

import { useEffect, useState } from "react";
import type { Telltales } from "../types/telemetry";

interface HudProps {
  host: string;
  busySessionCount: number;
  totalSessionCount: number;
  telltales: Telltales;
  /** True once the real session registry is streaming (Phase 1). */
  rosterLive: boolean;
}

/** "HH:MM:SS" in local time, same format as the mockup's clock. */
function formatClock(): string {
  return new Date().toTimeString().slice(0, 8);
}

export function Hud({ host, busySessionCount, totalSessionCount, telltales, rosterLive }: HudProps) {
  const [clock, setClock] = useState(formatClock);

  useEffect(() => {
    const intervalId = window.setInterval(() => setClock(formatClock()), 1000);
    return () => window.clearInterval(intervalId);
  }, []);

  return (
    <header className="hud a-hud" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <h1 data-tauri-drag-region>
          Claude<b>Visualizer</b>
        </h1>
        <span className="tag" data-tauri-drag-region>Instrument Cluster</span>
      </div>
      <div className="hud-stat">
        <span className="k">Host</span>
        <span className="v">{host}</span>
      </div>
      <div className="hud-stat">
        <span className="k">Sessions</span>
        <span className="v">
          <em>{busySessionCount}</em> / {totalSessionCount}
        </span>
      </div>
      <div className="hud-stat">
        <span className="k">Clock</span>
        <span className="v">{clock}</span>
      </div>
      <div className="tell-tales">
        <div className={`lamp${telltales.telemetryConnected ? " on-green" : ""}`} title="Telemetry connected">◉</div>
        <div className={`lamp${telltales.permissionWaiting ? " on-amber" : ""}`} title="Permission prompt waiting">!</div>
        <div className={`lamp${telltales.apiError ? " on-red" : ""}`} title="API error">▲</div>
        <div className={`lamp${telltales.rateLimited ? " on-amber" : ""}`} title="Rate limit">↯</div>
      </div>
      <span className="badge">
        {rosterLive ? "Live · Tier A" : "Simulated Data"}
      </span>
    </header>
  );
}
