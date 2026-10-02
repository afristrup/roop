use crate::{Block, StmtKind, Token};
use crate::{Err, TokenInput, expr, ident, place};
use chumsky::prelude::*;

pub fn borrow_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Borrow)
        .ignore_then(ident())
        .then_ignore(just(Token::Assign))
        .then(place(expr()))
        .then(blk)
        .map(|((name, source), body)| StmtKind::Borrow { name, source, body })
}
