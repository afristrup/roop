/// A layout: text and the places where it may break, laid out by `render`.
#[derive(Clone, Debug)]
pub enum Doc {
    Text(String),
    Concat(Vec<Doc>),
    /// Indents the lines it breaks by one level.
    Nest(Box<Doc>),
    /// Stays on one line if it fits, otherwise breaks all its own lines.
    Group(Box<Doc>),
    /// A space, or a newline when the group breaks.
    Line,
    /// Nothing, or a newline when the group breaks.
    SoftLine,
    /// Always a newline, and the groups around it break.
    HardLine,
    /// Text that only appears when the group breaks.
    IfBreak(&'static str),
    /// Text that goes at the end of the line and does not count for its width.
    Suffix(String),
}

impl Doc {
    pub fn text(s: impl Into<String>) -> Doc {
        Doc::Text(s.into())
    }

    pub fn concat(parts: Vec<Doc>) -> Doc {
        Doc::Concat(parts)
    }

    pub fn nest(inner: Doc) -> Doc {
        Doc::Nest(Box::new(inner))
    }

    pub fn group(inner: Doc) -> Doc {
        Doc::Group(Box::new(inner))
    }

    pub fn nothing() -> Doc {
        Doc::Concat(Vec::new())
    }

    pub fn join(parts: Vec<Doc>, separator: Doc) -> Doc {
        let mut out = Vec::new();
        for (i, part) in parts.into_iter().enumerate() {
            if i > 0 {
                out.push(separator.clone());
            }
            out.push(part);
        }
        Doc::Concat(out)
    }
}
