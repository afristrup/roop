use crate::symbol_of;
use roop_syntax::parse_spanned;
use serde_json::Value;

/// The functions, structs, enums and sessions of the text, for an outline.
pub fn document_symbols(text: &str) -> Vec<Value> {
    parse_spanned(text)
        .map(|items| {
            items
                .iter()
                .filter_map(|(item, span)| symbol_of(text, item, *span))
                .collect()
        })
        .unwrap_or_default()
}
