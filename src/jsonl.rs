use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub tool_calls: Vec<String>,
    pub timestamp: String,
    pub session_id: String,
}

impl ParsedMessage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_creation_tokens
    }
}

#[derive(Debug, Default)]
pub struct SessionStats {
    pub total_tokens: u64,
    pub total_tool_calls: u64,
    pub tool_breakdown: HashMap<String, u64>,
    pub messages: Vec<ParsedMessage>,
}

#[derive(Deserialize)]
struct JsonlEntry {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    message: Option<MessageBody>,
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct MessageBody {
    usage: Option<Usage>,
    content: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
}

pub fn parse_jsonl_line(line: &str) -> Option<ParsedMessage> {
    let entry: JsonlEntry = serde_json::from_str(line).ok()?;

    if entry.entry_type.as_deref() != Some("assistant") {
        return None;
    }

    let message = entry.message?;
    let usage = message.usage?;

    let mut tool_calls = Vec::new();
    if let Some(serde_json::Value::Array(content)) = &message.content {
        for block in content {
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                    tool_calls.push(name.to_string());
                }
            }
        }
    }

    Some(ParsedMessage {
        input_tokens: usage.input_tokens.unwrap_or(0),
        output_tokens: usage.output_tokens.unwrap_or(0),
        cache_read_tokens: usage.cache_read_input_tokens.unwrap_or(0),
        cache_creation_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
        tool_calls,
        timestamp: entry.timestamp.unwrap_or_default(),
        session_id: entry.session_id.unwrap_or_default(),
    })
}

pub fn scan_sessions_since(dir: &Path, since: Option<chrono::DateTime<chrono::Utc>>) -> SessionStats {
    let mut stats = SessionStats::default();

    let pattern = format!("{}/**/*.jsonl", dir.display());
    let paths: Vec<_> = glob::glob(&pattern)
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    for path in paths {
        if let Some(since) = since {
            if let Ok(meta) = fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    let modified: chrono::DateTime<chrono::Utc> = modified.into();
                    if modified < since {
                        continue;
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some(msg) = parse_jsonl_line(line) {
                    stats.total_tokens += msg.total_tokens();
                    stats.total_tool_calls += msg.tool_calls.len() as u64;
                    for tool in &msg.tool_calls {
                        *stats.tool_breakdown.entry(tool.clone()).or_insert(0) += 1;
                    }
                    stats.messages.push(msg);
                }
            }
        }
    }

    stats
}
