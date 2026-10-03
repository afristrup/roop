use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_roop"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Client {
            child,
            input,
            output,
        }
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        write!(self.input, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        self.input.flush().unwrap();
    }

    fn receive(&mut self) -> Value {
        let mut length = 0;
        loop {
            let mut line = String::new();
            self.output.read_line(&mut line).unwrap();
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(n) = line.strip_prefix("Content-Length:") {
                length = n.trim().parse().unwrap();
            }
        }
        let mut body = vec![0; length];
        self.output.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn request(&mut self, id: i64, method: &str, params: Value) -> Value {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        self.receive()
    }

    fn open(&mut self, uri: &str, text: &str) -> Value {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": uri, "languageId": "roop", "version": 1, "text": text } },
        }));
        self.receive()
    }

    fn finish(mut self) {
        self.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
        assert!(self.child.wait().unwrap().success());
    }
}

#[test]
fn the_server_says_what_it_can_do() {
    let mut client = Client::start();
    let reply = client.request(1, "initialize", json!({}));
    assert_eq!(reply["id"], 1);
    assert_eq!(
        reply["result"]["capabilities"]["documentFormattingProvider"],
        true
    );
    client.finish();
}

#[test]
fn a_syntax_error_is_a_diagnostic_at_its_place() {
    let mut client = Client::start();
    let published = client.open("file:///a.roop", "fn f(x: &mut i64) {\n    x += ;\n}\n");
    let diagnostics = published["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{published}");
    assert_eq!(diagnostics[0]["severity"], 1);
    assert_eq!(diagnostics[0]["range"]["start"]["line"], 1, "{published}");
    client.finish();
}

#[test]
fn something_the_checker_rejects_is_a_diagnostic() {
    let mut client = Client::start();
    let published = client.open("file:///b.roop", "fn f(x: &mut i64) { x += x; }\n");
    let diagnostics = published["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{published}");
    assert!(
        diagnostics[0]["message"]
            .as_str()
            .unwrap()
            .contains("appears on its own"),
        "{published}"
    );
    client.finish();
}

#[test]
fn a_clean_file_has_no_diagnostics() {
    let mut client = Client::start();
    let published = client.open("file:///c.roop", "fn f(x: &mut i64, y: &i64) { x += y; }\n");
    assert_eq!(published["params"]["diagnostics"], json!([]));
    client.finish();
}

#[test]
fn formatting_replaces_the_document_with_the_formatted_text() {
    let mut client = Client::start();
    client.open("file:///d.roop", "fn f(x:&mut i64,y:&i64){x+=y;}");
    let reply = client.request(
        2,
        "textDocument/formatting",
        json!({ "textDocument": { "uri": "file:///d.roop" } }),
    );
    let edits = reply["result"].as_array().unwrap();
    assert_eq!(edits.len(), 1, "{reply}");
    assert_eq!(
        edits[0]["newText"],
        "fn f(x: &mut i64, y: &i64) {\n    x += y;\n}\n"
    );
    client.finish();
}

#[test]
fn the_outline_lists_the_items() {
    let mut client = Client::start();
    client.open(
        "file:///e.roop",
        "struct P { a: i64 }\nenum E { A, B }\nfn go(x: &mut i64) { x += 1; }\n",
    );
    let reply = client.request(
        3,
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": "file:///e.roop" } }),
    );
    let names: Vec<&str> = reply["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["P", "E", "go"], "{reply}");
    client.finish();
}

#[test]
fn an_unknown_request_gets_an_error_and_a_notification_is_ignored() {
    let mut client = Client::start();
    let reply = client.request(4, "textDocument/hover", json!({}));
    assert_eq!(reply["error"]["code"], -32601);
    client.send(json!({ "jsonrpc": "2.0", "method": "$/setTrace", "params": {} }));
    let reply = client.request(5, "shutdown", json!({}));
    assert_eq!(reply["result"], Value::Null);
    client.finish();
}
