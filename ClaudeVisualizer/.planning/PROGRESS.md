# ClaudeVisualizer — Progress

**Status: COMPLETE** — all SPEC §7 phases shipped 2026-07-02.

## Phase 0: Scaffold [DONE]
- [x] Tauri 2 + React/TS/Vite project under `app/`
- [x] Dark resizable window (min 720×480, titleBarStyle Overlay, window-state restore)
- [x] Mockup ported to components (Hud, GaugeCluster, SessionLanes, Trace, Feed, Odometers)
- [x] Non-scrolling full-viewport grid, reduced-motion, simulated data

## Phase 1: Session roster [DONE]
- [x] SessionRegistryWatcher on ~/.claude/sessions (notify + 2s rescan)
- [x] Dead-PID pruning via kill(pid,0); model peek from transcript tail
- [x] 10 Hz Tauri Channel to frontend

## Phase 2: Transcript telemetry [DONE]
- [x] Byte-offset TranscriptTailer (complete lines only, starts at EOF)
- [x] Aggregator: 5s rate windows, usage deltas deduped by requestId
- [x] Tach, cache gauge, live trace, per-lane current tool, diagnostic feed w/ durations

## Phase 3: Odometers + tasks + context [DONE]
- [x] model_config.rs price table + context limits (docs-sourced 2026-06)
- [x] Burn rate (60s cost window), context fuel (prompt size ÷ model window)
- [x] Daily odometers with local-midnight rollover + startup backfill
- [x] Task progress in lanes from ~/.claude/tasks/<sessionId>/
- [x] Window-drag fix (core:window:allow-start-dragging)

## Phase 4: Optional enrichment [DONE]
- [x] Tier B: OTLP/gRPC receiver on :4317 (authoritative burn rate, error lamps)
- [x] Tier C: hook receiver on :4319 + opt-in installer (permission telltale)
- [x] ⚙ popover UI, tier badge, real telltales

## Phase 5: Polish & package [DONE]
- [x] Custom gauge icon (SVG source in icons/), README, 26 Rust tests
- [x] Release build: .app (ad-hoc signed) + ClaudeVisualizer_0.1.0_x64.dmg

## Deferred / user-action items
- [ ] Developer ID signing + notarization (needs Apple Developer cert)
- [x] Native Apple Silicon build — arm64 .dmg built 2026-07-03 on home M3 Max
      (`ClaudeVisualizer_0.1.0_aarch64.dmg`, verified `lipo -archs` = arm64).
      Note: brew is Intel-prefix so rustup runs under Rosetta; must build with
      `--target aarch64-apple-darwin` for native. Universal build still open if wanted.
- [ ] Menu-bar companion + launch-at-login (SPEC §8: deferred from v1)
- [ ] Price-table upkeep in src-tauri/src/model_config.rs when Anthropic pricing changes
