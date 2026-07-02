// Snapshot types pushed to the frontend over the Tauri Channel.
//
// Field names serialize as camelCase to match the TypeScript contract in
// src/types/telemetry.ts — the frontend uses these payloads verbatim.

use serde::Serialize;

/// The tool a session is currently running (filled from transcripts in Phase 2).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolActivity {
    pub tool_name: String,
    pub arg_summary: String,
}

/// One lane in the Concurrent Sessions panel (SPEC §4).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session_id: String,
    pub name: String,
    pub model: String,
    pub cwd: String,
    /// "busy" | "idle" — anything else the registry invents is coerced to "idle".
    pub status: String,
    /// 0 until Phase 3 wires the statusline `used_percentage`.
    pub context_percent: f64,
    /// 0 until Phase 2 computes per-session output tok/s.
    pub activity_percent: f64,
    pub current_tool: Option<ToolActivity>,
    /// Registry `startedAt` (epoch ms) — lets the frontend show per-session uptime.
    pub started_at_ms: i64,
}

/// What the registry watcher knows: the live session roster (Tier A1).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySnapshot {
    pub generated_at_ms: i64,
    pub host: String,
    pub sessions: Vec<SessionSnapshot>,
}
