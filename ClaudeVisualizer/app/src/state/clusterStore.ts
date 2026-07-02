// Frontend snapshot store.
//
// Composes each ClusterSnapshot from two layers:
//   1. The simulator (gauges, trace, feed, odometers — real in Phase 2/3).
//   2. The real session roster streamed from the Rust registry watcher over a
//      Tauri Channel (Phase 1). When the channel is up, real sessions and host
//      replace the simulated ones; everything else stays simulated for now.
//
// Outside Tauri (plain `vite` in a browser) the invoke fails and the store
// falls back to full simulation, so the UI is still developable in a browser.

import { useSyncExternalStore } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import type { ClusterSnapshot, RegistrySnapshot } from "../types/telemetry";
import { SNAPSHOT_INTERVAL_MS, seedInitialFeed, takeSnapshot } from "./simulation";

let latestSnapshot: ClusterSnapshot | null = null;
const listeners = new Set<() => void>();

let intervalId: number | null = null;

// Latest roster received from the Rust backend; null until the first message.
let liveRegistry: RegistrySnapshot | null = null;
let registryChannelStarted = false;

/** Ask the backend to stream registry snapshots into a Channel. Called once. */
function connectRegistryChannel(): void {
  if (registryChannelStarted) return;
  registryChannelStarted = true;

  const channel = new Channel<RegistrySnapshot>();
  channel.onmessage = (registry) => {
    liveRegistry = registry;
  };
  invoke("subscribe_registry", { channel }).catch((error: unknown) => {
    // Not running inside Tauri, or the command is missing — the UI keeps
    // rendering from the simulator and the badge says "Simulated Data".
    console.warn("Registry channel unavailable; showing simulated sessions.", error);
  });
}

/** Merge the simulated base snapshot with whatever real data has arrived. */
function composeSnapshot(): ClusterSnapshot {
  const simulated = takeSnapshot(Date.now());
  if (liveRegistry === null) return simulated;
  return {
    ...simulated,
    host: liveRegistry.host,
    sessions: liveRegistry.sessions,
    rosterLive: true,
  };
}

function startSnapshotLoopIfNeeded(): void {
  if (intervalId !== null) return;
  connectRegistryChannel();
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
