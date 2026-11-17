use crate::Token;

/// A word as a token: `and`, `or` and `not` are operators, the rest are names.
pub fn word(text: &str) -> Token {
    match text {
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        _ => Token::Ident(text.into()),
    }
}
