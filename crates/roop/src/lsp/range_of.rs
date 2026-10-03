use crate::position;
use roop_syntax::Span;
use serde_json::{Value, json};

/// The LSP range of a span of `text`. An empty span, such as the end of the
/// input, is widened to the character before it so an editor can underline it.
pub fn range_of(text: &str, span: Span) -> Value {
    let end = span.end.max(span.start);
    let start = if span.start == end {
        text[..text.floor_char_boundary(span.start)]
            .chars()
            .next_back()
            .map_or(0, |c| span.start - c.len_utf8())
    } else {
        span.start
    };
    json!({ "start": position(text, start), "end": position(text, end) })
}
