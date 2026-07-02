// Startup backfill for the odometers: the tailer only sees lines appended
// after launch, but "Tokens Today" should include everything since local
// midnight. This module scans transcripts modified today, replays their lines
// through the same parsing/dedupe rules the live path uses, and returns totals.
//
// Runs once, on its own thread, off the hot path. Read-only.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::aggregator::DailyTotals;
use crate::model_config;
use crate::transcript::{parse_line, TranscriptLine, UsageCounts};

/// Sum today's activity across all projects.
///
/// Only lines with `day_start_ms <= timestamp < cutoff_ms` count. The cutoff
/// is the app's start time: everything at or after it is (or will be) counted
/// by the live tailer, so including it here would double-count.
pub fn scan_today(home: &Path, day_start_ms: i64, cutoff_ms: i64) -> DailyTotals {
    let mut totals = DailyTotals::default();
    let projects_dir = home.join(".claude/projects");

    // Missing dir = Claude Code never ran; zero totals are correct.
    let Ok(project_dirs) = fs::read_dir(&projects_dir) else {
        return totals;
    };

    for project_dir in project_dirs.flatten() {
        let Ok(transcripts) = fs::read_dir(project_dir.path()) else {
            continue; // unreadable project dir (permissions) — skip it
        };
        for entry in transcripts.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            // Cheap pre-filter: a file not touched since midnight has no
            // lines from today (transcripts are append-only).
            let modified_today = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .is_some_and(|d| (d.as_millis() as i64) >= day_start_ms);
            if !modified_today {
                continue;
            }
            accumulate_file(&path, day_start_ms, cutoff_ms, &mut totals);
        }
    }
    totals
}

/// Replay one transcript's lines from today into the totals.
fn accumulate_file(path: &Path, day_start_ms: i64, cutoff_ms: i64, totals: &mut DailyTotals) {
    // Whole-file read: transcripts are a few MB at most, and this runs once
    // at startup on a background thread.
    let Ok(text) = fs::read_to_string(path) else {
        return; // unreadable file — its totals are simply missing
    };

    // Same dedupe rule as the live path: usage repeats across lines of one
    // request, so keep only the latest usage (and model) per request id, then
    // sum the finals.
    let mut final_usage_by_request: HashMap<String, (UsageCounts, String)> = HashMap::new();

    for line in text.lines() {
        // Lines without a parseable timestamp fall back to 0 → outside the
        // window → correctly excluded.
        let Some(parsed) = parse_line(line, 0) else {
            continue;
        };
        match parsed {
            TranscriptLine::Assistant {
                request_id,
                timestamp_ms,
                model,
                usage,
                tool_uses,
            } => {
                if timestamp_ms < day_start_ms || timestamp_ms >= cutoff_ms {
                    continue;
                }
                let model_id = model.unwrap_or_else(|| "claude-opus".to_string());
                final_usage_by_request.insert(request_id, (usage, model_id));
                for tool_use in tool_uses {
                    totals.lines_edited += tool_use.lines_changed;
                    totals.commits += u64::from(tool_use.is_git_commit);
                }
            }
            TranscriptLine::ToolResult { timestamp_ms, .. } => {
                if timestamp_ms >= day_start_ms && timestamp_ms < cutoff_ms {
                    totals.tool_calls += 1;
                }
            }
        }
    }

    for (usage, model_id) in final_usage_by_request.values() {
        totals.tokens += usage.input_tokens
            + usage.output_tokens
            + usage.cache_read_tokens
            + usage.cache_creation_tokens;
        totals.cost_usd += model_config::estimate_cost_usd(
            model_id,
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_tokens,
            usage.cache_creation_tokens,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backfill_counts_only_todays_lines_and_dedupes_requests() {
        let dir = std::env::temp_dir().join(format!("cv-backfill-test-{}", std::process::id()));
        let project = dir.join(".claude/projects/-test-project");
        fs::create_dir_all(&project).unwrap();

        // Times: day starts at 0, app started (cutoff) at 100_000.
        // Line A: yesterday (excluded). B+C: same request today (counted once).
        // D: after cutoff (excluded — the live tailer owns it). E: tool result today.
        let lines = [
            r#"{"type":"assistant","requestId":"old","timestamp":"1969-12-31T00:00:00.000Z","message":{"model":"claude-opus-4-8","usage":{"input_tokens":1,"output_tokens":1},"content":[]}}"#,
            r#"{"type":"assistant","requestId":"req_B","timestamp":"1970-01-01T00:00:10.000Z","message":{"model":"claude-opus-4-8","usage":{"input_tokens":10,"output_tokens":100},"content":[]}}"#,
            r#"{"type":"assistant","requestId":"req_B","timestamp":"1970-01-01T00:00:11.000Z","message":{"model":"claude-opus-4-8","usage":{"input_tokens":10,"output_tokens":100},"content":[]}}"#,
            r#"{"type":"assistant","requestId":"req_D","timestamp":"1970-01-01T00:10:00.000Z","message":{"model":"claude-opus-4-8","usage":{"input_tokens":5,"output_tokens":5},"content":[]}}"#,
            r#"{"type":"user","timestamp":"1970-01-01T00:00:20.000Z","message":{"content":[{"tool_use_id":"toolu_1","type":"tool_result","content":"ok"}]}}"#,
        ];
        fs::write(project.join("abc.jsonl"), lines.join("\n")).unwrap();

        let totals = scan_today(&dir, 0, 100_000);
        assert_eq!(totals.tokens, 110); // req_B once: 10 in + 100 out
        assert_eq!(totals.tool_calls, 1);
        assert!(totals.cost_usd > 0.0);

        fs::remove_dir_all(&dir).unwrap();
    }
}
