// SessionLanes — the Concurrent Sessions roster. One lane per live session,
// LED + rail lit when busy, showing model, cwd, current tool, and Ctx/Act chips.
// Renders the SPEC §6 empty state when no sessions are running.

import type { SessionSnapshot } from "../types/telemetry";

/** Chip color escalates as the percentage climbs (teal → amber → red). */
function chipColor(percent: number): string {
  if (percent > 85) return "var(--red)";
  if (percent > 65) return "var(--amber)";
  return "var(--teal)";
}

function Lane({ session }: { session: SessionSnapshot }) {
  const isBusy = session.status === "busy";
  return (
    <div className={`lane ${isBusy ? "busy" : "idle"}`}>
      <span className="rail" />
      <span className={`led ${isBusy ? "busy" : "idle"}`} />
      <div className="idblock">
        <div className="name">{session.name}</div>
        <div className="model">{session.model} · {session.cwd}</div>
      </div>
      <div className="doing">
        {isBusy ? (
          <>
            <span className="spin" />
            <span className="txt">
              {session.currentTool ? (
                <>
                  <span className="tool">{session.currentTool.toolName}</span> · {session.currentTool.argSummary}
                </>
              ) : (
                // Busy but no tool info yet (current-tool tracking lands in Phase 2).
                "working…"
              )}
            </span>
          </>
        ) : (
          <span className="txt">idle — awaiting prompt</span>
        )}
      </div>
      {/* A 0 means "not measured yet" (context arrives in Phase 3, activity in
          Phase 2) — show a dash rather than a misleading 0%. */}
      <div className="stats">
        <div className="chip">
          <div className="cl">Ctx</div>
          <div className="cv" style={{ color: session.contextPercent > 0 ? chipColor(session.contextPercent) : "var(--label-dim)" }}>
            {session.contextPercent > 0 ? `${Math.round(session.contextPercent)}%` : "—"}
          </div>
        </div>
        <div className="chip">
          <div className="cl">Act</div>
          <div className="cv" style={{ color: isBusy && session.activityPercent > 0 ? chipColor(session.activityPercent) : "var(--label-dim)" }}>
            {isBusy && session.activityPercent > 0 ? `${Math.round(session.activityPercent)}%` : "—"}
          </div>
        </div>
      </div>
      <span className={`st ${isBusy ? "busy" : "idle"}`}>{isBusy ? "Busy" : "Idle"}</span>
    </div>
  );
}

export function SessionLanes({ sessions }: { sessions: SessionSnapshot[] }) {
  return (
    <section className="panel a-lanes">
      <div className="panel-head">
        <span className="dot" />
        <h2>Concurrent Sessions</h2>
        <span className="right">~/.claude/sessions/*.json</span>
      </div>
      <div className="panel-body">
        {sessions.length === 0 ? (
          <div className="lanes-empty">
            <span className="lanes-empty-glow" />
            No active sessions
          </div>
        ) : (
          <div className="lanes">
            {sessions.map((session) => (
              <Lane key={session.sessionId} session={session} />
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
