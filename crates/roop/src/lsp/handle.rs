use crate::{capabilities, diagnostics, document_symbols, format_edits};
use serde_json::{Value, json};
use std::collections::HashMap;

fn reply(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn publish(uri: &str, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics(text) },
    })
}

/// What the server sends back for one message from the editor, keeping the
/// text of each open document up to date.
pub fn handle(documents: &mut HashMap<String, String>, message: &Value) -> Vec<Value> {
    let params = &message["params"];
    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
    let text =
        |documents: &HashMap<String, String>| documents.get(uri).cloned().unwrap_or_default();
    match message["method"].as_str() {
        Some("initialize") => vec![reply(&message["id"], capabilities())],
        Some("shutdown") => vec![reply(&message["id"], Value::Null)],
        Some("textDocument/didOpen") => {
            let opened = params["textDocument"]["text"].as_str().unwrap_or_default();
            documents.insert(uri.to_string(), opened.to_string());
            vec![publish(uri, opened)]
        }
        Some("textDocument/didChange") => {
            let changed = params["contentChanges"][0]["text"]
                .as_str()
                .unwrap_or_default();
            documents.insert(uri.to_string(), changed.to_string());
            vec![publish(uri, changed)]
        }
        Some("textDocument/didClose") => {
            documents.remove(uri);
            vec![json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": { "uri": uri, "diagnostics": [] },
            })]
        }
        Some("textDocument/formatting") => {
            vec![reply(
                &message["id"],
                Value::Array(format_edits(&text(documents))),
            )]
        }
        Some("textDocument/documentSymbol") => {
            vec![reply(
                &message["id"],
                Value::Array(document_symbols(&text(documents))),
            )]
        }
        Some(_) if message["id"].is_null() => Vec::new(),
        Some(_) => vec![json!({
            "jsonrpc": "2.0",
            "id": message["id"],
            "error": { "code": -32601, "message": "method not found" },
        })],
        None => Vec::new(),
    }
}
