// ClaudeVisualizer — Tauri backend entry point.
//
// Phase 0: the backend only opens the window and restores its last size/position.
// Phase 1 will add the SessionRegistryWatcher (watching ~/.claude/sessions) and
// push real snapshots to the frontend over a tauri::ipc::Channel (SPEC §3).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // window-state persists size/position to disk on close and restores it
        // on launch, so the cluster reopens exactly where the user left it.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        // If Tauri itself fails to start (corrupt config, no display server),
        // there is nothing sensible to recover to — crash with a clear message.
        .expect("error while running tauri application");
}
