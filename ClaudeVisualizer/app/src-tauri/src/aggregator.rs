// Aggregator — turns parsed transcript lines into the rolling metrics the
// gauges and lanes display (SPEC §3): per-session token rates over a sliding
// window, cache-hit ratio, current tool, activity %, and the diagnostic feed.

use std::collections::{HashMap, VecDeque};

use crate::snapshot::{FeedEvent, ToolActivity};
use crate::transcript::{TranscriptLine, UsageCounts};

/// Sliding window for rate computation (SPEC: "rolling ~5 s window").
const RATE_WINDOW_MS: i64 = 5_000;
/// Keep samples a bit longer than the window so a late prune can't cut into it.
const SAMPLE_RETENTION_MS: i64 = 10_000;
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

/// Everything the aggregator tracks for one live session.
#[derive(Default)]
struct SessionTelemetry {
    samples: VecDeque<UsageSample>,
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

/// Rates computed for one session at push time.
#[derive(Clone, Copy, Default)]
pub struct SessionRates {
    pub output_tokens_per_sec: f64,
    pub input_tokens_per_sec: f64,
    pub cache_read_tokens_per_sec: f64,
    /// cache_read ÷ (cache_read + input) over the window; None = no traffic.
    pub cache_hit_ratio: Option<f64>,
}

pub struct Aggregator {
    sessions: HashMap<String, SessionTelemetry>,
    feed_ring: VecDeque<FeedEvent>,
    next_event_id: u64,
    session_name_by_id: HashMap<String, String>,
    /// Slow-decaying max of any session's output rate — the denominator for
    /// each lane's activity % ("normalized to a rolling machine max").
    rolling_max_output_rate: f64,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            feed_ring: VecDeque::with_capacity(FEED_RING_CAPACITY),
            next_event_id: 1,
            session_name_by_id: HashMap::new(),
            rolling_max_output_rate: 0.0,
        }
    }

    /// Tell the aggregator a session's display name (feed rows show it).
    pub fn set_session_name(&mut self, session_id: &str, name: &str) {
        self.session_name_by_id
            .insert(session_id.to_string(), name.to_string());
    }

    /// Feed one parsed transcript line into the per-session state.
    pub fn ingest(&mut self, session_id: &str, line: TranscriptLine) {
        let telemetry = self.sessions.entry(session_id.to_string()).or_default();

        match line {
            TranscriptLine::Assistant {
                request_id,
                timestamp_ms,
                model,
                usage,
                tool_uses,
            } => {
                if model.is_some() {
                    telemetry.live_model_id = model;
                }

                // Delta against the previous line of the SAME request; the
                // first line of a request counts in full. saturating_sub
                // guards against usage ever going down (would underflow u64).
                let previous = telemetry
                    .last_usage_by_request
                    .insert(request_id.clone(), usage)
                    .unwrap_or_default();
                let sample = UsageSample {
                    timestamp_ms,
                    output_tokens: usage.output_tokens.saturating_sub(previous.output_tokens),
                    input_tokens: usage.input_tokens.saturating_sub(previous.input_tokens),
                    cache_read_tokens: usage
                        .cache_read_tokens
                        .saturating_sub(previous.cache_read_tokens),
                };
                if sample.output_tokens > 0 || sample.input_tokens > 0 || sample.cache_read_tokens > 0 {
                    telemetry.samples.push_back(sample);
                }

                // Cap the per-request map: requests complete quickly, so only
                // the most recent handful matter. Drop everything on overflow
                // (worst case: one over-counted sample after a 64-request burst).
                if telemetry.last_usage_by_request.len() > 64 {
                    telemetry.last_usage_by_request.clear();
                    telemetry.last_usage_by_request.insert(request_id, usage);
                }

                for tool_use in tool_uses {
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
            }
            TranscriptLine::ToolResult {
                tool_use_id,
                timestamp_ms,
                is_error,
            } => {
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

    /// Drop state for sessions that ended and expire stale pending tools.
    pub fn prune(&mut self, live_ids: &[String], now_ms: i64) {
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
        let denominator = (cache_read + input) as f64;
        SessionRates {
            output_tokens_per_sec: output as f64 / seconds,
            input_tokens_per_sec: input as f64 / seconds,
            cache_read_tokens_per_sec: cache_read as f64 / seconds,
            cache_hit_ratio: (denominator > 0.0).then(|| cache_read as f64 / denominator),
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

    #[test]
    fn repeated_usage_for_same_request_counts_once() {
        let mut agg = Aggregator::new();
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
        let mut agg = Aggregator::new();
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        agg.ingest("s1", assistant("req_B", 1_500, 300, vec![]));
        let rates = agg.session_rates("s1", 2_000);
        assert!((rates.output_tokens_per_sec - 160.0).abs() < 0.01); // 800 / 5s
    }

    #[test]
    fn old_samples_fall_out_of_the_window() {
        let mut agg = Aggregator::new();
        agg.ingest("s1", assistant("req_A", 1_000, 500, vec![]));
        let rates = agg.session_rates("s1", 20_000); // 19s later
        assert_eq!(rates.output_tokens_per_sec, 0.0);
    }

    #[test]
    fn tool_lifecycle_produces_feed_event_with_duration() {
        let mut agg = Aggregator::new();
        agg.set_session_name("s1", "playground");
        let tool = ToolUseBlock {
            tool_use_id: "toolu_1".into(),
            tool_name: "Bash".into(),
            arg_summary: "npm test".into(),
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
    fn cache_ratio_follows_spec_formula() {
        let mut agg = Aggregator::new();
        agg.ingest("s1", assistant("req_A", 1_000, 100, vec![]));
        let rates = agg.session_rates("s1", 2_000);
        // cache_read=90, input=10 → 90%
        assert!((rates.cache_hit_ratio.unwrap() - 0.9).abs() < 1e-9);
    }
}
