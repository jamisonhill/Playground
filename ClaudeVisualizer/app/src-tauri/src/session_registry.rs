// SessionRegistryWatcher — Tier A1 of the ingestion pipeline (SPEC §2).
//
// Watches ~/.claude/sessions/*.json (one file per running Claude Code process,
// named by PID), keeps a shared Vec<SessionSnapshot> up to date, and prunes
// sessions whose process has died without deleting its file. Strictly
// read-only: this module never writes anything under ~/.claude.

use std::ffi::CStr;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use serde::Deserialize;

use crate::snapshot::SessionSnapshot;

/// A live session as the registry watcher sees it: the display snapshot plus
/// the transcript path the tailer needs (built from the *original* cwd — the
/// snapshot's cwd is shortened for display and useless for path building).
#[derive(Clone)]
pub struct SessionRecord {
    pub snapshot: SessionSnapshot,
    pub transcript_path: PathBuf,
}

/// Verified shape of ~/.claude/sessions/<pid>.json (SPEC §2 A1). Fields we
/// don't use are simply not declared — serde ignores unknown fields.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryFile {
    pid: i32,
    session_id: String,
    cwd: String,
    #[serde(default)]
    started_at: i64,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

/// The user's home directory. Claude Code itself requires $HOME, so if it is
/// somehow unset there are no sessions to show anyway — fall back to "/" and
/// let every scan come back empty rather than crashing the app.
pub fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

/// This machine's hostname for the HUD, e.g. "jamison-mbp" (".local" trimmed).
pub fn hostname() -> String {
    let mut buffer = [0u8; 256];
    // gethostname is the portable libc call; a non-zero return means the
    // buffer was too small or the syscall failed — use a generic fallback.
    let failed = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0;
    if failed {
        return "localhost".to_string();
    }
    let name = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) };
    name.to_string_lossy().trim_end_matches(".local").to_string()
}

/// Is the process still running? Registry files can outlive their process
/// (crash, force-quit), so each scan re-checks with `kill(pid, 0)` — the
/// classic "send no signal, just check" probe (SPEC §2 A1).
fn is_pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    // EPERM = the process exists but belongs to another user; still alive.
    // ESRCH (or anything else) = no such process; treat the file as stale.
    std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// Encode a cwd the way Claude Code names its project transcript directories:
/// "/" becomes "-" (verified: /Users/x/Ai/Playground → -Users-x-Ai-Playground).
fn encode_cwd(cwd: &str) -> String {
    cwd.replace('/', "-")
}

/// Show "~/Ai/Playground" instead of the full absolute path in a lane.
fn shorten_cwd(cwd: &str, home: &Path) -> String {
    match home.to_str().and_then(|h| cwd.strip_prefix(h)) {
        Some(rest) => format!("~{rest}"),
        None => cwd.to_string(),
    }
}

/// "claude-opus-4-8" → "Opus 4.8", "claude-fable-5" → "Fable 5",
/// "claude-haiku-4-5-20251001" → "Haiku 4.5" (date suffixes dropped).
pub fn model_display_name(model_id: &str) -> String {
    let Some(rest) = model_id.strip_prefix("claude-") else {
        return model_id.to_string(); // unrecognized scheme — show the raw id
    };
    let mut parts = rest.split('-');
    let family = match parts.next() {
        Some(f) if !f.is_empty() => {
            let mut chars = f.chars();
            // Capitalize the family name: "opus" → "Opus".
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => return model_id.to_string(),
            }
        }
        _ => return model_id.to_string(),
    };
    // Remaining segments are version digits; 8-digit segments are datestamps.
    let version: Vec<&str> = parts.filter(|p| p.len() < 8).collect();
    if version.is_empty() {
        family
    } else {
        format!("{family} {}", version.join("."))
    }
}

/// How many bytes of transcript tail to search for the latest model id.
const MODEL_PEEK_BYTES: u64 = 262_144;

/// The transcript for a session lives at
/// ~/.claude/projects/<encoded-cwd>/<sessionId>.jsonl (SPEC §2 A2).
pub fn transcript_path(home: &Path, cwd: &str, session_id: &str) -> PathBuf {
    home.join(".claude/projects")
        .join(encode_cwd(cwd))
        .join(format!("{session_id}.jsonl"))
}

/// Read the last `assistant` line of the session's transcript to learn which
/// model it is running (the registry file itself has no model field). Returns
/// None if the transcript doesn't exist yet (fresh session) or has no
/// assistant turns — the caller falls back to the CLI version string.
fn peek_model(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    // Only read the tail — transcripts grow to many MB and the newest
    // assistant line is what we want anyway.
    let start = length.saturating_sub(MODEL_PEEK_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut tail = String::new();
    // Seeking may have landed mid-UTF-8-sequence; read lossily via bytes.
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    tail.push_str(&String::from_utf8_lossy(&bytes));

    for line in tail.lines().rev() {
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }
        // A line can be a partial JSON fragment if we cut it at `start` —
        // parse errors just mean "keep looking at older lines".
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if let Some(model) = value.pointer("/message/model").and_then(|m| m.as_str()) {
            return Some(model.to_string());
        }
    }
    None
}

/// One full scan of the registry directory → the current session roster.
pub fn scan_sessions(home: &Path) -> Vec<SessionRecord> {
    let dir = home.join(".claude/sessions");
    let mut sessions = Vec::new();

    // Directory missing just means Claude Code has never run on this machine —
    // an empty roster (the frontend shows the "No active sessions" state).
    let Ok(entries) = fs::read_dir(&dir) else {
        return sessions;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // A file can be mid-write (Claude Code rewrites it on every status
        // change) — unreadable or partial JSON is skipped; the next scan,
        // at most 2s away, will pick it up.
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(registry) = serde_json::from_str::<RegistryFile>(&text) else {
            continue;
        };
        // Prune: file left behind by a crashed / force-quit process.
        if !is_pid_alive(registry.pid) {
            continue;
        }

        let transcript = transcript_path(home, &registry.cwd, &registry.session_id);
        let model = peek_model(&transcript)
            .map(|id| model_display_name(&id))
            // No transcript yet — show the CLI version so the line isn't blank.
            .unwrap_or_else(|| format!("v{}", registry.version.as_deref().unwrap_or("?")));

        let status = match registry.status.as_deref() {
            Some("busy") => "busy",
            _ => "idle",
        };

        sessions.push(SessionRecord {
            snapshot: SessionSnapshot {
                session_id: registry.session_id,
                name: registry
                    .name
                    .unwrap_or_else(|| format!("pid-{}", registry.pid)),
                model,
                cwd: shorten_cwd(&registry.cwd, home),
                status: status.to_string(),
                context_percent: 0.0,  // Phase 3
                activity_percent: 0.0, // filled by the aggregator at push time
                current_tool: None,    // filled by the aggregator at push time
                started_at_ms: registry.started_at,
            },
            transcript_path: transcript,
        });
    }

    // Stable lane order: oldest session first, session id as tiebreaker.
    sessions.sort_by(|a, b| {
        a.snapshot
            .started_at_ms
            .cmp(&b.snapshot.started_at_ms)
            .then_with(|| a.snapshot.session_id.cmp(&b.snapshot.session_id))
    });
    sessions
}

/// Spawn the watcher thread. It rescans the registry when the directory
/// changes (create/modify/delete via `notify`) and at least every 2 seconds
/// regardless — a process can die without any file event, and only a rescan
/// notices the dead PID.
pub fn spawn_watcher(shared_sessions: Arc<Mutex<Vec<SessionRecord>>>) {
    std::thread::spawn(move || {
        let home = home_dir();
        let sessions_dir = home.join(".claude/sessions");

        // The watcher pings this mpsc channel on every filesystem event; the
        // loop below treats a ping and a 2s timeout identically (rescan).
        let (event_tx, event_rx) = std::sync::mpsc::channel::<()>();
        let mut watcher = notify::recommended_watcher(move |_event| {
            // Full event details don't matter — any change triggers a rescan.
            let _ = event_tx.send(());
        })
        .ok();

        match watcher.as_mut() {
            Some(w) => {
                // If the directory doesn't exist yet, watching fails — that's
                // fine, the 2-second periodic rescan still covers everything;
                // we just lose the instant-update latency.
                if let Err(error) = w.watch(&sessions_dir, RecursiveMode::NonRecursive) {
                    eprintln!("[registry] watch unavailable ({error}); polling every 2s");
                }
            }
            None => eprintln!("[registry] notify watcher failed to start; polling every 2s"),
        }

        // Log roster changes (not every scan) — handy when debugging why a
        // session does or doesn't appear, quiet the rest of the time.
        let mut last_roster_fingerprint = String::new();

        loop {
            let scanned = scan_sessions(&home);
            let fingerprint = scanned
                .iter()
                .map(|s| format!("{}:{}", s.snapshot.name, s.snapshot.status))
                .collect::<Vec<_>>()
                .join(",");
            if fingerprint != last_roster_fingerprint {
                eprintln!("[registry] {} live session(s): {}", scanned.len(), fingerprint);
                last_roster_fingerprint = fingerprint;
            }
            // A poisoned mutex means the pusher thread panicked while holding
            // the lock — unrecoverable, so propagate the panic here too.
            *shared_sessions.lock().expect("session list mutex poisoned") = scanned;

            // Sleep until the next filesystem event or 2s, whichever is first…
            let _ = event_rx.recv_timeout(Duration::from_secs(2));
            // …then collapse any burst of queued events into this one rescan.
            while event_rx.try_recv().is_ok() {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_cwd_like_claude_code() {
        assert_eq!(
            encode_cwd("/Users/jamisonhill/Ai/Playground"),
            "-Users-jamisonhill-Ai-Playground"
        );
    }

    #[test]
    fn shortens_home_prefixed_paths() {
        let home = Path::new("/Users/jamisonhill");
        assert_eq!(shorten_cwd("/Users/jamisonhill/Ai/coffee", home), "~/Ai/coffee");
        assert_eq!(shorten_cwd("/opt/other", home), "/opt/other");
    }

    #[test]
    fn formats_model_display_names() {
        assert_eq!(model_display_name("claude-opus-4-8"), "Opus 4.8");
        assert_eq!(model_display_name("claude-fable-5"), "Fable 5");
        assert_eq!(model_display_name("claude-haiku-4-5-20251001"), "Haiku 4.5");
        assert_eq!(model_display_name("gpt-oss"), "gpt-oss");
    }

    #[test]
    fn dead_pid_is_not_alive() {
        assert!(!is_pid_alive(0));
        assert!(!is_pid_alive(-4));
        // PID 1 (launchd) always exists on macOS.
        assert!(is_pid_alive(1));
    }
}
