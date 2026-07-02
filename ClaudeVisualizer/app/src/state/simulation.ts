// Phase 0 simulated backend.
//
// This module plays the role the Rust aggregator will take over in Phase 1+:
// it produces a ClusterSnapshot ~10×/second with plausible fake data (the same
// behaviors as mockup.html). Nothing outside this file knows the data is fake —
// swapping in the real Tauri Channel later means replacing only this module.

import type {
  ClusterSnapshot,
  FeedEvent,
  SessionSnapshot,
  ToolActivity,
} from "../types/telemetry";
import { clamp, lerp } from "../lib/easing";

export const SNAPSHOT_INTERVAL_MS = 100; // ~10 Hz, matching the planned Channel rate

// ---------------------------------------------------------------------------
// Simulated sessions and tools (verbatim from mockup.html).
// ---------------------------------------------------------------------------

interface SimSession {
  sessionId: string;
  name: string;
  model: string;
  cwd: string;
  busy: boolean;
  contextPercent: number;
  activityPercent: number;
  tool: ToolActivity | null;
}

const simSessions: SimSession[] = [
  { sessionId: "sim-1", name: "playground-71", model: "Opus 4.8",  cwd: "~/Ai/Playground", busy: true,  contextPercent: 41, activityPercent: 78, tool: { toolName: "Edit", argSummary: "dashboard.tsx" } },
  { sessionId: "sim-2", name: "mhit-ops",      model: "Sonnet 5",  cwd: "~/Ai/MHIT/OPS",   busy: true,  contextPercent: 63, activityPercent: 55, tool: { toolName: "Bash", argSummary: "pytest -q" } },
  { sessionId: "sim-3", name: "devotions",     model: "Opus 4.8",  cwd: "~/Ai/Devotions",  busy: true,  contextPercent: 29, activityPercent: 91, tool: { toolName: "WebSearch", argSummary: "liturgy" } },
  { sessionId: "sim-4", name: "coffee-log",    model: "Haiku 4.5", cwd: "~/Ai/coffee",     busy: false, contextPercent: 12, activityPercent: 0,  tool: null },
];

const simTools: ToolActivity[] = [
  { toolName: "Bash", argSummary: "npm run build" },
  { toolName: "Read", argSummary: "statusline.sh" },
  { toolName: "Edit", argSummary: "gauges.rs" },
  { toolName: "Grep", argSummary: "session.id" },
  { toolName: "WebSearch", argSummary: "tauri fs plugin" },
  { toolName: "Write", argSummary: "spec.md" },
  { toolName: "Bash", argSummary: "git commit" },
  { toolName: "Read", argSummary: "monitoring" },
  { toolName: "Task", argSummary: "verify build" },
  { toolName: "WebFetch", argSummary: "v2.tauri.app" },
  { toolName: "Glob", argSummary: "**/*.jsonl" },
  { toolName: "Edit", argSummary: "trace.ts" },
];

// ---------------------------------------------------------------------------
// Mutable simulation state.
// ---------------------------------------------------------------------------

const sim = {
  // Gauge targets (the frontend eases toward these, as it will with real data).
  tachTarget: 0,
  costTarget: 0,
  contextTarget: 40,
  cacheTarget: 80,
  // Odometer accumulators (seeded like the mockup so the strip looks lived-in).
  tokensToday: 4_812_400,
  costTodayUsd: 6.42,
  toolCalls: 1_183,
  linesEdited: 2_740,
  commits: 14,
  appStartMs: Date.now(),
  // Telltales.
  permissionWaiting: true,
  apiError: false,
  rateLimited: false,
  // Timers for the slower-than-snapshot behaviors.
  lastRetargetMs: 0,
  lastSessionShuffleMs: 0,
  lastFeedEventMs: 0,
  lastTelltaleMs: 0,
  nextEventId: 1,
  // Events created since the last snapshot was taken.
  pendingEvents: [] as FeedEvent[],
};

/** Pick new gauge targets — the "engine load" changes every couple of seconds. */
function retargetGauges(): void {
  const heat = Math.random(); // 0 = coasting, 1 = flat out
  sim.tachTarget = lerp(250, 1950, heat * heat);
  sim.costTarget = lerp(3, 34, heat);
  sim.contextTarget = clamp(sim.contextTarget + lerp(-6, 10, Math.random()), 18, 96);
  sim.cacheTarget = lerp(72, 96, Math.random());
}

/** Drift the busy sessions' stats and occasionally switch their current tool. */
function shuffleSessions(): void {
  for (const session of simSessions) {
    if (!session.busy) continue;
    session.activityPercent = Math.floor(clamp(session.activityPercent + (Math.random() - 0.5) * 22, 20, 99));
    session.contextPercent = Math.floor(clamp(session.contextPercent + (Math.random() - 0.45) * 2, 8, 97));
    if (Math.random() < 0.25) {
      session.tool = simTools[Math.floor(Math.random() * simTools.length)];
    }
  }
}

/** Emit one simulated tool_result event and bump the odometers it implies. */
function emitFeedEvent(timeMs: number): void {
  const session = simSessions[Math.floor(Math.random() * simSessions.length)];
  const tool = simTools[Math.floor(Math.random() * simTools.length)];
  const isError = Math.random() < 0.06;
  sim.pendingEvents.push({
    id: sim.nextEventId++,
    timeMs,
    sessionName: session.name,
    toolName: tool.toolName,
    argSummary: tool.argSummary,
    durationMs: isError ? null : Math.floor(30 + Math.random() * 3200),
    isError,
  });
  sim.toolCalls += 1;
  sim.tokensToday += 800 + Math.random() * 4200;
  sim.costTodayUsd += 0.001 + Math.random() * 0.006;
  if (tool.toolName === "Edit" || tool.toolName === "Write") {
    sim.linesEdited += Math.floor(2 + Math.random() * 40);
  }
}

function shuffleTelltales(): void {
  sim.permissionWaiting = Math.random() < 0.55;
  sim.apiError = Math.random() < 0.12;
  sim.rateLimited = Math.random() < 0.08;
}

// ---------------------------------------------------------------------------
// Snapshot production.
// ---------------------------------------------------------------------------

/** Advance the simulation to `now` and build the snapshot the "backend" would push. */
export function takeSnapshot(now: number): ClusterSnapshot {
  // Run each slow behavior on its own cadence (same periods as the mockup).
  if (now - sim.lastRetargetMs >= 2200) { retargetGauges(); sim.lastRetargetMs = now; }
  if (now - sim.lastSessionShuffleMs >= 2000) { shuffleSessions(); sim.lastSessionShuffleMs = now; }
  if (now - sim.lastFeedEventMs >= 1400) { emitFeedEvent(now); sim.lastFeedEventMs = now; }
  if (now - sim.lastTelltaleMs >= 3000) { shuffleTelltales(); sim.lastTelltaleMs = now; }

  const sessions: SessionSnapshot[] = simSessions.map((s, index) => ({
    sessionId: s.sessionId,
    name: s.name,
    model: s.model,
    cwd: s.cwd,
    status: s.busy ? "busy" : "idle",
    contextPercent: s.contextPercent,
    activityPercent: s.activityPercent,
    currentTool: s.busy ? s.tool : null,
    // Pretend each session started progressively earlier (30 min apart).
    startedAtMs: sim.appStartMs - (index + 1) * 30 * 60_000,
    tasksDone: 0,
    tasksTotal: 0,
    activeTaskForm: null,
  }));

  // Per-snapshot jitter makes the trace look like live telemetry rather than
  // a smooth synthetic curve (ratios match the mockup's pushTrace()).
  const jitter = () => Math.random() - 0.5;
  const trace = {
    outputTokensPerSec: clamp(sim.tachTarget + jitter() * 220, 0, 2100),
    inputTokensPerSec: clamp(sim.tachTarget * 0.35 + jitter() * 120, 0, 1400),
    cacheReadTokensPerSec: clamp(sim.tachTarget * 0.9 + jitter() * 260, 0, 2600),
  };

  return {
    generatedAtMs: now,
    host: "jamison-mbp",
    rosterLive: false, // the store flips this when the real registry connects
    sessions,
    gauges: {
      outputTokensPerSec: sim.tachTarget,
      costPerHour: sim.costTarget,
      contextPercent: sim.contextTarget,
      cacheHitPercent: sim.cacheTarget,
    },
    trace,
    feedRows: [], // the store accumulates feed rows (see drainSimFeedEvents)
    odometers: {
      tokensToday: sim.tokensToday,
      costTodayUsd: sim.costTodayUsd,
      toolCalls: sim.toolCalls,
      linesEdited: sim.linesEdited,
      commits: sim.commits,
      appStartMs: sim.appStartMs,
    },
    telltales: {
      telemetryConnected: true,
      permissionWaiting: sim.permissionWaiting,
      apiError: sim.apiError,
      rateLimited: sim.rateLimited,
    },
  };
}

/** Simulated feed events created since the last drain (the store accumulates). */
export function drainSimFeedEvents(): FeedEvent[] {
  const events = sim.pendingEvents;
  sim.pendingEvents = [];
  return events;
}

/** Backfill so the feed isn't empty on first paint (mockup seeds 12 rows). */
export function seedInitialFeed(now: number): void {
  for (let i = 12; i > 0; i--) {
    emitFeedEvent(now - i * (1000 + Math.random() * 4000));
  }
}
