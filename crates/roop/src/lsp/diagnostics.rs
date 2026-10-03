use crate::range_of;
use roop_syntax::{Item, parse};
use serde_json::{Value, json};

fn diagnostic(text: &str, span: roop_syntax::Span, message: String) -> Value {
    json!({
        "range": range_of(text, span),
        "severity": 1,
        "source": "roop",
        "message": message,
    })
}

/// What is wrong with the text: a syntax error, or else, when the file stands
/// alone and uses no other module, what the checker rejects. A file that uses
/// modules is not checked here, since the checker's positions would be in a
/// program assembled from several files.
pub fn diagnostics(text: &str) -> Vec<Value> {
    let program = match parse(text) {
        Ok(program) => program,
        Err(error) => return vec![diagnostic(text, error.span, error.message)],
    };
    let alone = !program
        .items
        .iter()
        .any(|item| matches!(item, Item::Mod(_) | Item::Use(_)));
    if !alone {
        return Vec::new();
    }
    match roop_check::check(&program) {
        Ok(()) => Vec::new(),
        Err(error) => vec![diagnostic(text, error.span(), error.to_string())],
    }
}
