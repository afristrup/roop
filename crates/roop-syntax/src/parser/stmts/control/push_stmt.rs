use crate::{Err, TokenInput, expr, place};
use crate::{StmtKind, Token};
use chumsky::prelude::*;

/// `push stack <- place;`
pub fn push_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Push)
        .ignore_then(place(expr()))
        .then_ignore(just(Token::LArrow))
        .then(place(expr()))
        .then_ignore(just(Token::Semi))
        .map(|(stack, source)| StmtKind::Push { stack, source })
}
