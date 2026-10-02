use crate::{Err, TokenInput, expr, ident, place};
use crate::{StmtKind, Token};
use chumsky::prelude::*;

/// `recv c -> place;`
pub fn recv_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Recv)
        .ignore_then(ident())
        .then_ignore(just(Token::Arrow))
        .then(place(expr()))
        .then_ignore(just(Token::Semi))
        .map(|(chan, target)| StmtKind::Recv { chan, target })
}
