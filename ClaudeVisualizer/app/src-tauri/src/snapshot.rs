// Snapshot types pushed to the frontend over the Tauri Channel.
//
// Field names serialize as camelCase to match the TypeScript contract in
// src/types/telemetry.ts — the frontend uses these payloads verbatim.

use serde::Serialize;

/// The tool a session is currently running, from transcript `tool_use` blocks.
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
    /// Output tok/s normalized to the rolling machine max (SPEC §3 formulas).
    pub activity_percent: f64,
    pub current_tool: Option<ToolActivity>,
    /// Registry `startedAt` (epoch ms) — lets the frontend show per-session uptime.
    pub started_at_ms: i64,
    /// Task-list progress from ~/.claude/tasks/<sessionId>/ (SPEC A3); 0/0 = no list.
    pub tasks_done: u32,
    pub tasks_total: u32,
    /// activeForm of the in-progress task, e.g. "Running tests".
    pub active_task_form: Option<String>,
}

/// One row for the Diagnostic Feed: a completed tool call (tool_use matched
/// to its tool_result in the transcript).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedEvent {
    /// Monotonically increasing — the frontend dedupes rows by this.
    pub id: u64,
    pub time_ms: i64,
    pub session_name: String,
    pub tool_name: String,
    pub arg_summary: String,
    /// None when the tool errored (no meaningful duration).
    pub duration_ms: Option<u64>,
    pub is_error: bool,
}

/// Machine-wide token rates over the sliding window (drives tach + trace).
#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Throughput {
    pub output_tokens_per_sec: f64,
    pub input_tokens_per_sec: f64,
    pub cache_read_tokens_per_sec: f64,
}

/// Daily running totals for the odometer strip.
#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdometerTotals {
    pub tokens_today: u64,
    pub cost_today_usd: f64,
    pub tool_calls: u64,
    pub lines_edited: u64,
    pub commits: u64,
}

/// Everything the backend knows, pushed ~10×/second (Tier A, Phases 1–3).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub generated_at_ms: i64,
    pub host: String,
    pub sessions: Vec<SessionSnapshot>,
    pub throughput: Throughput,
    /// Aggregate cache-read ratio 0–100 over the window; 0 when no traffic.
    pub cache_hit_percent: f64,
    /// Estimated burn rate in USD/hour over the last minute (price table).
    pub cost_per_hour: f64,
    /// Aggregate context fuel 0–100: the fullest session's tank (SPEC: max).
    pub context_percent: f64,
    /// Ring of the last ~40 feed events, oldest→newest (frontend dedupes by id).
    pub recent_events: Vec<FeedEvent>,
    pub odometers: OdometerTotals,
}
