use crate::{StmtKind, Token};
use crate::{Err, TokenInput, expr, ident, place};
use chumsky::prelude::*;

/// `send c <- place;`
pub fn send_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Send)
        .ignore_then(ident())
        .then_ignore(just(Token::LArrow))
        .then(place(expr()))
        .then_ignore(just(Token::Semi))
        .map(|(chan, source)| StmtKind::Send { chan, source })
}
