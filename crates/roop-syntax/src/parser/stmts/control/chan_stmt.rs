use crate::{Block, StmtKind, Token, Type};
use crate::{Err, TokenInput, ident, ty};
use chumsky::prelude::*;

pub fn chan_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Chan)
        .ignore_then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .then(blk)
        .map(|((name, ty), body): ((String, Type), Block)| StmtKind::Chan { name, ty, body })
}
