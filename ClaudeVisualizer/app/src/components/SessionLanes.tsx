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
        {isBusy && session.currentTool ? (
          <>
            <span className="spin" />
            <span className="txt">
              <span className="tool">{session.currentTool.toolName}</span> · {session.currentTool.argSummary}
            </span>
          </>
        ) : (
          <span className="txt">idle — awaiting prompt</span>
        )}
      </div>
      <div className="stats">
        <div className="chip">
          <div className="cl">Ctx</div>
          <div className="cv" style={{ color: chipColor(session.contextPercent) }}>
            {session.contextPercent}%
          </div>
        </div>
        <div className="chip">
          <div className="cl">Act</div>
          <div className="cv" style={{ color: isBusy ? chipColor(session.activityPercent) : "var(--label-dim)" }}>
            {isBusy ? `${session.activityPercent}%` : "—"}
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
