// Feed — the diagnostic ticker of tool_result events, newest first.
//
// Snapshots deliver only the events created since the previous snapshot; this
// component accumulates them, deduplicates by id (so React StrictMode's
// double-run of effects can't insert a row twice), and trims the list to what
// fits the panel height so the window never scrolls.

import { useEffect, useRef, useState } from "react";
import type { FeedEvent } from "../types/telemetry";

// Approximate row height in px, used to compute how many rows fit (mockup value).
const ROW_HEIGHT_PX = 27;
// Keep a bit more history than the tallest realistic panel can show, so
// enlarging the window immediately reveals older rows instead of blanks.
const MAX_RETAINED_ROWS = 80;

function formatTime(timeMs: number): string {
  return new Date(timeMs).toTimeString().slice(0, 8);
}

export function Feed({ recentEvents }: { recentEvents: FeedEvent[] }) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [rows, setRows] = useState<FeedEvent[]>([]); // newest first
  const [capacity, setCapacity] = useState(12);
  const lastSeenIdRef = useRef(0);

  // Recompute how many rows fit whenever the panel is resized.
  useEffect(() => {
    const body = bodyRef.current;
    if (!body) return;
    const measure = () =>
      setCapacity(Math.max(4, Math.floor(body.clientHeight / ROW_HEIGHT_PX)));
    measure();
    const resizeObserver = new ResizeObserver(measure);
    resizeObserver.observe(body);
    return () => resizeObserver.disconnect();
  }, []);

  // Fold newly-arrived events into the accumulated list.
  useEffect(() => {
    const fresh = recentEvents.filter((event) => event.id > lastSeenIdRef.current);
    if (fresh.length === 0) return;
    lastSeenIdRef.current = fresh[fresh.length - 1].id;
    setRows((previous) =>
      [...fresh].reverse().concat(previous).slice(0, MAX_RETAINED_ROWS),
    );
  }, [recentEvents]);

  return (
    <section className="panel a-feed">
      <div className="panel-head">
        <span className="dot" />
        <h2>Diagnostic Feed</h2>
        <span className="right">claude_code.tool_result</span>
      </div>
      <div className="panel-body feed-body" ref={bodyRef}>
        {rows.slice(0, capacity).map((event) => (
          <div key={event.id} className={`evt new${event.isError ? " err" : ""}`}>
            <span className="t">{formatTime(event.timeMs)}</span>
            <span className="sess">{event.sessionName}</span>
            <span className="act">
              <b>{event.toolName}</b> <span className="arg">{event.argSummary}</span>
              {event.isError ? " — error" : ""}
            </span>
            <span className="dur">{event.durationMs === null ? "—" : `${event.durationMs}ms`}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
