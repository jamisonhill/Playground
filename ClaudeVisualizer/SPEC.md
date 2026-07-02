# ClaudeVisualizer — Technical Specification

A real-time desktop **instrument cluster** that visualizes everything Claude Code is
doing on this machine, across all concurrent sessions, in a single resizable,
non-scrolling window styled like a car dashboard.

- **Visual source of truth:** [`mockup.html`](./mockup.html) — the approved concept. The
  shipped app must reproduce its layout, palette, motion, and information design with real data.
- **Status:** Design approved. Ready to build with Fable 5. See [`BUILD_PROMPT.md`](./BUILD_PROMPT.md).
- **Target platform:** macOS (Apple Silicon), Tauri 2 desktop app.

---

## 1. Goals & non-goals

**Goals**
- Show, at a glance, whether Claude Code is "working hard" right now — across *every* running session.
- Rich, verbose, real-time statistics: tokens, cost, cache, tools, context, tasks.
- Car-dashboard aesthetic: gauges/speedometers, a live throughput trace, warning lamps, odometers.
- One window that always fits its own shape and size, never scrolls, and scales its widgets to fit.
- Read-only observability. The app watches; it never mutates Claude Code state.

**Non-goals (v1)**
- No remote/multi-machine aggregation (local machine only).
- No historical database / long-term analytics (in-memory rolling windows only).
- No control surface (can't approve prompts or stop sessions from the dashboard).

---

## 2. Data sources (verified on this machine)

Claude Code exposes its activity locally through four channels. The app uses a **tiered
ingestion** strategy: the filesystem tier works with zero configuration; the telemetry and
hook tiers are optional enrichment.

### Tier A — Filesystem (always on, no setup)

#### A1. Session registry — `~/.claude/sessions/*.json`
One file **per running Claude Code process**, named by PID (e.g. `64453.json`). This is the
authoritative "who is running right now" list. Verified shape:

```json
{
  "pid": 64453,
  "sessionId": "0a395bf8-6e91-45a1-bf1f-3c225a05db35",
  "cwd": "/Users/jamisonhill/Ai/Playground",
  "startedAt": 1783007348837,
  "procStart": "Thu Jul  2 15:49:07 2026",
  "version": "2.1.198",
  "kind": "interactive",
  "entrypoint": "cli",
  "name": "playground-71",
  "nameSource": "derived",
  "status": "busy",            // "busy" | "idle"  ← drives the LED / activity state
  "updatedAt": 1783007686617,
  "statusUpdatedAt": 1783007686617,
  "bridgeSessionId": "session_01Hd..."
}
```
- **Watch** this directory (create/modify/delete). Each file = one session lane.
- `status` toggles `busy`/`idle`. A file's disappearance = session ended. Stale files
  (pid no longer alive) should be pruned: check the pid with `kill -0` semantics.

#### A2. Session transcripts — `~/.claude/projects/<encoded-cwd>/<sessionId>.jsonl`
Append-only JSON Lines, one file per session. `<encoded-cwd>` is the cwd with `/` → `-`
(e.g. `-Users-jamisonhill-Ai-Playground`). Filenames are the session UUID — **glob `*.jsonl`**,
do not assume a literal `transcript.jsonl`. This is the real-time telemetry stream.

Each line is an object with a `type`. Types seen: `mode`, `permission-mode`, `bridge-session`,
`file-history-snapshot`, `user`, `attachment`, `ai-title`, `last-prompt`, **`assistant`**.

The `assistant` line is the workhorse. Verified shape:
```jsonc
{
  "type": "assistant",
  "uuid": "…", "parentUuid": "…", "requestId": "…",
  "timestamp": "2026-07-02T15:54:55.548Z",   // ISO 8601 — use for tok/s windows
  "sessionId": "0a395bf8-…",
  "cwd": "/Users/jamisonhill/Ai/Playground",
  "version": "2.1.198",
  "gitBranch": "main",
  "message": {
    "model": "claude-opus-4-8",              // per-turn model
    "role": "assistant",
    "stop_reason": "tool_use",
    "content": [
      { "type": "thinking", … },
      { "type": "text", … },
      { "type": "tool_use", "name": "Bash", "input": { "command": "npm test" } }  // ← feed
    ],
    "usage": {
      "input_tokens": 7290,
      "output_tokens": 567,
      "cache_creation_input_tokens": 23173,
      "cache_read_input_tokens": 0,
      "server_tool_use": { "web_search_requests": 0, "web_fetch_requests": 0 },
      "service_tier": "standard",
      "cache_creation": { "ephemeral_1h_input_tokens": 23173, "ephemeral_5m_input_tokens": 0 },
      "speed": …,                             // inference speed signal
      "iterations": [ { "input_tokens": …, "output_tokens": …, … } ]
    }
  }
}
```
- **Tail incrementally**: track a byte offset per file; on modify, read only the new tail and
  parse complete lines. Never re-read the whole file (they grow large).
- **Which file is "live"**: the `*.jsonl` whose `sessionId` matches an entry in A1, and/or the
  most-recently-modified file per project dir.
- `tool_use` blocks → the diagnostic feed and each lane's "current tool".
- `usage` deltas between consecutive assistant lines → throughput, cache ratio, cost estimate.

#### A3. Task lists — `~/.claude/tasks/<uuid>/*.json`
Per-session todo items. Verified shape:
```json
{ "id": "1", "subject": "Phase 1: Scaffold …", "description": "…",
  "activeForm": "Scaffolding Xcode project", "status": "completed",
  "blocks": [], "blockedBy": [] }
```
- Optional panel / lane sub-widget: progress ring "3 / 7 tasks done", current `activeForm`.

#### A4. Statusline input (schema reference)
Claude Code feeds the statusline command a JSON blob on stdin with the fields the app also wants:
`model.display_name` / `model.id`, `workspace.current_dir`, `context_window.used_percentage`,
`context_window.total_input_tokens`. The app does not read statusline directly, but
`used_percentage` is the **authoritative context-fuel value**; if unavailable, derive fuel from
`total_input_tokens ÷ model_context_limit`.

### Tier B — OpenTelemetry (optional, canonical metrics/cost)
Enabled by the user via env before launching Claude Code:
```
CLAUDE_CODE_ENABLE_TELEMETRY=1
OTEL_METRICS_EXPORTER=otlp
OTEL_LOGS_EXPORTER=otlp
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```
Meter name: `com.anthropic.claude_code`. All signals carry `session.id` (default on via
`OTEL_METRICS_INCLUDE_SESSION_ID`) → group by it for per-session lanes; events also carry
`prompt.id` to stitch a prompt's lifecycle.

**8 metrics:** `claude_code.session.count`, `claude_code.token.usage`
(attrs: `type` ∈ {input, output, cacheRead, cacheCreation}, `model`, `query_source`, `speed`,
`effort`, `agent.name`, `skill.name`, `plugin.name`, `mcp_server.name`, `mcp_tool.name`),
`claude_code.cost.usage` (USD, `model`), `claude_code.lines_of_code.count`,
`claude_code.commit.count`, `claude_code.pull_request.count`,
`claude_code.code_edit_tool.decision`, `claude_code.active_time.total`.

**Event/log records:** `claude_code.user_prompt` (`prompt_length`),
`claude_code.tool_result` (`tool_name`, `duration_ms`, `success`, `error_type`),
`claude_code.api_request` (`duration_ms`, token counts, `cost_usd`),
`claude_code.api_error` (`status_code`, `error`, `attempt`),
`claude_code.tool_decision` (`decision`, `source`).

**Traces (beta):** `CLAUDE_CODE_ENHANCED_TELEMETRY_BETA=1` → spans
`claude_code.interaction`, `claude_code.llm_request`, `claude_code.tool`
(+ `tool.blocked_on_user`, `tool.execution`), `claude_code.hook`. Treat as best-effort;
span names may change between releases.

The app should **bundle a minimal OTLP receiver** (gRPC on `:4317`) in the Rust backend so no
external collector is required. `cost.usage` is the only source of *authoritative* dollar cost;
without Tier B, cost is estimated from token counts × model price table.

### Tier C — Hooks (optional, lowest-latency events)
The app can install HTTP hooks into `settings.json` that POST to a local port on
`PreToolUse` / `PostToolUse` / `UserPromptSubmit` / `Notification` / `Stop` / `SessionStart` /
`SessionEnd`. Payload: `session_id`, `transcript_path`, `cwd`, `permission_mode`,
`hook_event_name`; tool events add `tool_name`, `tool_input`; `PostToolUse` adds `tool_response`.
Use for instant feed updates and the "permission waiting" telltale (fires before the transcript
is flushed). Installing hooks mutates `settings.json`, so make it opt-in with clear disclosure.

---

## 3. Ingestion → aggregation → render pipeline

```
                 ┌─────────────────────────── Rust backend (Tauri) ───────────────────────────┐
 Tier A files ──▶│ SessionRegistryWatcher  ─┐                                                   │
 (~/.claude)     │ TranscriptTailer (per session, byte-offset) ─┐                               │
                 │ TaskWatcher ─────────────┘                   ├─▶ Aggregator ─▶ Channel ──────┼─▶ Frontend store
 Tier B :4317 ──▶│ OtlpReceiver (optional)  ─────────────────────┘   (per-session +             │      │
 Tier C :port ──▶│ HookServer (optional)    ─────────────────────┘    aggregate snapshots)      │      ▼
                 └───────────────────────────────────────────────────────────────────────────────┘   rAF render:
                                                                                                        canvas gauges,
                                                                                                        uPlot trace,
                                                                                                        DOM roster/feed/odos
```

- **Watchers** use the Tauri `fs` plugin (`watchImmediate` on `~/.claude/sessions`,
  `~/.claude/projects`, `~/.claude/tasks`) and `readTextFileLines` for streaming reads.
- **Aggregator** maintains, keyed by `session.id`: rolling token/sec (sliding ~5 s window over
  transcript timestamps), cache-hit ratio, cost accumulator, current tool, context %, task
  progress. It also computes machine-wide aggregates for the gauge cluster.
- **Transport**: push snapshots to the frontend over a Tauri **Channel** (`tauri::ipc::Channel`)
  at ~10 Hz — not the event bus (Channel is the recommended high-throughput path).
- **Frontend** keeps the latest snapshot in a store and animates toward it with a
  `requestAnimationFrame` loop (needles ease to targets; trace pushes to a ring buffer).

### Derived-metric formulas
| Metric | Formula |
|---|---|
| Throughput (tach) | Σ `output_tokens` across sessions ÷ sliding window seconds (from `timestamp` deltas) |
| Burn rate (speedometer) | Σ `cost.usage` rate; else Σ(tokens × model price) ÷ elapsed |
| Context fuel | statusline `used_percentage` (per session), aggregate = max or mean |
| Cache temp | `cache_read_input_tokens ÷ (cache_read + input_tokens)` over window |
| Session activity % | per-session output tok/s normalized to a rolling machine max |

---

## 4. Widget ↔ data contract (must match `mockup.html`)

| Widget | Bound field(s) | Source |
|---|---|---|
| **HUD** sessions count | count of live registry files | A1 |
| **HUD** clock / host | system | — |
| **Telltale: permission** | `Notification` / `permission_mode`, `tool_decision` | C / B |
| **Telltale: API error** | `claude_code.api_error` | B (else transcript `stop_reason`) |
| **Telltale: rate limit** | `api_error` `status_code` 429 | B |
| **Tachometer** (Throughput) | aggregate output tok/s | A2 / B |
| **Speedometer** (Burn Rate) | aggregate $/hr | B (else estimate) |
| **Fuel gauge** (Context) | `used_percentage` | A4 |
| **Cache Temp gauge** | cache-read ratio | A2 / B |
| **Live trace** | output / input / cache-read tok/s | A2 / B |
| **Session lane** LED/status | `status` busy/idle | A1 |
| **Session lane** model·cwd | `model`, `cwd` | A1 / A2 |
| **Session lane** current tool + spinner | latest `tool_use` | A2 / C |
| **Session lane** Ctx / Act chips | context %, activity % | A4 / A2 |
| **Diagnostic feed** rows | `tool_result` (tool, duration_ms, success) | C / B / A2 |
| **Odometer** tokens | Σ token usage today | A2 / B |
| **Odometer** cost | `cost.usage` today | B (else estimate) |
| **Odometer** tool calls | count of `tool_result` | A2 / B |
| **Odometer** lines / commits | `lines_of_code.count`, `commit.count` | B (else transcript) |

---

## 5. Tech stack

| Layer | Choice | Why |
|---|---|---|
| Desktop shell | **Tauri 2** | ~30–40 MB RAM (vs Electron 200–300), native `fs` watch + line streaming, Channel IPC, small signed `.dmg` |
| Backend | **Rust** | file watchers, byte-offset tailing, optional OTLP gRPC receiver + hook HTTP server |
| Frontend framework | **React + TypeScript + Vite** (Tauri default) | matches "TypeScript for larger projects"; Svelte acceptable if preferred |
| Line charts | **uPlot** | ~50 KB, ~10% CPU at 3.6k pts/60fps — best for high-frequency streaming |
| Gauges | **Custom `<canvas>`** (as in mockup) | proportional-to-radius drawing, no license cost (avoids commercial Highcharts) |
| Layout | CSS grid, `100%` height, `overflow:hidden`, `clamp()` type | single non-scrolling window that reshapes to fit |

**Rust crates:** `tauri`, `tauri-plugin-fs` (with `watch`), `notify`, `serde`/`serde_json`,
`tokio`; optional `tonic` + `opentelemetry-proto` (OTLP receiver), `axum`/`hyper` (hook server).

**Conventions** (from project CLAUDE.md): TypeScript, ES modules, descriptive names, comments
that explain intent (learning), explicit error handling with a comment on the failure mode.

---

## 6. Window & app behavior
- Resizable, min size ~720×480, remembers size/position. Dark, no native chrome clutter;
  consider `titleBarStyle: Overlay` so the cluster fills the glass.
- Optional: launch at login; menu-bar quick-glance (aggregate tach + session count).
- **Reduced motion**: honor `prefers-reduced-motion` — needles snap, no ticker scroll/pulse.
- **Empty state**: no sessions running → gauges at rest, "No active sessions" with a soft idle glow.
- Never write to `~/.claude` except the explicit opt-in hook install (Tier C), which is reversible.

---

## 7. Phased build plan
- **Phase 0 — Scaffold.** Tauri 2 + React/TS/Vite. Resizable dark window. Port the mockup's
  static layout/tokens into components (HUD, GaugeCluster, SessionLanes, Trace, Feed, Odometers).
- **Phase 1 — Session roster (real).** `SessionRegistryWatcher` → live lanes with busy/idle,
  model, cwd, uptime. Prune dead PIDs. First real data on screen.
- **Phase 2 — Transcript telemetry (real).** `TranscriptTailer` (byte-offset) → Aggregator →
  Channel. Drives tachometer, cache gauge, live trace, per-lane current tool, diagnostic feed.
- **Phase 3 — Odometers + tasks + context.** Token/cost/lines odometers; task progress; fuel gauge.
- **Phase 4 — Optional enrichment.** Bundled OTLP receiver (`:4317`) for authoritative cost +
  metrics; opt-in HTTP hook installer for instant events and the permission telltale.
- **Phase 5 — Polish & package.** Motion tuning, reduced-motion, window-shape responsiveness,
  empty/error states, signed `.dmg`, optional launch-at-login and menu-bar mode.

Each phase compiles and runs on its own; verify with real sessions before advancing.

---

## 8. Open items to confirm during build
- Exact model→price table for cost estimation when Tier B is off (keep in one config file).
- Model context-window limits for fuel fallback when statusline `used_percentage` is absent.
- Whether the bundled OTLP receiver should also expose Prometheus for power users.
- Menu-bar companion: in v1 or deferred.
