// ClaudeVisualizer — Tauri backend entry point.
//
// Phase 1 pipeline (SPEC §3): SessionRegistryWatcher keeps a shared session
// roster fresh; a pusher thread snapshots it ~10×/second and sends it to every
// subscribed frontend over a tauri::ipc::Channel (the recommended
// high-throughput IPC path — not the event bus).

mod session_registry;
mod snapshot;

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::ipc::Channel;
use tauri::State;

use snapshot::{RegistrySnapshot, SessionSnapshot};

/// Shared between the watcher thread, the pusher thread, and the command
/// handler. Arc = shared ownership across threads; Mutex = one writer at a time.
struct AppState {
    subscribers: Arc<Mutex<Vec<Channel<RegistrySnapshot>>>>,
}

/// The frontend calls this once at startup, handing us a Channel to stream
/// registry snapshots into. Multiple subscribers are fine (React StrictMode
/// mounts twice in dev); dead channels are dropped by the pusher on send failure.
#[tauri::command]
fn subscribe_registry(state: State<AppState>, channel: Channel<RegistrySnapshot>) {
    state
        .subscribers
        .lock()
        .expect("subscriber list mutex poisoned") // only poisons if a holder panicked
        .push(channel);
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // Only fails if the system clock is set before 1970 — treat as epoch.
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The live session roster, owned jointly by the watcher (writes) and the
    // pusher (reads).
    let sessions: Arc<Mutex<Vec<SessionSnapshot>>> = Arc::new(Mutex::new(Vec::new()));
    let subscribers: Arc<Mutex<Vec<Channel<RegistrySnapshot>>>> = Arc::new(Mutex::new(Vec::new()));

    // Thread 1: watch ~/.claude/sessions and keep `sessions` current.
    session_registry::spawn_watcher(sessions.clone());

    // Thread 2: push a snapshot to every subscriber at ~10 Hz.
    {
        let sessions = sessions.clone();
        let subscribers = subscribers.clone();
        let host = session_registry::hostname();
        std::thread::spawn(move || loop {
            let snapshot = RegistrySnapshot {
                generated_at_ms: now_epoch_ms(),
                host: host.clone(),
                sessions: sessions.lock().expect("session list mutex poisoned").clone(),
            };
            // A failed send means that frontend (e.g. an old StrictMode mount)
            // is gone — retain() silently drops it from the list.
            subscribers
                .lock()
                .expect("subscriber list mutex poisoned")
                .retain(|channel| channel.send(snapshot.clone()).is_ok());
            std::thread::sleep(Duration::from_millis(100));
        });
    }

    tauri::Builder::default()
        // window-state persists size/position to disk on close and restores it
        // on launch, so the cluster reopens exactly where the user left it.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(AppState { subscribers })
        .invoke_handler(tauri::generate_handler![subscribe_registry])
        .run(tauri::generate_context!())
        // If Tauri itself fails to start (corrupt config, no display server),
        // there is nothing sensible to recover to — crash with a clear message.
        .expect("error while running tauri application");
}
