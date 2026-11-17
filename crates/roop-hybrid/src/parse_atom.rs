use crate::{Cursor, Expr, HybridError, Token, parse_expr};

/// `!e`, a literal, a name, or a parenthesised expression. Negation binds
/// looser than comparison, so `!x <= 3` is `!(x <= 3)`.
pub fn parse_atom(c: &mut Cursor) -> Result<Expr, HybridError> {
    match c.peek().clone() {
        Token::Not => {
            c.bump();
            Ok(Expr::Not(Box::new(parse_expr(c, 4)?)))
        }
        Token::Num(n) => {
            c.bump();
            Ok(Expr::Num(n))
        }
        Token::LParen => {
            c.bump();
            let inner = parse_expr(c, 0)?;
            c.expect(&Token::RParen)?;
            Ok(inner)
        }
        Token::Ident(w) if matches!(w.as_str(), "tt" | "true") => {
            c.bump();
            Ok(Expr::Bool(true))
        }
        Token::Ident(w) if matches!(w.as_str(), "ff" | "false") => {
            c.bump();
            Ok(Expr::Bool(false))
        }
        Token::Ident(_) => c.name().map(Expr::Var),
        _ => Err(c.error("an expression")),
    }
}
