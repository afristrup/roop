use crate::{Err, TokenInput, expr, place};
use crate::{StmtKind, Token};
use chumsky::prelude::*;

/// `keep place;`
pub fn keep_stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Keep)
        .ignore_then(place(expr()))
        .then_ignore(just(Token::Semi))
        .map(StmtKind::Keep)
}
