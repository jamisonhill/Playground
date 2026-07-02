# Resume: ClaudeVisualizer

**Paused:** 2026-07-02 ~5:45pm
**Reason:** Project complete — pausing to finalize documentation.
**Phase:** 6/6 done (Phases 0–5, commits `28bab22` → `210f73d` + this pause commit)

**What's working (all verified on this machine):**
- Release app runs from `app/src-tauri/target/release/bundle/macos/ClaudeVisualizer.app`
- Live Tier A data: roster (3 sessions seen), transcript telemetry, odometers
  (backfill found 75.5M tokens / ~$114 / 601 tool calls that day), task progress
- Tier B (:4317) and Tier C (:4319) servers verified listening; hook installer
  unit-tested (never live-tested against a real permission prompt — see below)
- 26 Rust tests green; frontend tsc/vite clean

**Not yet verified end-to-end (first user to try it confirms):**
1. Tier B with a real exporter — launch Claude Code with the env vars from the
   ⚙ popover and watch the green ◉ lamp + burn-rate takeover.
2. Tier C permission lamp — install hooks via ⚙, start a NEW Claude Code
   session, trigger a permission prompt, watch the amber "!".
3. The .dmg on the other Mac (unsigned → right-click Open; Intel build → Rosetta
   on Apple Silicon).

**Key decisions on record:**
- Cost is a price-table ESTIMATE (list API prices) unless Tier B is live;
  table lives in `app/src-tauri/src/model_config.rs`.
- Usage lines repeating per requestId are DELTA'd, never summed (regression-tested).
- Hooks installer is the app's only write to ~/.claude: backup created at
  `settings.json.claudevisualizer-backup`, entries tagged with our URL, refuses
  malformed JSON.
- Tailer starts at EOF; backfill covers pre-launch lines (cutoff = app start).

**To restart dev:**
```sh
cd ~/Ai/Playground/ClaudeVisualizer/app
export PATH="/usr/local/opt/rustup/bin:$PATH"   # brew rustup is keg-only
npm run tauri dev
```
Tests: `cd app/src-tauri && cargo test`. Release: `npm run tauri build`.

**Next action if resuming:** pick from the Deferred list in PROGRESS.md —
most likely Developer ID signing, or a native arm64 build on the other Mac.
