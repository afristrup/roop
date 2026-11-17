use crate::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
}
