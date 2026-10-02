use crate::{Err, TokenInput, expr, place};
use crate::{StmtKind, Token};
use chumsky::prelude::*;

/// `pop stack -> place;`
pub fn pop_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Pop)
        .ignore_then(place(expr()))
        .then_ignore(just(Token::Arrow))
        .then(place(expr()))
        .then_ignore(just(Token::Semi))
        .map(|(stack, target)| StmtKind::Pop { stack, target })
}
