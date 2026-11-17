use crate::{Expr, Token};

type Build = fn(Box<Expr>, Box<Expr>) -> Expr;

/// The precedence of a binary operator, whether it groups to the right, and
/// the expression it builds.
pub fn binary(token: &Token) -> Option<(u8, bool, Build)> {
    Some(match token {
        Token::Or => (1, false, Expr::Or),
        Token::And => (2, false, Expr::And),
        Token::Le => (4, false, Expr::Le),
        Token::Eq => (4, false, Expr::Eq),
        Token::Plus => (5, false, Expr::Add),
        Token::Star => (6, false, Expr::Mul),
        Token::Caret => (7, true, Expr::Pow),
        _ => return None,
    })
}
