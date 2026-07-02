// ClaudeVisualizer — Tauri backend entry point.
//
// Phase 2 pipeline (SPEC §3):
//   SessionRegistryWatcher  ──▶ shared roster ──┐
//   TranscriptTailer + Aggregator ──▶ telemetry ─┼─▶ pusher ─▶ Channel ─▶ frontend
// Three threads share state through Arc<Mutex<…>>; the pusher composes a full
// TelemetrySnapshot ~10×/second and streams it to every subscribed frontend.

mod aggregator;
mod session_registry;
mod snapshot;
mod transcript;

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use notify::{RecursiveMode, Watcher};
use tauri::ipc::Channel;
use tauri::State;

use aggregator::Aggregator;
use session_registry::SessionRecord;
use snapshot::{TelemetrySnapshot, Throughput};
use transcript::TranscriptTailer;

/// Shared with the command handler so new frontends can subscribe.
struct AppState {
    subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>>,
}

/// The frontend calls this once at startup, handing us a Channel to stream
/// snapshots into. Multiple subscribers are fine (React StrictMode mounts
/// twice in dev); dead channels are dropped by the pusher on send failure.
#[tauri::command]
fn subscribe_registry(state: State<AppState>, channel: Channel<TelemetrySnapshot>) {
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

/// Thread 2: tail every live session's transcript and feed new lines into the
/// aggregator. Wakes on any change under ~/.claude/projects (or every 250ms
/// as a fallback — notify can miss events, and new sessions need pickup).
fn spawn_transcript_tailer(
    roster: Arc<Mutex<Vec<SessionRecord>>>,
    shared_aggregator: Arc<Mutex<Aggregator>>,
) {
    std::thread::spawn(move || {
        let projects_dir = session_registry::home_dir().join(".claude/projects");

        let (event_tx, event_rx) = std::sync::mpsc::channel::<()>();
        let mut watcher = notify::recommended_watcher(move |_event| {
            let _ = event_tx.send(());
        })
        .ok();
        if let Some(w) = watcher.as_mut() {
            // Recursive: transcripts live one directory deep (per-project dirs).
            // If this fails (dir missing on a fresh machine) the 250ms poll
            // still covers everything; we only lose wake-up latency.
            if let Err(error) = w.watch(&projects_dir, RecursiveMode::Recursive) {
                eprintln!("[tailer] watch unavailable ({error}); polling every 250ms");
            }
        }

        let mut tailer = TranscriptTailer::default();

        loop {
            // Snapshot the roster (tiny) so we don't hold its lock while
            // doing file IO.
            let records: Vec<SessionRecord> =
                roster.lock().expect("session list mutex poisoned").clone();
            let live_ids: Vec<String> = records
                .iter()
                .map(|r| r.snapshot.session_id.clone())
                .collect();

            let arrival_ms = now_epoch_ms();
            for record in &records {
                let session_id = &record.snapshot.session_id;
                // New sessions start tailing at EOF: rate windows and the feed
                // only want live activity, not a replay of history.
                tailer.start_at_end(session_id, &record.transcript_path);
                let lines = tailer.read_new_lines(session_id, &record.transcript_path);
                if lines.is_empty() {
                    continue;
                }
                let mut agg = shared_aggregator.lock().expect("aggregator mutex poisoned");
                agg.set_session_name(session_id, &record.snapshot.name);
                for line in &lines {
                    if let Some(parsed) = transcript::parse_line(line, arrival_ms) {
                        agg.ingest(session_id, parsed);
                    }
                }
            }

            tailer.retain_sessions(&live_ids);
            shared_aggregator
                .lock()
                .expect("aggregator mutex poisoned")
                .prune(&live_ids, arrival_ms);

            // Wake on a filesystem event or after 250ms, whichever is first…
            let _ = event_rx.recv_timeout(Duration::from_millis(250));
            // …then collapse any burst of queued events into one pass.
            while event_rx.try_recv().is_ok() {}
        }
    });
}

/// Thread 3: compose a TelemetrySnapshot from roster + aggregator state and
/// push it to every subscriber at ~10 Hz.
fn spawn_pusher(
    roster: Arc<Mutex<Vec<SessionRecord>>>,
    shared_aggregator: Arc<Mutex<Aggregator>>,
    subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>>,
) {
    let host = session_registry::hostname();
    std::thread::spawn(move || loop {
        let now_ms = now_epoch_ms();
        let records: Vec<SessionRecord> =
            roster.lock().expect("session list mutex poisoned").clone();

        let snapshot = {
            let mut agg = shared_aggregator.lock().expect("aggregator mutex poisoned");

            // Per-session rates, then machine-wide sums for the gauges/trace.
            let rates: Vec<(String, aggregator::SessionRates)> = records
                .iter()
                .map(|r| {
                    let id = r.snapshot.session_id.clone();
                    let rate = agg.session_rates(&id, now_ms);
                    (id, rate)
                })
                .collect();
            let output_rates: Vec<(String, f64)> = rates
                .iter()
                .map(|(id, r)| (id.clone(), r.output_tokens_per_sec))
                .collect();
            let activity = agg.activity_percents(&output_rates);

            let mut throughput = Throughput::default();
            let (mut cache_read_sum, mut input_sum) = (0.0, 0.0);
            for (_, rate) in &rates {
                throughput.output_tokens_per_sec += rate.output_tokens_per_sec;
                throughput.input_tokens_per_sec += rate.input_tokens_per_sec;
                throughput.cache_read_tokens_per_sec += rate.cache_read_tokens_per_sec;
                cache_read_sum += rate.cache_read_tokens_per_sec;
                input_sum += rate.input_tokens_per_sec;
            }
            // SPEC formula: cache_read ÷ (cache_read + input); 0 when quiet.
            let cache_hit_percent = if cache_read_sum + input_sum > 0.0 {
                cache_read_sum / (cache_read_sum + input_sum) * 100.0
            } else {
                0.0
            };

            let sessions = records
                .iter()
                .map(|record| {
                    let mut session = record.snapshot.clone();
                    let id = &session.session_id;
                    // The live transcript knows the model better than the
                    // startup peek (it updates when the user switches models).
                    if let Some(model_id) = agg.live_model_id(id) {
                        session.model = session_registry::model_display_name(&model_id);
                    }
                    session.current_tool = agg.current_tool(id);
                    session.activity_percent = activity.get(id).copied().unwrap_or(0.0);
                    session
                })
                .collect();

            TelemetrySnapshot {
                generated_at_ms: now_ms,
                host: host.clone(),
                sessions,
                throughput,
                cache_hit_percent,
                recent_events: agg.feed_events(),
            }
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let roster: Arc<Mutex<Vec<SessionRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_aggregator = Arc::new(Mutex::new(Aggregator::new()));
    let subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>> = Arc::new(Mutex::new(Vec::new()));

    session_registry::spawn_watcher(roster.clone());
    spawn_transcript_tailer(roster.clone(), shared_aggregator.clone());
    spawn_pusher(roster, shared_aggregator, subscribers.clone());

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
