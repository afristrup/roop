use serde_json::{Value, json};

/// A byte offset in `text` as an LSP position: a zero-based line, and the
/// column counted in UTF-16 units, which is what the protocol asks for.
pub fn position(text: &str, offset: usize) -> Value {
    let offset = offset.min(text.len());
    let before = &text[..text.floor_char_boundary(offset)];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let character: usize = before[start..].chars().map(char::len_utf16).sum();
    json!({ "line": line, "character": character })
}
