use crate::{Comment, Span};

/// Every comment of `src`, in order. A `//` inside a string or byte literal
/// is not one.
pub fn comments(src: &str) -> Vec<Comment> {
    let bytes = src.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            quote @ (b'"' | b'\'') => {
                i += 1;
                while i < bytes.len() && bytes[i] != quote && bytes[i] != b'\n' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let end = src[i..].find('\n').map_or(src.len(), |n| i + n);
                let text = src[i..end].trim_end();
                found.push(Comment {
                    span: Span::from(i..i + text.len()),
                    text: text.to_string(),
                });
                i = end;
            }
            _ => i += 1,
        }
    }
    found
}
