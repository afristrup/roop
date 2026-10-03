use crate::position;
use roop_config::FormatConfig;
use roop_fmt::format_source;
use serde_json::{Value, json};

/// The edit that formats the whole document, or none when it already is, or
/// cannot be formatted: the formatter refuses a text it cannot lay out without
/// losing a comment.
pub fn format_edits(text: &str) -> Vec<Value> {
    match format_source(text, &FormatConfig::default()) {
        Ok(formatted) if formatted != text => vec![json!({
            "range": { "start": position(text, 0), "end": position(text, text.len()) },
            "newText": formatted,
        })],
        _ => Vec::new(),
    }
}
