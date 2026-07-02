// ClaudeVisualizer — Tauri backend entry point.
//
// Phase 2 pipeline (SPEC §3):
//   SessionRegistryWatcher  ──▶ shared roster ──┐
//   TranscriptTailer + Aggregator ──▶ telemetry ─┼─▶ pusher ─▶ Channel ─▶ frontend
// Three threads share state through Arc<Mutex<…>>; the pusher composes a full
// TelemetrySnapshot ~10×/second and streams it to every subscribed frontend.

mod aggregator;
mod backfill;
mod hooks;
mod model_config;
mod otlp;
mod session_registry;
mod snapshot;
mod transcript;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use notify::{RecursiveMode, Watcher};
use tauri::ipc::Channel;
use tauri::State;

use aggregator::Aggregator;
use session_registry::SessionRecord;
use snapshot::{OdometerTotals, TelemetrySnapshot, Telltales, Throughput};
use transcript::TranscriptTailer;

/// Shared with the command handlers.
struct AppState {
    subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>>,
    /// Cached "our hooks are in settings.json" flag — re-checked only when the
    /// install/uninstall commands run, so the 10 Hz pusher never touches disk.
    hooks_installed: Arc<AtomicBool>,
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

/// Opt-in Tier C: add our hook entries to ~/.claude/settings.json (backs the
/// original file up first). Returns the new installed state.
#[tauri::command]
fn install_hooks(state: State<AppState>) -> Result<bool, String> {
    hooks::install(&session_registry::home_dir())?;
    state.hooks_installed.store(true, Ordering::Relaxed);
    Ok(true)
}

/// Reverse of install_hooks — removes exactly the entries we added.
#[tauri::command]
fn uninstall_hooks(state: State<AppState>) -> Result<bool, String> {
    hooks::uninstall(&session_registry::home_dir())?;
    state.hooks_installed.store(false, Ordering::Relaxed);
    Ok(false)
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
            {
                let mut agg = shared_aggregator.lock().expect("aggregator mutex poisoned");
                agg.prune(&live_ids, arrival_ms);
                // Seed each session's context gauge from the startup peek —
                // a no-op once the live tailer has measured it.
                for record in &records {
                    if record.initial_context_tokens > 0 {
                        agg.seed_context_tokens(
                            &record.snapshot.session_id,
                            record.initial_context_tokens,
                        );
                    }
                }
            }

            // Wake on a filesystem event or after 250ms, whichever is first…
            let _ = event_rx.recv_timeout(Duration::from_millis(250));
            // …then collapse any burst of queued events into one pass.
            while event_rx.try_recv().is_ok() {}
        }
    });
}

/// Everything the pusher needs beyond the Tier A state.
struct EnrichmentState {
    otlp: Arc<Mutex<otlp::OtlpState>>,
    hook: Arc<Mutex<hooks::HookState>>,
    hooks_installed: Arc<AtomicBool>,
}

/// Thread 3: compose a TelemetrySnapshot from roster + aggregator state and
/// push it to every subscriber at ~10 Hz.
fn spawn_pusher(
    roster: Arc<Mutex<Vec<SessionRecord>>>,
    shared_aggregator: Arc<Mutex<Aggregator>>,
    subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>>,
    enrichment: EnrichmentState,
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

            let mut max_context_percent = 0.0f64;
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
                    session.context_percent = agg.context_percent(id).unwrap_or(0.0);
                    max_context_percent = max_context_percent.max(session.context_percent);
                    session
                })
                .collect();

            // Tier B/C enrichment: authoritative burn rate when the OTLP
            // exporter is live, warning lamps from OTLP errors + hook events.
            let (otlp_burn, telemetry_connected, api_error, rate_limited) = {
                let otlp = enrichment.otlp.lock().expect("otlp state mutex poisoned");
                (
                    otlp.burn_rate_usd_per_hour(now_ms),
                    otlp.is_connected(now_ms),
                    otlp.api_error_active(now_ms),
                    otlp.rate_limit_active(now_ms),
                )
            };
            let permission_waiting = enrichment
                .hook
                .lock()
                .expect("hook state mutex poisoned")
                .any_permission_waiting(now_ms);

            let today = agg.today_totals();
            TelemetrySnapshot {
                generated_at_ms: now_ms,
                host: host.clone(),
                sessions,
                throughput,
                cache_hit_percent,
                // SPEC §4: cost.usage is the only authoritative dollar source —
                // prefer it whenever Tier B is exporting; else the estimate.
                cost_per_hour: otlp_burn.unwrap_or_else(|| agg.burn_rate_usd_per_hour(now_ms)),
                // SPEC: aggregate fuel = max across sessions (the fullest tank
                // is the one about to run out).
                context_percent: max_context_percent,
                recent_events: agg.feed_events(),
                odometers: OdometerTotals {
                    tokens_today: today.tokens,
                    cost_today_usd: today.cost_usd,
                    tool_calls: today.tool_calls,
                    lines_edited: today.lines_edited,
                    commits: today.commits,
                },
                telltales: Telltales {
                    telemetry_connected,
                    permission_waiting,
                    api_error,
                    rate_limited,
                },
                hooks_installed: enrichment.hooks_installed.load(Ordering::Relaxed),
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
    // Local UTC offset must be read before any threads exist — the time crate
    // refuses to read the environment's timezone from a multithreaded process
    // (a Unix soundness rule). Falls back to UTC, which only shifts the
    // odometers' midnight reset.
    let utc_offset_ms = time::UtcOffset::current_local_offset()
        .map(|offset| offset.whole_seconds() as i64 * 1000)
        .unwrap_or(0);
    let app_start_ms = now_epoch_ms();

    let roster: Arc<Mutex<Vec<SessionRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_aggregator = Arc::new(Mutex::new(Aggregator::new(utc_offset_ms, app_start_ms)));
    let subscribers: Arc<Mutex<Vec<Channel<TelemetrySnapshot>>>> = Arc::new(Mutex::new(Vec::new()));
    let otlp_state = Arc::new(Mutex::new(otlp::OtlpState::default()));
    let hook_state = Arc::new(Mutex::new(hooks::HookState::default()));
    let hooks_installed = Arc::new(AtomicBool::new(hooks::is_installed(
        &session_registry::home_dir(),
    )));

    session_registry::spawn_watcher(roster.clone());
    spawn_transcript_tailer(roster.clone(), shared_aggregator.clone());
    spawn_pusher(
        roster.clone(),
        shared_aggregator.clone(),
        subscribers.clone(),
        EnrichmentState {
            otlp: otlp_state.clone(),
            hook: hook_state.clone(),
            hooks_installed: hooks_installed.clone(),
        },
    );

    // Tier B + C servers share one small tokio runtime on its own thread.
    {
        let otlp_state = otlp_state.clone();
        let hook_state = hook_state.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build();
            let Ok(runtime) = runtime else {
                // No async runtime = no optional tiers; Tier A keeps working.
                eprintln!("[enrichment] tokio runtime failed to start; Tiers B/C disabled");
                return;
            };
            runtime.block_on(async {
                let otlp_addr: std::net::SocketAddr = "127.0.0.1:4317"
                    .parse()
                    .expect("hardcoded address is valid");
                let otlp_server = async {
                    // Port 4317 taken (a real collector is running) — fine,
                    // the user's collector wins and Tier B stays off here.
                    if let Err(error) = otlp::serve(otlp_state, otlp_addr).await {
                        eprintln!("[otlp] cannot serve on :4317 ({error}) — Tier B disabled");
                    }
                };
                tokio::join!(otlp_server, hooks::serve(hook_state));
            });
        });
    }

    // One-shot backfill of today's totals (everything before app start; the
    // live tailer owns everything after). Runs off the hot path.
    {
        let shared_aggregator = shared_aggregator.clone();
        std::thread::spawn(move || {
            let home = session_registry::home_dir();
            // Local midnight expressed in UTC epoch ms.
            let day_start_ms =
                (app_start_ms + utc_offset_ms).div_euclid(86_400_000) * 86_400_000 - utc_offset_ms;
            let totals = backfill::scan_today(&home, day_start_ms, app_start_ms);
            eprintln!(
                "[backfill] today so far: {} tokens, ${:.2}, {} tool calls",
                totals.tokens, totals.cost_usd, totals.tool_calls
            );
            shared_aggregator
                .lock()
                .expect("aggregator mutex poisoned")
                .add_backfill(&totals);
        });
    }

    tauri::Builder::default()
        // window-state persists size/position to disk on close and restores it
        // on launch, so the cluster reopens exactly where the user left it.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(AppState {
            subscribers,
            hooks_installed,
        })
        .invoke_handler(tauri::generate_handler![
            subscribe_registry,
            install_hooks,
            uninstall_hooks
        ])
        .run(tauri::generate_context!())
        // If Tauri itself fails to start (corrupt config, no display server),
        // there is nothing sensible to recover to — crash with a clear message.
        .expect("error while running tauri application");
}
