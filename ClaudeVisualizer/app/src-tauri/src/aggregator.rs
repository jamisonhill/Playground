// Aggregator — turns parsed transcript lines into the rolling metrics the
// gauges and lanes display (SPEC §3): per-session token rates over a sliding
// window, cache-hit ratio, current tool, activity %, and the diagnostic feed.

use std::collections::{HashMap, VecDeque};

use crate::model_config;
use crate::snapshot::{FeedEvent, ToolActivity};
use crate::transcript::{TranscriptLine, UsageCounts};

/// Sliding window for rate computation (SPEC: "rolling ~5 s window").
const RATE_WINDOW_MS: i64 = 5_000;
/// Keep samples a bit longer than the window so a late prune can't cut into it.
const SAMPLE_RETENTION_MS: i64 = 10_000;
/// Burn rate averages cost over a longer window — cost arrives in lumpy
/// per-turn increments, so 5s would make the speedometer jump wildly.
const COST_WINDOW_MS: i64 = 60_000;
const COST_RETENTION_MS: i64 = 120_000;
/// The feed ring sent with every snapshot; the frontend dedupes by event id.
const FEED_RING_CAPACITY: usize = 40;
/// A started tool with no result after this long is considered abandoned
/// (session was killed mid-tool, or the result line was never written).
const PENDING_TOOL_TIMEOUT_MS: i64 = 10 * 60 * 1000;

/// One usage delta attributed to a moment in time.
struct UsageSample {
    timestamp_ms: i64,
    output_tokens: u64,
    input_tokens: u64,
    cache_read_tokens: u64,
}

/// A tool_use we saw start, awaiting its tool_result for the feed row.
struct PendingTool {
    tool_name: String,
    arg_summary: String,
    started_ms: i64,
}

/// Running totals for the odometer strip ("today", this machine).
#[derive(Clone, Copy, Default)]
pub struct DailyTotals {
    /// All tokens processed: input + output + cache reads + cache writes.
    pub tokens: u64,
    pub cost_usd: f64,
    pub tool_calls: u64,
    pub lines_edited: u64,
    pub commits: u64,
}

impl DailyTotals {
    fn add(&mut self, other: &DailyTotals) {
        self.tokens += other.tokens;
        self.cost_usd += other.cost_usd;
        self.tool_calls += other.tool_calls;
        self.lines_edited += other.lines_edited;
        self.commits += other.commits;
    }
}

/// Everything the aggregator tracks for one live session.
#[derive(Default)]
struct SessionTelemetry {
    samples: VecDeque<UsageSample>,
    /// Size of the latest request's full prompt (input + cache read + cache
    /// creation) — this IS the context currently in use, for the fuel gauge.
    last_context_tokens: Option<u64>,
    /// Usage of the last line seen for each in-flight request id. Claude Code
    /// writes several assistant lines per API request with *identical or
    /// cumulative* usage (verified on this machine), so the true increment is
    /// the delta against the previous line of the same request — never the sum.
    last_usage_by_request: HashMap<String, UsageCounts>,
    pending_tools: HashMap<String, PendingTool>,
    current_tool: Option<ToolActivity>,
    /// Latest model id seen in a transcript line, e.g. "claude-opus-4-8".
    pub live_model_id: Option<String>,
}

/// Rates computed for one session at push time. The aggregate cache-hit ratio
/// is derived from these sums by the pusher (SPEC formula), so it isn't stored here.
#[derive(Clone, Copy, Default)]
pub struct SessionRates {
    pub output_tokens_per_sec: f64,
    pub input_tokens_per_sec: f64,
    pub cache_read_tokens_per_sec: f64,
}

pub struct Aggregator {
    sessions: HashMap<String, SessionTelemetry>,
    feed_ring: VecDeque<FeedEvent>,
    next_event_id: u64,
    session_name_by_id: HashMap<String, String>,
    /// Slow-decaying max of any session's output rate — the denominator for
    /// each lane's activity % ("normalized to a rolling machine max").
    rolling_max_output_rate: f64,
    /// Machine-wide (timestamp, estimated USD) samples for the burn-rate gauge.
    cost_samples: VecDeque<(i64, f64)>,
    /// Odometer totals for the current local day.
    today: DailyTotals,
    /// Local-midnight offset from UTC in ms — lets us know when "today" rolls
    /// over without repeatedly asking the OS (which is thread-restricted).
    utc_offset_ms: i64,
    /// Days since epoch (local) that `today` belongs to.
    current_day: i64,
}

impl Aggregator {
    pub fn new(utc_offset_ms: i64, now_ms: i64) -> Self {
        Self {
            sessions: HashMap::new(),
            feed_ring: VecDeque::with_capacity(FEED_RING_CAPACITY),
            next_event_id: 1,
            session_name_by_id: HashMap::new(),
            rolling_max_output_rate: 0.0,
            cost_samples: VecDeque::new(),
            today: DailyTotals::default(),
            utc_offset_ms,
            current_day: local_day_number(now_ms, utc_offset_ms),
        }
    }

    /// Merge historical totals (startup backfill of today's transcripts).
    pub fn add_backfill(&mut self, totals: &DailyTotals) {
        self.today.add(totals);
    }

    /// At local midnight the odometers reset — "Tokens Today" means today.
    fn roll_day_if_needed(&mut self, now_ms: i64) {
        let day = local_day_number(now_ms, self.utc_offset_ms);
        if day != self.current_day {
            self.current_day = day;
            self.today = DailyTotals::default();
        }
    }

    /// Tell the aggregator a session's display name (feed rows show it).
    pub fn set_session_name(&mut self, session_id: &str, name: &str) {
        self.session_name_by_id
            .insert(session_id.to_string(), name.to_string());
    }

    /// Feed one parsed transcript line into the per-session state.
    pub fn ingest(&mut self, session_id: &str, line: TranscriptLine) {
        match line {
            TranscriptLine::Assistant {
                request_id,
                timestamp_ms,
                model,
                usage,
                tool_uses,
            } => {
                // -- Per-session updates first (holds the sessions borrow) --
                let telemetry = self.sessions.entry(session_id.to_string()).or_default();
                if model.is_some() {
                    telemetry.live_model_id = model.clone();
                }
                let model_id = model
                    .or_else(|| telemetry.live_model_id.clone())
                    .unwrap_or_else(|| "claude-opus".to_string()); // pricing fallback

                // The latest request's prompt size = the context currently in
                // use by this session (fuel gauge).
                let context_tokens =
                    usage.input_tokens + usage.cache_read_tokens + usage.cache_creation_tokens;
                if context_tokens > 0 {
                    telemetry.last_context_tokens = Some(context_tokens);
                }

                // Delta against the previous line of the SAME request; the
                // first line of a request counts in full. saturating_sub
                // guards against usage ever going down (would underflow u64).
                let previous = telemetry
                    .last_usage_by_request
                    .insert(request_id.clone(), usage)
                    .unwrap_or_default();
                let delta_output = usage.output_tokens.saturating_sub(previous.output_tokens);
                let delta_input = usage.input_tokens.saturating_sub(previous.input_tokens);
                let delta_cache_read = usage
                    .cache_read_tokens
                    .saturating_sub(previous.cache_read_tokens);
                let delta_cache_creation = usage
                    .cache_creation_tokens
                    .saturating_sub(previous.cache_creation_tokens);
                let has_delta =
                    delta_output > 0 || delta_input > 0 || delta_cache_read > 0 || delta_cache_creation > 0;

                if has_delta {
                    telemetry.samples.push_back(UsageSample {
                        timestamp_ms,
                        output_tokens: delta_output,
                        input_tokens: delta_input,
                        cache_read_tokens: delta_cache_read,
                    });
                }

                // Cap the per-request map: requests complete quickly, so only
                // the most recent handful matter. Drop everything on overflow
                // (worst case: one over-counted sample after a 64-request burst).
                if telemetry.last_usage_by_request.len() > 64 {
                    telemetry.last_usage_by_request.clear();
                    telemetry.last_usage_by_request.insert(request_id, usage);
                }

                let mut lines_edited = 0u64;
                let mut commits = 0u64;
                for tool_use in tool_uses {
                    lines_edited += tool_use.lines_changed;
                    commits += u64::from(tool_use.is_git_commit);
                    telemetry.current_tool = Some(ToolActivity {
                        tool_name: tool_use.tool_name.clone(),
                        arg_summary: tool_use.arg_summary.clone(),
                    });
                    telemetry.pending_tools.insert(
                        tool_use.tool_use_id,
                        PendingTool {
                            tool_name: tool_use.tool_name,
                            arg_summary: tool_use.arg_summary,
                            started_ms: timestamp_ms,
                        },
                    );
                }

                // -- Machine-wide updates (sessions borrow released above) --
                self.roll_day_if_needed(timestamp_ms);
                if has_delta {
                    let cost = model_config::estimate_cost_usd(
                        &model_id,
                        delta_input,
                        delta_output,
                        delta_cache_read,
                        delta_cache_creation,
                    );
                    self.cost_samples.push_back((timestamp_ms, cost));
                    self.today.tokens +=
                        delta_input + delta_output + delta_cache_read + delta_cache_creation;
                    self.today.cost_usd += cost;
                }
                self.today.lines_edited += lines_edited;
                self.today.commits += commits;
            }
            TranscriptLine::ToolResult {
                tool_use_id,
                timestamp_ms,
                is_error,
            } => {
                self.roll_day_if_needed(timestamp_ms);
                self.today.tool_calls += 1;

                let telemetry = self.sessions.entry(session_id.to_string()).or_default();
                // A result for a tool we never saw start (started before the
                // app launched) is silently ignored — no duration to compute.
                let Some(pending) = telemetry.pending_tools.remove(&tool_use_id) else {
                    return;
                };
                // The tool finished, so the lane goes back to "working…" until
                // the next tool_use (only if this tool was the one displayed).
                if telemetry
                    .current_tool
                    .as_ref()
                    .is_some_and(|t| t.tool_name == pending.tool_name && t.arg_summary == pending.arg_summary)
                {
                    telemetry.current_tool = None;
                }

                let session_name = self
                    .session_name_by_id
                    .get(session_id)
                    .cloned()
                    .unwrap_or_else(|| session_id.chars().take(8).collect());
                let event = FeedEvent {
                    id: self.next_event_id,
                    time_ms: timestamp_ms,
                    session_name,
                    tool_name: pending.tool_name,
                    arg_summary: pending.arg_summary,
                    duration_ms: if is_error {
                        None
                    } else {
                        Some(timestamp_ms.saturating_sub(pending.started_ms).max(0) as u64)
                    },
                    is_error,
                };
                self.next_event_id += 1;
                if self.feed_ring.len() == FEED_RING_CAPACITY {
                    self.feed_ring.pop_front();
                }
                self.feed_ring.push_back(event);
            }
        }
    }

    /// Estimated burn rate in USD/hour: cost over the last 60s, extrapolated.
    pub fn burn_rate_usd_per_hour(&self, now_ms: i64) -> f64 {
        let window_start = now_ms - COST_WINDOW_MS;
        let cost_in_window: f64 = self
            .cost_samples
            .iter()
            .filter(|(t, _)| *t >= window_start)
            .map(|(_, c)| c)
            .sum();
        cost_in_window * (3_600_000.0 / COST_WINDOW_MS as f64)
    }

    /// Context used by one session, 0–100 against its model's window.
    pub fn context_percent(&self, session_id: &str) -> Option<f64> {
        let telemetry = self.sessions.get(session_id)?;
        let used = telemetry.last_context_tokens? as f64;
        let limit = model_config::context_limit_for(
            telemetry.live_model_id.as_deref().unwrap_or("claude-opus"),
        ) as f64;
        Some((used / limit * 100.0).clamp(0.0, 100.0))
    }

    /// Seed context from the startup transcript peek (only if the live tailer
    /// hasn't measured it yet — live data always wins).
    pub fn seed_context_tokens(&mut self, session_id: &str, tokens: u64) {
        let telemetry = self.sessions.entry(session_id.to_string()).or_default();
        if telemetry.last_context_tokens.is_none() && tokens > 0 {
            telemetry.last_context_tokens = Some(tokens);
        }
    }

    /// Today's odometer totals.
    pub fn today_totals(&self) -> DailyTotals {
        self.today
    }

    /// Drop state for sessions that ended and expire stale pending tools.
    pub fn prune(&mut self, live_ids: &[String], now_ms: i64) {
        while self
            .cost_samples
            .front()
            .is_some_and(|(t, _)| now_ms - t > COST_RETENTION_MS)
        {
            self.cost_samples.pop_front();
        }
        self.sessions.retain(|id, _| live_ids.iter().any(|live| live == id));
        self.session_name_by_id
            .retain(|id, _| live_ids.iter().any(|live| live == id));
        for telemetry in self.sessions.values_mut() {
            telemetry
                .pending_tools
                .retain(|_, tool| now_ms - tool.started_ms < PENDING_TOOL_TIMEOUT_MS);
            while telemetry
                .samples
                .front()
                .is_some_and(|s| now_ms - s.timestamp_ms > SAMPLE_RETENTION_MS)
            {
                telemetry.samples.pop_front();
            }
        }
    }

    /// Rates for one session over the sliding window, as of `now_ms`.
    pub fn session_rates(&self, session_id: &str, now_ms: i64) -> SessionRates {
        let Some(telemetry) = self.sessions.get(session_id) else {
            return SessionRates::default();
        };
        let window_start = now_ms - RATE_WINDOW_MS;
        let (mut output, mut input, mut cache_read) = (0u64, 0u64, 0u64);
        for sample in telemetry.samples.iter().filter(|s| s.timestamp_ms >= window_start) {
            output += sample.output_tokens;
            input += sample.input_tokens;
            cache_read += sample.cache_read_tokens;
        }
        let seconds = RATE_WINDOW_MS as f64 / 1000.0;
        SessionRates {
            output_tokens_per_sec: output as f64 / seconds,
            input_tokens_per_sec: input as f64 / seconds,
            cache_read_tokens_per_sec: cache_read as f64 / seconds,
        }
    }

    pub fn current_tool(&self, session_id: &str) -> Option<ToolActivity> {
        self.sessions.get(session_id)?.current_tool.clone()
    }

    pub fn live_model_id(&self, session_id: &str) -> Option<String> {
        self.sessions.get(session_id)?.live_model_id.clone()
    }

    /// Activity % for a lane: this session's output rate against the rolling
    /// machine max. Call once per push with all current rates so the max both
    /// tracks new peaks instantly and decays slowly when things calm down.
    pub fn activity_percents(&mut self, rates: &[(String, f64)]) -> HashMap<String, f64> {
        let current_max = rates.iter().map(|(_, r)| *r).fold(0.0, f64::max);
        // 0.999 per 100ms tick ≈ 45s half-life: peaks linger, then fade.
        self.rolling_max_output_rate = (self.rolling_max_output_rate * 0.999).max(current_max);
        let denominator = self.rolling_max_output_rate.max(1.0);
        rates
            .iter()
            .map(|(id, rate)| (id.clone(), (rate / denominator * 100.0).clamp(0.0, 100.0)))
            .collect()
    }

    /// The last ~40 feed events, oldest→newest (frontend dedupes by id).
    pub fn feed_events(&self) -> Vec<FeedEvent> {
        self.feed_ring.iter().cloned().collect()
    }
}

/// Days since the Unix epoch in local time — changes exactly at local midnight.
fn local_day_number(now_ms: i64, utc_offset_ms: i64) -> i64 {
    (now_ms + utc_offset_ms).div_euclid(86_400_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::ToolUseBlock;

    fn assistant(request_id: &str, t: i64, out: u64, tools: Vec<ToolUseBlock>) -> TranscriptLine {
        TranscriptLine::Assistant {
            request_id: request_id.into(),
            timestamp_ms: t,
            model: Some("claude-opus-4-8".into()),
            usage: UsageCounts {
                output_tokens: out,
                input_tokens: 10,
                cache_read_tokens: 90,
                cache_creation_tokens: 0,
            },
            tool_uses: tools,
        }
    }

    fn new_agg() -> Aggregator {
        Aggregator::new(0, 0)
    }

    #[test]
    fn repeated_usage_for_same_request_counts_once() {
        let mut agg = new_agg();
        // Claude Code writes several lines per request with identical usage —
        // the exact pattern verified in a real transcript.
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        agg.ingest("s1", assistant("req_A", 1_100, 500, vec![]));
        agg.ingest("s1", assistant("req_A", 1_200, 500, vec![]));
        let rates = agg.session_rates("s1", 2_000);
        assert!((rates.output_tokens_per_sec - 100.0).abs() < 0.01); // 500 / 5s
    }

    #[test]
    fn different_requests_accumulate() {
        let mut agg = new_agg();
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        agg.ingest("s1", assistant("req_B", 1_500, 300, vec![]));
        let rates = agg.session_rates("s1", 2_000);
        assert!((rates.output_tokens_per_sec - 160.0).abs() < 0.01); // 800 / 5s
    }

    #[test]
    fn old_samples_fall_out_of_the_window() {
        let mut agg = new_agg();
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        let rates = agg.session_rates("s1", 20_000); // 19s later
        assert_eq!(rates.output_tokens_per_sec, 0.0);
    }

    #[test]
    fn tool_lifecycle_produces_feed_event_with_duration() {
        let mut agg = new_agg();
        agg.set_session_name("s1", "playground");
        let tool = ToolUseBlock {
            tool_use_id: "toolu_1".into(),
            tool_name: "Bash".into(),
            arg_summary: "npm test".into(),
            lines_changed: 0,
            is_git_commit: false,
        };
        agg.ingest("s1", assistant("req_A", 1_000, 100, vec![tool]));
        assert!(agg.current_tool("s1").is_some());

        agg.ingest(
            "s1",
            TranscriptLine::ToolResult {
                tool_use_id: "toolu_1".into(),
                timestamp_ms: 3_500,
                is_error: false,
            },
        );
        assert!(agg.current_tool("s1").is_none()); // tool finished
        let events = agg.feed_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tool_name, "Bash");
        assert_eq!(events[0].duration_ms, Some(2_500));
        assert_eq!(events[0].session_name, "playground");
    }

    #[test]
    fn daily_totals_accumulate_and_roll_over_at_midnight() {
        let mut agg = new_agg();
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        let totals = agg.today_totals();
        assert_eq!(totals.tokens, 500 + 10 + 90); // out + in + cache_read
        assert!(totals.cost_usd > 0.0);

        // A line stamped on the next local day resets the counters first.
        agg.ingest("s1", assistant("req_B", 86_400_000 + 1_000, 100, vec![]));
        assert_eq!(agg.today_totals().tokens, 100 + 10 + 90);
    }

    #[test]
    fn burn_rate_extrapolates_window_cost_to_an_hour() {
        let mut agg = new_agg();
        // 1000 output tokens on Opus ≈ $0.025 (plus small input/cache-read cost).
        agg.ingest("s1", assistant("req_A", 10_000, 1_000, vec![]));
        let rate = agg.burn_rate_usd_per_hour(20_000);
        // Window is 60s → ×60 to the hour: at least 0.025 × 60 = $1.5/hr.
        assert!(rate > 1.5, "rate was {rate}");
        // Far in the future the window is empty again.
        assert_eq!(agg.burn_rate_usd_per_hour(10_000_000), 0.0);
    }

    #[test]
    fn context_percent_uses_model_window() {
        let mut agg = new_agg();
        // assistant() uses claude-opus-4-8 → 1M window; context = 10 + 90 = 100 tokens.
        agg.ingest("s1", assistant("req_A", 1_000, 50, vec![]));
        let pct = agg.context_percent("s1").unwrap();
        assert!((pct - 0.01).abs() < 1e-9); // 100 / 1M = 0.01%
        assert!(agg.context_percent("unknown").is_none());
    }

    #[test]
    fn seeded_context_yields_to_live_data() {
        let mut agg = new_agg();
        agg.seed_context_tokens("s1", 500_000);
        assert!((agg.context_percent("s1").unwrap() - 50.0).abs() < 1e-9);
        // Live line (context 100 tokens) overrides the seed.
        agg.ingest("s1", assistant("req_A", 1_000, 50, vec![]));
        assert!(agg.context_percent("s1").unwrap() < 1.0);
        // Re-seeding after live data is ignored.
        agg.seed_context_tokens("s1", 900_000);
        assert!(agg.context_percent("s1").unwrap() < 1.0);
    }

    #[test]
    fn cache_ratio_follows_spec_formula() {
        let mut agg = new_agg();
        agg.ingest("s1", assistant("req_A", 1_000, 100, vec![]));
        let rates = agg.session_rates("s1", 2_000);
        // cache_read=90, input=10 → ratio from the rate sums = 90%
        // (the pusher derives the aggregate the same way, per the SPEC formula)
        let ratio = rates.cache_read_tokens_per_sec
            / (rates.cache_read_tokens_per_sec + rates.input_tokens_per_sec);
        assert!((ratio - 0.9).abs() < 1e-9);
    }
}
