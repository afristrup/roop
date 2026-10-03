use crate::range_of;
use roop_syntax::{Item, Span};
use serde_json::{Value, json};

/// The outline entry of an item, with the LSP symbol kind: 12 function,
/// 23 struct, 10 enum, 5 class (a session).
pub fn symbol_of(text: &str, item: &Item, span: Span) -> Option<Value> {
    let (name, kind) = match item {
        Item::Fn(f) => (f.name.clone(), 12),
        Item::Struct(def) => (def.name.clone(), 23),
        Item::Enum(def) => (def.name.clone(), 10),
        Item::Session(def) => (def.name.clone(), 5),
        Item::Mod(_) | Item::Use(_) => return None,
    };
    let range = range_of(text, span);
    Some(json!({ "name": name, "kind": kind, "range": range, "selectionRange": range }))
}
