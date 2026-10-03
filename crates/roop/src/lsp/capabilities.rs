use serde_json::{Value, json};

/// What the server tells the editor it can do. Whole documents are sent on every
/// change, since roop files are small and parsing one is quick.
pub fn capabilities() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": { "openClose": true, "change": 1 },
            "documentFormattingProvider": true,
            "documentSymbolProvider": true,
        },
        "serverInfo": { "name": "roop" },
    })
}
