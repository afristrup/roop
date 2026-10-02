use crate::{Err, TokenInput, expr, place};
use crate::{OverwriteOp, StmtKind, Token};
use chumsky::prelude::*;

/// `place = e;`, `place *= e;`, `place /= e;`, `place %= e;`
pub fn overwrite_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    let op = select! {
        Token::Assign => OverwriteOp::Assign,
        Token::StarEq => OverwriteOp::Mul,
        Token::SlashEq => OverwriteOp::Div,
        Token::PercentEq => OverwriteOp::Rem,
    };
    place(expr())
        .then(op)
        .then(expr())
        .then_ignore(just(Token::Semi))
        .map(|((target, op), value)| StmtKind::Overwrite { target, op, value })
}
