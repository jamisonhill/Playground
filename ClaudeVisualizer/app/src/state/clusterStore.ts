// Frontend snapshot store.
//
// Holds the latest ClusterSnapshot and notifies React via useSyncExternalStore.
// In Phase 0 the snapshots come from the local simulator; in Phase 1+ the
// subscribe() body swaps to listening on a Tauri Channel — the components
// never change.

import { useSyncExternalStore } from "react";
import type { ClusterSnapshot } from "../types/telemetry";
import { SNAPSHOT_INTERVAL_MS, seedInitialFeed, takeSnapshot } from "./simulation";

let latestSnapshot: ClusterSnapshot | null = null;
const listeners = new Set<() => void>();

let intervalId: number | null = null;

function startSimulationIfNeeded(): void {
  if (intervalId !== null) return;
  seedInitialFeed(Date.now());
  latestSnapshot = takeSnapshot(Date.now());
  intervalId = window.setInterval(() => {
    latestSnapshot = takeSnapshot(Date.now());
    for (const listener of listeners) listener();
  }, SNAPSHOT_INTERVAL_MS);
}

function stopSimulationIfIdle(): void {
  // No components listening (app unmounting) — stop the timer so nothing leaks.
  if (listeners.size === 0 && intervalId !== null) {
    window.clearInterval(intervalId);
    intervalId = null;
  }
}

function subscribe(onChange: () => void): () => void {
  listeners.add(onChange);
  startSimulationIfNeeded();
  return () => {
    listeners.delete(onChange);
    stopSimulationIfIdle();
  };
}

function getSnapshot(): ClusterSnapshot {
  // subscribe() always runs before the first read in useSyncExternalStore,
  // but guard anyway: a null here would crash every widget at once.
  if (latestSnapshot === null) {
    startSimulationIfNeeded();
    latestSnapshot = takeSnapshot(Date.now());
  }
  return latestSnapshot;
}

/** The one hook components use to read live cluster state (~10 renders/sec). */
export function useClusterSnapshot(): ClusterSnapshot {
  return useSyncExternalStore(subscribe, getSnapshot);
}
