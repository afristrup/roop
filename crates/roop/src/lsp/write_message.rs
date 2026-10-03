use serde_json::Value;
use std::io::{self, Write};

/// Sends one JSON-RPC message with its `Content-Length` header.
pub fn write_message(output: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = message.to_string();
    write!(output, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    output.flush()
}
