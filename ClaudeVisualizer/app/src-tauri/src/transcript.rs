// TranscriptTailer — Tier A2 of the ingestion pipeline (SPEC §2).
//
// Transcripts (~/.claude/projects/<encoded-cwd>/<sessionId>.jsonl) are
// append-only JSON Lines and grow to many megabytes. We therefore keep a byte
// offset per session and, on every poll, read only the newly appended bytes,
// parsing just the complete lines (a line still being written is left for the
// next poll). Read-only, like everything else in Tier A.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Token counts from one assistant line's `usage` object.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct UsageCounts {
    pub output_tokens: u64,
    pub input_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

/// A `tool_use` content block: Claude started running a tool.
#[derive(Clone, Debug)]
pub struct ToolUseBlock {
    /// "toolu_…" id — the later tool_result refers back to this.
    pub tool_use_id: String,
    pub tool_name: String,
    pub arg_summary: String,
}

/// The subset of a transcript line the aggregator cares about.
#[derive(Debug)]
pub enum TranscriptLine {
    /// An assistant turn: token usage plus any tools it started.
    Assistant {
        request_id: String,
        timestamp_ms: i64,
        model: Option<String>,
        usage: UsageCounts,
        tool_uses: Vec<ToolUseBlock>,
    },
    /// A tool finished (tool_result blocks arrive inside `user` lines).
    ToolResult {
        tool_use_id: String,
        timestamp_ms: i64,
        is_error: bool,
    },
}

/// "2026-07-02T15:54:55.548Z" → epoch ms. Falls back to `fallback_ms` (arrival
/// time) if the timestamp is missing or malformed — a wrong-but-close time
/// only slightly skews the 5s rate window, which beats dropping the sample.
fn parse_timestamp_ms(value: Option<&str>, fallback_ms: i64) -> i64 {
    value
        .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
        .map(|t| (t.unix_timestamp_nanos() / 1_000_000) as i64)
        .unwrap_or(fallback_ms)
}

/// Pick a short human-readable summary out of a tool's `input` object, e.g.
/// {"command":"npm test"} → "npm test". Tries well-known keys first, then any
/// string value, so unknown tools still show something useful.
fn summarize_tool_input(input: &serde_json::Value) -> String {
    const PREFERRED_KEYS: [&str; 9] = [
        "command", "file_path", "path", "pattern", "query", "url", "prompt", "description", "subject",
    ];
    let text = PREFERRED_KEYS
        .iter()
        .find_map(|key| input.get(key).and_then(|v| v.as_str()))
        .or_else(|| {
            input
                .as_object()
                .and_then(|map| map.values().find_map(|v| v.as_str()))
        })
        .unwrap_or("");
    truncate_chars(text, 46)
}

/// Cut at a char boundary (byte slicing would panic mid-UTF-8) and add an ellipsis.
fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut result: String = text.chars().take(max_chars).collect();
    result.push('…');
    result
}

/// Parse one raw transcript line. Returns None for the many line types the
/// dashboard doesn't consume (`mode`, `file-history-snapshot`, plain user
/// prompts, …) and for malformed JSON (a line can be cut short by a crash).
pub fn parse_line(line: &str, arrival_ms: i64) -> Option<TranscriptLine> {
    // Cheap substring pre-filter before paying for full JSON parsing — the
    // overwhelming majority of lines are types we ignore.
    let looks_assistant = line.contains("\"type\":\"assistant\"");
    let looks_tool_result = line.contains("\"tool_result\"");
    if !looks_assistant && !looks_tool_result {
        return None;
    }

    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let line_type = value.get("type")?.as_str()?;
    let timestamp_ms = parse_timestamp_ms(value.get("timestamp").and_then(|t| t.as_str()), arrival_ms);

    match line_type {
        "assistant" => {
            let message = value.get("message")?;
            let usage = message.get("usage");
            let read_count = |key: &str| -> u64 {
                usage
                    .and_then(|u| u.get(key))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            };
            let tool_uses = message
                .get("content")
                .and_then(|c| c.as_array())
                .map(|blocks| {
                    blocks
                        .iter()
                        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                        .filter_map(|b| {
                            Some(ToolUseBlock {
                                tool_use_id: b.get("id")?.as_str()?.to_string(),
                                tool_name: b.get("name")?.as_str()?.to_string(),
                                arg_summary: summarize_tool_input(b.get("input").unwrap_or(&serde_json::Value::Null)),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(TranscriptLine::Assistant {
                request_id: value
                    .get("requestId")
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string(),
                timestamp_ms,
                model: message
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(String::from),
                usage: UsageCounts {
                    output_tokens: read_count("output_tokens"),
                    input_tokens: read_count("input_tokens"),
                    cache_read_tokens: read_count("cache_read_input_tokens"),
                    cache_creation_tokens: read_count("cache_creation_input_tokens"),
                },
                tool_uses,
            })
        }
        "user" => {
            // tool_result blocks ride inside user lines; take the first one.
            let block = value
                .get("message")?
                .get("content")?
                .as_array()?
                .iter()
                .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))?;
            Some(TranscriptLine::ToolResult {
                tool_use_id: block.get("tool_use_id")?.as_str()?.to_string(),
                timestamp_ms,
                is_error: block
                    .get("is_error")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false),
            })
        }
        _ => None,
    }
}

/// Byte-offset tailer: remembers how far into each session's transcript we
/// have read and returns only complete new lines on each call.
#[derive(Default)]
pub struct TranscriptTailer {
    offsets: HashMap<String, u64>,
}

impl TranscriptTailer {
    /// First sighting of a session: start tailing from the file's current end.
    /// History is deliberately skipped — rate windows and the feed only want
    /// what happens from now on (Phase 3 handles daily totals differently).
    pub fn start_at_end(&mut self, session_id: &str, path: &Path) {
        if self.offsets.contains_key(session_id) {
            return;
        }
        let length = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        self.offsets.insert(session_id.to_string(), length);
    }

    /// Forget sessions that ended so the offset map doesn't grow forever.
    pub fn retain_sessions(&mut self, live_ids: &[String]) {
        self.offsets.retain(|id, _| live_ids.iter().any(|live| live == id));
    }

    /// Read newly appended complete lines. The trailing partial line (no '\n'
    /// yet) is left unread; the offset only advances past complete lines.
    pub fn read_new_lines(&mut self, session_id: &str, path: &Path) -> Vec<String> {
        let offset = self.offsets.entry(session_id.to_string()).or_insert(0);

        let Ok(mut file) = fs::File::open(path) else {
            return Vec::new(); // transcript not created yet — nothing to read
        };
        let length = file.metadata().map(|m| m.len()).unwrap_or(0);
        if length < *offset {
            // File shrank (rotated or recreated) — restart from the beginning
            // rather than reading garbage from beyond the end.
            *offset = 0;
        }
        if length == *offset {
            return Vec::new();
        }

        if file.seek(SeekFrom::Start(*offset)).is_err() {
            return Vec::new();
        }
        let mut buffer = Vec::with_capacity((length - *offset) as usize);
        if file.read_to_end(&mut buffer).is_err() {
            return Vec::new(); // transient read failure — retry next poll
        }

        // Only consume up to the last newline; the remainder is mid-write.
        let Some(last_newline) = buffer.iter().rposition(|&b| b == b'\n') else {
            return Vec::new();
        };
        *offset += (last_newline + 1) as u64;

        String::from_utf8_lossy(&buffer[..=last_newline])
            .lines()
            .map(String::from)
            .filter(|l| !l.trim().is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_assistant_usage_and_tool_use() {
        let line = r#"{"type":"assistant","requestId":"req_1","timestamp":"2026-07-02T15:54:55.548Z","message":{"model":"claude-opus-4-8","usage":{"input_tokens":7290,"output_tokens":567,"cache_creation_input_tokens":23173,"cache_read_input_tokens":0},"content":[{"type":"tool_use","id":"toolu_9","name":"Bash","input":{"command":"npm test"}}]}}"#;
        match parse_line(line, 0) {
            Some(TranscriptLine::Assistant { request_id, usage, tool_uses, model, timestamp_ms }) => {
                assert_eq!(request_id, "req_1");
                assert_eq!(usage.output_tokens, 567);
                assert_eq!(usage.input_tokens, 7290);
                assert_eq!(usage.cache_creation_tokens, 23173);
                assert_eq!(model.as_deref(), Some("claude-opus-4-8"));
                assert_eq!(tool_uses.len(), 1);
                assert_eq!(tool_uses[0].tool_name, "Bash");
                assert_eq!(tool_uses[0].arg_summary, "npm test");
                assert!(timestamp_ms > 1_700_000_000_000);
            }
            other => panic!("expected Assistant, got {other:?}"),
        }
    }

    #[test]
    fn parses_tool_result_from_user_line() {
        let line = r#"{"type":"user","timestamp":"2026-07-02T20:16:51.772Z","message":{"content":[{"tool_use_id":"toolu_9","type":"tool_result","content":"ok","is_error":false}]}}"#;
        match parse_line(line, 0) {
            Some(TranscriptLine::ToolResult { tool_use_id, is_error, .. }) => {
                assert_eq!(tool_use_id, "toolu_9");
                assert!(!is_error);
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[test]
    fn ignores_irrelevant_and_broken_lines() {
        assert!(parse_line(r#"{"type":"mode","mode":"default"}"#, 0).is_none());
        assert!(parse_line(r#"{"type":"assistant","message":"#, 0).is_none()); // truncated
        assert!(parse_line("", 0).is_none());
    }

    #[test]
    fn tailer_reads_only_new_complete_lines() {
        let dir = std::env::temp_dir().join(format!("cv-tailer-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.jsonl");
        fs::write(&path, "old line\n").unwrap();

        let mut tailer = TranscriptTailer::default();
        // First sighting starts at EOF — the pre-existing line is skipped.
        tailer.start_at_end("s1", &path);
        assert!(tailer.read_new_lines("s1", &path).is_empty());

        // Append one complete line and one partial (no trailing newline yet).
        let mut content = fs::read_to_string(&path).unwrap();
        content.push_str("new line\npartial");
        fs::write(&path, &content).unwrap();
        assert_eq!(tailer.read_new_lines("s1", &path), vec!["new line"]);

        // Completing the partial line makes it readable on the next poll.
        let mut content = fs::read_to_string(&path).unwrap();
        content.push('\n');
        fs::write(&path, &content).unwrap();
        assert_eq!(tailer.read_new_lines("s1", &path), vec!["partial"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn truncates_long_tool_args_safely() {
        let long = "x".repeat(100);
        let summary = summarize_tool_input(&serde_json::json!({ "command": long }));
        assert_eq!(summary.chars().count(), 47); // 46 + ellipsis
    }
}
