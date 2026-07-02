// Telemetry types — the widget ↔ data contract from SPEC §4.
//
// In Phase 0 these are filled by the frontend simulator (src/state/simulation.ts).
// From Phase 1 on, the exact same shape arrives from the Rust aggregator over a
// Tauri Channel, so every widget is already wired to its final field names.

/** Mirrors the `status` field of ~/.claude/sessions/<pid>.json (Tier A1). */
export type SessionStatus = "busy" | "idle";

/** The tool a session is currently running, from transcript `tool_use` blocks (Tier A2). */
export interface ToolActivity {
  /** e.g. "Bash", "Edit", "WebSearch" */
  toolName: string;
  /** Short human-readable summary of the tool input, e.g. "npm run build" */
  argSummary: string;
}

/** One lane in the Concurrent Sessions panel. */
export interface SessionSnapshot {
  /** Stable key — the session UUID from the registry file. */
  sessionId: string;
  /** Display name (registry `name` field), e.g. "playground-71". */
  name: string;
  /** Per-turn model from the transcript, e.g. "Opus 4.8". */
  model: string;
  /** Working directory, shortened for display, e.g. "~/Ai/Playground". */
  cwd: string;
  status: SessionStatus;
  /** Context window used, 0–100 (statusline `used_percentage`, Tier A4). */
  contextPercent: number;
  /** Output tok/s normalized to a rolling machine max, 0–100 (SPEC §3 formulas). */
  activityPercent: number;
  /** Null when the session is idle / between tools. */
  currentTool: ToolActivity | null;
}

/** One row in the Diagnostic Feed (claude_code.tool_result). */
export interface FeedEvent {
  /** Monotonically increasing — lets the feed dedupe events across snapshots. */
  id: number;
  /** Wall-clock time of the event in epoch ms. */
  timeMs: number;
  sessionName: string;
  toolName: string;
  argSummary: string;
  /** Null when the tool errored before a duration was measured. */
  durationMs: number | null;
  isError: boolean;
}

/** Machine-wide targets for the four gauges. The frontend eases needles toward these. */
export interface AggregateGauges {
  /** Tachometer: Σ output tokens/sec across all sessions. */
  outputTokensPerSec: number;
  /** Speedometer: burn rate in $/hour. */
  costPerHour: number;
  /** Fuel: aggregate context-window usage, 0–100. */
  contextPercent: number;
  /** Cache temp: cache-read ratio, 0–100. */
  cacheHitPercent: number;
}

/** Instantaneous per-series rates for the live throughput trace. */
export interface TraceSample {
  outputTokensPerSec: number;
  inputTokensPerSec: number;
  cacheReadTokensPerSec: number;
}

/** Running totals for the odometer strip (today, this machine). */
export interface OdometerTotals {
  tokensToday: number;
  costTodayUsd: number;
  toolCalls: number;
  linesEdited: number;
  commits: number;
  /** Epoch ms when the visualizer started — the Uptime odometer derives from this. */
  appStartMs: number;
}

/** Warning-lamp states for the HUD telltale row. */
export interface Telltales {
  telemetryConnected: boolean;
  permissionWaiting: boolean;
  apiError: boolean;
  rateLimited: boolean;
}

/** The full state pushed to the frontend ~10×/second. */
export interface ClusterSnapshot {
  generatedAtMs: number;
  host: string;
  sessions: SessionSnapshot[];
  gauges: AggregateGauges;
  trace: TraceSample;
  /** Only events created since the previous snapshot (the feed accumulates them). */
  recentEvents: FeedEvent[];
  odometers: OdometerTotals;
  telltales: Telltales;
}
