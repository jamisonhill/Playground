// Feed — the diagnostic ticker of tool_result events, newest first.
//
// The store hands us already-accumulated, already-deduplicated rows; this
// component only measures how many fit the panel height so the window never
// scrolls, and renders that many.

import { useEffect, useRef, useState } from "react";
import type { FeedEvent } from "../types/telemetry";

// Approximate row height in px, used to compute how many rows fit (mockup value).
const ROW_HEIGHT_PX = 27;

function formatTime(timeMs: number): string {
  return new Date(timeMs).toTimeString().slice(0, 8);
}

export function Feed({ rows }: { rows: FeedEvent[] }) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [capacity, setCapacity] = useState(12);

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
