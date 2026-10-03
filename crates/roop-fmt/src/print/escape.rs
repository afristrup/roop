/// The text of a string or byte literal for `bytes`, with the escapes the
/// lexer reads back.
pub fn escape(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            0 => out.push_str("\\0"),
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\'' => out.push_str("\\'"),
            0x20..=0x7e => out.push(b as char),
            _ => {
                let width = utf8_width(b);
                match std::str::from_utf8(&bytes[i..(i + width).min(bytes.len())]) {
                    Ok(text) if width > 1 && text.chars().count() == 1 => {
                        out.push_str(text);
                        i += width;
                        continue;
                    }
                    _ => out.push_str(&format!("\\x{b:02x}")),
                }
            }
        }
        i += 1;
    }
    out
}

fn utf8_width(first: u8) -> usize {
    match first {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => 1,
    }
}
