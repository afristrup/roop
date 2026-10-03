use crate::Span;

/// A `//` comment: its span runs from the slashes to the end of the line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub text: String,
}
