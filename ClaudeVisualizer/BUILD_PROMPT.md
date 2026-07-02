# Build Prompt for Fable 5 — ClaudeVisualizer

Paste the block below to Fable 5 (`claude-fable-5`) to build the app. It assumes Fable 5 is
running in this repo (`~/Ai/Playground/ClaudeVisualizer`) with `SPEC.md` and `mockup.html`
present. Build **one phase per run**, verifying against real Claude Code sessions before advancing.

---

## The prompt

You are building **ClaudeVisualizer**, a macOS desktop app that visualizes what Claude Code is
doing in real time, across every concurrent session, as a car-dashboard **instrument cluster** in
a single resizable, non-scrolling window.

**Read first, treat as authoritative:**
- `SPEC.md` — architecture, verified data schemas, file paths, and the widget↔data contract.
- `mockup.html` — the approved visual design. Reproduce its layout, palette, typography, motion,
  and information design exactly, but driven by real data. It is the source of truth for how the
  app looks and feels.

**Stack (do not substitute without asking):**
- Tauri 2 desktop shell, Rust backend.
- React + TypeScript + Vite frontend (ES modules; TypeScript throughout).
- `uPlot` for the live throughput trace; custom `<canvas>` gauges (proportional to radius, per the
  mockup — do not pull in a commercial charting lib).
- Rust: `tauri-plugin-fs` (with the `watch` feature), `notify`, `serde`/`serde_json`, `tokio`.

**Hard requirements:**
1. **Single non-scrolling window.** Full-viewport CSS grid, `overflow:hidden` on the body, `clamp()`
   type, canvas widgets drawn proportionally to their computed size. It must reshape to any window
   size/aspect without scrolling — verify by resizing to tall-skinny and short-wide.
2. **Read-only.** Never mutate `~/.claude` except the explicit, reversible, opt-in hook install
   (Tier C). All other ingestion is passive watching.
3. **Real data via tiered ingestion** (see SPEC §2–3):
   - **Tier A (build first, no setup):** watch `~/.claude/sessions/*.json` (session roster,
     busy/idle, prune dead PIDs); byte-offset **tail** `~/.claude/projects/<encoded-cwd>/*.jsonl`
     transcripts for `usage` + `tool_use`; read `~/.claude/tasks/<uuid>/*.json`.
   - **Tier B (optional):** bundle a minimal OTLP gRPC receiver on `:4317` for authoritative
     `claude_code.cost.usage` and metrics; group by `session.id`.
   - **Tier C (optional, opt-in):** local HTTP hook receiver for instant `PostToolUse` events and
     the permission telltale.
4. **Aggregator in Rust**, keyed by `session.id`, computes rolling tok/s, cache ratio, cost,
   context %, current tool, task progress, and machine-wide aggregates. Push snapshots to the
   frontend over a Tauri **Channel** at ~10 Hz. Frontend eases needles toward targets with `rAF`.
5. Wire **every widget** to the field in SPEC §4 "Widget↔data contract."
6. Honor `prefers-reduced-motion`; handle the **empty state** (no sessions) and stale/dead sessions.
7. Follow the repo's code conventions: descriptive names, comments explaining intent and each
   error's failure mode (the owner is learning), explicit error handling.

**Build this phase now: Phase 0 — Scaffold.**
Create the Tauri 2 + React/TS/Vite project. Configure a resizable dark window (min ~720×480, remember
size/position, `titleBarStyle: Overlay`). Port the mockup's layout and design tokens into React
components — `Hud`, `GaugeCluster` (4 canvas gauges), `SessionLanes`, `Trace` (uPlot), `Feed`,
`Odometers` — rendering with placeholder/simulated data for now, matching `mockup.html` pixel-for-feel.
Confirm it compiles, launches, and stays non-scrolling at every window shape. Then stop and report
what to verify before Phase 1.

(Subsequent runs: Phase 1 session roster → Phase 2 transcript telemetry → Phase 3 odometers/tasks/
context → Phase 4 OTLP + hooks → Phase 5 polish & signed `.dmg`. See SPEC §7.)

---

## Notes for the operator
- Give Fable 5 one phase at a time; run the app against real sessions (open a couple of other Claude
  Code windows) to verify each phase before continuing.
- Tier A needs no configuration and proves the concept end-to-end; treat Tiers B/C as enrichment.
- If cost must be exact, enable Tier B telemetry env vars (SPEC §2, Tier B) before launching Claude
  Code sessions you want measured.
