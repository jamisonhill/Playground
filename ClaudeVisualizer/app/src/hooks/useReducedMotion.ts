// Tracks the OS "Reduce Motion" accessibility setting (SPEC §6: needles snap,
// no ticker scroll or pulsing when this is on). Live-updates if the user
// toggles the setting while the app is open.

import { useSyncExternalStore } from "react";

const query = window.matchMedia("(prefers-reduced-motion: reduce)");

function subscribe(onChange: () => void): () => void {
  query.addEventListener("change", onChange);
  return () => query.removeEventListener("change", onChange);
}

export function useReducedMotion(): boolean {
  return useSyncExternalStore(subscribe, () => query.matches);
}
