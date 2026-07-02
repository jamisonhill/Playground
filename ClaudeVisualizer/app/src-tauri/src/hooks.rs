// HookServer + installer — Tier C of the ingestion pipeline (SPEC §2).
//
// Claude Code hooks POST their JSON payload to this local receiver the moment
// an event happens — most importantly `Notification`, which fires when Claude
// is *waiting for permission* (something the transcript can't show until the
// prompt is answered). That drives the amber "!" telltale.
//
// Installing the hooks is the ONE place this app writes under ~/.claude, and
// it is strictly opt-in from the settings popover: a backup of settings.json
// is written first, our entries are tagged with our URL so uninstall removes
// exactly what install added, and nothing else in the file is touched.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::{Path as UrlPath, State};
use axum::routing::post;
use serde_json::{json, Value};

/// Port for the hook receiver. 4317/4318 are OTLP's; this is ours.
pub const HOOK_PORT: u16 = 4319;

/// A permission prompt with no answer after this long is assumed abandoned
/// (the user answered while our Stop/Submit hook failed to fire, or quit).
const PERMISSION_LAMP_TIMEOUT_MS: i64 = 120_000;

/// The hook events we install, and what each one means to us.
const HOOK_EVENTS: [&str; 3] = ["Notification", "UserPromptSubmit", "Stop"];

#[derive(Default)]
pub struct HookState {
    /// session_id → deadline until which that session counts as "waiting".
    permission_waiting_until: HashMap<String, i64>,
}

impl HookState {
    pub fn any_permission_waiting(&self, now_ms: i64) -> bool {
        self.permission_waiting_until
            .values()
            .any(|deadline| now_ms < *deadline)
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// One hook delivery. Claude Code pipes the payload JSON to our curl command.
async fn handle_hook(
    UrlPath(event): UrlPath<String>,
    State(state): State<Arc<Mutex<HookState>>>,
    body: String,
) -> &'static str {
    let payload: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
    let session_id = payload
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut state = state.lock().expect("hook state mutex poisoned");
    match event.as_str() {
        "Notification" => {
            // Notification fires for permission requests AND idle reminders —
            // only the permission ones should light the lamp.
            let message = payload
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("");
            if message.to_lowercase().contains("permission") {
                state
                    .permission_waiting_until
                    .insert(session_id, now_ms() + PERMISSION_LAMP_TIMEOUT_MS);
            }
        }
        // Any sign of the session moving again clears its lamp.
        "UserPromptSubmit" | "Stop" => {
            state.permission_waiting_until.remove(&session_id);
        }
        _ => {}
    }
    "ok"
}

/// Serve the hook receiver until the process exits.
pub async fn serve(state: Arc<Mutex<HookState>>) {
    let app = axum::Router::new()
        .route("/hook/{event}", post(handle_hook))
        .with_state(state);
    let addr = format!("127.0.0.1:{HOOK_PORT}");
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("[hooks] receiver stopped ({error}) — permission lamp disabled");
            }
        }
        // Port taken (another instance running?) — Tier C simply stays off.
        Err(error) => eprintln!("[hooks] cannot bind {addr} ({error}) — permission lamp disabled"),
    }
}

// ---------------------------------------------------------------------------
// Installer: the explicit, reversible, opt-in mutation of ~/.claude/settings.json.
// ---------------------------------------------------------------------------

fn settings_path(home: &Path) -> PathBuf {
    home.join(".claude/settings.json")
}

/// The command Claude Code will run for `event`: pipe the hook payload to us.
/// `-m 2` caps the hang if this app isn't running; `|| true` keeps a failed
/// delivery from ever failing the user's hook chain.
fn hook_command(event: &str) -> String {
    format!(
        "curl -s -m 2 -X POST -H 'Content-Type: application/json' --data-binary @- \
         http://127.0.0.1:{HOOK_PORT}/hook/{event} >/dev/null 2>&1 || true"
    )
}

/// Marker that identifies entries as ours (both for status and uninstall).
fn our_marker() -> String {
    format!("127.0.0.1:{HOOK_PORT}/hook/")
}

/// Does this settings document already contain our hooks?
pub fn is_installed_in(settings: &Value) -> bool {
    serde_json::to_string(settings)
        .map(|s| s.contains(&our_marker()))
        .unwrap_or(false)
}

/// Add our hook entries to a settings document (pure — file IO lives apart).
pub fn install_into(settings: &mut Value) {
    if !settings.is_object() {
        *settings = json!({});
    }
    let hooks = settings
        .as_object_mut()
        .expect("settings coerced to object above")
        .entry("hooks")
        .or_insert_with(|| json!({}));
    if !hooks.is_object() {
        // A malformed hooks section — leave it alone rather than clobber it.
        return;
    }
    for event in HOOK_EVENTS {
        let entries = hooks
            .as_object_mut()
            .expect("checked is_object above")
            .entry(event)
            .or_insert_with(|| json!([]));
        let Some(list) = entries.as_array_mut() else {
            continue; // malformed event entry — skip it, install the others
        };
        let already = serde_json::to_string(&list)
            .map(|s| s.contains(&our_marker()))
            .unwrap_or(false);
        if !already {
            list.push(json!({
                "hooks": [{ "type": "command", "command": hook_command(event) }]
            }));
        }
    }
}

/// Remove exactly the entries `install_into` added (pure).
pub fn remove_from(settings: &mut Value) {
    let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };
    let marker = our_marker();
    for event in HOOK_EVENTS {
        let Some(list) = hooks.get_mut(event).and_then(|e| e.as_array_mut()) else {
            continue;
        };
        list.retain(|group| {
            !serde_json::to_string(group)
                .map(|s| s.contains(&marker))
                .unwrap_or(false)
        });
    }
    // Tidy up: drop event arrays we emptied (a user's own entries survive).
    hooks.retain(|_, v| v.as_array().map(|a| !a.is_empty()).unwrap_or(true));
    if hooks.is_empty() {
        settings.as_object_mut().map(|o| o.remove("hooks"));
    }
}

fn read_settings(home: &Path) -> Result<Value, String> {
    let path = settings_path(home);
    if !path.exists() {
        return Ok(json!({}));
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text)
        // A settings file we can't parse must NOT be rewritten — we'd destroy it.
        .map_err(|e| format!("{} is not valid JSON ({e}); not touching it", path.display()))
}

fn write_settings(home: &Path, settings: &Value) -> Result<(), String> {
    let path = settings_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let pretty = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("serialize failed: {e}"))?;
    fs::write(&path, pretty + "\n").map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Install the hooks (with a one-time backup of the original settings.json).
pub fn install(home: &Path) -> Result<(), String> {
    let mut settings = read_settings(home)?;
    // Backup before the first-ever mutation; never overwrite an existing
    // backup (it holds the true pre-ClaudeVisualizer state).
    let path = settings_path(home);
    let backup = path.with_extension("json.claudevisualizer-backup");
    if path.exists() && !backup.exists() {
        fs::copy(&path, &backup).map_err(|e| format!("backup failed: {e} — aborting install"))?;
    }
    install_into(&mut settings);
    write_settings(home, &settings)
}

/// Remove our hooks (leaves everything else, including the backup, in place).
pub fn uninstall(home: &Path) -> Result<(), String> {
    let mut settings = read_settings(home)?;
    if !is_installed_in(&settings) {
        return Ok(()); // nothing of ours present — done
    }
    remove_from(&mut settings);
    write_settings(home, &settings)
}

/// Are our hooks currently present in settings.json?
pub fn is_installed(home: &Path) -> bool {
    read_settings(home).map(|s| is_installed_in(&s)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_then_uninstall_leaves_user_settings_untouched() {
        // A settings file with the user's own model choice AND their own hook.
        let mut settings = json!({
            "model": "opus",
            "hooks": {
                "Stop": [{ "hooks": [{ "type": "command", "command": "say done" }] }]
            }
        });

        install_into(&mut settings);
        assert!(is_installed_in(&settings));
        // The user's own Stop hook coexists with ours.
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
        assert!(settings["hooks"]["Notification"].is_array());

        // Installing twice must not duplicate entries.
        install_into(&mut settings);
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);

        remove_from(&mut settings);
        assert!(!is_installed_in(&settings));
        assert_eq!(settings["model"], "opus");
        // The user's own hook survives; our empty event arrays are tidied away.
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert!(settings["hooks"].get("Notification").is_none());
    }

    #[test]
    fn install_into_empty_settings_creates_structure() {
        let mut settings = json!({});
        install_into(&mut settings);
        assert!(is_installed_in(&settings));
        for event in HOOK_EVENTS {
            assert_eq!(settings["hooks"][event].as_array().unwrap().len(), 1);
        }
        remove_from(&mut settings);
        // Fully clean: the hooks key itself is gone.
        assert!(settings.get("hooks").is_none());
    }

    #[test]
    fn permission_lamp_lifecycle() {
        let mut state = HookState::default();
        assert!(!state.any_permission_waiting(1_000));
        state.permission_waiting_until.insert("s1".into(), 5_000);
        assert!(state.any_permission_waiting(1_000));
        assert!(!state.any_permission_waiting(6_000)); // deadline passed
        state.permission_waiting_until.remove("s1");
        assert!(!state.any_permission_waiting(1_000));
    }
}
