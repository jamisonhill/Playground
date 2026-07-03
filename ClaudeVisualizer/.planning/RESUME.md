# Resume: ClaudeVisualizer

**Paused:** 2026-07-03 ~9:50am
**Reason:** Native arm64 .dmg rebuilt on the home machine — good stopping point.
**Phase:** 6/6 done. Project still COMPLETE; this session added the native
Apple Silicon build (was a deferred item).

**What this session did (home Mac, Apple M3 Max):**
- Discovered a fresh checkout: no `target/`, no `node_modules`, no Rust toolchain
  (this is a DIFFERENT machine than the M1 that built the original x64 dmg).
- Reinstalled: `brew install rustup` → `rustup default stable` (installed
  x86_64 toolchain because brew is Intel-prefix / Rosetta) →
  `rustup target add aarch64-apple-darwin`. Ran `npm install` in `app/`.
- Built native arm64: `npm run tauri build -- --target aarch64-apple-darwin`
  (4m31s cold). Verified `lipo -archs` = arm64. Opened the dmg.

**Artifacts (NOT in git — rebuild if gone):**
- `app/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/ClaudeVisualizer_0.1.0_aarch64.dmg` (4.3M)
- `.../bundle/macos/ClaudeVisualizer.app`

**Key facts to remember next time:**
- Both of Jamison's Macs are Apple Silicon (work=M1, home=M3 Max) but run an
  Intel-prefix Homebrew (`/usr/local`), so rustup runs under Rosetta and the
  DEFAULT build is x86_64. For a native arm build you MUST pass
  `--target aarch64-apple-darwin`. (Memory `rust-toolchain-path` updated to match.)
- Still unsigned/ad-hoc → on any other Mac, right-click → Open the first time.
- Build outputs (`target/`, `.app`, `.dmg`) are never committed; a fresh checkout
  needs toolchain reinstall + `npm install` before building.

**To restart dev:**
```sh
cd ~/Ai/Playground/ClaudeVisualizer/app
export PATH="/usr/local/opt/rustup/bin:$PATH"   # keg-only brew rustup
npm run tauri dev
```
Native release build:
```sh
npm run tauri build -- --target aarch64-apple-darwin
```
Tests: `cd app/src-tauri && cargo test`.

**Next action if resuming — pick from PROGRESS.md Deferred list:**
- Developer ID signing + notarization (needs Apple Developer cert), OR
- Universal (Intel+ARM) build via `--target universal-apple-darwin`, OR
- Menu-bar companion + launch-at-login (SPEC §8), OR
- Price-table refresh in `src-tauri/src/model_config.rs`.
