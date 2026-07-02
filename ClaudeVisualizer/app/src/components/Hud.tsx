// HUD — top strip: brand, host, session count, clock, telltale lamps, and the
// data-sources popover (Tier B instructions + the opt-in Tier C hook installer).
// The whole strip is a Tauri drag region so the window (which has no title bar)
// can still be moved by grabbing the top edge.

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Telltales } from "../types/telemetry";

interface HudProps {
  host: string;
  busySessionCount: number;
  totalSessionCount: number;
  telltales: Telltales;
  /** True once the real session registry is streaming (Phase 1). */
  rosterLive: boolean;
  /** Tier C hooks present in ~/.claude/settings.json. */
  hooksInstalled: boolean;
}

/** "HH:MM:SS" in local time, same format as the mockup's clock. */
function formatClock(): string {
  return new Date().toTimeString().slice(0, 8);
}

/** The env vars a user sets to point Claude Code's telemetry at us (Tier B). */
const OTLP_ENV_SNIPPET = [
  "export CLAUDE_CODE_ENABLE_TELEMETRY=1",
  "export OTEL_METRICS_EXPORTER=otlp",
  "export OTEL_LOGS_EXPORTER=otlp",
  "export OTEL_EXPORTER_OTLP_PROTOCOL=grpc",
  "export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317",
].join("\n");

function badgeText(rosterLive: boolean, telemetryConnected: boolean, hooksInstalled: boolean): string {
  if (!rosterLive) return "Simulated Data";
  let tiers = "A";
  if (telemetryConnected) tiers += "+B";
  if (hooksInstalled) tiers += "+C";
  return `Live · Tier ${tiers}`;
}

export function Hud({
  host,
  busySessionCount,
  totalSessionCount,
  telltales,
  rosterLive,
  hooksInstalled,
}: HudProps) {
  const [clock, setClock] = useState(formatClock);
  const [popoverOpen, setPopoverOpen] = useState(false);
  const [hookActionError, setHookActionError] = useState<string | null>(null);
  const [hookActionBusy, setHookActionBusy] = useState(false);

  useEffect(() => {
    const intervalId = window.setInterval(() => setClock(formatClock()), 1000);
    return () => window.clearInterval(intervalId);
  }, []);

  /** Install or remove the Tier C hooks — the one write the app ever makes. */
  async function toggleHooks() {
    setHookActionBusy(true);
    setHookActionError(null);
    try {
      await invoke(hooksInstalled ? "uninstall_hooks" : "install_hooks");
      // The badge/status updates on the next snapshot from the backend.
    } catch (error: unknown) {
      // Typical failure: settings.json is malformed JSON (we refuse to touch it).
      setHookActionError(String(error));
    } finally {
      setHookActionBusy(false);
    }
  }

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
        <div className={`lamp${telltales.telemetryConnected ? " on-green" : ""}`} title="OTLP telemetry connected (Tier B)">◉</div>
        <div className={`lamp${telltales.permissionWaiting ? " on-amber" : ""}`} title="Permission prompt waiting (Tier C)">!</div>
        <div className={`lamp${telltales.apiError ? " on-red" : ""}`} title="API error">▲</div>
        <div className={`lamp${telltales.rateLimited ? " on-amber" : ""}`} title="Rate limit">↯</div>
        <button
          className={`lamp gear${popoverOpen ? " on-green" : ""}`}
          title="Data sources"
          onClick={() => setPopoverOpen((open) => !open)}
        >
          ⚙
        </button>
      </div>
      <span className="badge">{badgeText(rosterLive, telltales.telemetryConnected, hooksInstalled)}</span>

      {popoverOpen && (
        <>
          {/* Click anywhere else to dismiss. */}
          <div className="popover-backdrop" onClick={() => setPopoverOpen(false)} />
          <div className="popover">
            <h3>Data Sources</h3>

            <div className="tier">
              <div className="tier-head">
                <span className="tier-dot on" />
                <b>Tier A — Filesystem</b>
                <span className="tier-status">always on</span>
              </div>
              <p>Watches ~/.claude read-only: sessions, transcripts, tasks.</p>
            </div>

            <div className="tier">
              <div className="tier-head">
                <span className={`tier-dot${telltales.telemetryConnected ? " on" : ""}`} />
                <b>Tier B — OpenTelemetry</b>
                <span className="tier-status">
                  {telltales.telemetryConnected ? "connected" : "waiting on :4317"}
                </span>
              </div>
              <p>
                Authoritative cost + error lamps. Launch Claude Code with these
                env vars to enable:
              </p>
              <pre>{OTLP_ENV_SNIPPET}</pre>
            </div>

            <div className="tier">
              <div className="tier-head">
                <span className={`tier-dot${hooksInstalled ? " on" : ""}`} />
                <b>Tier C — Hooks</b>
                <span className="tier-status">{hooksInstalled ? "installed" : "not installed"}</span>
              </div>
              <p>
                Lights the permission telltale the instant Claude waits for
                approval. Installing adds 3 hook entries to
                ~/.claude/settings.json (a backup is created; fully
                reversible). Running sessions pick them up on restart.
              </p>
              <button className="tier-btn" onClick={toggleHooks} disabled={hookActionBusy || !rosterLive}>
                {hookActionBusy ? "…" : hooksInstalled ? "Remove hooks" : "Install hooks"}
              </button>
              {hookActionError && <p className="tier-error">{hookActionError}</p>}
            </div>
          </div>
        </>
      )}
    </header>
  );
}
