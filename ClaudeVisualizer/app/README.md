# ClaudeVisualizer

A macOS instrument cluster for Claude Code: every concurrent session, live token
throughput, burn rate, context fuel, cache temperature, tool activity, and daily
odometers — in one resizable, non-scrolling window.

Design source of truth: [`../mockup.html`](../mockup.html) · Architecture: [`../SPEC.md`](../SPEC.md)

## Run

```sh
# Requires Rust (this machine: export PATH="/usr/local/opt/rustup/bin:$PATH") and Node
npm install
npm run tauri dev      # development, hot reload
npm run tauri build    # release .app + .dmg in src-tauri/target/release/bundle/
```

## Data tiers

| Tier | Source | Setup | Drives |
|---|---|---|---|
| **A** | Watches `~/.claude` (sessions, transcripts, tasks) read-only | none | everything: roster, gauges, trace, feed, odometers, task progress |
| **B** | Bundled OTLP/gRPC receiver on `127.0.0.1:4317` | launch Claude Code with the env vars below | authoritative burn rate, API-error + rate-limit lamps, telemetry lamp |
| **C** | Hook receiver on `127.0.0.1:4319` | opt-in via the ⚙ popover (edits `~/.claude/settings.json`, backup created, reversible) | instant permission-waiting telltale |

Tier B env vars (also shown in the in-app ⚙ popover):

```sh
export CLAUDE_CODE_ENABLE_TELEMETRY=1
export OTEL_METRICS_EXPORTER=otlp
export OTEL_LOGS_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

Without Tier B, cost figures are estimates: token counts × the price table in
`src-tauri/src/model_config.rs` (update that file when Anthropic's pricing changes).

## Layout

- `src-tauri/src/` — Rust backend: `session_registry` (Tier A1 watcher),
  `transcript` (byte-offset tailer, A2), `aggregator` (rates/cost/context/day totals),
  `backfill` (today's totals at startup), `otlp` (Tier B), `hooks` (Tier C receiver
  + settings installer), `model_config` (prices/context limits), `snapshot` (the
  Channel payload).
- `src/` — React frontend: `state/clusterStore` merges the live Channel over the
  simulator (`state/simulation`, used when running outside Tauri); components map
  1:1 to the mockup's panels.

`cargo test` covers parsing, request-dedupe, rate windows, pricing, day rollover,
backfill windowing, the hook installer JSON transform, and an in-process OTLP
gRPC round-trip.
