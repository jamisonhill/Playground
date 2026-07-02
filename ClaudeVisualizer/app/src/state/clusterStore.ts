// Frontend snapshot store.
//
// Composes each ClusterSnapshot from two layers:
//   1. The simulator — still authoritative for burn rate, context fuel, and
//      odometers (real in Phase 3/4).
//   2. The real telemetry streamed from the Rust backend over a Tauri Channel
//      (Phase 1: roster/host; Phase 2: throughput, cache ratio, current tool,
//      activity %, diagnostic feed). When the channel is up, real values
//      replace the simulated ones.
//
// Outside Tauri (plain `vite` in a browser) the invoke fails and the store
// falls back to full simulation, so the UI is still developable in a browser.

import { useSyncExternalStore } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import type { ClusterSnapshot, FeedEvent, TelemetrySnapshot } from "../types/telemetry";
import {
  SNAPSHOT_INTERVAL_MS,
  drainSimFeedEvents,
  seedInitialFeed,
  takeSnapshot,
} from "./simulation";

/** Keep more rows than the tallest realistic feed panel can show. */
const MAX_FEED_ROWS = 80;

let latestSnapshot: ClusterSnapshot | null = null;
const listeners = new Set<() => void>();

let intervalId: number | null = null;

// Latest payload received from the Rust backend; null until the first message.
let liveTelemetry: TelemetrySnapshot | null = null;
let telemetryChannelStarted = false;

// Accumulated feed rows, newest first. Sim and live event ids are independent
// counters, so the two sources are never mixed: on first live data the sim
// rows are discarded wholesale.
let feedRows: FeedEvent[] = [];
let lastLiveEventId = 0;
let usingLiveFeed = false;

/** Ask the backend to stream telemetry snapshots into a Channel. Called once. */
function connectTelemetryChannel(): void {
  if (telemetryChannelStarted) return;
  telemetryChannelStarted = true;

  const channel = new Channel<TelemetrySnapshot>();
  channel.onmessage = (telemetry) => {
    liveTelemetry = telemetry;
  };
  invoke("subscribe_registry", { channel }).catch((error: unknown) => {
    // Not running inside Tauri, or the command is missing — the UI keeps
    // rendering from the simulator and the badge says "Simulated Data".
    console.warn("Telemetry channel unavailable; showing simulated data.", error);
  });
}

/** Prepend fresh events (oldest→newest in, newest-first out) and cap. */
function prependFeedRows(fresh: FeedEvent[]): void {
  if (fresh.length === 0) return;
  feedRows = [...fresh].reverse().concat(feedRows).slice(0, MAX_FEED_ROWS);
}

/** Merge the simulated base snapshot with whatever real data has arrived. */
function composeSnapshot(): ClusterSnapshot {
  const simulated = takeSnapshot(Date.now());

  if (liveTelemetry === null) {
    prependFeedRows(drainSimFeedEvents());
    return { ...simulated, feedRows };
  }

  // First live data: drop the simulated feed so real rows aren't mixed with
  // fakes (their id counters are unrelated).
  if (!usingLiveFeed) {
    usingLiveFeed = true;
    feedRows = [];
  }
  drainSimFeedEvents(); // keep the simulator's internal queue from growing
  const freshLive = liveTelemetry.recentEvents.filter((e) => e.id > lastLiveEventId);
  if (freshLive.length > 0) {
    lastLiveEventId = freshLive[freshLive.length - 1].id;
    prependFeedRows(freshLive);
  }

  return {
    ...simulated,
    host: liveTelemetry.host,
    sessions: liveTelemetry.sessions,
    rosterLive: true,
    feedRows,
    trace: liveTelemetry.throughput,
    gauges: {
      ...simulated.gauges, // costPerHour + contextPercent stay simulated (Phase 3)
      outputTokensPerSec: liveTelemetry.throughput.outputTokensPerSec,
      cacheHitPercent: liveTelemetry.cacheHitPercent,
    },
  };
}

function startSnapshotLoopIfNeeded(): void {
  if (intervalId !== null) return;
  connectTelemetryChannel();
  seedInitialFeed(Date.now());
  latestSnapshot = composeSnapshot();
  intervalId = window.setInterval(() => {
    latestSnapshot = composeSnapshot();
    for (const listener of listeners) listener();
  }, SNAPSHOT_INTERVAL_MS);
}

function stopSnapshotLoopIfIdle(): void {
  // No components listening (app unmounting) — stop the timer so nothing leaks.
  if (listeners.size === 0 && intervalId !== null) {
    window.clearInterval(intervalId);
    intervalId = null;
  }
}

function subscribe(onChange: () => void): () => void {
  listeners.add(onChange);
  startSnapshotLoopIfNeeded();
  return () => {
    listeners.delete(onChange);
    stopSnapshotLoopIfIdle();
  };
}

function getSnapshot(): ClusterSnapshot {
  // subscribe() always runs before the first read in useSyncExternalStore,
  // but guard anyway: a null here would crash every widget at once.
  if (latestSnapshot === null) {
    startSnapshotLoopIfNeeded();
    latestSnapshot = composeSnapshot();
  }
  return latestSnapshot;
}

/** The one hook components use to read live cluster state (~10 renders/sec). */
export function useClusterSnapshot(): ClusterSnapshot {
  return useSyncExternalStore(subscribe, getSnapshot);
}
