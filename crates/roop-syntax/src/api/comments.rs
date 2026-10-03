use crate::{Comment, Span};

/// Every comment of `src`, in order. The language has no strings, so a `//`
/// always starts one.
pub fn comments(src: &str) -> Vec<Comment> {
    let mut found = Vec::new();
    let mut offset = 0;
    for line in src.split_inclusive('\n') {
        if let Some(at) = line.find("//") {
            let text = line[at..].trim_end();
            let start = offset + at;
            found.push(Comment {
                span: Span::from(start..start + text.len()),
                text: text.to_string(),
            });
        }
        offset += line.len();
    }
    found
}
